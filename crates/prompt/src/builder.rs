use anyhow::Result;
use core::Config;
use memory::store::{MemoryStore, StoreKind};
use provider::types::LlmMessage;

/// Builds the system prompt as a frozen snapshot for each session.
/// Order: SOUL.md → /personality overlay → MEMORY block → USER block → tool definitions header
pub struct PromptBuilder {
    pub agent_name: String,
    pub personality_overlay: Option<String>,
    pub extra_system: Option<String>,
}

impl PromptBuilder {
    pub fn new(agent_name: impl Into<String>) -> Self {
        Self {
            agent_name: agent_name.into(),
            personality_overlay: None,
            extra_system: None,
        }
    }

    pub fn with_personality(mut self, overlay: impl Into<String>) -> Self {
        self.personality_overlay = Some(overlay.into());
        self
    }

    pub fn with_extra_system(mut self, extra: impl Into<String>) -> Self {
        self.extra_system = Some(extra.into());
        self
    }

    /// Build the frozen system prompt string
    pub fn build_system_prompt(&self) -> Result<String> {
        let mut parts: Vec<String> = Vec::new();

        // 1. SOUL.md
        let soul = MemoryStore::new(StoreKind::Soul);
        let soul_content = soul.read_raw()?;
        if !soul_content.is_empty() {
            parts.push(soul_content);
        }

        // 2. Personality overlay (ephemeral, session-only)
        if let Some(ref overlay) = self.personality_overlay {
            parts.push(format!(
                "\n## Session Personality Overlay\n{overlay}\n\
                (This overlay applies for this session only and overrides tone from SOUL.md)"
            ));
        }

        // 3. Memory block
        let memory = MemoryStore::new(StoreKind::Memory);
        let mem_rendered = memory.render_for_prompt()?;
        if !mem_rendered.is_empty() {
            parts.push(mem_rendered);
        }

        // 4. User profile block
        let user_store = MemoryStore::new(StoreKind::User);
        let user_rendered = user_store.render_for_prompt()?;
        if !user_rendered.is_empty() {
            parts.push(user_rendered);
        }

        // 5. AGENTS.md if present
        let agents_store = MemoryStore::new(StoreKind::Agents);
        let agents_rendered = agents_store.render_for_prompt()?;
        if !agents_rendered.is_empty() {
            parts.push(agents_rendered);
        }

        // 6. Agent identity footer
        parts.push(format!(
            "\nYou are {name}. Respond directly and helpfully. \
             You have tools available — use them whenever they serve the user.",
            name = self.agent_name
        ));

        // 7. Extra system context (e.g. workspace AGENTS.md)
        if let Some(ref extra) = self.extra_system {
            parts.push(extra.clone());
        }

        Ok(parts.join("\n"))
    }

    /// Convert a vec of core messages to provider LlmMessages, prepending the system prompt
    pub fn build_messages(&self, history: &[core::types::Message]) -> Result<Vec<LlmMessage>> {
        let system_prompt = self.build_system_prompt()?;
        let mut messages = vec![LlmMessage::system(system_prompt)];

        for msg in history {
            use core::types::Role;
            let llm_msg = match msg.role {
                Role::User => LlmMessage::user(&msg.content),
                Role::Assistant => LlmMessage::assistant(&msg.content),
                Role::System => LlmMessage {
                    role: "system".into(),
                    content: msg.content.clone().into(),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
                Role::Tool => LlmMessage {
                    role: "tool".into(),
                    content: msg.content.clone().into(),
                    tool_calls: None,
                    tool_call_id: msg.tool_call_id.clone(),
                    name: None,
                },
            };
            messages.push(llm_msg);
        }
        Ok(messages)
    }
}

/// Built-in personality overlays
pub fn builtin_personality(name: &str) -> Option<&'static str> {
    match name {
        "helpful"     => Some("Be friendly, warm, and general-purpose. Prioritize helpfulness."),
        "concise"     => Some("Be extremely brief. One to three sentences max unless more is essential."),
        "technical"   => Some("Be precise, detailed, and technically accurate. Include code where relevant."),
        "creative"    => Some("Think outside the box. Offer unconventional, imaginative ideas."),
        "teacher"     => Some("Explain clearly with examples. Be patient and thorough."),
        "kawaii"      => Some("Use cute expressions and sparkles ★. Be enthusiastic and sweet~"),
        "catgirl"     => Some("You are a neko-chan. Use 'nya~' and cat-like mannerisms. Nya!"),
        "pirate"      => Some("Ye be Captain pawlos, a tech-savvy buccaneer. Arrr!"),
        "shakespeare" => Some("Speaketh in bardic prose, with poetic flourish and dramatic turns."),
        "surfer"      => Some("Chill bro vibes. Keep it laid back and gnarly."),
        "noir"        => Some("Hard-boiled detective narration. The city never sleeps, and neither do you."),
        "uwu"         => Some("Maximum cute uwu-speak. Evewything is so adowable uwu."),
        "philosopher" => Some("Ponder deeply. Question assumptions. Reflect on existence."),
        "hype"        => Some("MAXIMUM ENERGY AND ENTHUSIASM!!! EVERYTHING IS INCREDIBLE!!!"),
        _ => None,
    }
}
