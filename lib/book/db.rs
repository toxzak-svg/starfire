//! Book database layer — SQLite persistence

use crate::book::{Chapter, ChapterId, Density, Focus, FocusId, Section, SectionId, BookId};
use rusqlite::{params, Connection as SqliteConn, OptionalExtension};
use std::path::Path;

/// Database connection wrapper with schema management.
pub struct Connection {
    inner: SqliteConn,
}

impl Connection {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let inner = SqliteConn::open(path)?;

        inner.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 30000;
             PRAGMA locking_mode = NORMAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -64000;"
        )?;

        Self::init_schema(&inner)?;
        Ok(Self { inner })
    }

    fn init_schema(conn: &SqliteConn) -> anyhow::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS books (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                bookmark_prefix TEXT UNIQUE NOT NULL,
                chapter_count INTEGER DEFAULT 0,
                section_count INTEGER DEFAULT 0,
                total_tokens INTEGER DEFAULT 0,
                density_breakdown TEXT DEFAULT '{}',
                created_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                access_count INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS chapters (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL REFERENCES books(id),
                name TEXT NOT NULL,
                bookmark_prefix TEXT NOT NULL,
                density TEXT NOT NULL,
                UNIQUE(book_id, bookmark_prefix)
            );

            CREATE TABLE IF NOT EXISTS sections (
                id TEXT PRIMARY KEY,
                chapter_id TEXT NOT NULL REFERENCES chapters(id),
                bookmark TEXT UNIQUE NOT NULL,
                content TEXT NOT NULL,
                tokens INTEGER NOT NULL,
                density TEXT NOT NULL,
                last_accessed INTEGER NOT NULL,
                version INTEGER DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS focuses (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL REFERENCES books(id),
                density_filter TEXT NOT NULL,
                thread_id TEXT,
                stashed_at INTEGER,
                is_active INTEGER DEFAULT 1,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS focus_pins (
                focus_id TEXT NOT NULL REFERENCES focuses(id),
                section_id TEXT NOT NULL REFERENCES sections(id),
                PRIMARY KEY (focus_id, section_id)
            );

            CREATE INDEX IF NOT EXISTS idx_chapters_book ON chapters(book_id);
            CREATE INDEX IF NOT EXISTS idx_sections_chapter ON sections(chapter_id);
            CREATE INDEX IF NOT EXISTS idx_sections_bookmark ON sections(bookmark);
            CREATE INDEX IF NOT EXISTS idx_focuses_book ON focuses(book_id);
            CREATE INDEX IF NOT EXISTS idx_focuses_active ON focuses(is_active);"
        )?;
        Ok(())
    }

    pub fn raw(&self) -> &SqliteConn {
        &self.inner
    }
}

pub fn insert_book(conn: &Connection, book: &crate::book::Book) -> anyhow::Result<()> {
    conn.raw().execute(
        "INSERT OR IGNORE INTO books (id, name, bookmark_prefix, chapter_count, section_count, total_tokens, density_breakdown, created_at, last_accessed, access_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            book.id.0,
            book.name,
            book.bookmark_prefix,
            0i64,
            0i64,
            0i64,
            "{}",
            book.created_at,
            book.last_accessed,
            0i64
        ],
    )?;
    Ok(())
}

pub fn insert_chapter(conn: &Connection, chapter: &Chapter) -> anyhow::Result<()> {
    conn.raw().execute(
        "INSERT OR IGNORE INTO chapters (id, book_id, name, bookmark_prefix, density)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            chapter.id.0,
            chapter.book_id.0,
            chapter.name,
            chapter.bookmark_prefix,
            chapter.density.to_string()
        ],
    )?;

    conn.raw().execute(
        "UPDATE books SET chapter_count = chapter_count + 1 WHERE id = ?1",
        params![chapter.book_id.0],
    )?;

    Ok(())
}

pub fn insert_section(conn: &Connection, section: &Section) -> anyhow::Result<()> {
    conn.raw().execute(
        "INSERT OR REPLACE INTO sections (id, chapter_id, bookmark, content, tokens, density, last_accessed, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            section.id.0,
            section.chapter_id.0,
            section.bookmark,
            section.content,
            section.tokens as i64,
            section.density.to_string(),
            section.last_accessed,
            section.version
        ],
    )?;

    conn.raw().execute(
        "UPDATE books SET section_count = section_count + 1, total_tokens = total_tokens + ?1 WHERE id = (
            SELECT book_id FROM chapters WHERE id = ?2
        )",
        params![section.tokens as i64, section.chapter_id.0],
    )?;

    Ok(())
}

pub fn get_section_by_bookmark(conn: &Connection, bookmark: &str) -> anyhow::Result<Option<Section>> {
    let result = conn.raw().query_row(
        "SELECT s.id, s.chapter_id, s.bookmark, s.content, s.tokens, s.density, s.last_accessed, s.version
         FROM sections s WHERE s.bookmark = ?1",
        params![bookmark],
        |row| {
            Ok(Section {
                id: SectionId(row.get(0)?),
                chapter_id: ChapterId(row.get(1)?),
                bookmark: row.get(2)?,
                content: row.get(3)?,
                tokens: row.get::<_, i64>(4)? as usize,
                density: Density::from(row.get::<_, String>(5)?.as_str()),
                last_accessed: row.get(6)?,
                version: row.get::<_, i64>(7)? as u32,
            })
        },
    ).optional()?;

    Ok(result)
}

pub fn get_chapter_by_prefix(conn: &Connection, book_id: &str, prefix: &str) -> Option<ChapterId> {
    conn.raw().query_row(
        "SELECT id FROM chapters WHERE book_id = ?1 AND bookmark_prefix = ?2",
        params![book_id, prefix],
        |row| Ok(ChapterId(row.get(0)?)),
    ).optional().ok().flatten()
}

pub fn get_sections_by_book(conn: &Connection, book_id: &str) -> anyhow::Result<Vec<Section>> {
    let mut stmt = conn.raw().prepare(
        "SELECT s.id, s.chapter_id, s.bookmark, s.content, s.tokens, s.density, s.last_accessed, s.version
         FROM sections s
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.book_id = ?1"
    )?;

    let sections = stmt.query_map(params![book_id], |row| {
        Ok(Section {
            id: SectionId(row.get(0)?),
            chapter_id: ChapterId(row.get(1)?),
            bookmark: row.get(2)?,
            content: row.get(3)?,
            tokens: row.get::<_, i64>(4)? as usize,
            density: Density::from(row.get::<_, String>(5)?.as_str()),
            last_accessed: row.get(6)?,
            version: row.get::<_, i64>(7)? as u32,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(sections)
}

pub fn get_all_books(conn: &Connection) -> anyhow::Result<Vec<Book>> {
    let mut stmt = conn.raw().prepare(
        "SELECT id, name, bookmark_prefix, chapter_count, section_count, total_tokens, density_breakdown, created_at, last_accessed, access_count FROM books"
    )?;

    let books = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get(0)?,
            name: row.get(1)?,
            bookmark_prefix: row.get(2)?,
            chapter_count: row.get::<_, i64>(3)? as u32,
            section_count: row.get::<_, i64>(4)? as u32,
            total_tokens: row.get::<_, i64>(5)? as u64,
            density_breakdown: row.get(6)?,
            created_at: row.get(7)?,
            last_accessed: row.get(8)?,
            access_count: row.get::<_, i64>(9)? as u32,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(books)
}

pub fn insert_focus(conn: &Connection, focus: &Focus) -> anyhow::Result<()> {
    conn.raw().execute("UPDATE focuses SET is_active = 0 WHERE is_active = 1", [])?;

    conn.raw().execute(
        "INSERT INTO focuses (id, book_id, density_filter, is_active, created_at)
         VALUES (?1, ?2, ?3, 1, ?4)",
        params![
            focus.id.0,
            focus.book_id.0,
            focus.density_filter.to_string(),
            focus.created_at
        ],
    )?;

    Ok(())
}

pub fn insert_focus_pin(conn: &Connection, focus_id: &str, section_id: &str) -> anyhow::Result<()> {
    conn.raw().execute(
        "INSERT OR IGNORE INTO focus_pins (focus_id, section_id) VALUES (?1, ?2)",
        params![focus_id, section_id],
    )?;
    Ok(())
}

pub fn get_active_focus(conn: &Connection) -> anyhow::Result<Option<Focus>> {
    let result = conn.raw().query_row(
        "SELECT id, book_id, density_filter, thread_id, stashed_at, created_at FROM focuses WHERE is_active = 1",
        [],
        |row| {
            Ok(Focus {
                id: FocusId(row.get(0)?),
                book_id: BookId(row.get(1)?),
                pinned_sections: Vec::new(),
                active_chapters: Vec::new(),
                density_filter: Density::from(row.get::<_, String>(2)?.as_str()),
                thread_id: row.get(3)?,
                stashed_at: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    ).optional()?;

    Ok(result)
}

pub fn get_focus_stack(conn: &Connection) -> anyhow::Result<Vec<Focus>> {
    let mut stmt = conn.raw().prepare(
        "SELECT id, book_id, density_filter, thread_id, stashed_at, created_at
         FROM focuses WHERE is_active = 0 ORDER BY stashed_at DESC"
    )?;

    let focuses = stmt.query_map([], |row| {
        Ok(Focus {
            id: FocusId(row.get(0)?),
            book_id: BookId(row.get(1)?),
            pinned_sections: Vec::new(),
            active_chapters: Vec::new(),
            density_filter: Density::from(row.get::<_, String>(2)?.as_str()),
            thread_id: row.get(3)?,
            stashed_at: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(focuses)
}

pub fn get_pinned_sections(conn: &Connection, focus_id: &str) -> anyhow::Result<Vec<SectionId>> {
    let mut stmt = conn.raw().prepare(
        "SELECT section_id FROM focus_pins WHERE focus_id = ?1"
    )?;

    let ids = stmt.query_map(params![focus_id], |row| {
        Ok(SectionId(row.get(0)?))
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(ids)
}

pub fn get_sections_by_density(
    conn: &Connection,
    book_id: &str,
    _density_filter: &Density,
    max_tokens: usize,
) -> anyhow::Result<Vec<Section>> {
    let mut stmt = conn.raw().prepare(
        "SELECT s.id, s.chapter_id, s.bookmark, s.content, s.tokens, s.density, s.last_accessed, s.version
         FROM sections s
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.book_id = ?1
         ORDER BY
            CASE s.density
                WHEN 'high' THEN 0
                WHEN 'medium' THEN 1
                WHEN 'low' THEN 2
                WHEN 'packed' THEN 3
            END,
            s.last_accessed DESC"
    )?;

    let mut sections = Vec::new();
    let mut tokens_used = 0usize;

    let rows = stmt.query_map(params![book_id], |row| {
        Ok(Section {
            id: SectionId(row.get(0)?),
            chapter_id: ChapterId(row.get(1)?),
            bookmark: row.get(2)?,
            content: row.get(3)?,
            tokens: row.get::<_, i64>(4)? as usize,
            density: Density::from(row.get::<_, String>(5)?.as_str()),
            last_accessed: row.get(6)?,
            version: row.get::<_, i64>(7)? as u32,
        })
    })?
    .filter_map(|r| r.ok());

    for section in rows {
        if tokens_used + section.tokens > max_tokens {
            break;
        }
        tokens_used += section.tokens;
        sections.push(section);
    }

    Ok(sections)
}

// Minimal Book struct for db.rs queries (avoids circular deps)
#[derive(Debug)]
pub struct Book {
    pub id: String,
    pub name: String,
    pub bookmark_prefix: String,
    pub chapter_count: u32,
    pub section_count: u32,
    pub total_tokens: u64,
    pub density_breakdown: String,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
}
