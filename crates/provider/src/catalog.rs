//! Model catalog with all available providers and models
//! Includes local model detection and fallback logic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical model catalog - all available models organized by provider
pub struct ModelCatalog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: u32,
    pub supports_tools: bool,
    pub is_local: bool,
    pub recommended_for: Vec<String>, // "fast", "coding", "reasoning", "balanced"
}

impl ModelCatalog {
    /// Get all available models for a provider
    pub fn models_for_provider(provider: &str) -> Vec<ModelInfo> {
        match provider {
            "openai" => Self::openai_models(),
            "anthropic" => Self::anthropic_models(),
            "openrouter" => Self::openrouter_models(),
            "google" | "gemini" => Self::gemini_models(),
            "deepseek" => Self::deepseek_models(),
            "minimax" => Self::minimax_models(),
            "qwen" | "alibaba" => Self::qwen_models(),
            "moonshot" | "kimi" => Self::kimi_models(),
            "pawlos" | "local" => Self::pawlos_local_models(),
            _ => vec![],
        }
    }

    /// All available providers with metadata
    pub fn all_providers() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("openai", "OpenAI", "GPT-4o, GPT-4o-mini, GPT-5 series"),
            ("anthropic", "Anthropic", "Claude 4 (Opus, Sonnet, Haiku)"),
            ("openrouter", "OpenRouter", "100+ models via unified API"),
            ("google", "Google AI", "Gemini 2.5, Gemini Flash"),
            ("deepseek", "DeepSeek", "DeepSeek V3, Coder, Reasoner"),
            ("minimax", "MiniMax", "M2.5, M2.7 - fast & cheap"),
            ("qwen", "Qwen (Alibaba)", "Qwen3 - excellent coding"),
            ("moonshot", "Moonshot (Kimi)", "Kimi k2.6 - great reasoning"),
            ("pawlos", "Pawlos Local", "Local Ollama models on your device"),
        ]
    }

    fn openai_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "gpt-5".into(), name: "GPT-5".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "gpt-5-mini".into(), name: "GPT-5 Mini".into(), context_length: 100000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gpt-5-nano".into(), name: "GPT-5 Nano".into(), context_length: 50000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gpt-4o".into(), name: "GPT-4o".into(), context_length: 128000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "gpt-4o-mini".into(), name: "GPT-4o Mini".into(), context_length: 128000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gpt-4.1".into(), name: "GPT-4.1".into(), context_length: 128000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "gpt-4.1-mini".into(), name: "GPT-4.1 Mini".into(), context_length: 128000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gpt-4.1-nano".into(), name: "GPT-4.1 Nano".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
        ]
    }

    fn anthropic_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "claude-opus-4-7".into(), name: "Claude Opus 4.7".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into(), "reasoning".into()] },
            ModelInfo { id: "claude-opus-4-6".into(), name: "Claude Opus 4.6".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into(), "reasoning".into()] },
            ModelInfo { id: "claude-sonnet-4-6".into(), name: "Claude Sonnet 4.6".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "claude-sonnet-4-5".into(), name: "Claude Sonnet 4.5".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "claude-haiku-4-5".into(), name: "Claude Haiku 4.5".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
        ]
    }

    fn openrouter_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "moonshotai/kimi-k2.6".into(), name: "Kimi K2.6".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["reasoning".into(), "coding".into()] },
            ModelInfo { id: "anthropic/claude-opus-4.7".into(), name: "Claude Opus 4.7".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "qwen/qwen3.6-plus".into(), name: "Qwen 3.6 Plus".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "minimax/minimax-m2.7".into(), name: "MiniMax M2.7".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "minimax/minimax-m2.5".into(), name: "MiniMax M2.5".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "x-ai/grok-4.20".into(), name: "Grok 4.20".into(), context_length: 131072, supports_tools: true, is_local: false, recommended_for: vec!["reasoning".into()] },
            ModelInfo { id: "deepseek-ai/deepseek-v3.2".into(), name: "DeepSeek V3.2".into(), context_length: 64000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
        ]
    }

    fn gemini_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "gemini-2.5-pro".into(), name: "Gemini 2.5 Pro".into(), context_length: 1000000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "gemini-2.5-flash".into(), name: "Gemini 2.5 Flash".into(), context_length: 1000000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gemini-3.1-pro".into(), name: "Gemini 3.1 Pro".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "gemini-3.1-flash".into(), name: "Gemini 3.1 Flash".into(), context_length: 1000000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
        ]
    }

    fn deepseek_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "deepseek-chat".into(), name: "DeepSeek Chat".into(), context_length: 64000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "deepseek-reasoner".into(), name: "DeepSeek Reasoner".into(), context_length: 64000, supports_tools: true, is_local: false, recommended_for: vec!["reasoning".into()] },
            ModelInfo { id: "deepseek-coder".into(), name: "DeepSeek Coder".into(), context_length: 16000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
        ]
    }

    fn minimax_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "MiniMax-M2.7".into(), name: "MiniMax M2.7".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into(), "coding".into()] },
            ModelInfo { id: "MiniMax-M2.5".into(), name: "MiniMax M2.5".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["fast".into()] },
        ]
    }

    fn qwen_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "qwen3.6-plus".into(), name: "Qwen 3.6 Plus".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "qwen3.5-32b".into(), name: "Qwen 3.5 32B".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "qwen2.5-coder".into(), name: "Qwen 2.5 Coder".into(), context_length: 16000, supports_tools: true, is_local: false, recommended_for: vec!["coding".into()] },
        ]
    }

    fn kimi_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo { id: "kimi-k2.6".into(), name: "Kimi K2.6".into(), context_length: 200000, supports_tools: true, is_local: false, recommended_for: vec!["reasoning".into(), "coding".into()] },
            ModelInfo { id: "kimi-k2.5".into(), name: "Kimi K2.5".into(), context_length: 128000, supports_tools: true, is_local: false, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "kimi-k2-thinking".into(), name: "Kimi K2 Thinking".into(), context_length: 32000, supports_tools: true, is_local: false, recommended_for: vec!["reasoning".into()] },
        ]
    }

    /// Pawlos local models - these are loaded from Ollama on the user's device
    fn pawlos_local_models() -> Vec<ModelInfo> {
        vec![
            // === HIGH-END (8B+ params) - needs GPU with 16GB+ VRAM ===
            ModelInfo { id: "llama3.1:70b".into(), name: "Llama 3.1 70B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["reasoning".into(), "balanced".into()] },
            ModelInfo { id: "qwen2.5:14b".into(), name: "Qwen 2.5 14B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["coding".into(), "balanced".into()] },
            ModelInfo { id: "qwen2.5:7b".into(), name: "Qwen 2.5 7B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "llama3.1:8b".into(), name: "Llama 3.1 8B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["balanced".into()] },
            ModelInfo { id: "mistral:7b".into(), name: "Mistral 7B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "phi4:14b".into(), name: "Phi 4 14B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "codellama:7b".into(), name: "CodeLlama 7B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "gemma2:9b".into(), name: "Gemma 2 9B".into(), context_length: 8192, supports_tools: true, is_local: true, recommended_for: vec!["balanced".into()] },
            
            // === MEDIUM (3-7B params) - works on modern laptops with 8GB+ RAM ===
            ModelInfo { id: "phi3:14b".into(), name: "Phi 3 14B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "qwen2.5:4b".into(), name: "Qwen 2.5 4B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "phi3.5:3.8b".into(), name: "Phi 3.5 3.8B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            
            // === LIGHT (1-3B params) - works on any device, even old laptops ===
            ModelInfo { id: "phi3:3.8b".into(), name: "Phi 3 3.8B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "gemma2:2b".into(), name: "Gemma 2 2B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "llama3.2:3b".into(), name: "Llama 3.2 3B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "llama3.2:1b".into(), name: "Llama 3.2 1B".into(), context_length: 4096, supports_tools: true, is_local: true, recommended_for: vec!["fast".into()] },
            
            // === TINY (≤1B params) - works on anything, even Raspberry Pi ===
            ModelInfo { id: "tinyllama:1.1b".into(), name: "TinyLlama 1.1B".into(), context_length: 2048, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
            ModelInfo { id: "phi2:2.7b".into(), name: "Phi 2 2.7B".into(), context_length: 2048, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
            ModelInfo { id: "stableLM:3b".into(), name: "StableLM 3B".into(), context_length: 4096, supports_tools: false, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "mobiuslab:qwen-0.5b".into(), name: "Qwen 0.5B".into(), context_length: 1024, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
            ModelInfo { id: "aya:0.5b".into(), name: "Aya 0.5B".into(), context_length: 1024, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
            ModelInfo { id: "SmolLM2:360m".into(), name: "SmolLM2 360M".into(), context_length: 1024, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
            ModelInfo { id: "SmolLM2:1.7b".into(), name: "SmolLM2 1.7B".into(), context_length: 2048, supports_tools: false, is_local: true, recommended_for: vec!["fast".into()] },
            ModelInfo { id: "deepseek-coder:1.3b".into(), name: "DeepSeek Coder 1.3B".into(), context_length: 2048, supports_tools: true, is_local: true, recommended_for: vec!["coding".into()] },
            ModelInfo { id: "deepseek-coder:0.5b".into(), name: "DeepSeek Coder 0.5B".into(), context_length: 1024, supports_tools: false, is_local: true, recommended_for: vec!["ultra_fast".into()] },
        ]
    }

    /// Get the default model for a provider (first in the list)
    pub fn default_model_for(provider: &str) -> String {
        Self::models_for_provider(provider)
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| format!("{}/default", provider))
    }
}

/// Local model detection and device capability assessment
pub struct LocalModelDetector;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceCapabilities {
    pub has_gpu: bool,
    pub gpu_vram_gb: f64,
    pub ram_gb: f64,
    pub cpu_cores: usize,
    pub os: String, // "windows", "linux", "macos"
}

/// Performance tiers for local models
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceTier {
    /// Can run 70B+ models (32GB+ RAM or 16GB+ GPU)
    HighEnd,
    /// Can run 14B-70B models (16GB RAM or 8GB GPU)
    MidRange,
    /// Can run 7B-14B models (8GB RAM)
    EntryLevel,
    /// Can run 1B-7B models (4-8GB RAM)
    LowEnd,
    /// Can only run ≤1B models (<4GB RAM, old laptops, Pi)
    UltraLowEnd,
}

impl LocalModelDetector {
    /// Detect device capabilities and determine best local model tier
    pub fn detect_capabilities() -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();

        // Detect OS
        #[cfg(target_os = "windows")]
        {
            caps.os = "windows".into();
        }
        #[cfg(target_os = "linux")]
        {
            caps.os = "linux".into();
        }
        #[cfg(target_os = "macos")]
        {
            caps.os = "macos".into();
        }

        // Get CPU core count
        caps.cpu_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        // Estimate RAM (this is a rough estimate)
        #[cfg(windows)]
        {
            // On Windows, try to get total physical memory
            use std::mem::MaybeUninit;
            #[link(name = "kernel32")]
            extern "system" {
                fn GetGlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
            }
            #[repr(C)]
            struct MEMORYSTATUSEX {
                dwLength: u32,
                dwMemoryLoad: u32,
                ullTotalPhys: u64,
                ullAvailPhys: u64,
                ullTotalPageFile: u64,
                ullAvailPageFile: u64,
                ullTotalVirtual: u64,
                ullAvailVirtual: u64,
                ullAvailExtendedVirtual: u64,
            }
            let mut mem = MEMORYSTATUSEX { 
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            };
            if unsafe { GetGlobalMemoryStatusEx(&mut mem) } != 0 {
                caps.ram_gb = mem.ullTotalPhys as f64 / 1_073_741_824.0;
            }
        }
        #[cfg(not(windows))]
        {
            // Simplified fallback - assume typical values
            caps.ram_gb = 16.0; // Default assumption
        }

        // GPU detection (simplified - would need platform-specific code for real detection)
        // For now, we'll use RAM as a proxy - systems with more RAM often have dedicated GPU
        caps.has_gpu = caps.ram_gb >= 16.0;
        caps.gpu_vram_gb = if caps.has_gpu { 8.0 } else { 0.0 };

        caps
    }

    /// Determine device tier based on capabilities
    pub fn device_tier(caps: &DeviceCapabilities) -> DeviceTier {
        // High-end: 32GB+ RAM or dedicated GPU with 16GB+ VRAM
        if caps.ram_gb >= 32.0 || (caps.has_gpu && caps.gpu_vram_gb >= 16.0) {
            return DeviceTier::HighEnd;
        }
        // Mid-range: 16GB RAM or GPU with 8GB VRAM
        if caps.ram_gb >= 16.0 || (caps.has_gpu && caps.gpu_vram_gb >= 8.0) {
            return DeviceTier::MidRange;
        }
        // Entry-level: 8GB RAM
        if caps.ram_gb >= 8.0 {
            return DeviceTier::EntryLevel;
        }
        // Low-end: 4-8GB RAM - can still run small models
        if caps.ram_gb >= 4.0 {
            return DeviceTier::LowEnd;
        }
        // Ultra low-end: <4GB RAM - only tiny models work
        DeviceTier::UltraLowEnd
    }

    /// Recommend the best local model based on device capabilities
    pub fn recommended_local_model(caps: &DeviceCapabilities) -> String {
        let tier = Self::device_tier(caps);
        
        let models = ModelCatalog::pawlos_local_models();
        
        match tier {
            DeviceTier::HighEnd => {
                // Recommend best available
                models.iter()
                    .find(|m| m.id.contains("70b") || m.id.contains("qwen2.5:14b"))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "llama3.1:70b".into())
            }
            DeviceTier::MidRange => {
                // Good balance of performance
                models.iter()
                    .find(|m| m.id.contains("14b") || m.id.contains("qwen2.5:14b"))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "qwen2.5:14b".into())
            }
            DeviceTier::EntryLevel => {
                // Works on most laptops
                models.iter()
                    .find(|m| m.id.contains("7b") || m.id.contains("qwen2.5:7b"))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "qwen2.5:7b".into())
            }
            DeviceTier::LowEnd => {
                // Lightweight model for older devices (3B params)
                models.iter()
                    .find(|m| m.id.contains("3b") || m.id.contains("phi3:3.8b") || m.id.contains("gemma2:2b"))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "phi3:3.8b".into())
            }
            DeviceTier::UltraLowEnd => {
                // Tiny models for very old/low-RAM devices (≤1B params)
                models.iter()
                    .find(|m| m.id.contains("360m") || m.id.contains("0.5b") || m.id.contains("1.1b"))
                    .map(|m| m.id.clone())
                    .unwrap_or_else(|| "tinyllama:1.1b".into())
            }
        }
    }

    /// Check if Ollama is available on the system
    pub async fn is_ollama_available() -> bool {
        use tokio::net::TcpStream;
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            TcpStream::connect("localhost:11434")
        ).await.is_ok()
    }
}

/// Fallback strategy when API fails
pub struct ModelFallback;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// Try next provider in the list
    NextProvider,
    /// Fall back to local Ollama
    LocalOllama,
    /// No fallback available
    None,
}

impl ModelFallback {
    /// Determine fallback strategy based on error type
    pub fn determine(error: &str) -> FallbackStrategy {
        let error_lower = error.to_lowercase();
        
        // Network/connection errors -> try local
        if error_lower.contains("connection") 
            || error_lower.contains("network") 
            || error_lower.contains("timeout") {
            return FallbackStrategy::LocalOllama;
        }
        
        // Auth errors -> try next provider
        if error_lower.contains("auth") 
            || error_lower.contains("401") 
            || error_lower.contains("403")
            || error_lower.contains("api key") {
            return FallbackStrategy::NextProvider;
        }
        
        // Rate limiting -> try next provider
        if error_lower.contains("429") 
            || error_lower.contains("rate limit") {
            return FallbackStrategy::NextProvider;
        }
        
        // Model not found -> try next provider
        if error_lower.contains("404") 
            || error_lower.contains("model not found") {
            return FallbackStrategy::NextProvider;
        }
        
        // Server errors -> try local as last resort
        if error_lower.contains("500") 
            || error_lower.contains("503") 
            || error_lower.contains("server error") {
            return FallbackStrategy::LocalOllama;
        }
        
        FallbackStrategy::NextProvider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_tier_detection() {
        let caps = DeviceCapabilities {
            has_gpu: true,
            gpu_vram_gb: 16.0,
            ram_gb: 32.0,
            cpu_cores: 8,
            os: "windows".into(),
        };
        assert_eq!(LocalModelDetector::device_tier(&caps), DeviceTier::HighEnd);
    }
    
    #[test]
    fn test_fallback_strategy() {
        let conn_error = "connection refused";
        assert_eq!(ModelFallback::determine(conn_error), FallbackStrategy::LocalOllama);
        
        let auth_error = "401 unauthorized";
        assert_eq!(ModelFallback::determine(auth_error), FallbackStrategy::NextProvider);
    }
}