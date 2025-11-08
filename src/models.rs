use crate::config::ProviderKind;
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

pub async fn fetch_available_models(
    provider: ProviderKind,
    api_key: Option<&str>,
    api_base: &str,
) -> Result<Vec<String>> {
    match provider {
        ProviderKind::Anthropic => {
            let key =
                api_key.ok_or_else(|| anyhow!("Anthropic API key required to list models"))?;
            fetch_anthropic_models(key, api_base).await
        }
        ProviderKind::OpenAi => {
            let key = api_key.ok_or_else(|| anyhow!("OpenAI API key required to list models"))?;
            fetch_openai_style_models(Some(key), api_base, provider).await
        }
        ProviderKind::OpenRouter => {
            let key =
                api_key.ok_or_else(|| anyhow!("OpenRouter API key required to list models"))?;
            fetch_openai_style_models(Some(key), api_base, provider).await
        }
        ProviderKind::LmStudio => fetch_openai_style_models(None, api_base, provider).await,
    }
}

async fn fetch_anthropic_models(api_key: &str, api_base: &str) -> Result<Vec<String>> {
    let client = Client::new();
    let base = api_base.trim_end_matches('/');
    let url = format!("{base}/models");

    let response = client
        .get(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .context("Failed to call Anthropic models endpoint")?
        .error_for_status()
        .context("Anthropic model request returned an error")?;

    let payload: AnthropicModelsResponse = response
        .json()
        .await
        .context("Failed to parse Anthropic model response")?;

    Ok(normalize_models(
        payload.data.into_iter().map(|entry| entry.id).collect(),
        ProviderKind::Anthropic,
    ))
}

async fn fetch_openai_style_models(
    api_key: Option<&str>,
    api_base: &str,
    provider: ProviderKind,
) -> Result<Vec<String>> {
    let client = Client::new();
    let base = api_base.trim_end_matches('/');
    let url = format!("{base}/models");

    let mut request = client.get(url);
    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = request
        .send()
        .await
        .context("Failed to call OpenAI-compatible models endpoint")?
        .error_for_status()
        .context("OpenAI-compatible model request returned an error")?;

    let payload: OpenAiModelsResponse = response
        .json()
        .await
        .context("Failed to parse OpenAI-compatible model response")?;

    Ok(normalize_models(
        payload.data.into_iter().map(|entry| entry.id).collect(),
        provider,
    ))
}

pub fn normalize_models(mut models: Vec<String>, provider: ProviderKind) -> Vec<String> {
    if matches!(provider, ProviderKind::OpenAi) {
        models.retain(|id| is_chat_model_id(id));
    }

    models.sort();
    models.dedup();
    models
}

fn is_chat_model_id(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    id.contains("gpt") || id.starts_with("o1") || id.starts_with("o3") || id.contains("omni")
}
