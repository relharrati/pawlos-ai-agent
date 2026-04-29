use rusqlite::{Connection, params};
use std::path::Path;
use anyhow::Result;
use crate::Config;

/// Wrapper around a SQLite connection with schema management
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open (and initialise) the SQLite database at ~/.pawlos/pawlos.db
    pub fn open() -> Result<Self> {
        let db_path = Config::base_dir().join("pawlos.db");
        std::fs::create_dir_all(db_path.parent().unwrap())?;
        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory DB (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // Sessions table
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                agent_name  TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                metadata    TEXT NOT NULL DEFAULT '{}'
            );",
        )?;

        // Messages table
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS messages (
                id             TEXT PRIMARY KEY,
                session_id     TEXT NOT NULL REFERENCES sessions(id),
                role           TEXT NOT NULL,
                content        TEXT NOT NULL,
                tool_calls     TEXT,
                tool_call_id   TEXT,
                timestamp      TEXT NOT NULL
            );",
        )?;

        // Task log — heartbeat / scheduled tasks
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS task_log (
                id          TEXT PRIMARY KEY,
                agent_name  TEXT NOT NULL,
                description TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'pending',
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                result      TEXT
            );",
        )?;

        // File index — tracks files the agent has worked on
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS file_index (
                path        TEXT PRIMARY KEY,
                language    TEXT,
                last_seen   TEXT NOT NULL,
                metadata    TEXT NOT NULL DEFAULT '{}'
            );",
        )?;

        // Skills registry
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS skills (
                name        TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                path        TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                used_count  INTEGER NOT NULL DEFAULT 0
            );",
        )?;

        Ok(())
    }

    /// Log a task to the task_log table
    pub fn log_task(&self, id: &str, agent: &str, description: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO task_log (id, agent_name, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?4)",
            params![id, agent, description, now],
        )?;
        Ok(())
    }

    /// Update task status
    pub fn update_task_status(&self, id: &str, status: &str, result: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE task_log SET status=?1, result=?2, updated_at=?3 WHERE id=?4",
            params![status, result, now, id],
        )?;
        Ok(())
    }

    /// Index a file path
    pub fn index_file(&self, path: &str, language: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO file_index (path, language, last_seen) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET language=excluded.language, last_seen=excluded.last_seen",
            params![path, language, now],
        )?;
        Ok(())
    }

    /// Register a skill
    pub fn register_skill(&self, name: &str, description: &str, path: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO skills (name, description, path, created_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(name) DO UPDATE SET description=excluded.description, path=excluded.path",
            params![name, description, path, now],
        )?;
        Ok(())
    }

    /// List all skills
    pub fn list_skills(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT name, description FROM skills ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }
}
