//! SQLite peer store for presence tracking

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub public_key: String,
    pub domain: String,
    pub status: String,
    pub last_heartbeat: i64,
    pub ygg_address: Option<String>,
    pub latency_ms: Option<i64>,
}

pub struct PeerStore {
    conn: Connection,
}

impl PeerStore {
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
            CREATE TABLE IF NOT EXISTS peer_status (
                public_key TEXT PRIMARY KEY,
                domain TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'offline',
                last_heartbeat INTEGER NOT NULL DEFAULT 0,
                ygg_address TEXT,
                latency_ms INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_status ON peer_status(status);
            CREATE TABLE IF NOT EXISTS heartbeat_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_key TEXT NOT NULL,
                status TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (public_key) REFERENCES peer_status(public_key)
            );
            CREATE INDEX IF NOT EXISTS idx_hb_ts ON heartbeat_log(timestamp);
            INSERT OR IGNORE INTO _migrations (version, description) VALUES (1, 'initial');
            "
        )?;
        Ok(())
    }

    pub fn update_peer(&mut self, public_key: &str, domain: &str, status: &str, timestamp: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO peer_status (public_key, domain, status, last_heartbeat) VALUES (?1, ?2, ?3, ?4)",
            (public_key, domain, status, timestamp),
        )?;
        self.conn.execute(
            "INSERT INTO heartbeat_log (public_key, status, timestamp) VALUES (?1, ?2, ?3)",
            (public_key, status, timestamp),
        )?;
        Ok(())
    }

    pub fn get_online_peers(&self, limit: usize) -> Result<Vec<PeerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT public_key, domain, status, last_heartbeat, ygg_address, latency_ms FROM peer_status WHERE status != 'offline' ORDER BY last_heartbeat DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok(PeerInfo {
                public_key: row.get(0)?,
                domain: row.get(1)?,
                status: row.get(2)?,
                last_heartbeat: row.get(3)?,
                ygg_address: row.get(4)?,
                latency_ms: row.get(5)?,
            })
        })?;
        let mut peers = Vec::new();
        for row in rows {
            peers.push(row?);
        }
        Ok(peers)
    }

    pub fn get_peer_status(&self, public_key: &str) -> Result<Option<PeerInfo>> {
        let result = self.conn.query_row(
            "SELECT public_key, domain, status, last_heartbeat, ygg_address, latency_ms FROM peer_status WHERE public_key = ?1",
            [public_key],
            |row| Ok(PeerInfo {
                public_key: row.get(0)?,
                domain: row.get(1)?,
                status: row.get(2)?,
                last_heartbeat: row.get(3)?,
                ygg_address: row.get(4)?,
                latency_ms: row.get(5)?,
            }),
        );
        match result {
            Ok(peer) => Ok(Some(peer)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_status(&mut self, public_key: &str, status: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        self.conn.execute(
            "UPDATE peer_status SET status = ?1, last_heartbeat = ?2 WHERE public_key = ?3",
            (status, now, public_key),
        )?;
        Ok(())
    }
}
