use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use core::Config;

/// A skill stored as a YAML file in ~/.pawlos/skills/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
    #[serde(default)]
    pub tool_requirements: Vec<String>,
}

impl Skill {
    pub fn path(name: &str) -> PathBuf {
        Config::skills_dir().join(format!("{name}.skill"))
    }

    pub fn load(name: &str) -> Result<Self> {
        let path = Self::path(name);
        if !path.exists() {
            anyhow::bail!("Skill '{}' not found at {:?}", name, path);
        }
        let content = std::fs::read_to_string(&path)?;
        let skill: Skill = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse skill '{name}': {e}"))?;
        Ok(skill)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Config::skills_dir();
        std::fs::create_dir_all(&dir)?;
        let path = Self::path(&self.name);
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| anyhow::anyhow!("Skill serialization error: {e}"))?;
        std::fs::write(&path, yaml)?;
        Ok(())
    }

    pub fn list_all() -> Result<Vec<String>> {
        let dir = Config::skills_dir();
        if !dir.exists() { return Ok(Vec::new()); }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".skill") {
                names.push(name.trim_end_matches(".skill").to_string());
            }
        }
        names.sort();
        Ok(names)
    }
}

pub fn skill_load_definition() -> serde_json::Value {
    serde_json::json!({
        "name": "skill_load",
        "description": "Load a named skill and inject its prompt into context.",
        "parameters": {
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Skill name (without .skill extension)" }
            },
            "required": ["name"]
        }
    })
}

pub fn skill_create_definition() -> serde_json::Value {
    serde_json::json!({
        "name": "skill_create",
        "description": "Create and save a new reusable skill.",
        "parameters": {
            "type": "object",
            "properties": {
                "name":        { "type": "string" },
                "description": { "type": "string" },
                "prompt":      { "type": "string" },
                "tool_requirements": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["name", "description", "prompt"]
        }
    })
}
