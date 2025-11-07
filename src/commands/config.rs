use crate::config::Config;
use anyhow::{Context, Result};

pub fn show() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    if !config.is_configured() {
        println!("No configuration found. Run 'tt setup' to configure.");
        return Ok(());
    }

    let api_key_display = if config.provider.requires_api_key() {
        config
            .api_key_preview()
            .unwrap_or_else(|| "<not configured>".to_string())
    } else {
        "n/a (local provider)".to_string()
    };

    println!("Current configuration:");
    println!("  Provider: {}", config.provider.display_name());
    println!("  API Base: {}", config.api_base());
    println!("  Default Model: {}", config.default_model);
    println!("  API Key: {}", api_key_display);
    println!("\nConfig location: {}", Config::config_path()?.display());

    Ok(())
}
