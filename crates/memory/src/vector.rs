use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use core::Config;

/// Vector embedding entry for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub store: String,       // Which memory store (longterm, dailylog, etc.)
    pub content: String,     // The text content
    pub embedding: Vec<f32>, // The embedding vector (simplified - in production use actual embeddings)
    pub created_at: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl VectorEntry {
    pub fn new(store: &str, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            store: store.to_string(),
            content,
            embedding: Vec::new(), // Placeholder - real impl would generate embeddings
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Vector store with dual storage: SQLite for production, JSON for portability
pub struct VectorStore {
    sql_path: PathBuf,
    json_path: PathBuf,
}

impl VectorStore {
    pub fn new() -> Self {
        let base = Config::skills_dir();
        Self {
            sql_path: base.join("vector.db"),
            json_path: base.join("vector_store.json"),
        }
    }

    /// Initialize SQLite table for vector storage
    pub fn init_sql(&self) -> Result<()> {
        if let Some(parent) = self.sql_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = rusqlite::Connection::open(&self.sql_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id TEXT PRIMARY KEY,
                store TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB,
                created_at TEXT NOT NULL,
                metadata TEXT DEFAULT '{}'
            )",
            [],
        )?;
        
        // Index for semantic search
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vectors_store ON vectors(store)",
            [],
        )?;
        
        Ok(())
    }

    /// Add a vector entry to both stores
    pub fn add(&self, entry: &VectorEntry) -> Result<()> {
        // SQL storage
        if let Err(e) = self.add_sql(entry) {
            tracing::warn!("SQL vector storage failed, using JSON: {}", e);
        }

        // JSON storage (backup/portable)
        self.add_json(entry)?;

        Ok(())
    }

    fn add_sql(&self, entry: &VectorEntry) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.sql_path)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO vectors (id, store, content, embedding, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                entry.id,
                entry.store,
                entry.content,
                rusqlite::types::Blob::from_bytes(
                    &bincode::serialize(&entry.embedding).unwrap_or_default()
                ),
                entry.created_at,
                serde_json::to_string(&entry.metadata).unwrap_or_default(),
            ],
        )?;
        
        Ok(())
    }

    fn add_json(&self, entry: &VectorEntry) -> Result<()> {
        let mut entries = self.load_json()?;
        entries.push(entry.clone());
        self.save_json(&entries)
    }

    /// Search vectors by content similarity (simplified - real impl would use cosine similarity)
    pub fn search(&self, store: &str, query: &str, limit: usize) -> Result<Vec<VectorEntry>> {
        // Try SQL first
        if let Ok(results) = self.search_sql(store, query, limit) {
            return Ok(results);
        }
        
        // Fall back to JSON
        let entries = self.load_json()?;
        let query_lower = query.to_lowercase();
        
        let mut results: Vec<_> = entries
            .into_iter()
            .filter(|e| e.store == store && e.content.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect();
        
        Ok(results)
    }

    fn search_sql(&self, store: &str, query: &str, limit: usize) -> Result<Vec<VectorEntry>> {
        let conn = rusqlite::Connection::open(&self.sql_path)?;
        
        let query_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, store, content, embedding, created_at, metadata 
             FROM vectors 
             WHERE store = ?1 AND content LIKE ?2
             LIMIT ?3"
        )?;
        
        let entries = stmt.query_map(
            rusqlite::params![store, query_pattern, limit as i64],
            |row| {
                let id: String = row.get(0)?;
                let store: String = row.get(1)?;
                let content: String = row.get(2)?;
                let _embedding: Option<Vec<u8>> = row.get(3)?;
                let created_at: String = row.get(4)?;
                let metadata_str: String = row.get(5)?;
                
                Ok(VectorEntry {
                    id,
                    store,
                    content,
                    embedding: Vec::new(), // Simplified
                    created_at,
                    metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
                })
            },
        )?;
        
        let mut results = Vec::new();
        for entry in entries {
            if let Ok(e) = entry {
                results.push(e);
            }
        }
        
        Ok(results)
    }

    /// Get all entries for a store
    pub fn get_all(&self, store: &str) -> Result<Vec<VectorEntry>> {
        if let Ok(results) = self.get_all_sql(store) {
            return Ok(results);
        }
        
        // JSON fallback
        let entries = self.load_json()?;
        Ok(entries.into_iter().filter(|e| e.store == store).collect())
    }

    fn get_all_sql(&self, store: &str) -> Result<Vec<VectorEntry>> {
        let conn = rusqlite::Connection::open(&self.sql_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, store, content, embedding, created_at, metadata 
             FROM vectors WHERE store = ?1"
        )?;
        
        let entries = stmt.query_map([store], |row| {
            Ok(VectorEntry {
                id: row.get(0)?,
                store: row.get(1)?,
                content: row.get(2)?,
                embedding: Vec::new(),
                created_at: row.get(4)?,
                metadata: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            })
        })?;
        
        let mut results = Vec::new();
        for entry in entries {
            if let Ok(e) = entry {
                results.push(e);
            }
        }
        
        Ok(results)
    }

    /// Delete entries older than specified days
    pub fn cleanup(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();

        let mut removed = 0;

        // SQL cleanup
        if let Ok(conn) = rusqlite::Connection::open(&self.sql_path) {
            let n = conn.execute(
                "DELETE FROM vectors WHERE created_at < ?1",
                [&cutoff_str],
            )?;
            removed += n;
        }

        // JSON cleanup
        let mut entries = self.load_json()?;
        let before = entries.len();
        entries.retain(|e| e.created_at >= cutoff_str);
        let json_removed = before - entries.len();
        removed += json_removed;
        
        if removed > 0 {
            self.save_json(&entries)?;
        }

        Ok(removed)
    }

    // JSON file operations
    fn load_json(&self) -> Result<Vec<VectorEntry>> {
        if !self.json_path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.json_path)?;
        let entries: Vec<VectorEntry> = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(entries)
    }

    fn save_json(&self, entries: &[VectorEntry]) -> Result<()> {
        if let Some(parent) = self.json_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(entries)?;
        std::fs::write(&self.json_path, json)?;
        Ok(())
    }

    /// Sync JSON to SQL (for migration/backup)
    pub fn sync_json_to_sql(&self) -> Result<usize> {
        let entries = self.load_json()?;
        let mut synced = 0;
        
        for entry in &entries {
            if self.add_sql(entry).is_ok() {
                synced += 1;
            }
        }
        
        Ok(synced)
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}