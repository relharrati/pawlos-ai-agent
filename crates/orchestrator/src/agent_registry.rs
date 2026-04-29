use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use core::{Config, types::AgentConfig};

/// Registry of all sub-agent configs
pub struct AgentRegistry {
    agents: HashMap<String, AgentConfig>,
}

impl AgentRegistry {
    pub fn load() -> Result<Self> {
        let agents_dir = Config::agents_dir();
        let mut agents = HashMap::new();

        if !agents_dir.exists() {
            std::fs::create_dir_all(&agents_dir)?;
            // Write a default agent config
            let default = AgentConfig {
                name: "default".into(),
                description: Some("Main pawlos agent".into()),
                personality_overlay: None,
                allowed_tools: None,
                model: None,
                parent: None,
            };
            let yaml = serde_yaml::to_string(&default)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            std::fs::write(agents_dir.join("default.yaml"), yaml)?;
            agents.insert("default".into(), default);
            return Ok(Self { agents });
        }

        for entry in std::fs::read_dir(&agents_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                let cfg: AgentConfig = serde_yaml::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Error parsing agent {:?}: {e}", path))?;
                agents.insert(cfg.name.clone(), cfg);
            }
        }

        Ok(Self { agents })
    }

    pub fn get(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    pub fn create_agent(&mut self, cfg: AgentConfig) -> Result<()> {
        let path = Config::agents_dir().join(format!("{}.yaml", cfg.name));
        let yaml = serde_yaml::to_string(&cfg)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, yaml)?;
        self.agents.insert(cfg.name.clone(), cfg);
        Ok(())
    }
}
