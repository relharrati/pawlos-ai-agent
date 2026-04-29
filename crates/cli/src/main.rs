mod onboard;
mod repl;
mod commands;
mod ui;

// Re-export for commands module
pub use self::commands::*;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use tokio::sync::Mutex;

use pawlos_core::{Config, Database};
use memory::store::MemoryStore;
use orchestrator::{SessionManager, Heartbeat, AgentRegistry, WebServer};
use orchestrator::web_server::AppState;
use provider::registry::ProviderRegistry;

#[derive(Parser)]
#[command(
    name = "pawlos",
    about = "🤖 pawlos — persistent, self-evolving AI agent",
    version = "0.1.0",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Re-run onboarding setup
    Onboard,
    /// Start the agent (default behaviour)
    Start,
    /// Start web UI only (no terminal REPL)
    Web,
    /// Manage memory stores
    Memory {
        #[arg(value_enum)]
        action: MemoryCmd,
        store: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List / run a skill
    Skill {
        #[arg(value_enum)]
        action: SkillCmd,
        name: Option<String>,
    },
    /// Manage MCP servers
    MCP {
        #[arg(value_enum)]
        action: McpCmd,
    },
    /// Print current config
    Config,
}

#[derive(clap::ValueEnum, Clone)]
enum MemoryCmd { Read, Add, Remove, Consolidate }

#[derive(clap::ValueEnum, Clone)]
pub enum SkillCmd { List, Load, Create }

#[derive(clap::ValueEnum, Clone)]
pub enum McpCmd { List, Add, Remove }

#[tokio::main]
async fn main() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("pawlos=info,warn")),
        )
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Start) {
        Command::Onboard => {
            onboard::run_onboarding().await?;
        }
        Command::Config => {
            let cfg = Config::load()?;
            let yaml = serde_yaml::to_string(&cfg).unwrap_or_default();
            println!("{yaml}");
        }
        Command::Memory { action, store, args } => {
            commands::memory_cmd(action, &store, &args)?;
        }
        Command::Skill { action, name } => {
            commands::skill_cmd(action, name.as_deref())?;
        }
        Command::MCP { action } => {
            commands::mcp_cmd(action)?;
        }
        Command::Web => {
            let cfg = Config::load()?;
            if cfg.agent.name.is_empty() || !Config::config_path().exists() {
                onboard::run_onboarding().await?;
            }
            let cfg = Config::load()?;
            run_services(&cfg, false).await?;
        }
        Command::Start => {
            // Check if configured
            if !Config::config_path().exists() {
                onboard::run_onboarding().await?;
            }
            let cfg = Config::load()?;
            if cfg.agent.name.is_empty() || cfg.user.name.is_empty() {
                onboard::run_onboarding().await?;
            }
            let cfg = Config::load()?;
            run_services(&cfg, true).await?;
        }
    }

    Ok(())
}

async fn run_services(cfg: &Config, run_repl: bool) -> Result<()> {
    let db = Arc::new(Mutex::new(Database::open()?));
    let session_mgr = Arc::new(SessionManager::new(db.clone()));
    let provider = Arc::new(ProviderRegistry::from_config(cfg)?);

    // Bootstrap memory files
    MemoryStore::bootstrap_defaults(&cfg.agent.name, &cfg.user.name)?;

    // Agent registry
    let _agent_registry = AgentRegistry::load()?;

    let model = cfg.models.default.clone();
    let agent_name = cfg.agent.name.clone();

    // Start heartbeat
    Heartbeat::from_config(cfg).start(
        db.clone(),
        session_mgr.clone(),
        provider.clone(),
        model.clone(),
    );

    // Start web server in background
    let web_state = AppState {
        session_mgr: session_mgr.clone(),
        provider: provider.clone(),
        model: model.clone(),
        agent_name: agent_name.clone(),
    };
    let web = WebServer::from_config(cfg);
    let web_host = cfg.server.host.clone();
    let web_port = cfg.server.port;
    tokio::spawn(async move {
        if let Err(e) = web.run(web_state).await {
            tracing::error!("Web server error: {e}");
        }
    });

    // Open browser
    if cfg.server.port != 0 {
        let url = format!("http://{}:{}", web_host, web_port);
        let _ = open_browser(&url);
        println!("🌐 Web UI: {url}");
    }

    if run_repl {
        repl::run_repl(
            &agent_name,
            session_mgr,
            provider,
            &model,
        )
        .await?;
    } else {
        // Web-only: block forever
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/C", "start", url]).spawn()?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
