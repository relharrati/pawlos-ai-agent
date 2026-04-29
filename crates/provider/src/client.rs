use std::pin::Pin;
use futures::Stream;
use anyhow::Result;
use async_trait::async_trait;
use crate::types::{ChatRequest, ChatResponse, StreamChunk};

pub type StreamResult = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Trait implemented by every provider backend
#[async_trait]
pub trait ProviderClient: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, req: ChatRequest) -> Result<StreamResult>;
}

/// OpenAI-compatible client (covers OpenAI, OpenRouter, Ollama, vLLM, etc.)
pub struct OpenAiCompatClient {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub http: reqwest::Client,
}

impl OpenAiCompatClient {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ProviderClient for OpenAiCompatClient {
    fn name(&self) -> &str { &self.name }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self.http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        parse_response(&resp)
    }

    async fn chat_stream(&self, req: ChatRequest) -> Result<StreamResult> {
        use futures::StreamExt;
        use bytes::Bytes;

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut stream_req = req;
        stream_req.stream = true;

        let resp = self.http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&stream_req)
            .send()
            .await?
            .error_for_status()?;

        let byte_stream = resp.bytes_stream();

        let stream = byte_stream.filter_map(|chunk| async move {
            let bytes: Bytes = chunk.ok()?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            // SSE format: lines starting with "data: "
            let mut last_chunk = None;
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { break; }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = val["choices"][0]["delta"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let finish = val["choices"][0]["finish_reason"].as_str().map(|s| s.to_string());
                        last_chunk = Some(Ok(StreamChunk { delta, tool_call_delta: None, finish_reason: finish }));
                    }
                }
            }
            last_chunk
        });

        Ok(Box::pin(stream))
    }
}

fn parse_response(val: &serde_json::Value) -> Result<ChatResponse> {
    let choice = &val["choices"][0];
    let message = &choice["message"];
    let content = message["content"].as_str().unwrap_or("").to_string();
    let finish_reason = choice["finish_reason"].as_str().unwrap_or("stop").to_string();

    // Parse tool calls if present
    let tool_calls = if let Some(arr) = message["tool_calls"].as_array() {
        arr.iter().filter_map(|tc| {
            Some(crate::types::LlmToolCall {
                id: tc["id"].as_str()?.to_string(),
                kind: tc["type"].as_str().unwrap_or("function").to_string(),
                function: crate::types::LlmFunction {
                    name: tc["function"]["name"].as_str()?.to_string(),
                    arguments: tc["function"]["arguments"].as_str().unwrap_or("{}").to_string(),
                },
            })
        }).collect()
    } else {
        Vec::new()
    };

    let usage = val.get("usage").and_then(|u| {
        Some(crate::types::Usage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        })
    });

    Ok(ChatResponse {
        id: val["id"].as_str().unwrap_or("").to_string(),
        model: val["model"].as_str().unwrap_or("").to_string(),
        content,
        tool_calls,
        finish_reason,
        usage,
    })
}
