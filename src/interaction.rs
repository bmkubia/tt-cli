use crate::config::{ProviderKind, SystemPromptStyle};
use crate::models;
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};

pub fn select_provider(current: ProviderKind) -> Result<ProviderKind> {
    let providers = [
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::OpenRouter,
        ProviderKind::LmStudio,
    ];

    let labels: Vec<&str> = providers.iter().map(|p| p.display_name()).collect();
    let default_index = providers.iter().position(|p| *p == current).unwrap_or(0);

    let index = Select::new()
        .with_prompt("Select model provider")
        .items(&labels)
        .default(default_index)
        .interact()
        .context("Failed to read provider selection")?;

    Ok(providers[index])
}

pub fn prompt_api_key(provider: ProviderKind, existing: Option<&str>) -> Result<String> {
    loop {
        let mut input = Input::new();
        input = input.with_prompt(format!("Enter your {} API key", provider.display_name()));
        if let Some(value) = existing {
            input = input.with_initial_text(value);
        }

        let key: String = input.interact_text().context("Failed to read API key")?;
        let trimmed = key.trim();

        if trimmed.is_empty() {
            println!("API key cannot be empty. Please try again.");
            continue;
        }

        return Ok(trimmed.to_string());
    }
}

pub fn prompt_lmstudio_base(preset: Option<&str>) -> Result<String> {
    loop {
        let mut input = Input::new();
        input = input.with_prompt("Enter LM Studio API base (e.g., http://localhost:1234/v1)");
        let default_base = preset
            .filter(|base| !base.trim().is_empty())
            .unwrap_or(ProviderKind::LmStudio.default_api_base());
        input = input.with_initial_text(default_base);

        let value: String = input
            .interact_text()
            .context("Failed to read LM Studio API base")?;
        let trimmed = value.trim();

        if trimmed.is_empty() {
            println!("API base cannot be empty. Please try again.");
            continue;
        }

        return Ok(trimmed.trim_end_matches('/').to_string());
    }
}

pub async fn select_model(
    provider: ProviderKind,
    api_key: Option<&str>,
    api_base: &str,
    current_model: Option<&str>,
) -> Result<String> {
    match models::fetch_available_models(provider, api_key, api_base).await {
        Ok(models) if !models.is_empty() => {
            select_model_from_list(provider, &models, current_model)
        }
        Ok(_) => {
            println!(
                "No models returned by {}. Please enter a model name manually.",
                provider.display_name()
            );
            prompt_custom_model(provider, current_model)
        }
        Err(err) => {
            eprintln!(
                "Warning: could not fetch models from {} ({err}). Enter a model name manually.",
                provider.display_name()
            );
            prompt_custom_model(provider, current_model)
        }
    }
}

fn select_model_from_list(
    provider: ProviderKind,
    models: &[String],
    current_model: Option<&str>,
) -> Result<String> {
    if models.is_empty() {
        return prompt_custom_model(provider, current_model);
    }

    let mut options: Vec<String> = models.to_vec();
    options.sort();
    options.dedup();

    let custom_index = options.len();
    let mut display = options.clone();
    display.push("Other (enter manually)".to_string());

    let default_index = current_model
        .and_then(|current| options.iter().position(|m| m == current))
        .unwrap_or(0)
        .min(display.len().saturating_sub(1));

    let selection = Select::new()
        .with_prompt(format!(
            "Select your default {} model",
            provider.display_name()
        ))
        .items(&display)
        .default(default_index)
        .interact()
        .context("Failed to read model selection")?;

    if selection == custom_index {
        prompt_custom_model(provider, current_model)
    } else {
        Ok(options[selection].clone())
    }
}

fn prompt_custom_model(provider: ProviderKind, current_model: Option<&str>) -> Result<String> {
    loop {
        let mut input = Input::new();
        input = input.with_prompt(format!(
            "Enter the {} model identifier",
            provider.display_name()
        ));
        if let Some(value) = current_model {
            input = input.with_initial_text(value);
        }

        let model: String = input.interact_text().context("Failed to read model name")?;
        let trimmed = model.trim();

        if trimmed.is_empty() {
            println!("Model name cannot be empty. Please try again.");
            continue;
        }

        return Ok(trimmed.to_string());
    }
}

pub fn prompt_toggle(prompt: &str, default: bool) -> Result<bool> {
    Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to read option")
}

pub fn select_prompt_style(current: SystemPromptStyle) -> Result<SystemPromptStyle> {
    let styles = [
        (
            SystemPromptStyle::Command,
            "Command mode — terse answers like `brew upgrade`.",
        ),
        (
            SystemPromptStyle::Sidekick,
            "Sidekick mode — quick context before or around the command.",
        ),
        (
            SystemPromptStyle::Exploration,
            "Exploration mode — deeper explanations with the final command at the end.",
        ),
    ];

    let labels: Vec<&str> = styles.iter().map(|(_, desc)| *desc).collect();
    let default_index = styles
        .iter()
        .position(|(style, _)| *style == current)
        .unwrap_or(0);

    let selection = Select::new()
        .with_prompt("System prompt style")
        .items(&labels)
        .default(default_index)
        .interact()
        .context("Failed to read prompt style selection")?;

    Ok(styles[selection].0)
}
