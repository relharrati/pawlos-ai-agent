use anyhow::Result;
use colored::Colorize;
use crate::{MemoryCmd, SkillCmd, McpCmd};
use memory::store::{MemoryStore, StoreKind};
use tools::skill::Skill;
use pawlos_core::Config;

pub fn memory_cmd(action: MemoryCmd, store_str: &str, args: &[String]) -> Result<()> {
    let kind = StoreKind::from_str(store_str)
        .ok_or_else(|| anyhow::anyhow!("Unknown store: {store_str}"))?;
    let ms = MemoryStore::new(kind);

    match action {
        MemoryCmd::Read => {
            let rendered = ms.render_for_prompt()?;
            if rendered.is_empty() {
                println!("{}", "(empty)".dimmed());
            } else {
                println!("{rendered}");
            }
        }
        MemoryCmd::Add => {
            let content = args.join(" ");
            ms.add(&content)?;
            println!("{} Added to {store_str}.", "✓".green());
        }
        MemoryCmd::Remove => {
            let pattern = args.join(" ");
            let n = ms.remove(&pattern)?;
            println!("{} Removed {n} entries.", "✓".green());
        }
        MemoryCmd::Consolidate => {
            let n = ms.consolidate()?;
            println!("{} Consolidated: {n} removed.", "✓".green());
        }
    }
    Ok(())
}

pub fn skill_cmd(action: SkillCmd, name: Option<&str>) -> Result<()> {
    match action {
        SkillCmd::List => {
            let skills = Skill::list_all()?;
            if skills.is_empty() {
                println!("{}", "No skills found. Create one with `pawlos skill create <name>`.".dimmed());
            } else {
                for s in &skills {
                    println!("  • {}", s.cyan());
                }
            }
        }
        SkillCmd::Load => {
            let name = name.ok_or_else(|| anyhow::anyhow!("Provide a skill name"))?;
            let skill = Skill::load(name)?;
            println!("{}\n{}", format!("# {}: {}", skill.name, skill.description).bold(), skill.prompt);
        }
        SkillCmd::Create => {
            println!("{}", "Use the agent chat to create skills dynamically: \"Remember how to X, call it Y\"".dimmed());
        }
    }
    Ok(())
}

pub fn mcp_cmd(action: McpCmd) -> Result<()> {
    let cfg = Config::load()?;
    
    match action {
        McpCmd::List => {
            if cfg.mcp_servers.is_empty() {
                println!("{}", "No MCP servers configured.".dimmed());
                println!("\nAdd MCPs during onboarding or edit config.yaml manually.");
                println!("\nAvailable MCPs:");
                println!("  • github      - GitHub API (issues, PRs, repos)");
                println!("  • fetch       - Web content → markdown");
                println!("  • brave_search - Web search");
                println!("  • memory      - Knowledge graph");
                println!("  • git         - Git operations");
                println!("  • sqlite      - SQLite database");
                println!("  • playwright  - Browser automation");
                println!("  • context7    - Documentation lookup");
            } else {
                println!("{}", "Configured MCP servers:".bold());
                for (name, config) in &cfg.mcp_servers {
                    let cmd = config.get("command").and_then(|v| v.as_str()).unwrap_or("npx");
                    let args = config.get("args").and_then(|v| v.as_array())
                        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
                        .unwrap_or_default();
                    println!("  • {} → {} {}", name.cyan(), cmd, args.dimmed());
                }
            }
        }
        McpCmd::Add => {
            println!("{}", "To add an MCP server, edit ~/.pawlos/config.yaml".dimmed());
            println!("Or re-run onboarding: pawlos onboard");
        }
        McpCmd::Remove => {
            println!("{}", "To remove an MCP server, edit ~/.pawlos/config.yaml".dimmed());
        }
    }
    Ok(())
}
