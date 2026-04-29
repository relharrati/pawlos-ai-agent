use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use pawlos_core::types::Message;
use provider::registry::ProviderRegistry;
use provider::types::{ChatRequest, LlmMessage};
use prompt::builder::{PromptBuilder, builtin_personality};
use tools::executor::ToolExecutor;
use crate::session::SessionManager;

/// Run a single conversation turn: user message → LLM → tool calls → final response
/// Returns (assistant_content, used_tools: bool)
pub async fn run_turn(
    session_id: Uuid,
    user_message: String,
    agent_name: &str,
    session_mgr: &Arc<SessionManager>,
    provider: &Arc<ProviderRegistry>,
    model_spec: &str,
    allow_tools: bool,
) -> Result<(String, bool)> {
    // Add user message to session
    let user_msg = Message::user(&user_message);
    session_mgr.push_message(session_id, user_msg).await?;

    let session = session_mgr.get(session_id).await
        .ok_or_else(|| anyhow::anyhow!("Session {session_id} not found"))?;

    let prompt_builder = PromptBuilder::new(agent_name);
    let messages = prompt_builder.build_messages(&session.messages)?;

    let (provider_name, model_id) = ProviderRegistry::parse_model_spec(model_spec);
    let client = provider.get_client(provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{provider_name}' not configured"))?;

    let tools = if allow_tools {
        Some(
            ToolExecutor::all_definitions()
                .into_iter()
                .map(|t| core::types::ToolDefinition {
                    name: t["name"].as_str().unwrap_or("").to_string(),
                    description: t["description"].as_str().unwrap_or("").to_string(),
                    parameters: t["parameters"].clone(),
                })
                .collect()
        )
    } else {
        None
    };

    let req = ChatRequest {
        model: model_id.to_string(),
        messages,
        tools,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(4096),
    };

    let mut response = client.chat(req).await?;
    let mut used_tools = false;
    let mut tool_iterations = 0;
    const MAX_TOOL_ITERS: u32 = 8;

    // Agentic loop: keep executing tool calls until done or limit hit
    loop {
        if response.tool_calls.is_empty() || !allow_tools || tool_iterations >= MAX_TOOL_ITERS {
            break;
        }
        used_tools = true;
        tool_iterations += 1;

        // Add assistant turn with tool calls
        let assistant_msg = Message::assistant(&response.content);
        session_mgr.push_message(session_id, assistant_msg).await?;

        // Execute all tool calls
        for tc in &response.tool_calls {
            let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                .unwrap_or(serde_json::json!({}));
            let result = ToolExecutor::execute(&tc.function.name, &args).await
                .unwrap_or_else(|e| format!("Tool error: {e}"));

            let tool_result_msg = Message {
                id: uuid::Uuid::new_v4(),
                role: core::types::Role::Tool,
                content: result,
                tool_calls: None,
                tool_call_id: Some(tc.id.clone()),
                timestamp: chrono::Utc::now(),
            };
            session_mgr.push_message(session_id, tool_result_msg).await?;
        }

        // Re-fetch session and re-run
        let session = session_mgr.get(session_id).await
            .ok_or_else(|| anyhow::anyhow!("Session gone"))?;
        let messages = prompt_builder.build_messages(&session.messages)?;
        let req = ChatRequest {
            model: model_id.to_string(),
            messages,
            tools: if allow_tools {
                Some(ToolExecutor::all_definitions().into_iter().map(|t| core::types::ToolDefinition {
                    name: t["name"].as_str().unwrap_or("").to_string(),
                    description: t["description"].as_str().unwrap_or("").to_string(),
                    parameters: t["parameters"].clone(),
                }).collect())
            } else { None },
            stream: false,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };
        response = client.chat(req).await?;
    }

    // Persist final assistant message
    let assistant_msg = Message::assistant(&response.content);
    session_mgr.push_message(session_id, assistant_msg).await?;

    Ok((response.content, used_tools))
}
