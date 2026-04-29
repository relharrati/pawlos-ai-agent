use anyhow::Result;
use dialoguer::{Input, Select, Password, Confirm, MultiSelect};
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
    print_awakening_animation();

    // Show branding
    ui::logo::print();
    ui::banner::print();
    
    println!();
    println!("{}{}  🤖 pawlos — first run setup{}", colors::DIM, colors::BOLD, colors::RESET);
    println!("{}", "═".repeat(50).dimmed());
    println!();

    // Agent name
    let agent_name: String = Input::new()
        .with_prompt("🤖 Who am I?  (agent name, e.g. \"pawlos\", \"Rusty\", \"Athena\")")
        .default("pawlos".into())
        .interact_text()?;

    // User name
    let user_name: String = Input::new()
        .with_prompt("👤 Who are you?  (your name or handle)")
        .interact_text()?;

    println!();

    // Provider selection
    let providers = vec![
        "OpenAI (GPT-4o)",
        "Anthropic (Claude)",
        "OpenRouter (200+ models)",
        "Google (Gemini)",
        "Groq (fast inference)",
        "Ollama / local",
        "Custom HTTP endpoint",
    ];
    let provider_idx = Select::new()
        .with_prompt("🔌 Choose your LLM provider")
        .items(&providers)
        .default(0)
        .interact()?;

    let (provider_key, default_model, base_url_override): (&str, &str, Option<&str>) = match provider_idx {
        0 => ("openai",     "openai/gpt-4o",            None),
        1 => ("anthropic",  "anthropic/claude-3-5-sonnet-20241022", None),
        2 => ("openrouter", "openrouter/openai/gpt-4o",  None),
        3 => ("google",     "google/gemini-2.0-flash",   None),
        4 => ("groq",       "groq/llama-3.3-70b-versatile", None),
        5 => ("local",      "local/llama3.1:8b",         Some("http://localhost:11434/v1")),
        _ => ("openai",     "openai/gpt-4o",            None),
    };

    let api_key = if provider_idx == 5 {
        "ollama".to_string()
    } else if provider_idx == 6 {
        Input::new().with_prompt("API key").interact_text()?
    } else {
        Password::new()
            .with_prompt(format!("🔑 {provider_key} API key"))
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
    let port: u16 = Input::new()
        .with_prompt("🌐 Web UI port")
        .default(9797_u16)
        .interact_text()?;

    // ── MCP servers ─────────────────────────────────────────────
    println!("\n{}", "═".repeat(50).dimmed());
    println!("{}", "  🔌 MCP Servers (optional)".bold());
    println!("{}\n", "═".repeat(50).dimmed());

    let mcp_options: Vec<&str> = AVAILABLE_MCPS.iter().map(|(_, desc)| desc).collect();
    let mcp_indices = MultiSelect::new()
        .with_prompt("Select MCPs to enable (space to select, enter to confirm)")
        .items(&mcp_options)
        .interact()?;

    // Collect tokens/keys for selected MCPs
    let mut mcp_configs = HashMap::new();

    for idx in &mcp_indices {
        let (name, _) = AVAILABLE_MCPS[*idx];
        match name {
            "github" => {
                let token = Password::new()
                    .with_prompt("  🔑 GitHub token (GITHUB_TOKEN)")
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
                let key = Password::new()
                    .with_prompt("  🔑 Brave Search API key")
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
                let root = Input::new()
                    .with_prompt("  📁 Git repo path (default: current dir)")
                    .default(".".to_string())
                    .interact_text()?;
                mcp_configs.insert("git".to_string(), serde_json::json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-git", root]
                }));
            }
            "sqlite" => {
                let db_path = Input::new()
                    .with_prompt("  📁 SQLite database path")
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
        println!("\n{} {}", "✅".green(), format!("{} MCP servers configured", mcp_configs.len()).green());
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

    cfg.save()?;
    ui::success("Config saved");

    // Bootstrap memory files
    MemoryStore::bootstrap_defaults(&agent_name, &user_name)?;
    ui::success("Memory files initialised");

    println!();
    ui::dashboard::print_with_label("READY");
    println!();
    println!("{}{}{} is ready!{}",
             colors::BRIGHT_CYAN, colors::BOLD, agent_name, colors::RESET);
    println!("{}{}Run `pawlos` to start chatting{}",
             colors::DIM, colors::BRIGHT_GREEN, colors::RESET);
    println!();

    Ok(())
}

fn print_awakening_animation() {
    // Show random funny startup message
    print!("{}Initializing...{}", colors::DIM, quotes::random());
    std::thread::sleep(std::time::Duration::from_millis(400));
    ui::clear_line();
    quotes::print_random();
    std::thread::sleep(std::time::Duration::from_millis(200));
}
