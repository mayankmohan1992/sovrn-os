//! SQLite event storage (append-only log pattern)

use anyhow::Result;
use rusqlite::Connection;
use crate::event::{Event, ReactionSummary};
use std::path::Path;

pub struct FeedStore {
    conn: Connection,
}

impl FeedStore {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(Self { conn })
    }

    pub fn init_tables(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                kind INTEGER NOT NULL,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                sig TEXT NOT NULL,
                raw_json TEXT,
                inserted_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                CHECK(kind >= 0 AND kind <= 9999)
            );
            CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind);
            CREATE INDEX IF NOT EXISTS idx_events_author ON events(author);
            CREATE INDEX IF NOT EXISTS idx_events_created ON events(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_events_kind_author ON events(kind, author);
            CREATE TABLE IF NOT EXISTS follows (
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                removed_at INTEGER,
                PRIMARY KEY(source, target)
            );
            CREATE INDEX IF NOT EXISTS idx_follows_source ON follows(source);
            CREATE TABLE IF NOT EXISTS blocks (
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                removed_at INTEGER,
                PRIMARY KEY(source, target)
            );
            CREATE TABLE IF NOT EXISTS reactions (
                id TEXT PRIMARY KEY,
                target_event TEXT NOT NULL,
                kind INTEGER NOT NULL,
                author TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(target_event) REFERENCES events(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_reactions_target ON reactions(target_event);
            CREATE TABLE IF NOT EXISTS media_refs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                cid TEXT NOT NULL,
                media_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                width INTEGER,
                height INTEGER,
                alt_text TEXT,
                FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_media_event ON media_refs(event_id);
            CREATE TABLE IF NOT EXISTS timeline_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                event_id TEXT NOT NULL,
                rank REAL NOT NULL,
                seen INTEGER DEFAULT 0,
                FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_timeline_user ON timeline_cache(user_id, rank DESC);
            CREATE TABLE IF NOT EXISTS content_cache (
                cid TEXT PRIMARY KEY,
                local_path TEXT NOT NULL,
                fetched_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            INSERT OR IGNORE INTO _migrations (version, description) VALUES (1, 'initial');
            "
        )?;
        Ok(())
    }

    pub fn create_event(&mut self, event: &Event) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO events (id, kind, author, content, created_at, sig, raw_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (&event.id, event.kind, &event.author, &event.content, event.created_at, &event.sig, &event.raw_json),
        )?;
        Ok(())
    }

    pub fn get_event(&self, event_id: &str) -> Result<Option<Event>> {
        let result = self.conn.query_row(
            "SELECT id, kind, author, content, created_at, sig, raw_json FROM events WHERE id = ?1",
            [event_id],
            |row| Ok(Event {
                id: row.get(0)?,
                kind: row.get(1)?,
                author: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                sig: row.get(5)?,
                raw_json: row.get(6)?,
                tags: vec![],
            }),
        );
        match result {
            Ok(event) => Ok(Some(event)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_event(&mut self, event_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM events WHERE id = ?1", [event_id])?;
        Ok(())
    }

    pub fn get_timeline(&self, _user_id: &str, cursor: Option<&str>, limit: usize) -> Result<(Vec<Event>, Option<String>, bool)> {
        let mut events = Vec::new();
        let after_ts = cursor.and_then(|c| c.parse::<i64>().ok()).unwrap_or(i64::MAX);

        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.kind, e.author, e.content, e.created_at, e.sig, e.raw_json
             FROM events e
             WHERE e.created_at < ?1 AND e.kind IN (1, 2, 6)
             ORDER BY e.created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map((after_ts, limit + 1), |row| Ok(Event {
            id: row.get(0)?,
            kind: row.get(1)?,
            author: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            sig: row.get(5)?,
            raw_json: row.get(6)?,
            tags: vec![],
        }))?;

        for row in rows {
            events.push(row?);
        }

        let has_more = events.len() > limit;
        if has_more { events.truncate(limit); }
        let next_cursor = events.last().map(|e| e.created_at.to_string());
        Ok((events, next_cursor, has_more))
    }

    pub fn get_reactions(&self, event_id: &str) -> Result<ReactionSummary> {
        let likes: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM reactions WHERE target_event = ?1 AND kind = 7",
            [event_id],
            |row| row.get(0),
        ).unwrap_or(0);
        let boosts: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM reactions WHERE target_event = ?1 AND kind = 6",
            [event_id],
            |row| row.get(0),
        ).unwrap_or(0);
        Ok(ReactionSummary { like: likes, boost: boosts })
    }

    pub fn add_reaction(&mut self, id: &str, target_event: &str, kind: i32, author: &str) -> Result<()> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        self.conn.execute(
            "INSERT OR IGNORE INTO reactions (id, target_event, kind, author, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (id, target_event, kind, author, now),
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, kind: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        let pattern = format!("%{}%", query);
        let mut events = Vec::new();

        if let Some(k) = kind {
            let mut stmt = self.conn.prepare(
                "SELECT id, kind, author, content, created_at, sig, raw_json FROM events WHERE content LIKE ?1 AND kind = ?2 ORDER BY created_at DESC LIMIT ?3"
            )?;
            let rows = stmt.query_map((pattern.as_str(), k, limit as i64), Self::row_to_event)?;
            for row in rows { events.push(row?); }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, kind, author, content, created_at, sig, raw_json FROM events WHERE content LIKE ?1 ORDER BY created_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map((pattern.as_str(), limit as i64), Self::row_to_event)?;
            for row in rows { events.push(row?); }
        }

        Ok(events)
    }

    fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
        Ok(Event {
            id: row.get(0)?, kind: row.get(1)?, author: row.get(2)?, content: row.get(3)?,
            created_at: row.get(4)?, sig: row.get(5)?, raw_json: row.get(6)?, tags: vec![],
        })
    }
}
