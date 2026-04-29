use anyhow::Result;
use dialoguer::{Input, Select, Password, MultiSelect};
use colored::Colorize;
use pawlos_core::{Config, config::*};
use memory::store::MemoryStore;
use std::collections::HashMap;
use crate::ui::{self, colors, quotes};

/// MCP servers available during onboarding
const AVAILABLE_MCPS: &[(&str, &str)] = &[
    ("github", "GitHub (issues, PRs, repos)"),
    ("brave_search", "Brave Search (web search)"),
    ("fetch", "Web Fetch (URL → markdown)"),
    ("memory", "Knowledge Graph Memory"),
    ("sqlite", "SQLite Database"),
    ("git", "Git Operations"),
    ("playwright", "Browser Automation"),
    ("context7", "Documentation Lookup"),
];

pub async fn run_onboarding() -> Result<()> {
    // Clear screen and show branding
    print!("\x1b[2J\x1b[H");
    ui::logo::print();
    ui::banner::print();
    
    println!();
    println!("{}", "━".repeat(60).bright_cyan());
    println!("{}  pawlos — First Run Setup", " ".bright_cyan());
    println!("{}", "━".repeat(60).bright_cyan());
    println!();

    // Agent name
    println!("{}", "Step 1/4: Agent Identity".bold().bright_blue());
    println!("{}", "─".repeat(30).dimmed());
    let agent_name: String = Input::new()
        .with_prompt("Who am I? (agent name)")
        .default("pawlos".into())
        .interact_text()?;
    println!("{} ✓ Agent name set to: {}", " ".dimmed(), agent_name.bright_cyan());

    // User name
    println!();
    println!("{}", "Step 2/4: User Identity".bold().bright_blue());
    println!("{}", "─".repeat(30).dimmed());
    let user_name: String = Input::new()
        .with_prompt("Who are you? (your name)")
        .interact_text()?;
    println!("{} ✓ User name set to: {}", " ".dimmed(), user_name.bright_cyan());

    // Provider selection
    println!();
    println!("{}", "Step 3/4: LLM Provider".bold().bright_blue());
    println!("{}", "─".repeat(30).dimmed());
    
    let providers = vec![
        "OpenAI (GPT-4o)",
        "Anthropic (Claude)",
        "OpenRouter (200+ models)",
        "Google (Gemini)",
        "Groq (fast inference)",
        "Ollama / Local (pawlos default)",
        "Custom HTTP endpoint",
    ];
    
    let provider_idx = Select::new()
        .with_prompt("Choose your LLM provider")
        .items(&providers)
        .default(5)  // Default to Ollama/local
        .interact()?;
    
    let (provider_key, default_model, base_url_override): (&str, &str, Option<&str>) = match provider_idx {
        0 => ("openai",     "openai/gpt-4o",            None),
        1 => ("anthropic",  "anthropic/claude-3-5-sonnet-20241022", None),
        2 => ("openrouter", "openrouter/openai/gpt-4o",  None),
        3 => ("google",     "google/gemini-2.0-flash",   None),
        4 => ("groq",       "groq/llama-3.3-70b-versatile", None),
        5 => ("pawlos",     "pawlos/qwen2.5:7b",         Some("http://localhost:11434/v1")),
        _ => ("custom",     "custom/model",             None),
    };
    
    println!("{} ✓ Provider: {}", " ".dimmed(), providers[provider_idx].bright_cyan());

    let api_key = if provider_idx == 5 {
        // Local Ollama doesn't need API key
        "ollama".to_string()
    } else if provider_idx == 6 {
        // Custom endpoint
        Input::new()
            .with_prompt("API key (leave empty if none)")
            .allow_empty(true)
            .interact_text()?
    } else {
        Password::new()
            .with_prompt(format!("{} API key", provider_key))
            .allow_empty_password(true)
            .interact()?
    };

    let base_url: Option<String> = if provider_idx == 6 {
        let u: String = Input::new().with_prompt("Base URL").interact_text()?;
        Some(u)
    } else {
        base_url_override.map(|s| s.to_string())
    };

    // Port
    println!();
    println!("{}", "Step 4/4: Web UI".bold().bright_blue());
    println!("{}", "─".repeat(30).dimmed());
    let port: u16 = Input::new()
        .with_prompt("Web UI port (0 to disable)")
        .default(9797_u16)
        .interact_text()?;
    println!("{} ✓ Web UI port: {}", " ".dimmed(), port.to_string().bright_cyan());

    // MCP servers
    println!();
    println!("{}", "Optional: MCP Servers".bold().bright_blue());
    println!("{}", "─".repeat(30).dimmed());
    println!("{}", "Select MCP servers to enable (space=select, enter=confirm)".dimmed());
    
    let mcp_options: Vec<&str> = AVAILABLE_MCPS.iter().map(|(_, desc)| *desc).collect();
    let mcp_indices = MultiSelect::new()
        .with_prompt("MCP Servers")
        .items(&mcp_options)
        .interact()?;

    // Collect tokens/keys for selected MCPs
    let mut mcp_configs = HashMap::new();
    
    for idx in &mcp_indices {
        let (name, _) = AVAILABLE_MCPS[*idx];
        match name {
            "github" => {
                println!();
                let token = Password::new()
                    .with_prompt("  GitHub token (GITHUB_TOKEN)")
                    .allow_empty_password(true)
                    .interact()?;
                if !token.is_empty() {
                    mcp_configs.insert("github".to_string(), serde_json::json!({
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-github"],
                        "env": { "GITHUB_TOKEN": token }
                    }));
                }
            }
            "brave_search" => {
                println!();
                let key = Password::new()
                    .with_prompt("  Brave Search API key")
                    .allow_empty_password(true)
                    .interact()?;
                if !key.is_empty() {
                    mcp_configs.insert("brave_search".to_string(), serde_json::json!({
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-brave-search"],
                        "env": { "BRAVE_API_KEY": key }
                    }));
                }
            }
            "fetch" => {
                mcp_configs.insert("fetch".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-fetch"]
                }));
            }
            "memory" => {
                mcp_configs.insert("memory".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-memory"]
                }));
            }
            "git" => {
                println!();
                let root = Input::new()
                    .with_prompt("  Git repo path (default: current dir)")
                    .default(".".to_string())
                    .interact_text()?;
                mcp_configs.insert("git".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-git", root]
                }));
            }
            "sqlite" => {
                println!();
                let db_path = Input::new()
                    .with_prompt("  SQLite database path")
                    .default("./data.db".to_string())
                    .interact_text()?;
                mcp_configs.insert("sqlite".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "mcp-server-sqlite", db_path]
                }));
            }
            "playwright" => {
                mcp_configs.insert("playwright".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@executeautomation/playwright-mcp-server"]
                }));
            }
            "context7" => {
                mcp_configs.insert("context7".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@context7/mcp-server"]
                }));
            }
            _ => {}
        }
    }

    if !mcp_configs.is_empty() {
        println!();
        println!("{} ✓ {} MCP servers configured", " ".dimmed(), mcp_configs.len().to_string().bright_cyan());
    }

    // Build config
    let mut providers_map = HashMap::new();
    providers_map.insert(
        provider_key.to_string(),
        ProviderConfig {
            api_key: Some(api_key),
            base_url,
            default_model: None,
            models: None,
        },
    );

    let cfg = Config {
        agent: AgentSection {
            name: agent_name.clone(),
            personalities: HashMap::new(),
        },
        user: UserSection { name: user_name.clone() },
        models: ModelsSection {
            default: default_model.to_string(),
            providers: providers_map,
        },
        messaging: MessagingSection::default(),
        server: ServerSection { port, host: "127.0.0.1".into() },
        heartbeat: HeartbeatSection { enabled: true, interval_secs: 300 },
        mcp_servers: mcp_configs,
    };

    // Save config (with backup if exists)
    cfg.save()?;
    println!();
    println!("{}", "━".repeat(60).bright_cyan());
    println!("{}", "  ✓ Setup Complete!".bold().bright_green());
    println!("{}", "━".repeat(60).bright_cyan());

    // Bootstrap memory files
    MemoryStore::bootstrap_defaults(&agent_name, &user_name)?;
    println!("{}", " ✓ Memory files initialized".dimmed());

    println!();
    println!("{}", "pawlos is ready!".bold().bright_cyan());
    println!();
    println!("  {} Run {} to start chatting", ">".bright_cyan(), "pawlos".bold());
    if port != 0 {
        println!("  {} Web UI: {}", ">".bright_cyan(), format!("http://127.0.0.1:{}", port).bright_blue());
    }
    println!();

    Ok(())
}
