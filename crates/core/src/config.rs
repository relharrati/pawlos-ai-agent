use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use dirs::home_dir;

/// Top-level config loaded from ~/.pawlos/config.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub agent: AgentSection,
    pub user: UserSection,
    pub models: ModelsSection,
    pub messaging: MessagingSection,
    pub server: ServerSection,
    pub heartbeat: HeartbeatSection,
    pub mcp_servers: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSection {
    pub name: String,
    pub personalities: HashMap<String, String>,
}

impl Default for AgentSection {
    fn default() -> Self {
        Self {
            name: "pawlos".into(),
            personalities: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSection {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsSection {
    pub default: String,
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for ModelsSection {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Add pawlos local provider (default - runs local Ollama models)
        providers.insert("pawlos".to_string(), ProviderConfig {
            api_key: None,
            base_url: Some("http://localhost:11434/v1".to_string()),
            default_model: Some("qwen2.5:7b".to_string()),
            models: Some(vec![
                // High-end (8B+)
                "llama3.1:70b".to_string(),
                "qwen2.5:14b".to_string(),
                "qwen2.5:7b".to_string(),
                "llama3.1:8b".to_string(),
                "mistral:7b".to_string(),
                "phi4:14b".to_string(),
                "codellama:7b".to_string(),
                "gemma2:9b".to_string(),
                // Medium (3-7B)
                "phi3:14b".to_string(),
                "qwen2.5:4b".to_string(),
                "phi3.5:3.8b".to_string(),
                // Light (1-3B)
                "phi3:3.8b".to_string(),
                "gemma2:2b".to_string(),
                "llama3.2:3b".to_string(),
                "llama3.2:1b".to_string(),
                // Tiny (≤1B)
                "tinyllama:1.1b".to_string(),
                "phi2:2.7b".to_string(),
                "stableLM:3b".to_string(),
                "SmolLM2:360m".to_string(),
                "SmolLM2:1.7b".to_string(),
                "deepseek-coder:1.3b".to_string(),
                "deepseek-coder:0.5b".to_string(),
            ]),
        });
        
        // Add openrouter as secondary (free tier available)
        providers.insert("openrouter".to_string(), ProviderConfig {
            api_key: None, // User needs to set OPENROUTER_API_KEY
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            default_model: Some("moonshotai/kimi-k2.6".to_string()),
            models: None,
        });
        
        Self {
            default: "pawlos/qwen2.5:7b".to_string(), // Local by default - fast & free
            providers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessagingSection {
    pub discord: Option<DiscordConfig>,
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub allowed_channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub allowed_users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    pub port: u16,
    pub host: String,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            port: 9797,
            host: "127.0.0.1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatSection {
    pub enabled: bool,
    pub interval_secs: u64,
}

impl Default for HeartbeatSection {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300, // 5 minutes
        }
    }
}

impl Config {
    /// Returns the base pawlos directory: ~/.pawlos/
    pub fn base_dir() -> PathBuf {
        home_dir()
            .expect("Cannot determine home directory")
            .join(".pawlos")
    }

    pub fn config_path() -> PathBuf {
        Self::base_dir().join("config.yaml")
    }

    /// Get the project root (for dev mode) or fall back to base_dir
    fn get_project_or_base(&self, folder: &str) -> PathBuf {
        // Check for project root folder first (grandparent of exe)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                if let Some(grandparent) = parent.parent() {
                    let candidate = grandparent.join(folder);
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }
        Self::base_dir().join(folder)
    }

    /// Context directory - memory files (SOUL.md, USER.md, MEMORY.md, LONGTERM.md)
    pub fn context_dir() -> PathBuf {
        Self::default().get_project_or_base("context")
    }

    /// Reusable skills created during chat (make_pdf.skill, etc.)
    pub fn skills_dir() -> PathBuf {
        Self::default().get_project_or_base("skills")
    }

    /// Sub-agents directory (each agent has their own folder)
    pub fn agents_dir() -> PathBuf {
        Self::default().get_project_or_base("agents")
    }

    /// Daily logs directory (YYYY-MM-DD.md files)
    pub fn memory_dir() -> PathBuf {
        Self::default().get_project_or_base("memory")
    }

    /// Legacy - backward compatibility
    pub fn memories_dir() -> PathBuf {
        Self::context_dir()
    }

    /// Vector embeddings storage (in memory/ folder)
    pub fn vector_store_path() -> PathBuf {
        Self::memory_dir().join("vector.db")
    }

    /// Backup JSON vector store
    pub fn vector_json_path() -> PathBuf {
        Self::memory_dir().join("vector_store.json")
    }

    pub fn logs_dir() -> PathBuf {
        Self::base_dir().join("logs")
    }

    pub fn vector_db_dir() -> PathBuf {
        Self::base_dir().join("vector_db")
    }

    pub fn plugins_dir() -> PathBuf {
        Self::base_dir().join("plugins")
    }

    pub fn mcp_servers_dir() -> PathBuf {
        Self::base_dir().join("mcp_servers")
    }

    /// Load config from disk; returns Default if file missing
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let cfg: Config = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Config parse error: {e}"))?;
        Ok(cfg)
    }

    /// Persist config to disk with automatic backup
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Create backup if config already exists
        if path.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let backup_path = path.with_extension(format!("yaml.backup-{}", timestamp));
            std::fs::copy(&path, &backup_path).ok(); // Ignore backup errors
            tracing::info!("Config backup created: {}", backup_path.display());
        }
        
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| anyhow::anyhow!("Config serialization error: {e}"))?;
        std::fs::write(&path, yaml)?;
        Ok(())
    }

    /// Expand ${ENV_VAR} references in strings
    pub fn expand_env(s: &str) -> String {
        let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
        re.replace_all(s, |caps: &regex::Captures| {
            std::env::var(&caps[1]).unwrap_or_default()
        })
        .into_owned()
    }

    /// Resolve API key for a provider (expands env vars)
    pub fn api_key(&self, provider: &str) -> Option<String> {
        self.models
            .providers
            .get(provider)
            .and_then(|p| p.api_key.clone())
            .map(|k| Self::expand_env(&k))
            .filter(|k| !k.is_empty())
    }
}
