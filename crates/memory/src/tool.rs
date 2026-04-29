use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::store::{MemoryStore, StoreKind};

/// Actions the LLM can invoke via the memory tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MemoryAction {
    Add     { store: String, content: String },
    Replace { store: String, old_entry: String, new_entry: String },
    Remove  { store: String, entry: String },
    Consolidate { store: String },
    Read    { store: String },
}

pub struct MemoryTool;

impl MemoryTool {
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "memory",
            "description": "Manage agent memory stores following OpenClaw template. \
                Core files: SOUL (identity), USER (profile), MEMORY (short-term), LONGTERM (curated), \
                DAILYLOG (daily notes in memory/YYYY-MM-DD.md). Use add/replace/remove/consolidate/read actions.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["add", "replace", "remove", "consolidate", "read"],
                        "description": "Action to perform on the memory store"
                    },
                    "store": {
                        "type": "string",
                        "enum": ["soul", "user", "memory", "longterm", "dailylog", "agents", "tools"],
                        "description": "Which memory store: soul (identity), user (profile), memory (short-term), \
                            longterm (curated learnings), dailylog (today's notes), agents (workspace), tools (capabilities)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to add (for add action)"
                    },
                    "old_entry": {
                        "type": "string",
                        "description": "Text pattern to find and replace (for replace action)"
                    },
                    "new_entry": {
                        "type": "string",
                        "description": "Replacement text (for replace action)"
                    },
                    "entry": {
                        "type": "string",
                        "description": "Pattern to match and remove (for remove action)"
                    }
                },
                "required": ["action", "store"]
            }
        })
    }

    pub async fn execute(action: MemoryAction) -> Result<String> {
        match action {
            MemoryAction::Add { store, content } => {
                let kind = parse_kind(&store)?;
                let ms = MemoryStore::new(kind);
                ms.add(&content)?;
                Ok(format!("Added to {store}: \"{content}\""))
            }
            MemoryAction::Replace { store, old_entry, new_entry } => {
                let kind = parse_kind(&store)?;
                let ms = MemoryStore::new(kind);
                let found = ms.replace(&old_entry, &new_entry)?;
                if found {
                    Ok(format!("Replaced in {store}."))
                } else {
                    Ok(format!("Pattern not found in {store}."))
                }
            }
            MemoryAction::Remove { store, entry } => {
                let kind = parse_kind(&store)?;
                let ms = MemoryStore::new(kind);
                let n = ms.remove(&entry)?;
                Ok(format!("Removed {n} entries from {store}."))
            }
            MemoryAction::Consolidate { store } => {
                let kind = parse_kind(&store)?;
                let ms = MemoryStore::new(kind);
                let n = ms.consolidate()?;
                Ok(format!("Consolidated {store}: removed {n} duplicate/empty entries."))
            }
            MemoryAction::Read { store } => {
                let kind = parse_kind(&store)?;
                let ms = MemoryStore::new(kind);
                let rendered = ms.render_for_prompt()?;
                if rendered.is_empty() {
                    Ok(format!("{store} is empty."))
                } else {
                    Ok(rendered)
                }
            }
        }
    }

    /// Parse a MemoryAction from a raw JSON tool call argument
    pub fn parse_action(args: &serde_json::Value) -> Result<MemoryAction> {
        let action = args["action"].as_str().unwrap_or("").to_string();
        let store  = args["store"].as_str().unwrap_or("memory").to_string();
        match action.as_str() {
            "add"  => Ok(MemoryAction::Add {
                store,
                content: args["content"].as_str().unwrap_or("").to_string(),
            }),
            "replace" => Ok(MemoryAction::Replace {
                store,
                old_entry: args["old_entry"].as_str().unwrap_or("").to_string(),
                new_entry: args["new_entry"].as_str().unwrap_or("").to_string(),
            }),
            "remove" => Ok(MemoryAction::Remove {
                store,
                entry: args["entry"].as_str().unwrap_or("").to_string(),
            }),
            "consolidate" => Ok(MemoryAction::Consolidate { store }),
            "read"        => Ok(MemoryAction::Read { store }),
            other => anyhow::bail!("Unknown memory action: {other}"),
        }
    }
}

fn parse_kind(s: &str) -> Result<StoreKind> {
    StoreKind::from_str(s)
        .ok_or_else(|| anyhow::anyhow!("Unknown memory store: {s}"))
}
