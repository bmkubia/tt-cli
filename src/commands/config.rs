use crate::config::Config;
use crate::ui;
use anyhow::{Context, Result};

pub fn show() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;
    let config_path = Config::config_path().context("Failed to determine config path")?;

    let api_key_display = if config.provider.requires_api_key() {
        config
            .api_key_preview()
            .unwrap_or_else(|| "<not configured>".to_string())
    } else {
        "n/a (local provider)".to_string()
    };

    let default_model = if config.default_model.trim().is_empty() {
        "<not set>".to_string()
    } else {
        config.default_model.clone()
    };

    let status = if config.is_configured() {
        "Ready".to_string()
    } else {
        "Not configured".to_string()
    };

    let mut rows = vec![
        ("Status".to_string(), status),
        (
            "Provider".to_string(),
            config.provider.display_name().to_string(),
        ),
        ("API Base".to_string(), config.api_base()),
        ("Default Model".to_string(), default_model),
        ("API Key".to_string(), api_key_display),
        ("Config File".to_string(), config_path.display().to_string()),
    ];

    if !config.is_configured() {
        rows.push((
            "Next Step".to_string(),
            "Run tt setup to finish configuring".to_string(),
        ));
    }

    ui::print_info_card("Current Configuration", rows);

    if !config.is_configured() {
        println!("No configuration found. Run 'tt setup' to configure.");
    }

    Ok(())
}
