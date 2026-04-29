use serde::{Deserialize, Serialize};
use anyhow::Result;
use core::Config;
use orchestrator::{AgentManager, AgentState};

/// Actions for managing sub-agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AgentAction {
    Spawn { name: String, description: String, instructions: String },
    Delegate { agent: String, task: String },
    List,
    Status { name: String },
    Complete { name: String },
    GetContext { name: String },
}

pub struct AgentTool;

impl AgentTool {
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "agent",
            "description": "Manage sub-agents. Spawn new agents, delegate tasks, check status. \
                Use spawn to create a new sub-agent, delegate to give them work, list to see all agents.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["spawn", "delegate", "list", "status", "complete", "get_context"],
                        "description": "Action to perform"
                    },
                    "name": {
                        "type": "string",
                        "description": "Agent name (for spawn, status, complete, get_context)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Agent description (for spawn)"
                    },
                    "instructions": {
                        "type": "string",
                        "description": "Agent instructions (for spawn)"
                    },
                    "agent": {
                        "type": "string",
                        "description": "Agent name to delegate task to"
                    },
                    "task": {
                        "type": "string",
                        "description": "Task to delegate"
                    }
                },
                "required": ["action"]
            }
        })
    }

    /// Execute an agent action
    pub fn execute(action: AgentAction) -> Result<String> {
        // Use a simple in-memory manager (in production this would be stateful)
        match action {
            AgentAction::Spawn { name, description, instructions } => {
                let mut manager = AgentManager::new();
                match manager.spawn(&name, &description, &instructions) {
                    Ok(instance) => {
                        Ok(format!(
                            "✅ Created agent '{}'\nDescription: {}\n\nUse /agent delegate {} '<task>' to give work",
                            instance.name,
                            instance.config.description.as_deref().unwrap_or("No description"),
                            instance.name
                        ))
                    }
                    Err(e) => Ok(format!("❌ Failed to spawn agent: {}", e))
                }
            }

            AgentAction::Delegate { agent, task } => {
                let mut manager = AgentManager::new();
                match manager.delegate(&agent, &task) {
                    Ok(_) => Ok(format!("📝 Delegated to '{}': {}", agent, task)),
                    Err(e) => Ok(format!("❌ Failed to delegate: {}", e))
                }
            }

            AgentAction::List => {
                let manager = AgentManager::new();
                let agents = manager.list();
                if agents.is_empty() {
                    Ok("No sub-agents yet. Say 'spawn an agent called X who does Y' to create one.".to_string())
                } else {
                    let mut output = "🤖 Sub-agents:\n".to_string();
                    for a in agents {
                        let state = match a.state {
                            AgentState::Idle => "🟢 Idle",
                            AgentState::Working => "🔵 Working",
                            AgentState::Waiting => "🟡 Waiting",
                            AgentState::Completed => "✅ Done",
                        };
                        output.push_str(&format!("- {}: {}\n", a.name, state));
                        if let Some(task) = &a.current_task {
                            output.push_str(&format!("  → {}\n", task));
                        }
                    }
                    Ok(output)
                }
            }

            AgentAction::Status { name } => {
                let manager = AgentManager::new();
                match manager.get(&name) {
                    Some(a) => {
                        let state = match a.state {
                            AgentState::Idle => "Idle",
                            AgentState::Working => "Working on a task",
                            AgentState::Waiting => "Waiting",
                            AgentState::Completed => "Task completed",
                        };
                        Ok(format!(
                            "Agent: {}\nState: {}\nCreated: {}\nLast active: {}",
                            a.name, state, a.created_at, a.last_active
                        ))
                    }
                    None => Ok(format!("Agent '{}' not found", name))
                }
            }

            AgentAction::Complete { name } => {
                let mut manager = AgentManager::new();
                match manager.complete_task(&name) {
                    Ok(_) => Ok(format!("✅ Marked task as complete for agent '{}'", name)),
                    Err(e) => Ok(format!("❌ Failed: {}", e))
                }
            }

            AgentAction::GetContext { name } => {
                let manager = AgentManager::new();
                match manager.get_agent_context(&name) {
                    Ok(ctx) => Ok(format!("# Agent Context for {}\n\n{}", name, ctx)),
                    Err(e) => Ok(format!("❌ Error: {}", e))
                }
            }
        }
    }

    /// Parse an AgentAction from JSON tool call args
    pub fn parse_action(args: &serde_json::Value) -> Result<AgentAction> {
        let action = args["action"].as_str().unwrap_or("").to_string();
        
        match action.as_str() {
            "spawn" => Ok(AgentAction::Spawn {
                name: args["name"].as_str().unwrap_or("").to_string(),
                description: args["description"].as_str().unwrap_or("").to_string(),
                instructions: args["instructions"].as_str().unwrap_or("").to_string(),
            }),
            "delegate" => Ok(AgentAction::Delegate {
                agent: args["agent"].as_str().unwrap_or("").to_string(),
                task: args["task"].as_str().unwrap_or("").to_string(),
            }),
            "list" => Ok(AgentAction::List),
            "status" => Ok(AgentAction::Status {
                name: args["name"].as_str().unwrap_or("").to_string(),
            }),
            "complete" => Ok(AgentAction::Complete {
                name: args["name"].as_str().unwrap_or("").to_string(),
            }),
            "get_context" => Ok(AgentAction::GetContext {
                name: args["name"].as_str().unwrap_or("").to_string(),
            }),
            other => anyhow::bail!("Unknown agent action: {}", other),
        }
    }
}