//! Pager — density-aware section loading with token budgeting

use crate::book::{Density, FocusId, LibraryManifest, Section, SectionId};
use super::db::Connection;
use rusqlite::OptionalExtension;

/// Page in sections for a focus, respecting token budget and density filter.
///
/// Strategy:
/// 1. Load pinned sections first (always included)
/// 2. Load sections by density priority: HIGH > MEDIUM > LOW > PACKED
/// 3. Budget remaining tokens after pinned
/// 4. Return loaded sections + whether truncation occurred
pub fn page_in(
    conn: &Connection,
    manifest: &LibraryManifest,
    focus_id: &FocusId,
    max_tokens: usize,
) -> anyhow::Result<Vec<Section>> {
    let focus = get_focus(conn, focus_id)?;
    let Some(focus) = focus else {
        return Ok(Vec::new());
    };

    // Get pinned sections
    let pinned_ids: Vec<SectionId> = super::db::get_pinned_sections(conn, focus_id.as_str())?
        .into_iter()
        .collect();

    let mut loaded: Vec<Section> = Vec::new();
    let mut tokens_used = 0;

    // Load pinned sections first
    for sid in &pinned_ids {
        if let Some(section) = get_section_by_id(conn, sid)? {
            loaded.push(section.clone());
            tokens_used += section.tokens;
        }
    }

    // Budget remaining
    let remaining = max_tokens.saturating_sub(tokens_used);

    // Load sections by density
    let sections = super::db::get_sections_by_density(
        conn,
        focus.book_id.0.as_str(),
        &focus.density_filter,
        remaining,
    )?;

    for section in sections {
        if tokens_used + section.tokens > max_tokens {
            break;
        }
        // Skip if already loaded as pinned
        if pinned_ids.contains(&section.id) {
            continue;
        }
        loaded.push(section.clone());
        tokens_used += section.tokens;
    }

    // Update last_accessed for loaded sections
    let now = crate::now_timestamp();
    for section in &loaded {
        update_section_access(conn, &section.id, now)?;
    }

    Ok(loaded)
}

fn get_focus(conn: &super::db::Connection, focus_id: &FocusId) -> anyhow::Result<Option<crate::book::Focus>> {
    let mut stmt = conn.raw().prepare(
        "SELECT id, book_id, density_filter, thread_id, stashed_at, created_at FROM focuses WHERE id = ?1"
    )?;

    let result = stmt.query_row([focus_id.0.clone()], |row| {
        Ok(crate::book::Focus {
            id: FocusId(row.get(0)?),
            book_id: crate::book::BookId(row.get(1)?),
            pinned_sections: Vec::new(),
            active_chapters: Vec::new(),
            density_filter: Density::from(row.get::<_, String>(2)?.as_str()),
            thread_id: row.get(3)?,
            stashed_at: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).optional()?;

    Ok(result)
}

fn get_section_by_id(conn: &super::db::Connection, section_id: &SectionId) -> anyhow::Result<Option<Section>> {
    let mut stmt = conn.raw().prepare(
        "SELECT id, chapter_id, bookmark, content, tokens, density, last_accessed, version FROM sections WHERE id = ?1"
    )?;

    let result = stmt.query_row([section_id.0.clone()], |row| {
        Ok(Section {
            id: SectionId(row.get(0)?),
            chapter_id: crate::book::ChapterId(row.get(1)?),
            bookmark: row.get(2)?,
            content: row.get(3)?,
            tokens: row.get::<_, i64>(4)? as usize,
            density: Density::from(row.get::<_, String>(5)?.as_str()),
            last_accessed: row.get(6)?,
            version: row.get::<_, i64>(7)? as u32,
        })
    }).optional()?;

    Ok(result)
}

fn update_section_access(conn: &super::db::Connection, section_id: &SectionId, now: i64) -> anyhow::Result<()> {
    conn.raw().execute(
        "UPDATE sections SET last_accessed = ?1 WHERE id = ?2",
        rusqlite::params![now, section_id.0],
    )?;
    Ok(())
}
