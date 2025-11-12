use crate::config::Config;
use crate::interaction;
use crate::ui;
use anyhow::{Context, Result};

pub async fn change() -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;

    if !config.is_configured() {
        anyhow::bail!("No configuration found. Run 'tt setup' first.");
    }

    let provider_display = config.provider.display_name();
    let previous_model = config.default_model.clone();

    ui::print_info_card(
        "Current Default",
        vec![
            ("Provider".to_string(), provider_display.to_string()),
            ("Model".to_string(), previous_model.clone()),
        ],
    );

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

    if config.default_model == previous_model {
        ui::print_info_card(
            "Model Unchanged",
            vec![
                ("Provider".to_string(), provider_display.to_string()),
                ("Default".to_string(), config.default_model.clone()),
            ],
        );
    } else {
        ui::print_info_card(
            "Model Updated",
            vec![
                ("Provider".to_string(), provider_display.to_string()),
                ("Previous".to_string(), previous_model),
                ("Now".to_string(), config.default_model.clone()),
            ],
        );
    }

    println!("Use: tt \"your question\" to chat with this default model.\n");

    Ok(())
}
