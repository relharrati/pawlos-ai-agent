use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use uuid::Uuid;

use crate::ui::{self, colors, quotes, responses};
use orchestrator::SessionManager;
use orchestrator::turn::run_turn;
use prompt::builder::builtin_personality;
use provider::registry::ProviderRegistry;

pub async fn run_repl(
    agent_name: &str,
    session_mgr: Arc<SessionManager>,
    provider: Arc<ProviderRegistry>,
    model: &str,
) -> Result<()> {
    // Show branded welcome
    ui::print_welcome(agent_name);
    
    println!(
        "{}{}Commands:{} /personality /model /skill /memory /help /exit",
        colors::DIM, colors::BRIGHT_CYAN, colors::RESET
    );
    println!("{}", "─".repeat(54).dimmed());

    let session_id = session_mgr.get_or_create(agent_name).await?;

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut current_model = model.to_string();

    loop {
        // Prompt - branded
        let prompt_str = ui::prompt(agent_name);
        print!("\n{} ", format!("{}you{}", colors::BRIGHT_BLUE, colors::BOLD));
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { break; } // EOF / Ctrl+D
        let input = line.trim().to_string();
        if input.is_empty() { continue; }

        // Slash commands
        if input == "/exit" || input == "/quit" { break; }

        if input.starts_with("/personality ") {
            let name = input["/personality ".len()..].trim();
            if let Some(desc) = builtin_personality(name) {
                ui::success(&format!("Personality switched to {name}: {desc}"));
            } else {
                ui::warn(&format!("Unknown personality '{name}'. Check config.yaml for custom ones."));
            }
            continue;
        }

        if input.starts_with("/model ") {
            current_model = input["/model ".len()..].trim().to_string();
            ui::success(&format!("Model switched to {current_model}"));
            continue;
        }

        if input.starts_with("/skill ") {
            let name = input["/skill ".len()..].trim();
            match tools::skill::Skill::load(name) {
                Ok(skill) => {
                    println!("\n{}{}{}:{}{}", colors::BRIGHT_GREEN, colors::BOLD, skill.name, colors::RESET);
                    println!("{}{}", colors::DIM, skill.description);
                    println!("\n{}", skill.prompt);
                }
                Err(e) => ui::error(&format!("{e}")),
            }
            continue;
        }

        if input == "/memory" || input == "/memory read" {
            for kind in [
                memory::store::StoreKind::Memory,
                memory::store::StoreKind::User,
            ] {
                let store = memory::store::MemoryStore::new(kind);
                let rendered = store.render_for_prompt().unwrap_or_default();
                if !rendered.is_empty() { println!("{rendered}"); }
            }
            continue;
        }

        if input == "/help" || input == "/?" {
            println!();
            println!("{}{}━━━ Available Commands ━━━{}", colors::CYAN, colors::BOLD);
            println!();
            println!("{}  /personality <name>   {}Switch conversation tone", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /model <name>         {}Change LLM model", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /skill <name>         {}Load a skill", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /memory               {}View memory stores", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /mcp                  {}List MCP servers", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /help                 {}Show this help", colors::BRIGHT_CYAN, colors::DIM);
            println!("{}  /exit                 {}Exit the chat", colors::BRIGHT_CYAN, colors::DIM);
            println!();
            println!("{}{}━━━━━━━━━━━━━━━━━━━━━━━{}", colors::DIM, colors::BRIGHT_CYAN);
            println!();
            continue;
        }

        // Normal message - branded agent response
        print!("\n{} ", format!("{}{}>{}", colors::BRIGHT_MAGENTA, colors::BOLD, agent_name));
        use std::io::Write;
        std::io::stdout().flush()?;

        match run_turn(
            session_id,
            input,
            agent_name,
            &session_mgr,
            &provider,
            &current_model,
            true,
        )
        .await
        {
            Ok((response, used_tools)) => {
                if used_tools {
                    print!("{} ", format!("{}{}[tools]{}", colors::DIM, colors::BRIGHT_YELLOW, colors::RESET));
                }
                // Random fun thought prefix
                print!("\n{}", responses::prefix());
                println!(" {}", response);
            }
            Err(e) => {
                ui::error(&format!("{e}"));
            }
        }
    }

    println!();
    ui::banner::print();
    println!("{}{}Goodbye! 👋{}", colors::DIM, colors::BRIGHT_CYAN, colors::RESET);
    Ok(())
}
