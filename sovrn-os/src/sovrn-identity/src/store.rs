//! SQLite key storage for identity service

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct KeyStore {
    conn: Connection,
}

impl KeyStore {
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
            CREATE TABLE IF NOT EXISTS keys (
                id TEXT PRIMARY KEY,
                signing_key_encrypted TEXT NOT NULL,
                verifying_key TEXT NOT NULL,
                x25519_public TEXT NOT NULL,
                pubkey_hash TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_keys_hash ON keys(pubkey_hash);
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT PRIMARY KEY REFERENCES keys(id),
                name TEXT,
                about TEXT,
                avatar_cid TEXT,
                domain TEXT,
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE TABLE IF NOT EXISTS aliases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id TEXT NOT NULL REFERENCES keys(id),
                name TEXT NOT NULL,
                domain TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_aliases_domain ON aliases(domain);
            CREATE TABLE IF NOT EXISTS delegated_apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id TEXT NOT NULL REFERENCES keys(id),
                app_id TEXT NOT NULL,
                username TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE INDEX IF NOT EXISTS idx_apps_key ON delegated_apps(key_id);
            INSERT OR IGNORE INTO _migrations (version, description) VALUES (1, 'initial');
            "
        )?;
        Ok(())
    }

    pub fn store_key(&self, id: &str, signing_key_enc: &str, verifying_key: &str, x25519_public: &str, pubkey_hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO keys (id, signing_key_encrypted, verifying_key, x25519_public, pubkey_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
            (id, signing_key_enc, verifying_key, x25519_public, pubkey_hash),
        )?;
        Ok(())
    }

    pub fn get_key_by_hash(&self, pubkey_hash: &str) -> Result<Option<(String, String, String, String)>> {
        let result = self.conn.query_row(
            "SELECT id, verifying_key, x25519_public, pubkey_hash FROM keys WHERE pubkey_hash = ?1",
            [pubkey_hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn store_profile(&self, id: &str, name: Option<&str>, about: Option<&str>, domain: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO profiles (id, name, about, domain) VALUES (?1, ?2, ?3, ?4)",
            (id, name, about, domain),
        )?;
        Ok(())
    }

    pub fn get_profile(&self, pubkey_hash: &str) -> Result<Option<(String, Option<String>, Option<String>, Option<String>, Option<String>)>> {
        let result = self.conn.query_row(
            "SELECT p.id, p.name, p.about, p.domain, p.avatar_cid FROM profiles p JOIN keys k ON p.id = k.id WHERE k.pubkey_hash = ?1",
            [pubkey_hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn add_alias(&self, key_id: &str, name: &str, domain: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO aliases (key_id, name, domain) VALUES (?1, ?2, ?3)",
            (key_id, name, domain),
        )?;
        Ok(())
    }

    pub fn list_aliases(&self, key_id: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, domain FROM aliases WHERE key_id = ?1"
        )?;
        let rows = stmt.query_map([key_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut aliases = Vec::new();
        for row in rows {
            aliases.push(row?);
        }
        Ok(aliases)
    }

    pub fn delete_alias(&self, domain: &str) -> Result<()> {
        self.conn.execute("DELETE FROM aliases WHERE domain = ?1", [domain])?;
        Ok(())
    }

    pub fn store_app_delegation(&self, key_id: &str, app_id: &str, username: &str, password_hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO delegated_apps (key_id, app_id, username, password_hash) VALUES (?1, ?2, ?3, ?4)",
            (key_id, app_id, username, password_hash),
        )?;
        Ok(())
    }

    pub fn get_app_delegation(&self, key_id: &str, app_id: &str) -> Result<Option<(String, String)>> {
        let result = self.conn.query_row(
            "SELECT username, password_hash FROM delegated_apps WHERE key_id = ?1 AND app_id = ?2",
            [key_id, app_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
