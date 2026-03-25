//! Knowledge Reader
//!
//! Reads content from files and URLs.
//! Supports: .txt, .md, .rst, .json, .html (strips tags), plain URLs.
//!
//! Usage:
//!   let reader = Reader::new();
//!   let content = reader.read_file("path/to/file.txt")?;
//!   let content = reader.read_url("https://example.com")?;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Reads files and URLs to extract text content.
pub struct Reader {
    /// Maximum file size to read (1MB)
    max_file_size: usize,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            max_file_size: 1024 * 1024,
        }
    }

    /// Read a file and extract its text content.
    pub fn read_file(&self, path: &Path) -> Result<String> {
        let metadata = fs::metadata(path)
            .context(format!("Failed to read file metadata: {:?}", path))?;
        
        if metadata.len() as usize > self.max_file_size {
            anyhow::bail!("File too large: {} bytes (max: {})", metadata.len(), self.max_file_size);
        }
        
        let content = fs::read_to_string(path)
            .context(format!("Failed to read file: {:?}", path))?;
        
        Ok(self.extract_text(&content, path))
    }

    /// Read a URL and extract its text content.
    pub fn read_url(&self, url: &str) -> Result<String> {
        let response = ureq::get(url)
            .call()
            .context("Failed to fetch URL")?;
        
        let content = response.into_string()
            .context("Failed to read response body")?;
        
        Ok(self.extract_text(&content, Path::new(url)))
    }

    /// Extract readable text from content based on file type.
    fn extract_text(&self, content: &str, path: &Path) -> String {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match ext.as_str() {
            "txt" | "md" | "rst" | "csv" | "log" => {
                // Plain text — clean and return
                clean_text(content)
            }
            "json" => {
                // Try to extract string values from JSON
                extract_from_json(content)
            }
            "html" | "htm" => {
                // Strip HTML tags
                strip_html(content)
            }
            _ => {
                // Default: try to clean as plain text
                clean_text(content)
            }
        }
    }

    /// Read a directory of files recursively.
    pub fn read_directory(&self, path: &Path) -> Result<Vec<(std::path::PathBuf, String)>> {
        let mut results = Vec::new();
        
        if !path.is_dir() {
            anyhow::bail!("Not a directory: {:?}", path);
        }
        
        for entry in fs::read_dir(path)
            .context(format!("Failed to read directory: {:?}", path))? 
        {
            let entry = entry
                .context("Failed to read directory entry")?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Recurse into subdirectories (limit depth)
                if let Ok(rel) = entry_path.strip_prefix(path) {
                    if rel.components().count() < 3 { // Max 3 levels deep
                        results.extend(self.read_directory(&entry_path)?);
                    }
                }
            } else {
                // Skip very large files
                if let Ok(metadata) = entry_path.metadata() {
                    if metadata.len() as usize <= self.max_file_size {
                        if let Ok(content) = self.read_file(&entry_path) {
                            if !content.trim().is_empty() {
                                results.push((entry_path, content));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}

/// Clean plain text — remove excess whitespace, normalize.
fn clean_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_whitespace = false;
    let mut prev_was_newline = false;
    
    for line in text.lines() {
        let line = line.trim();
        
        if line.is_empty() {
            if !prev_was_newline && !result.is_empty() {
                result.push('\n');
                result.push('\n');
                prev_was_newline = true;
            }
        } else {
            for ch in line.chars() {
                if ch.is_whitespace() {
                    if !in_whitespace && !result.is_empty() {
                        result.push(' ');
                        in_whitespace = true;
                    }
                } else {
                    result.push(ch);
                    in_whitespace = false;
                    prev_was_newline = false;
                }
            }
            result.push(' ');
        }
    }
    
    result.trim().to_string()
}

/// Strip HTML tags from content.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    
    let html_lower = html.to_lowercase();
    let bytes = html.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        let remaining = &bytes[i..];
        
        // Check for script/style tags
        if remaining.starts_with(b"<script") || remaining.starts_with(b"<SCRIPT") {
            in_script = true;
        } else if remaining.starts_with(b"</script>") || remaining.starts_with(b"</SCRIPT>") {
            in_script = false;
        } else if remaining.starts_with(b"<style") || remaining.starts_with(b"<STYLE") {
            in_style = true;
        } else if remaining.starts_with(b"</style>") || remaining.starts_with(b"</STYLE>") {
            in_style = false;
        }
        
        if in_script || in_style {
            i += 1;
            continue;
        }
        
        if bytes[i] == b'<' {
            in_tag = true;
        } else if bytes[i] == b'>' {
            in_tag = false;
        } else if !in_tag {
            result.push(bytes[i] as char);
        }
        
        i += 1;
    }
    
    clean_text(&result)
}

/// Extract readable text from JSON.
fn extract_from_json(json: &str) -> String {
    let mut result = Vec::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut current = String::new();
    
    for ch in json.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        
        if ch == '\\' {
            escape_next = true;
            continue;
        }
        
        if ch == '"' {
            if in_string {
                // End of string
                if !current.is_empty() {
                    result.push(current.clone());
                    result.push(" ".to_string());
                    current.clear();
                }
            }
            in_string = !in_string;
            continue;
        }
        
        if in_string {
            current.push(ch);
        }
    }
    
    clean_text(&result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Hello    world\n\n\n  This is    a test.  ";
        let cleaned = clean_text(input);
        assert!(!cleaned.contains("    "));
        assert!(cleaned.contains("Hello world"));
    }

    #[test]
    fn test_strip_html() {
        let input = "<p>Hello <b>world</b>!</p>";
        let stripped = strip_html(input);
        assert!(stripped.contains("Hello"));
        assert!(stripped.contains("world"));
        assert!(!stripped.contains("<"));
        assert!(!stripped.contains(">"));
    }
}
