//! Persistent message queue storage (SQLite WAL mode)

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub content: String,
    pub created_at: i64,
    pub sig: String,
    pub delivery_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub peer_id: String,
    pub last_message_id: String,
    pub last_message_at: i64,
    pub unread_count: i32,
}

pub struct MessageStore {
    conn: Connection,
}

impl MessageStore {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    pub fn init_tables(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE TABLE IF NOT EXISTS conversations (
                peer_id TEXT PRIMARY KEY,
                last_message_id TEXT,
                last_message_at INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                sig TEXT NOT NULL DEFAULT '',
                delivery_status TEXT NOT NULL DEFAULT 'pending',
                read_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_messages_peer ON messages(from_id, to_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_to ON messages(to_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(delivery_status);
            CREATE TABLE IF NOT EXISTS delivery_status (
                message_id TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'pending',
                attempts INTEGER NOT NULL DEFAULT 0,
                last_attempt INTEGER,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            );
            INSERT OR IGNORE INTO _migrations (version, description) VALUES (1, 'initial');
            "
        )?;
        Ok(())
    }

    pub fn send_dm(&mut self, from_id: &str, to_id: &str, content: &str, sig: &str) -> Result<String> {
        let id = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(from_id.as_bytes());
            hasher.update(to_id.as_bytes());
            hasher.update(content.as_bytes());
            hasher.update(&std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs().to_be_bytes());
            hex::encode(hasher.finalize())
        };
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

        self.conn.execute(
            "INSERT INTO messages (id, from_id, to_id, content, created_at, sig, delivery_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
            (&id, from_id, to_id, content, now, sig),
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO delivery_status (message_id, status) VALUES (?1, 'pending')",
            [&id],
        )?;

        // Update conversation
        let peer_id = to_id;
        self.conn.execute(
            "INSERT OR REPLACE INTO conversations (peer_id, last_message_id, last_message_at, unread_count)
             VALUES (?1, ?2, ?3, COALESCE((SELECT unread_count FROM conversations WHERE peer_id = ?1), 0))",
            (peer_id, &id, now),
        )?;

        Ok(id)
    }

    pub fn get_conversations(&self, user_id: &str, limit: usize) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT peer_id, COALESCE(last_message_id, ''), last_message_at, unread_count
             FROM conversations
             WHERE peer_id IN (
                SELECT DISTINCT from_id FROM messages WHERE to_id = ?1
                UNION
                SELECT DISTINCT to_id FROM messages WHERE from_id = ?1
             )
             ORDER BY last_message_at DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map((user_id, limit as i64), |row| Ok(Conversation {
            peer_id: row.get(0)?,
            last_message_id: row.get(1)?,
            last_message_at: row.get(2)?,
            unread_count: row.get(3)?,
        }))?;
        let mut convs = Vec::new();
        for row in rows { convs.push(row?); }
        Ok(convs)
    }

    pub fn get_messages(&self, peer_id: &str, user_id: &str, cursor: Option<i64>, limit: usize) -> Result<Vec<Message>> {
        let after_ts = cursor.unwrap_or(i64::MAX);
        let mut stmt = self.conn.prepare(
            "SELECT id, from_id, to_id, content, created_at, sig, delivery_status
             FROM messages
             WHERE ((from_id = ?1 AND to_id = ?2) OR (from_id = ?2 AND to_id = ?1))
               AND created_at < ?3
             ORDER BY created_at DESC LIMIT ?4"
        )?;
        let rows = stmt.query_map((peer_id, user_id, after_ts, limit as i64), |row| Ok(Message {
            id: row.get(0)?, from_id: row.get(1)?, to_id: row.get(2)?,
            content: row.get(3)?, created_at: row.get(4)?, sig: row.get(5)?,
            delivery_status: row.get(6)?,
        }))?;
        let mut msgs = Vec::new();
        for row in rows { msgs.push(row?); }
        Ok(msgs)
    }

    pub fn mark_read(&mut self, peer_id: &str, user_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        self.conn.execute(
            "UPDATE messages SET read_at = ?1 WHERE from_id = ?2 AND to_id = ?3 AND read_at IS NULL",
            (now, peer_id, user_id),
        )?;
        self.conn.execute(
            "UPDATE conversations SET unread_count = 0 WHERE peer_id = ?1",
            [peer_id],
        )?;
        Ok(())
    }

    pub fn receive_dm(&mut self, from_id: &str, to_id: &str, content: &str, sig: &str) -> Result<()> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        let id = hex::encode(sha2::Sha256::digest(format!("{}{}{}{}", from_id, to_id, content, now)));

        self.conn.execute(
            "INSERT INTO messages (id, from_id, to_id, content, created_at, sig, delivery_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'delivered')",
            (&id, from_id, to_id, content, now, sig),
        )?;

        // Update conversation unread count
        self.conn.execute(
            "INSERT INTO conversations (peer_id, last_message_id, last_message_at, unread_count)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(peer_id) DO UPDATE SET last_message_id = ?2, last_message_at = ?3, unread_count = unread_count + 1",
            (from_id, &id, now),
        )?;
        Ok(())
    }

    pub fn get_delivery_status(&self, message_id: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT status FROM delivery_status WHERE message_id = ?1",
            [message_id],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_queue_depth(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE delivery_status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}
