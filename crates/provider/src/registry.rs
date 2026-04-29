use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use pawlos_core::Config;
use crate::client::{OpenAiCompatClient, ProviderClient};

/// Registry of all configured provider clients
pub struct ProviderRegistry {
    clients: HashMap<String, Arc<dyn ProviderClient>>,
    pub default_model: String,
}

impl ProviderRegistry {
    pub fn from_config(cfg: &Config) -> Result<Self> {
        let mut clients: HashMap<String, Arc<dyn ProviderClient>> = HashMap::new();

        // Well-known base URLs
        let well_known: HashMap<&str, &str> = [
            ("openai",      "https://api.openai.com/v1"),
            ("anthropic",   "https://api.anthropic.com/v1"),
            ("openrouter",  "https://openrouter.ai/api/v1"),
            ("groq",        "https://api.groq.com/openai/v1"),
            ("together",    "https://api.together.xyz/v1"),
            ("fireworks",   "https://api.fireworks.ai/inference/v1"),
            ("mistral",     "https://api.mistral.ai/v1"),
            ("deepseek",    "https://api.deepseek.com/v1"),
            ("google",      "https://generativelanguage.googleapis.com/v1beta/openai"),
        ].into();

        for (name, provider_cfg) in &cfg.models.providers {
            let base_url = provider_cfg.base_url
                .clone()
                .or_else(|| well_known.get(name.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| format!("https://api.{name}.com/v1"));

            let api_key = provider_cfg.api_key
                .as_deref()
                .map(Config::expand_env)
                .unwrap_or_default();

            let client = OpenAiCompatClient::new(name.clone(), base_url, api_key);
            clients.insert(name.clone(), Arc::new(client));
        }

        // Add pawlos local provider (Ollama) if not already defined
        // pawlos is the primary local model provider
        if !clients.contains_key("pawlos") {
            let client = OpenAiCompatClient::new(
                "pawlos".to_string(),
                "http://localhost:11434/v1".to_string(),
                "ollama".to_string(),
            );
            clients.insert("pawlos".into(), Arc::new(client));
        }

        // Also add "local" as alias for backwards compatibility
        if !clients.contains_key("local") {
            let client = OpenAiCompatClient::new(
                "local".to_string(),
                "http://localhost:11434/v1".to_string(),
                "ollama".to_string(),
            );
            clients.insert("local".into(), Arc::new(client));
        }

        Ok(Self {
            clients,
            default_model: cfg.models.default.clone(),
        })
    }

    /// Parse "provider/model" string, return (provider_name, model_id)
    pub fn parse_model_spec(spec: &str) -> (&str, &str) {
        if let Some(idx) = spec.find('/') {
            (&spec[..idx], &spec[idx + 1..])
        } else {
            ("openai", spec)
        }
    }

    pub fn get_client(&self, provider: &str) -> Option<Arc<dyn ProviderClient>> {
        self.clients.get(provider).cloned()
    }

    pub fn list_providers(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }
}
