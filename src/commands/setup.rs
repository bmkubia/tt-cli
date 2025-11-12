use crate::config::{Config, ProviderKind};
use crate::interaction;
use anyhow::{Context, Result};

pub async fn run() -> Result<()> {
    println!("Setting up tt CLI...\n");
    let mut config = Config::load().context("Failed to load configuration")?;

    let provider = interaction::select_provider(config.provider)?;
    let api_base = resolve_api_base(provider, config.api_base_override.as_deref())?;

    let api_key = if provider.requires_api_key() {
        Some(interaction::prompt_api_key(
            provider,
            config.api_key.as_deref().filter(|k| !k.is_empty()),
        )?)
    } else {
        None
    };

    let current_model = if config.default_model.trim().is_empty() {
        None
    } else {
        Some(config.default_model.as_str())
    };

    let default_model =
        interaction::select_model(provider, api_key.as_deref(), &api_base, current_model).await?;

    let show_header =
        interaction::prompt_toggle("Show streaming response header", config.show_header)?;
    let show_model_in_header = if show_header {
        interaction::prompt_toggle("Show model name in header", config.show_model_in_header)?
    } else {
        false
    };
    let prompt_style = interaction::select_prompt_style(config.system_prompt_style)?;

    config.provider = provider;
    config.api_key = api_key;
    config.default_model = default_model;
    config.api_base_override = if provider == ProviderKind::LmStudio {
        Some(api_base)
    } else {
        None
    };
    config.show_header = show_header;
    config.show_model_in_header = show_model_in_header;
    config.system_prompt_style = prompt_style;

    config.save().context("Failed to save configuration")?;

    println!("\nConfiguration saved successfully!");
    println!("You can now use: tt <your question>");

    Ok(())
}

fn resolve_api_base(provider: ProviderKind, preset: Option<&str>) -> Result<String> {
    if provider == ProviderKind::LmStudio {
        interaction::prompt_lmstudio_base(
            preset
                .filter(|base| !base.trim().is_empty())
                .or(Some(provider.default_api_base())),
        )
    } else {
        Ok(provider.default_api_base().to_string())
    }
}
