
//! SQLite storage for DHT records and routing table persistence

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtRecord {
    pub domain: String,
    pub public_key: String,
    pub ygg_address: String,
    pub online: bool,
    pub registered_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub ttl: i64,
    pub created_at: i64,
}

pub struct DhtStore {
    conn: Connection,
}

impl DhtStore {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn init_tables(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE TABLE IF NOT EXISTS domains (
                domain TEXT PRIMARY KEY,
                public_key TEXT NOT NULL,
                ygg_address TEXT NOT NULL,
                online INTEGER NOT NULL DEFAULT 1,
                signature TEXT NOT NULL,
                registered_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                expires_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_domains_public_key ON domains(public_key);
            CREATE INDEX IF NOT EXISTS idx_domains_expires ON domains(expires_at);
            CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                ttl INTEGER NOT NULL DEFAULT 86400,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_kv_created ON kv_store(created_at);
            CREATE TABLE IF NOT EXISTS routing_entries (
                node_id TEXT PRIMARY KEY,
                public_key TEXT NOT NULL,
                ygg_address TEXT NOT NULL,
                last_seen INTEGER NOT NULL,
                latency_ms INTEGER
            );
            INSERT OR IGNORE INTO _migrations (version, description) VALUES (1, 'initial');
            "
        )?;
        Ok(())
    }

    pub fn lookup(&mut self, domain: &str) -> Result<Option<DhtRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT domain, public_key, ygg_address, online, registered_at, expires_at
             FROM domains WHERE domain = ?1 AND expires_at > strftime('%s','now')"
        )?;
        let result = stmt.query_row([domain], |row| {
            Ok(DhtRecord {
                domain: row.get(0)?,
                public_key: row.get(1)?,
                ygg_address: row.get(2)?,
                online: row.get::<_, i32>(3)? == 1,
                registered_at: row.get(4)?,
                expires_at: row.get(5)?,
            })
        });
        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn register(&mut self, name: &str, public_key: &str, signature: &str, ygg_address: &str) -> Result<String> {
        let domain = format!("{}.sovrn", name);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let expires_at = now + 86400 * 365; // 1 year TTL

        self.conn.execute(
            "INSERT OR REPLACE INTO domains (domain, public_key, ygg_address, online, signature, registered_at, expires_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6)",
            (domain.clone(), public_key, ygg_address, signature, now, expires_at),
        )?;
        Ok(domain)
    }

    pub fn check_available(&mut self, name: &str) -> Result<bool> {
        let domain = format!("{}.sovrn", name);
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM domains WHERE domain = ?1",
            [&domain],
            |row| row.get(0),
        )?;
        Ok(count == 0)
    }

    pub fn suggest_alternatives(&mut self, name: &str, limit: usize) -> Result<Vec<String>> {
        let suffixes = ["123", "1", "_mesh", "_sovrn", "01"];
        let mut suggestions = Vec::new();
        for suffix in suffixes.iter().take(limit) {
            let alt = format!("{}{}", name, suffix);
            suggestions.push(alt);
        }
        Ok(suggestions)
    }

    pub fn put(&mut self, key: &str, value: &str, ttl: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, ttl) VALUES (?1, ?2, ?3)",
            (key, value, ttl),
        )?;
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
