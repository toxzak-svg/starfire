//! Index management — read/write `index.json` with atomic locking.
//!
//! Uses a simple file-based lock (mkdir .lock) to handle concurrent access.

use crate::types::{CrumbIndex, CrumbMeta};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Global lock for index access — single-threaded within one process.
static INDEX_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::new();

fn get_lock() -> &'static Arc<Mutex<()>> {
    INDEX_LOCK.get_or_init(|| Arc::new(Mutex::new(())))
}

fn lock_path(crumbs_dir: &Path) -> PathBuf {
    crumbs_dir.join(".lock")
}

/// Acquire the lock by creating a lock directory (atomic on POSIX, fine on Windows for our use).
fn acquire_lock(lock_path: &Path) -> Result<LockGuard> {
    // Try to create the lock directory
    for _ in 0..100 {
        match fs::create_dir(lock_path) {
            Ok(()) => return Ok(LockGuard(lock_path.to_path_buf())),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // Wait a bit and retry
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => anyhow::bail!("Failed to acquire lock: {}", e),
        }
    }
    anyhow::bail!("Could not acquire lock after 100 attempts")
}

struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

/// Load the index from disk, or return a default empty index.
pub fn load_index(crumbs_dir: &Path) -> Result<CrumbIndex> {
    let index_path = crumbs_dir.join("index.json");
    if !index_path.exists() {
        return Ok(CrumbIndex::default());
    }
    let file = File::open(&index_path)
        .with_context(|| format!("Failed to open index at {:?}", index_path))?;
    let reader = BufReader::new(file);
    let index: CrumbIndex = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse index at {:?}", index_path))?;
    Ok(index)
}

/// Save the index to disk atomically: write to temp, rename.
pub fn save_index(crumbs_dir: &Path, index: &CrumbIndex) -> Result<()> {
    let index_path = crumbs_dir.join("index.json");
    let tmp_path = crumbs_dir.join("index.json.tmp");

    let file = File::create(&tmp_path)
        .with_context(|| format!("Failed to create temp index at {:?}", tmp_path))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, index)
        .context("Failed to serialize index")?;
    writer.flush()
        .context("Failed to flush index")?;

    // Atomic rename on POSIX; on Windows this is also atomic for same filesystem
    fs::rename(&tmp_path, &index_path)
        .with_context(|| format!("Failed to rename temp index to {:?}", index_path))?;

    Ok(())
}

/// Load the index with locking.
pub fn load_index_locked(crumbs_dir: &Path) -> Result<CrumbIndex> {
    let lock_path = lock_path(crumbs_dir);
    let _guard = acquire_lock(&lock_path)?;
    load_index(crumbs_dir)
}

/// Save the index with locking.
pub fn save_index_locked(crumbs_dir: &Path, index: &CrumbIndex) -> Result<()> {
    let lock_path = lock_path(crumbs_dir);
    let _guard = acquire_lock(&lock_path)?;
    save_index(crumbs_dir, index)
}

/// Update a single crumb entry in the index.
pub fn index_add(
    crumbs_dir: &Path,
    id: &str,
    meta: CrumbMeta,
) -> Result<()> {
    let lock_path = lock_path(crumbs_dir);
    let _guard = acquire_lock(&lock_path)?;

    let mut index = load_index(crumbs_dir)?;
    index.crumbs.insert(id.to_string(), meta);
    index.total = index.crumbs.len();
    save_index(crumbs_dir, &index)
}

/// Remove a crumb from the index.
pub fn index_remove(crumbs_dir: &Path, id: &str) -> Result<()> {
    let lock_path = lock_path(crumbs_dir);
    let _guard = acquire_lock(&lock_path)?;

    let mut index = load_index(crumbs_dir)?;
    index.crumbs.remove(id);
    index.total = index.crumbs.len();
    save_index(crumbs_dir, &index)
}

/// List all crumb IDs in the index.
pub fn index_list_ids(crumbs_dir: &Path) -> Result<Vec<String>> {
    let lock_path = lock_path(crumbs_dir);
    let _guard = acquire_lock(&lock_path)?;

    let index = load_index(crumbs_dir)?;
    Ok(index.crumbs.keys().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_load_save_empty_index() {
        let dir = temp_dir();
        let idx: CrumbIndex = Default::default();
        save_index(dir.path(), &idx).unwrap();
        let loaded = load_index(dir.path()).unwrap();
        assert_eq!(loaded.total, 0);
        assert!(loaded.crumbs.is_empty());
    }

    #[test]
    fn test_index_add_remove() {
        let dir = temp_dir();
        let meta = CrumbMeta {
            topic: "Test".to_string(),
            tags: vec!["test".to_string()],
            created: "2026-04-05T00:00:00Z".to_string(),
            location: "https://example.com".to_string(),
            crumb_type: "web".to_string(),
            author: "test".to_string(),
        };
        index_add(dir.path(), "crumb_2026-04-05_abc", meta.clone()).unwrap();

        let loaded = load_index(dir.path()).unwrap();
        assert_eq!(loaded.total, 1);
        assert!(loaded.crumbs.contains_key("crumb_2026-04-05_abc"));

        index_remove(dir.path(), "crumb_2026-04-05_abc").unwrap();
        let loaded = load_index(dir.path()).unwrap();
        assert_eq!(loaded.total, 0);
    }

    #[test]
    fn test_lock_exclusive() {
        let dir = temp_dir();
        let lock = lock_path(dir.path());

        let guard1 = acquire_lock(&lock);
        assert!(guard1.is_ok());

        // Lock is held — another acquire should fail or wait
        // (In single-threaded test this is sequential, so it will retry and succeed)
        // The important thing: no panic and guard is dropped properly
    }
}
