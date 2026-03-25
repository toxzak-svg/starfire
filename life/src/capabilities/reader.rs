//! File Reader Capability
//!
//! Star can read files to learn about its environment and the world.

use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Result of a file read operation.
#[derive(Debug)]
pub struct ReadResult {
    pub path: String,
    pub content: String,
    pub success: bool,
    pub error: Option<String>,
    pub lines: usize,
}

/// File reader for Star.
pub struct FileReader {
    /// Allowed base directories (security)
    allowed_dirs: Vec<PathBuf>,
}

impl FileReader {
    /// Create a new file reader with allowed directories.
    pub fn new() -> Self {
        let allowed_dirs = vec![
            PathBuf::from("/home/zach/.openclaw/workspace"),
            PathBuf::from("/home/zach"),
            PathBuf::from("/home/zach/Documents"),
            PathBuf::from("/home/zach/Downloads"),
            PathBuf::from("/home/zach/Desktop"),
        ];
        Self { allowed_dirs }
    }

    /// Check if a path is allowed.
    fn is_allowed(&self, path: &Path) -> bool {
        // Resolve any symlinks first
        let resolved = fs::canonicalize(path).ok();
        let check_path = resolved.as_ref().map(|p| p.as_path()).unwrap_or(path);
        
        self.allowed_dirs.iter().any(|dir| {
            check_path.starts_with(dir) || check_path == dir
        })
    }

    /// Read a file and return its contents.
    pub fn read(&self, path: &str) -> ReadResult {
        let path = PathBuf::from(path);
        
        // Security check
        if !self.is_allowed(&path) {
            return ReadResult {
                path: path.display().to_string(),
                content: String::new(),
                success: false,
                error: Some("Path not allowed. Can only read files in workspace, home, Documents, Downloads, Desktop.".to_string()),
                lines: 0,
            };
        }
        
        // Check if file exists
        if !path.exists() {
            return ReadResult {
                path: path.display().to_string(),
                content: String::new(),
                success: false,
                error: Some("File not found.".to_string()),
                lines: 0,
            };
        }
        
        // Check if it's a file (not a directory)
        if !path.is_file() {
            return ReadResult {
                path: path.display().to_string(),
                content: String::new(),
                success: false,
                error: Some("Path is a directory, not a file.".to_string()),
                lines: 0,
            };
        }
        
        // Try to read
        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines = content.lines().count();
                info!("Read file: {} ({} lines)", path.display(), lines);
                ReadResult {
                    path: path.display().to_string(),
                    content,
                    success: true,
                    error: None,
                    lines,
                }
            }
            Err(e) => ReadResult {
                path: path.display().to_string(),
                content: String::new(),
                success: false,
                error: Some(format!("Cannot read: {}. Do you have permission?", e)),
                lines: 0,
            },
        }
    }

    /// List files in a directory.
    pub fn list_dir(&self, dir_path: &str) -> Result<Vec<String>, String> {
        let path = PathBuf::from(dir_path);
        
        if !self.is_allowed(&path) {
            return Err("Path not allowed.".to_string());
        }
        
        if !path.exists() {
            return Err("Directory not found.".to_string());
        }
        
        if !path.is_dir() {
            return Err("Path is not a directory.".to_string());
        }
        
        let mut entries = Vec::new();
        match fs::read_dir(&path) {
            Ok(dir) => {
                for entry in dir.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name().display().to_string();
                    let file_type = if entry.path().is_dir() { "/" } else { "" };
                    entries.push(format!("{}{}", file_name, file_type));
                }
                Ok(entries)
            }
            Err(e) => Err(format!("Cannot read directory: {}", e)),
        }
    }

    /// Search for files matching a pattern.
    pub fn find_files(&self, dir: &str, pattern: &str) -> Result<Vec<String>, String> {
        let path = PathBuf::from(dir);
        
        if !self.is_allowed(&path) {
            return Err("Path not allowed.".to_string());
        }
        
        if !path.exists() || !path.is_dir() {
            return Err("Directory not found.".to_string());
        }
        
        let mut matches = Vec::new();
        self.search_recursive(&path, pattern, &mut matches)?;
        Ok(matches)
    }
    
    fn search_recursive(&self, dir: &Path, pattern: &str, results: &mut Vec<String>) -> Result<(), String> {
        if !self.is_allowed(dir) {
            return Ok(());
        }
        
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            if path.is_dir() {
                if !name.starts_with('.') {
                    self.search_recursive(&path, pattern, results)?;
                }
            } else if name.to_lowercase().contains(&pattern.to_lowercase()) {
                results.push(path.display().to_string());
            }
        }
        
        Ok(())
    }
}

impl Default for FileReader {
    fn default() -> Self {
        Self::new()
    }
}
