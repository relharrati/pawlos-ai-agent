use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use core::{Config, types::AgentConfig};

/// Agent state for tracking what an agent is doing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentState {
    Idle,
    Working,
    Waiting,
    Completed,
}

/// A running sub-agent instance
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInstance {
    pub name: String,
    pub config: AgentConfig,
    pub state: AgentState,
    pub current_task: Option<String>,
    pub created_at: String,
    pub last_active: String,
}

impl AgentInstance {
    pub fn new(name: String, config: AgentConfig) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name,
            config,
            state: AgentState::Idle,
            current_task: None,
            created_at: now.clone(),
            last_active: now,
        }
    }
}

/// Agent manager - handles spawning, delegating, and managing sub-agents
pub struct AgentManager {
    instances: HashMap<String, AgentInstance>,
}

impl AgentManager {
    /// Create a new agent manager and load existing agents
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Spawn a new sub-agent from conversation
    /// "Spawn an agent called X who does Y"
    pub fn spawn(&mut self, name: &str, description: &str, instructions: &str) -> Result<AgentInstance> {
        // Create the agent config
        let cfg = AgentConfig {
            name: name.to_string(),
            description: Some(description.to_string()),
            personality_overlay: None,
            allowed_tools: None,
            model: None,
            parent: Some("pawlos".to_string()),
        };

        // Create agent directory structure
        let agent_dir = Config::agents_dir().join(name);
        std::fs::create_dir_all(&agent_dir)?;
        std::fs::create_dir_all(agent_dir.join("skills"))?;
        std::fs::create_dir_all(agent_dir.join("tasks").join("done"))?;
        std::fs::create_dir_all(agent_dir.join("tasks").join("in_progress"))?;
        std::fs::create_dir_all(agent_dir.join("tasks").join("pending"))?;
        std::fs::create_dir_all(agent_dir.join("logs"))?;

        // Write AGENT.md
        let agent_md = format!(
r#"# AGENT.md - {name}

## Identity
**Name:** {name}
**Created:** {created}

## Role
{description}

## Instructions
{instructions}

## Parent Agent
- Main agent: pawlos

## Tasks
- (Managed via tasks/ folder)

---
_Last updated: {updated}_
"#,
            name = name,
            description = description,
            instructions = instructions,
            created = chrono::Local::now().format("%Y-%m-%d %H:%M"),
            updated = chrono::Local::now().format("%Y-%m-%d %H:%M")
        );
        std::fs::write(agent_dir.join("AGENT.md"), agent_md)?;

        // Save config
        let yaml = serde_yaml::to_string(&cfg)?;
        std::fs::write(agent_dir.join("agent.yaml"), yaml)?;

        // Create instance
        let instance = AgentInstance::new(name.to_string(), cfg);
        self.instances.insert(name.to_string(), instance.clone());

        Ok(instance)
    }

    /// Delegate a task to an agent
    pub fn delegate(&mut self, agent_name: &str, task: &str) -> Result<()> {
        if let Some(instance) = self.instances.get_mut(agent_name) {
            instance.state = AgentState::Working;
            instance.current_task = Some(task.to_string());
            instance.last_active = chrono::Utc::now().to_rfc3339();

            // Write to pending tasks
            let task_file = Config::agents_dir()
                .join(agent_name)
                .join("tasks")
                .join("pending")
                .join(format!("{}.md", uuid::Uuid::new_v4()));
            
            std::fs::create_dir_all(task_file.parent().unwrap())?;
            std::fs::write(&task_file, format!(
                "# Task\n\n{}\n\n## Status\n- Assigned: {}\n- Agent: {}\n",
                task,
                chrono::Local::now().format("%Y-%m-%d %H:%M"),
                agent_name
            ))?;

            Ok(())
        } else {
            anyhow::bail!("Agent '{}' not found", agent_name)
        }
    }

    /// Get agent by name
    pub fn get(&self, name: &str) -> Option<&AgentInstance> {
        self.instances.get(name)
    }

    /// List all active agents
    pub fn list(&self) -> Vec<&AgentInstance> {
        self.instances.values().collect()
    }

    /// Complete a task for an agent
    pub fn complete_task(&mut self, agent_name: &str) -> Result<()> {
        if let Some(instance) = self.instances.get_mut(agent_name) {
            if let Some(task) = instance.current_task.take() {
                // Move from in_progress to done
                let done_dir = Config::agents_dir()
                    .join(agent_name)
                    .join("tasks")
                    .join("done");
                std::fs::create_dir_all(&done_dir)?;
                
                let done_file = done_dir.join(format!(
                    "{}.md",
                    chrono::Local::now().format("%Y-%m-%d_%H-%M")
                ));
                std::fs::write(&done_file, format!(
                    "# Completed Task\n\n{}\n\n## Completed: {}",
                    task,
                    chrono::Local::now().format("%Y-%m-%d %H:%M")
                ))?;
            }
            instance.state = AgentState::Idle;
            instance.last_active = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            anyhow::bail!("Agent '{}' not found", agent_name)
        }
    }

    /// Get agent's AGENT.md content for context injection
    pub fn get_agent_context(&self, agent_name: &str) -> Result<String> {
        let path = Config::agents_dir()
            .join(agent_name)
            .join("AGENT.md");
        
        if path.exists() {
            Ok(std::fs::read_to_string(&path)?)
        } else {
            Ok(format!("# Agent: {}\n\n(No instructions set)", agent_name))
        }
    }

    /// Get all pending tasks for an agent
    pub fn get_pending_tasks(&self, agent_name: &str) -> Result<Vec<String>> {
        let tasks_dir = Config::agents_dir()
            .join(agent_name)
            .join("tasks")
            .join("pending");
        
        if !tasks_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for entry in std::fs::read_dir(&tasks_dir)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                let content = std::fs::read_to_string(entry.path())?;
                // Extract task content (between # Task and ##)
                if let Some(task) = content.find("# Task\n\n") {
                    let start = task + 8;
                    if let Some(end) = content[start..].find("\n\n##") {
                        tasks.push(content[start..start+end].to_string());
                    }
                }
            }
        }
        Ok(tasks)
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}