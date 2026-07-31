use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

pub type DbState = Mutex<Db>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

impl serde::Serialize for DbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct Db {
    conn: Connection,
    /// Absolute path to `app.db` (handy for later modules / debugging).
    #[allow(dead_code)]
    path: PathBuf,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn, path };
        db.migrate()?;
        Ok(db)
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    #[allow(dead_code)]
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS shortcuts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('url', 'app')),
                target TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS news_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                summary TEXT,
                published_at TEXT,
                fetched_at TEXT NOT NULL,
                score REAL NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS news_prefs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_url TEXT NOT NULL UNIQUE,
                title TEXT,
                weight REAL NOT NULL DEFAULT 1.0,
                enabled INTEGER NOT NULL DEFAULT 1,
                muted INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'USD',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT
            );

            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
                category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                amount REAL NOT NULL,
                description TEXT,
                posted_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                external_id TEXT
            );

            CREATE TABLE IF NOT EXISTS news_topic_affinity (
                term TEXT PRIMARY KEY,
                score REAL NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS emails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uid INTEGER NOT NULL,
                mailbox TEXT NOT NULL,
                message_id TEXT,
                from_addr TEXT,
                from_name TEXT,
                to_addrs TEXT NOT NULL DEFAULT '',
                subject TEXT NOT NULL DEFAULT '',
                preview TEXT NOT NULL DEFAULT '',
                date_iso TEXT,
                is_unread INTEGER NOT NULL DEFAULT 1,
                is_important INTEGER NOT NULL DEFAULT 0,
                importance_score REAL NOT NULL DEFAULT 0,
                has_list_unsubscribe INTEGER NOT NULL DEFAULT 0,
                is_junk INTEGER NOT NULL DEFAULT 0,
                message_url TEXT,
                synced_at TEXT NOT NULL,
                UNIQUE(mailbox, uid)
            );

            CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_shortcuts_sort ON shortcuts(sort_order ASC, id ASC);
            CREATE INDEX IF NOT EXISTS idx_news_items_score ON news_items(score DESC, published_at DESC);
            CREATE INDEX IF NOT EXISTS idx_transactions_posted ON transactions(posted_at DESC);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_external
                ON transactions(account_id, external_id)
                WHERE external_id IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_emails_important
                ON emails(is_important DESC, importance_score DESC, date_iso DESC);
            CREATE INDEX IF NOT EXISTS idx_emails_unread ON emails(is_unread DESC, date_iso DESC);
            "#,
        )?;
        Self::ensure_column(
            &self.conn,
            "news_items",
            "liked",
            "ALTER TABLE news_items ADD COLUMN liked INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(
            &self.conn,
            "news_items",
            "hidden",
            "ALTER TABLE news_items ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0",
        )?;
        Ok(())
    }

    fn ensure_column(
        conn: &Connection,
        table: &str,
        column: &str,
        alter_sql: &str,
    ) -> Result<(), DbError> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let exists = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|name| name == column);
        if !exists {
            conn.execute(alter_sql, [])?;
        }
        Ok(())
    }
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, DbError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
