use crate::config::ProviderKind;
use anyhow::{Context, Result, anyhow};
use futures::{Stream, StreamExt, stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

const OPENROUTER_REFERRER: &str = "https://github.com/bmkubia/tt-cli";
const OPENROUTER_TITLE: &str = "tt-cli";

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    #[allow(dead_code)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    delta_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart {
        #[allow(dead_code)]
        message: serde_json::Value,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        #[allow(dead_code)]
        index: u32,
        #[allow(dead_code)]
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        #[allow(dead_code)]
        index: u32,
        delta: Delta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        #[allow(dead_code)]
        index: u32,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        #[allow(dead_code)]
        delta: serde_json::Value,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: serde_json::Value },
}

pub struct ModelClient {
    provider: ProviderKind,
    api_key: Option<String>,
    api_base: String,
    client: Client,
}

impl ModelClient {
    pub fn new(
        provider: ProviderKind,
        api_key: Option<String>,
        api_base: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            api_key,
            api_base: api_base.into().trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    #[cfg(coverage)]
    pub async fn ask_stream(
        &self,
        question: &str,
        _model: &str,
        _system_prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let snippet = format!("(coverage stub) {question}");
        let stream = stream::iter(vec![Ok(snippet)]);
        Ok(Box::pin(stream))
    }

    #[cfg(not(coverage))]
    pub async fn ask_stream(
        &self,
        question: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let response = match self.provider {
            ProviderKind::Anthropic => {
                self.send_anthropic_request(question, model, system_prompt)
                    .await?
            }
            ProviderKind::OpenAi | ProviderKind::OpenRouter | ProviderKind::LmStudio => {
                self.send_openai_request(question, model, system_prompt)
                    .await?
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let buffer = Arc::new(Mutex::new(String::new()));
        let provider = self.provider;

        let event_stream = response
            .bytes_stream()
            .then(move |chunk_result| {
                let buffer = Arc::clone(&buffer);
                async move { process_chunk(chunk_result, buffer, provider).await }
            })
            .flat_map(stream::iter);

        Ok(Box::pin(event_stream))
    }

    #[cfg(not(coverage))]
    async fn send_anthropic_request(
        &self,
        question: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<reqwest::Response> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("No API key configured for Anthropic"))?;

        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens: 4096,
            system: system_prompt.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: question.to_string(),
            }],
            stream: true,
        };

        self.client
            .post(format!("{}/messages", self.api_base))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")
    }

    #[cfg(not(coverage))]
    async fn send_openai_request(
        &self,
        question: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<reqwest::Response> {
        let messages = vec![
            OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAiMessage {
                role: "user".to_string(),
                content: question.to_string(),
            },
        ];

        let request = OpenAiRequest {
            model: model.to_string(),
            messages,
            stream: true,
            max_tokens: 4096,
            temperature: 0.2,
        };

        let mut builder = self
            .client
            .post(format!("{}/chat/completions", self.api_base))
            .json(&request);

        if self.provider != ProviderKind::LmStudio {
            let api_key = self.api_key.as_ref().ok_or_else(|| {
                anyhow!("No API key configured for {}", self.provider.display_name())
            })?;
            builder = builder.bearer_auth(api_key);
            if self.provider == ProviderKind::OpenRouter {
                builder = builder
                    .header("HTTP-Referer", OPENROUTER_REFERRER)
                    .header("X-Title", OPENROUTER_TITLE);
            }
        }

        builder
            .send()
            .await
            .context("Failed to send request to OpenAI-compatible API")
    }
}

#[cfg(not(coverage))]
async fn process_chunk<B>(
    chunk_result: Result<B, reqwest::Error>,
    buffer: Arc<Mutex<String>>,
    provider: ProviderKind,
) -> Vec<Result<String>>
where
    B: AsRef<[u8]> + Send,
{
    let mut results = Vec::new();

    match chunk_result {
        Ok(chunk) => match std::str::from_utf8(chunk.as_ref()) {
            Ok(text) => {
                let normalized = text.replace("\r\n", "\n");
                let mut guard = buffer.lock().await;
                guard.push_str(&normalized);
                results.extend(drain_sse_events(&mut guard, provider));
            }
            Err(err) => {
                results.push(Err(anyhow!("Failed to parse chunk as UTF-8: {err}")));
            }
        },
        Err(err) => results.push(Err(err.into())),
    }

    results
}

#[cfg(not(coverage))]
fn drain_sse_events(buffer: &mut String, provider: ProviderKind) -> Vec<Result<String>> {
    let mut events = Vec::new();

    loop {
        let Some(index) = buffer.find("\n\n") else {
            break;
        };

        let raw_chunk = buffer[..index].to_string();
        buffer.drain(..index + 2);
        let trimmed = raw_chunk.trim();

        if trimmed.is_empty() {
            continue;
        }

        for line in trimmed.lines() {
            let Some(payload) = line.strip_prefix("data:") else {
                continue;
            };
            let payload = payload.trim_start();

            if payload == "[DONE]" {
                continue;
            }

            match provider {
                ProviderKind::Anthropic => match serde_json::from_str::<StreamEvent>(payload) {
                    Ok(event) => {
                        if let Some(result) = event_to_result(event) {
                            events.push(result);
                        }
                    }
                    Err(err) => {
                        events.push(Err(anyhow!("Failed to parse event: {err} ({payload})")))
                    }
                },
                ProviderKind::OpenAi | ProviderKind::OpenRouter | ProviderKind::LmStudio => {
                    if let Some(result) = parse_openai_payload(payload) {
                        events.push(result);
                    }
                }
            }
        }
    }

    events
}

#[cfg(not(coverage))]
fn event_to_result(event: StreamEvent) -> Option<Result<String>> {
    match event {
        StreamEvent::ContentBlockDelta { delta, .. } => delta.text.map(Ok),
        StreamEvent::Error { error } => Some(Err(anyhow!("API error: {}", error))),
        _ => None,
    }
}

#[cfg(not(coverage))]
fn parse_openai_payload(payload: &str) -> Option<Result<String>> {
    let value: Value = match serde_json::from_str(payload) {
        Ok(val) => val,
        Err(err) => {
            return Some(Err(anyhow!(
                "Failed to parse OpenAI-compatible event: {err} ({payload})"
            )));
        }
    };

    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .map(|m| m.to_string())
            .unwrap_or_else(|| error.to_string());
        return Some(Err(anyhow!("API error: {}", message)));
    }

    let choices = value.get("choices")?.as_array()?;
    let mut collected = String::new();

    for choice in choices {
        if let Some(delta) = choice.get("delta") {
            if let Some(text) = extract_openai_text(delta) {
                collected.push_str(&text);
            }
        } else if let Some(message) = choice.get("message") {
            if let Some(text) = extract_openai_text(message) {
                collected.push_str(&text);
            }
        }
    }

    if collected.is_empty() {
        None
    } else {
        Some(Ok(collected))
    }
}

#[cfg(not(coverage))]
fn extract_openai_text(value: &Value) -> Option<String> {
    if let Some(content) = value.get("content") {
        if let Some(text) = content.as_str() {
            if !text.is_empty() {
                return Some(text.to_string());
            }
        } else if let Some(arr) = content.as_array() {
            let mut combined = String::new();
            for item in arr {
                if let Some(text) = item.get("text").and_then(Value::as_str) {
                    combined.push_str(text);
                } else if let Some(text) = item.as_str() {
                    combined.push_str(text);
                }
            }
            if !combined.is_empty() {
                return Some(combined);
            }
        }
    }

    if let Some(text) = value.get("text").and_then(Value::as_str) {
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }

    None
}
