use anyhow::Result;
use std::path::Path;

pub struct FileTool;

impl FileTool {
    pub fn read_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "file_read",
            "description": "Read the contents of a file from the local filesystem.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute or relative file path" },
                    "offset": { "type": "integer", "description": "Line number to start from (1-indexed)" },
                    "limit":  { "type": "integer", "description": "Max lines to read" }
                },
                "required": ["path"]
            }
        })
    }

    pub fn write_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "file_write",
            "description": "Write content to a file (creates or overwrites).",
            "parameters": {
                "type": "object",
                "properties": {
                    "path":    { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }
        })
    }

    pub fn edit_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "file_edit",
            "description": "Replace a specific string in a file. Exact match required.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path":       { "type": "string" },
                    "old_string": { "type": "string", "description": "Exact text to replace" },
                    "new_string": { "type": "string", "description": "Replacement text" }
                },
                "required": ["path", "old_string", "new_string"]
            }
        })
    }

    pub async fn read(args: &serde_json::Value) -> Result<String> {
        let path = args["path"].as_str().unwrap_or("").to_string();
        let offset = args["offset"].as_u64().map(|n| n as usize).unwrap_or(1).saturating_sub(1);
        let limit  = args["limit"].as_u64().map(|n| n as usize).unwrap_or(usize::MAX);

        if !Path::new(&path).exists() {
            return Ok(format!("File not found: {path}"));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let slice = lines.iter().skip(offset).take(limit).cloned().collect::<Vec<_>>();
        Ok(slice.join("\n"))
    }

    pub async fn write(args: &serde_json::Value) -> Result<String> {
        let path    = args["path"].as_str().unwrap_or("").to_string();
        let content = args["content"].as_str().unwrap_or("").to_string();

        if let Some(parent) = Path::new(&path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, &content).await?;
        Ok(format!("Written {len} bytes to {path}", len = content.len()))
    }

    pub async fn edit(args: &serde_json::Value) -> Result<String> {
        let path       = args["path"].as_str().unwrap_or("").to_string();
        let old_string = args["old_string"].as_str().unwrap_or("");
        let new_string = args["new_string"].as_str().unwrap_or("");

        if !Path::new(&path).exists() {
            return Ok(format!("File not found: {path}"));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        if !content.contains(old_string) {
            return Ok(format!("old_string not found in {path}"));
        }
        let new_content = content.replacen(old_string, new_string, 1);
        tokio::fs::write(&path, new_content).await?;
        Ok(format!("Edit applied to {path}"))
    }
}
