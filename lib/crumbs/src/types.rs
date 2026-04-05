//! Crumb data types — mirrors the Python CrumbStore schema exactly.
//!
//! The shared storage format is JSON files. Both Python and Rust
//! read/write the same files — no duplication, no sync issues.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The primary crumb structure — stored as individual JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crumb {
    pub id: String,
    #[serde(rename = "type")]
    pub crumb_type: String,
    pub created: String,
    pub updated: String,
    pub location: String,
    #[serde(default)]
    pub display_location: String,
    pub topic: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub relevance: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Lightweight entry in the master index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrumbMeta {
    pub topic: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created: String,
    pub location: String,
    #[serde(rename = "type")]
    pub crumb_type: String,
    #[serde(default)]
    pub author: String,
}

/// The master index file — `~/.openclaw/crumbs/index.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrumbIndex {
    pub version: u32,
    pub updated: String,
    pub total: usize,
    #[serde(default)]
    pub crumbs: HashMap<String, CrumbMeta>,
}

/// Search query parameters.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub topic: Option<String>,
    pub crumb_type: Option<String>,
    pub author: Option<String>,
    pub limit: usize,
}

/// Fields that can be updated on an existing crumb.
#[derive(Debug, Clone, Default)]
pub struct CrumbUpdate {
    pub topic: Option<String>,
    pub summary: Option<String>,
    pub relevance: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub display_location: Option<String>,
    pub expires: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Statistics about the crumb store.
#[derive(Debug, Clone)]
pub struct CrumbStats {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub by_author: HashMap<String, usize>,
    pub by_tag: HashMap<String, usize>,
}

/// Initialize an empty CrumbIndex.
impl Default for CrumbIndex {
    fn default() -> Self {
        Self {
            version: 1,
            updated: timestamp_now(),
            total: 0,
            crumbs: HashMap::new(),
        }
    }
}

/// Get current UTC timestamp as ISO 8601 string.
pub fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Simple ISO 8601 without timezone suffix for compatibility with Python
    let tm = chrono_utc(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        tm.0, tm.1, tm.2, tm.3, tm.4, tm.5
    )
}

fn chrono_utc(secs: u64) -> (u64, u32, u32, u32, u32, u32) {
    // Simplified: just compute day + time without full chrono
    // Year/month/day from seconds since epoch
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    let seconds = rem % 60;

    // Julian day to YMD (algorithm from Fliegel & Van Flandern)
    let l = days + 68569;
    let n = (4 * l) / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1461001;
    let l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let day = l - (2447 * j) / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;

    (year, month as u32, day as u32, hours as u32, minutes as u32, seconds as u32)
}

/// Infer crumb type from a location string.
pub fn infer_location_type(location: &str) -> &'static str {
    if location.starts_with("http://") || location.starts_with("https://") {
        "web"
    } else if location.starts_with("file://") {
        "local"
    } else if location.starts_with("internal://") {
        "internal"
    } else if location.starts_with("gist://") {
        "gist"
    } else if location.starts_with("notion://") {
        "notion"
    } else {
        "unknown"
    }
}

/// Extract file metadata from a file:// URI.
pub fn extract_file_metadata(location: &str) -> Option<HashMap<String, serde_json::Value>> {
    if !location.starts_with("file://") {
        return None;
    }
    let path = &location[7..]; // strip "file://"
    let path = std::path::Path::new(path);
    if !path.exists() || !path.is_file() {
        return None;
    }
    let mut meta = HashMap::new();
    if let Ok(size) = std::fs::metadata(path) {
        meta.insert("file_size".to_string(), serde_json::json!(size.len()));
    }
    if let Some(ext) = path.extension() {
        meta.insert("file_type".to_string(), serde_json::json!(ext.to_string_lossy()));
    }
    Some(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_location_type() {
        assert_eq!(infer_location_type("https://arxiv.org/abs/1234"), "web");
        assert_eq!(infer_location_type("http://example.com"), "web");
        assert_eq!(infer_location_type("file:///home/user/file.txt"), "local");
        assert_eq!(infer_location_type("internal://curiosity-gap/123"), "internal");
        assert_eq!(infer_location_type("gist://abc123"), "gist");
        assert_eq!(infer_location_type("notion://workspace/page"), "notion");
        assert_eq!(infer_location_type("/some/random/path"), "unknown");
    }

    #[test]
    fn test_timestamp_now() {
        let ts = timestamp_now();
        // Should be ISO 8601 format: "2026-04-05T15:00:00Z"
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('-'));
    }

    #[test]
    fn test_default_index() {
        let idx: CrumbIndex = Default::default();
        assert_eq!(idx.version, 1);
        assert_eq!(idx.total, 0);
        assert!(idx.crumbs.is_empty());
    }

    #[test]
    fn test_chrono_utc() {
        // Jan 1 1970 00:00:00 UTC
        let tm = chrono_utc(0);
        assert_eq!(tm.0, 1970);
        assert_eq!(tm.1, 1);
        assert_eq!(tm.2, 1);

        // Jan 1 2000 00:00:00 UTC = 946684800
        let tm = chrono_utc(946684800);
        assert_eq!(tm.0, 2000);
        assert_eq!(tm.1, 1);
        assert_eq!(tm.2, 1);
    }
}
