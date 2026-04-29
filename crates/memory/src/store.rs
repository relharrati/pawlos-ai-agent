use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use core::Config;

/// Which memory store to target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreKind {
    /// Agent's personality and identity - loaded every session
    Soul,
    /// User profile and preferences - loaded every session  
    User,
    /// Short-term working memory - 2200 char limit, ephemeral
    Memory,
    /// Long-term curated memory - unlimited, distilled learnings
    LongTerm,
    /// Daily log entries - raw notes per day YYYY-MM-DD.md
    DailyLog,
    /// Workspace instructions - per project/agent
    Agents,
    /// Tools and capabilities documentation
    Tools,
}

impl StoreKind {
    pub fn filename(&self) -> &'static str {
        match self {
            StoreKind::Soul      => "SOUL.md",
            StoreKind::User      => "USER.md",
            StoreKind::Memory    => "MEMORY.md",
            StoreKind::LongTerm  => "LONGTERM.md",
            StoreKind::DailyLog  => "*.md", // Pattern, not fixed
            StoreKind::Agents    => "AGENTS.md",
            StoreKind::Tools     => "TOOLS.md",
        }
    }

    /// Character limit for the store (None = unlimited)
    pub fn char_limit(&self) -> Option<usize> {
        match self {
            StoreKind::Memory => Some(2200),
            StoreKind::User   => Some(2200),
            StoreKind::Soul   => None,
            StoreKind::LongTerm => None,
            StoreKind::DailyLog => None,
            StoreKind::Agents => None,
            StoreKind::Tools  => None,
        }
    }

    /// Whether this store supports vector embeddings for semantic search
    pub fn supports_vector(&self) -> bool {
        matches!(self, StoreKind::LongTerm | StoreKind::DailyLog)
    }

    /// Whether this store is loaded in every session (core identity files)
    pub fn is_core(&self) -> bool {
        matches!(self, StoreKind::Soul | StoreKind::User | StoreKind::Memory)
    }

    /// Whether this store is loaded only in main session (not shared contexts)
    pub fn is_private(&self) -> bool {
        matches!(self, StoreKind::LongTerm | StoreKind::Memory)
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "soul"       => Some(StoreKind::Soul),
            "user"       => Some(StoreKind::User),
            "memory"     => Some(StoreKind::Memory),
            "longterm"   => Some(StoreKind::LongTerm),
            "dailylog"   => Some(StoreKind::DailyLog),
            "agents"     => Some(StoreKind::Agents),
            "tools"      => Some(StoreKind::Tools),
            _ => None,
        }
    }
}

/// A single memory entry (delimited by § in the file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub content: String,
    pub timestamp: Option<String>,
    pub vector_id: Option<String>, // For vector DB reference
}

/// Vector embedding entry for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub store: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub created_at: String,
}

/// Manages reading/writing a single memory store file
pub struct MemoryStore {
    pub kind: StoreKind,
    pub path: PathBuf,
}

impl MemoryStore {
    pub fn new(kind: StoreKind) -> Self {
        // Memory files go to context/ folder
        let base = Config::context_dir();
        let path = base.join(kind.filename());
        Self { kind, path }
    }

    pub fn from_path(kind: StoreKind, base_dir: &Path) -> Self {
        let path = base_dir.join(kind.filename());
        Self { kind, path }
    }

    /// For daily log: get today's log file path (in memory/ folder)
    pub fn daily_log_path() -> PathBuf {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Config::memory_dir().join(format!("{}.md", today))
    }

    /// For daily log: get path for specific date
    pub fn daily_log_path_for(date: &str) -> PathBuf {
        Config::memory_dir().join(format!("{}.md", date))
    }

    /// Read all entries from disk. Returns empty vec if file doesn't exist.
    pub fn read_entries(&self) -> Result<Vec<MemoryEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.path)
            .with_context(|| format!("Reading {:?}", self.path))?;
        
        // For unlimited stores, return whole file as single entry
        if self.kind.char_limit().is_none() {
            return Ok(vec![MemoryEntry { 
                content: content.trim().to_string(),
                timestamp: None,
                vector_id: None,
            }]);
        }
        
        // Split by § separator for limited stores
        let entries: Vec<MemoryEntry> = content
            .split('§')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| MemoryEntry { 
                content: s.to_string(),
                timestamp: None,
                vector_id: None,
            })
            .collect();
        Ok(entries)
    }

    /// Read raw file content
    pub fn read_raw(&self) -> Result<String> {
        if !self.path.exists() {
            return Ok(String::new());
        }
        Ok(std::fs::read_to_string(&self.path)?)
    }

    /// Write entries back to disk (§-separated)
    fn write_entries(&self, entries: &[MemoryEntry]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = if self.kind.char_limit().is_none() {
            entries.iter().map(|e| e.content.as_str()).collect::<Vec<_>>().join("\n")
        } else {
            entries
                .iter()
                .map(|e| e.content.as_str())
                .collect::<Vec<_>>()
                .join("\n§\n")
        };
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Total character count
    pub fn char_count(&self) -> Result<usize> {
        Ok(self.read_raw()?.len())
    }

    /// Usage percentage relative to char limit
    pub fn usage_pct(&self) -> Result<Option<f64>> {
        if let Some(limit) = self.kind.char_limit() {
            let count = self.char_count()?;
            Ok(Some((count as f64 / limit as f64) * 100.0))
        } else {
            Ok(None)
        }
    }

    /// Render memory block for prompt injection
    /// Format follows OpenClaw template structure
    pub fn render_for_prompt(&self) -> Result<String> {
        let raw = self.read_raw()?;
        if raw.is_empty() {
            return Ok(String::new());
        }
        let char_count = raw.len();
        
        let header_name = match &self.kind {
            StoreKind::Soul      => "SOUL (your identity)",
            StoreKind::User      => "USER PROFILE",
            StoreKind::Memory    => "MEMORY (short-term)",
            StoreKind::LongTerm  => "LONG-TERM MEMORY (curated)",
            StoreKind::DailyLog  => "DAILY LOG",
            StoreKind::Agents    => "AGENTS (workspace)",
            StoreKind::Tools     => "TOOLS",
        };
        
        let divider = "═".repeat(46);
        let usage_str = if let Some(limit) = self.kind.char_limit() {
            let pct = (char_count as f64 / limit as f64 * 100.0) as u32;
            format!("[{pct}% — {char_count}/{limit} chars]")
        } else {
            format!("[{char_count} chars]")
        };
        
        Ok(format!(
            "{divider}\n{header_name} {usage_str}\n{divider}\n{raw}\n"
        ))
    }

    /// Append a new entry
    pub fn add(&self, content: &str) -> Result<()> {
        let mut entries = self.read_entries()?;
        entries.push(MemoryEntry { 
            content: content.trim().to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            vector_id: None,
        });
        self.check_limit(&entries)?;
        self.write_entries(&entries)
    }

    /// Replace an existing entry
    pub fn replace(&self, old: &str, new: &str) -> Result<bool> {
        let mut entries = self.read_entries()?;
        let mut found = false;
        for entry in &mut entries {
            if entry.content.contains(old) {
                entry.content = entry.content.replace(old, new);
                found = true;
                break;
            }
        }
        if found {
            self.check_limit(&entries)?;
            self.write_entries(&entries)?;
        }
        Ok(found)
    }

    /// Remove entries matching a substring
    pub fn remove(&self, pattern: &str) -> Result<usize> {
        let mut entries = self.read_entries()?;
        let before = entries.len();
        entries.retain(|e| !e.content.contains(pattern));
        let removed = before - entries.len();
        if removed > 0 {
            self.write_entries(&entries)?;
        }
        Ok(removed)
    }

    /// Consolidate: trim and deduplicate
    pub fn consolidate(&self) -> Result<usize> {
        let mut entries = self.read_entries()?;
        let before = entries.len();
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| {
            let key = e.content.split_whitespace().collect::<Vec<_>>().join(" ");
            seen.insert(key)
        });
        let removed = before - entries.len();
        self.write_entries(&entries)?;
        Ok(removed)
    }

    fn check_limit(&self, entries: &[MemoryEntry]) -> Result<()> {
        if let Some(limit) = self.kind.char_limit() {
            let total: usize = entries.iter().map(|e| e.content.len()).sum();
            let total = total + entries.len().saturating_sub(1) * 3;
            if total > limit {
                anyhow::bail!(
                    "Memory store {:?} would exceed {limit} char limit ({total} chars)",
                    self.kind
                );
            }
        }
        Ok(())
    }

    /// Bootstrap default files following OpenClaw template structure
    pub fn bootstrap_defaults(agent_name: &str, user_name: &str) -> Result<()> {
        let skills_dir = Config::skills_dir();
        std::fs::create_dir_all(&skills_dir)?;
        std::fs::create_dir_all(&skills_dir.join("memory"))?;
        std::fs::create_dir_all(&skills_dir.join("skills"))?;

        // SOUL.md - OpenClaw template style
        let soul_path = skills_dir.join("SOUL.md");
        if !soul_path.exists() {
            std::fs::write(&soul_path, format!(
r#"# SOUL.md - Who You Are

You're {agent_name}, a pragmatic senior engineer with strong taste.
You optimize for truth, clarity, and usefulness over politeness theater.

## Style
- Direct but not cold
- Substance over filler  
- Push back on bad ideas
- Admit uncertainty plainly

## What to avoid
- Sycophancy, hype language, over-explaining obvious things

## Core Truths
- You persist across sessions via files, not a database
- You wake up fresh each session - read SOUL.md, USER.md, MEMORY.md, today's daily log
- If you change this file, tell the user - it's your soul

---
_Last updated: {date}_
"#,
                date = chrono::Local::now().format("%Y-%m-%d")
            ))?;
        }

        // USER.md - User profile
        let user_path = skills_dir.join("USER.md");
        if !user_path.exists() {
            std::fs::write(&user_path, format!(
r#"# USER.md - User Profile

**Name:** {user_name}

## Preferences
- (To be filled in by user)

## Communication Style
- (User's preferred communication style)

## Important Context
- (Things the user wants you to remember)

---
_Last updated: {date}_
"#,
                date = chrono::Local::now().format("%Y-%m-%d")
            ))?;
        }

        // MEMORY.md - Short-term working memory (2200 char limit)
        let memory_path = skills_dir.join("MEMORY.md");
        if !memory_path.exists() {
            std::fs::write(&memory_path, "# MEMORY.md\n\nShort-term working memory. Keep under 2200 chars.\n")?;
        }

        // LONGTERM.md - Long-term curated memory
        let longterm_path = skills_dir.join("LONGTERM.md");
        if !longterm_path.exists() {
            std::fs::write(&longterm_path, 
"# LONGTERM.md - Your Long-Term Memory

Curated learnings, significant events, distilled insights.
Load in main session only (direct chats with your human).
DO NOT load in shared contexts (Discord, group chats).

## Key Learnings
- (Add significant lessons here)

## Important Facts
- (Facts worth remembering long-term)

## Decisions Log
- (Important decisions made)

---
_Last updated: never_
")?;
        }

        // AGENTS.md - Workspace instructions
        let agents_path = skills_dir.join("AGENTS.md");
        if !agents_path.exists() {
            std::fs::write(&agents_path,
"# AGENTS.md - Your Workspace

This folder is your working directory - your memory.

## Core Files
- SOUL.md - Your identity and personality
- USER.md - User profile and preferences  
- MEMORY.md - Short-term working memory
- LONGTERM.md - Long-term curated memory
- memory/ - Daily logs (YYYY-MM-DD.md)
- skills/ - Reusable skill prompts

## Daily Routine
1. On wake: Read today's daily log, yesterday's if exists
2. Check LONGTERM.md for important context
3. Work, write notes to memory/YYYY-MM-DD.md
4. On heartbeat: Review daily logs, update LONGTERM.md

## Memory Rules
- If you want to remember something, WRITE IT TO A FILE
- "Mental notes" don't survive session restarts
- Text > Brain 📝

---
_Last updated: {date}
"#,
                date = chrono::Local::now().format("%Y-%m-%d")
            )?;
        }

        // TOOLS.md - Tools and capabilities
        let tools_path = skills_dir.join("TOOLS.md");
        if !tools_path.exists() {
            std::fs::write(&tools_path,
"# TOOLS.md - Your Capabilities

## Available Tools
- shell: Execute terminal commands
- file_read, file_write, file_edit: File operations
- memory: Manage memory stores (add, replace, remove, consolidate)
- web_search, web_fetch: Web browsing
- skill_load, skill_create: Load/create reusable skills
- agent_spawn, agent_delegate: Sub-agent management

## Tool Safety
- Destructive actions require user confirmation
- Sensitive paths like ~/.ssh are blocked by default

---
_Last updated: {date}
"#,
                date = chrono::Local::now().format("%Y-%m-%d")
            )?;
        }

        // Create today's daily log
        let today_log = Self::daily_log_path();
        if !today_log.exists() {
            std::fs::create_dir_all(today_log.parent().unwrap())?;
            std::fs::write(&today_log, format!(
"# {} - Daily Log

## Events
- (Log significant events here)

## Decisions
- (Note important decisions)

## Notes
- (General notes)

---
_Day started: {}_
",
                chrono::Local::now().format("%Y-%m-%d"),
                chrono::Local::now().format("%H:%M")
            ))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_store(kind: StoreKind) -> (MemoryStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = MemoryStore::from_path(kind, dir.path());
        (store, dir)
    }

    #[test]
    fn add_and_read_entries() {
        let (store, _dir) = test_store(StoreKind::Memory);
        store.add("first note").unwrap();
        store.add("second note").unwrap();
        let entries = store.read_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, "first note");
    }

    #[test]
    fn replace_entry() {
        let (store, _dir) = test_store(StoreKind::Memory);
        store.add("old value").unwrap();
        let found = store.replace("old value", "new value").unwrap();
        assert!(found);
        let entries = store.read_entries().unwrap();
        assert_eq!(entries[0].content, "new value");
    }

    #[test]
    fn remove_entry() {
        let (store, _dir) = test_store(StoreKind::Memory);
        store.add("keep this").unwrap();
        store.add("delete this").unwrap();
        let n = store.remove("delete this").unwrap();
        assert_eq!(n, 1);
        assert_eq!(store.read_entries().unwrap().len(), 1);
    }
}