use crate::config::Config;
use crate::interaction;
use anyhow::{Context, Result};

pub async fn change() -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;

    if !config.is_configured() {
        anyhow::bail!("No configuration found. Run 'tt setup' first.");
    }

    println!("Current provider: {}", config.provider.display_name());
    println!("Current model: {}\n", config.default_model);

    let api_base = config.api_base();
    let new_model = interaction::select_model(
        config.provider,
        config.api_key.as_deref(),
        &api_base,
        Some(config.default_model.as_str()),
    )
    .await?;

    config.default_model = new_model;
    config.save().context("Failed to save configuration")?;

    println!(
        "\nDefault model updated to: {} ({})",
        config.default_model,
        config.provider.display_name()
    );

    Ok(())
}
