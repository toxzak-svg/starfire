//! Thread — stash and restore focus state as a serializable string

use crate::book::{FocusId, LibraryManifest, Density, Focus};
use super::db::Connection;
use super::db as db;
use rusqlite::OptionalExtension;

/// Stash a focus to a thread string.
///
/// Format: `thread:{book_id}:{density}:{pinned_sect1,sect2}:{timestamp}`
///
/// The thread string is a compact serialization that can be stored
/// anywhere (database, message, file) and used to restore the focus later.
pub fn stash(
    conn: &Connection,
    manifest: &LibraryManifest,
    focus_id: &FocusId,
) -> anyhow::Result<String> {
    // Get focus from DB
    let focus = get_focus(conn, focus_id)?;
    let Some(focus) = focus else {
        anyhow::bail!("Focus not found: {}", focus_id.0);
    };

    // Get pinned sections
    let pinned: Vec<String> = db::get_pinned_sections(conn, focus_id.as_str())?
        .into_iter()
        .map(|s| s.0)
        .collect();

    let pinned_str = if pinned.is_empty() {
        String::from("-")
    } else {
        pinned.join(",")
    };

    let thread = format!(
        "thread:{}:{}:{}:{}",
        focus.book_id.0,
        focus.density_filter,
        pinned_str,
        crate::now_timestamp()
    );

    // Mark focus as stashed in DB
    conn.raw().execute(
        "UPDATE focuses SET is_active = 0, thread_id = ?1, stashed_at = ?2 WHERE id = ?3",
        rusqlite::params![thread, crate::now_timestamp(), focus_id.0],
    )?;

    Ok(thread)
}

/// Restore a focus from a thread string.
pub fn restore(
    conn: &Connection,
    manifest: &mut LibraryManifest,
    thread: &str,
) -> anyhow::Result<Option<FocusId>> {
    // Parse thread string
    let parts: Vec<&str> = thread.split(':').collect();
    if parts.len() != 5 || parts[0] != "thread" {
        anyhow::bail!("Invalid thread string: {}", thread);
    }

    let book_id = parts[1];
    let _density = Density::from(parts[2]);
    let pinned_str = parts[3];
    let _timestamp: i64 = parts[4].parse().unwrap_or(0);

    // Deactivate any active focus
    conn.raw().execute("UPDATE focuses SET is_active = 0 WHERE is_active = 1", [])?;

    // Create new focus
    let focus_id = FocusId::new();
    conn.raw().execute(
        "INSERT INTO focuses (id, book_id, density_filter, is_active, created_at)
         VALUES (?1, ?2, ?3, 1, ?4)",
        rusqlite::params![
            focus_id.0,
            book_id,
            _density.to_string(),
            crate::now_timestamp()
        ],
    )?;

    // Restore pins if any
    if pinned_str != "-" {
        for sid in pinned_str.split(',') {
            if !sid.is_empty() {
                conn.raw().execute(
                    "INSERT OR IGNORE INTO focus_pins (focus_id, section_id) VALUES (?1, ?2)",
                    rusqlite::params![focus_id.0, sid],
                )?;
            }
        }
    }

    // Update manifest
    manifest.active_focus = Some(focus_id.clone());

    Ok(Some(focus_id))
}

fn get_focus(conn: &Connection, focus_id: &FocusId) -> anyhow::Result<Option<Focus>> {
    let mut stmt = conn.raw().prepare(
        "SELECT id, book_id, density_filter, thread_id, stashed_at, created_at FROM focuses WHERE id = ?1"
    )?;

    let result = stmt.query_row([focus_id.0.clone()], |row| {
        Ok(Focus {
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
