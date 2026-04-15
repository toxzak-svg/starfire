//! Book — Hierarchical Knowledge Library
//!
//! A bookmark-driven knowledge system for Starfire. Books are persistent
//! knowledge libraries organized into chapters and sections of varying
//! information density. Sections are paged in on-demand based on the
//! active focus, never loading an entire book at once.
//!
//! # Metaphor
//!
//! - Bookmark string = URL
//! - Book = Website
//! - Library = Browser with multiple tabs
//! - Active focus = Current tab
//! - Thread stash = Tab bookmark (serialized state)
//!
//! # Example
//!
//! ```ignore
//! let mut library = Library::new(&db_path)?;
//!
//! // Write a section
//! library.write_section(
//!     "railway:env:secrets",
//!     "GROQ_API_KEY=gsk_xxx\nRAILWAY_ENV=production",
//!     Density::High
//! )?;
//!
//! // Create a focus and page in
//! let focus = library.create_focus("railway", Density::Medium)?;
//! let sections = library.page_in(&focus, 512)?;
//! ```
//!
//! # Density
//!
//! Sections declare their density to enable smart budgeting:
//! - `High`: Full detail — variable names, values, states
//! - `Medium`: Summary — key patterns, sanitized examples
//! - `Low`: Compressed — status codes, boolean flags
//! - `Packed`: Raw data dump — load only on explicit request

pub mod db;
pub mod pager;
pub mod sweep;
pub mod thread;

use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Library — the top-level container for all books
pub struct Library {
    conn: Mutex<db::Connection>,
    manifest: LibraryManifest,
}

impl Library {
    /// Open or create a library at the given path.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = db::Connection::open(path)?;
        let manifest = LibraryManifest::load(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            manifest,
        })
    }

    /// Create a new book.
    pub fn create_book(&mut self, name: &str, bookmark_prefix: &str) -> anyhow::Result<BookId> {
        let id = BookId::new();
        let book = Book {
            id: id.clone(),
            name: name.to_string(),
            bookmark_prefix: bookmark_prefix.to_string(),
            chapters: Vec::new(),
            created_at: crate::now_timestamp(),
            last_accessed: crate::now_timestamp(),
            access_count: 0,
        };

        let conn = self.conn.lock().unwrap();
        db::insert_book(&*conn, &book)?;
        self.manifest.books.insert(id.clone(), book_header(&book));

        Ok(id)
    }

    /// Get a book by its bookmark prefix.
    pub fn book_by_prefix(&self, prefix: &str) -> Option<BookId> {
        self.manifest
            .books
            .values()
            .find(|b| b.bookmark_prefix == prefix)
            .map(|b| b.id.clone())
    }

    /// Write a section. Creates book/chapter if they don't exist.
    pub fn write_section(
        &mut self,
        bookmark: &str,
        content: &str,
        density: Density,
    ) -> anyhow::Result<SectionId> {
        let parsed = Bookmark::parse(bookmark)?;

        // Ensure book exists
        let book_id = match self.book_by_prefix(&parsed.book) {
            Some(id) => id,
            None => self.create_book(&parsed.book, &parsed.book)?,
        };

        // Ensure chapter exists
        let chapter_id = {
            let conn = self.conn.lock().unwrap();
            match db::get_chapter_by_prefix(&*conn, book_id.as_str(), &parsed.chapter) {
                Some(chapter_id) => chapter_id,
                None => {
                    let id = ChapterId::new();
                    db::insert_chapter(
                        &*conn,
                        &Chapter {
                            id: id.clone(),
                            book_id: book_id.clone(),
                            name: parsed.chapter.clone(),
                            bookmark_prefix: parsed.chapter.clone(),
                            density,
                            sections: Vec::new(),
                        },
                    )?;
                    id
                }
            }
        };

        // Insert or update section
        let section_id = SectionId::new();
        let tokens = tokenizer_count(content);
        let conn = self.conn.lock().unwrap();
        db::insert_section(
            &*conn,
            &Section {
                id: section_id.clone(),
                chapter_id,
                bookmark: bookmark.to_string(),
                content: content.to_string(),
                tokens,
                density,
                last_accessed: crate::now_timestamp(),
                version: 1,
            },
        )?;

        // Update manifest
        if let Some(book) = self.manifest.books.get_mut(&book_id) {
            book.section_count += 1;
            book.total_tokens += tokens;
            match density {
                Density::High => { let k = "high"; book.density_breakdown.insert(k.to_string(), *book.density_breakdown.get(k).unwrap_or(&0usize) + tokens); }
                Density::Medium => { let k = "medium"; book.density_breakdown.insert(k.to_string(), *book.density_breakdown.get(k).unwrap_or(&0usize) + tokens); }
                Density::Low => { let k = "low"; book.density_breakdown.insert(k.to_string(), *book.density_breakdown.get(k).unwrap_or(&0usize) + tokens); }
                Density::Packed => { let k = "packed"; book.density_breakdown.insert(k.to_string(), *book.density_breakdown.get(k).unwrap_or(&0usize) + tokens); }
            };
        }

        Ok(section_id)
    }

    /// Get a section by its full bookmark string.
    pub fn get_section(&self, bookmark: &str) -> anyhow::Result<Option<Section>> {
        let conn = self.conn.lock().unwrap();
        Ok(db::get_section_by_bookmark(&*conn, bookmark)?)
    }

    /// Create a new focus (active tab) for a book.
    pub fn create_focus(&mut self, bookmark_prefix: &str, density_filter: Density) -> anyhow::Result<FocusId> {
        let book_id = self
            .book_by_prefix(bookmark_prefix)
            .ok_or_else(|| anyhow::anyhow!("Book not found: {}", bookmark_prefix))?;

        let id = FocusId::new();
        let id_clone = id.clone();
        let focus = Focus {
            id,
            book_id: book_id.clone(),
            pinned_sections: Vec::new(),
            active_chapters: Vec::new(),
            density_filter,
            thread_id: None,
            stashed_at: None,
            created_at: crate::now_timestamp(),
        };

        let conn = self.conn.lock().unwrap();
        db::insert_focus(&*conn, &focus)?;

        self.manifest.focus_stack.push(id_clone.clone());
        self.manifest.active_focus = Some(id_clone.clone());

        Ok(id_clone)
    }

    /// Pin a section to a focus (always-loaded).
    pub fn pin_section(&mut self, focus_id: &FocusId, section_id: &SectionId) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        db::insert_focus_pin(&*conn, focus_id.as_str(), section_id.as_str())?;
        Ok(())
    }

    /// Page in sections for a focus, respecting token budget and density filter.
    pub fn page_in(&self, focus_id: &FocusId, max_tokens: usize) -> anyhow::Result<Vec<Section>> {
        pager::page_in(&*self.conn.lock().unwrap(), &self.manifest, focus_id, max_tokens)
    }

    /// Stash the current focus to a thread string (tab bookmark).
    pub fn stash_focus(&mut self, focus_id: &FocusId) -> anyhow::Result<String> {
        let thread_str = thread::stash(&*self.conn.lock().unwrap(), &self.manifest, focus_id)?;

        // Update manifest: clear active focus, push to stack
        self.manifest.active_focus = None;

        Ok(thread_str)
    }

    /// Restore a focus from a thread string.
    pub fn restore_focus(&mut self, thread: &str) -> anyhow::Result<Option<FocusId>> {
        thread::restore(&*self.conn.lock().unwrap(), &mut self.manifest, thread)
    }

    /// Sweep across all books for a query using Quanot semantic matching.
    pub fn sweep(&self, query: &str, max_results: usize) -> anyhow::Result<Vec<sweep::SweepResult>> {
        sweep::sweep(&*self.conn.lock().unwrap(), &self.manifest, query, max_results)
    }

    /// Get the library manifest (all book headers, no content).
    pub fn manifest(&self) -> &LibraryManifest {
        &self.manifest
    }

    /// Get stats for a book.
    pub fn book_stats(&self, book_id: &BookId) -> anyhow::Result<BookStats> {
        let conn = self.conn.lock().unwrap();
        let header = self.manifest.books.get(book_id)
            .ok_or_else(|| anyhow::anyhow!("Book not found"))?;
        let sections = db::get_sections_by_book(&*conn, book_id.as_str())?;
        Ok(BookStats {
            header: header.clone(),
            chapters: sections.into_iter().map(|s| s.chapter_id).collect::<std::collections::HashSet<_>>().len(),
        })
    }
}

/// Lightweight manifest — all book metadata without content.
#[derive(Debug, Clone)]
pub struct LibraryManifest {
    pub books: HashMap<BookId, BookHeader>,
    pub active_focus: Option<FocusId>,
    pub focus_stack: Vec<FocusId>,
}

impl LibraryManifest {
    fn load(conn: &db::Connection) -> anyhow::Result<Self> {
        let books = db::get_all_books(conn)?
            .into_iter()
            .map(|b| (BookId(b.id.clone()), book_header_from_book(&b)))
            .collect();

        let active_focus = db::get_active_focus(conn)?.map(|f| FocusId(f.id.0));
        let focus_stack = db::get_focus_stack(conn)?
            .into_iter()
            .map(|f| FocusId(f.id.0))
            .collect();

        Ok(Self {
            books,
            active_focus,
            focus_stack,
        })
    }
}

fn book_header(b: &Book) -> BookHeader {
    BookHeader {
        id: b.id.clone(),
        name: b.name.clone(),
        bookmark_prefix: b.bookmark_prefix.clone(),
        chapter_count: 0,
        section_count: 0,
        total_tokens: 0,
        density_breakdown: HashMap::new(),
        last_accessed: b.last_accessed,
    }
}

fn book_header_from_book(b: &db::Book) -> BookHeader {
    BookHeader {
        id: BookId(b.id.clone()),
        name: b.name.clone(),
        bookmark_prefix: b.bookmark_prefix.clone(),
        chapter_count: b.chapter_count as usize,
        section_count: b.section_count as usize,
        total_tokens: b.total_tokens as usize,
        density_breakdown: serde_json::from_str(&b.density_breakdown).unwrap_or_default(),
        last_accessed: b.last_accessed,
    }
}

/// Metadata for a book (no content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookHeader {
    pub id: BookId,
    pub name: String,
    pub bookmark_prefix: String,
    pub chapter_count: usize,
    pub section_count: usize,
    pub total_tokens: usize,
    pub density_breakdown: HashMap<String, usize>,
    pub last_accessed: i64,
}

/// Full Book with chapters (loaded on demand).
#[derive(Debug, Clone)]
pub struct Book {
    pub id: BookId,
    pub name: String,
    pub bookmark_prefix: String,
    pub chapters: Vec<Chapter>,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
}

/// A chapter within a book.
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: ChapterId,
    pub book_id: BookId,
    pub name: String,
    pub bookmark_prefix: String,
    pub density: Density,
    pub sections: Vec<Section>,
}

/// A section — the atomic unit of information in a book.
#[derive(Debug, Clone)]
pub struct Section {
    pub id: SectionId,
    pub chapter_id: ChapterId,
    pub bookmark: String,
    pub content: String,
    pub tokens: usize,
    pub density: Density,
    pub last_accessed: i64,
    pub version: u32,
}

/// Information density of a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Density {
    /// Full detail — variable names, values, states
    High,
    /// Summary — key patterns, sanitized examples
    Medium,
    /// Compressed — status codes, boolean flags
    Low,
    /// Raw data dump — load only on explicit request
    Packed,
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Density::High => write!(f, "high"),
            Density::Medium => write!(f, "medium"),
            Density::Low => write!(f, "low"),
            Density::Packed => write!(f, "packed"),
        }
    }
}

impl From<&str> for Density {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "high" => Density::High,
            "medium" => Density::Medium,
            "low" => Density::Low,
            "packed" => Density::Packed,
            _ => Density::Medium,
        }
    }
}

/// An active focus (tab) within a book.
#[derive(Debug, Clone)]
pub struct Focus {
    pub id: FocusId,
    pub book_id: BookId,
    pub pinned_sections: Vec<SectionId>,
    pub active_chapters: Vec<ChapterId>,
    pub density_filter: Density,
    pub thread_id: Option<String>,
    pub stashed_at: Option<i64>,
    pub created_at: i64,
}

/// Book stats summary.
#[derive(Debug)]
pub struct BookStats {
    pub header: BookHeader,
    pub chapters: usize,
}

// ID types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookId(String);

impl BookId {
    fn new() -> Self {
        Self(format!("book_{}", random::<u64>()))
    }
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(String);

impl ChapterId {
    fn new() -> Self {
        Self(format!("chap_{}", random::<u64>()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SectionId(String);

impl SectionId {
    fn new() -> Self {
        Self(format!("sect_{}", random::<u64>()))
    }
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FocusId(String);

impl FocusId {
    fn new() -> Self {
        Self(format!("focus_{}", random::<u64>()))
    }
    fn as_str(&self) -> &str {
        &self.0
    }
}

/// Parsed bookmark string.
#[derive(Debug)]
struct Bookmark {
    book: String,
    chapter: String,
}

impl Bookmark {
    fn parse(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            anyhow::bail!("Bookmark must have at least 2 parts: book:chapter");
        }
        Ok(Self {
            book: parts[0].to_string(),
            chapter: parts[1].to_string(),
        })
    }
}

/// Rough tokenizer — splits on whitespace + punctuation.
/// Accurate enough for token budgeting.
fn tokenizer_count(text: &str) -> usize {
    let mut count = 0;
    let mut in_token = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if !in_token {
                count += 1;
                in_token = true;
            }
        } else {
            in_token = false;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_parse() {
        let b = Bookmark::parse("railway:env").unwrap();
        assert_eq!(b.book, "railway");
        assert_eq!(b.chapter, "env");

        let b2 = Bookmark::parse("medical:symptoms:fever:child").unwrap();
        assert_eq!(b2.book, "medical");
        assert_eq!(b2.chapter, "symptoms");
    }

    #[test]
    fn test_tokenizer_count() {
        assert_eq!(tokenizer_count("hello world"), 2);
        assert_eq!(tokenizer_count("key=value"), 2);
        assert_eq!(tokenizer_count("Hello, world! How are you?"), 5);
    }

    #[test]
    fn test_density_from_str() {
        assert_eq!(Density::from("high"), Density::High);
        assert_eq!(Density::from("HIGH"), Density::High);
        assert_eq!(Density::from("medium"), Density::Medium);
        assert_eq!(Density::from("low"), Density::Low);
        assert_eq!(Density::from("packed"), Density::Packed);
        assert_eq!(Density::from("unknown"), Density::Medium);
    }
}
