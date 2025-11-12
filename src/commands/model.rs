use crate::config::Config;
use crate::interaction;
use anyhow::{Context, Result};
use unicode_width::UnicodeWidthStr;

pub async fn change() -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;

    if !config.is_configured() {
        anyhow::bail!("No configuration found. Run 'tt setup' first.");
    }

    let provider_display = config.provider.display_name();
    let previous_model = config.default_model.clone();

    print_model_card(
        "Current Default",
        vec![
            ("Provider", provider_display.to_string()),
            ("Model", previous_model.clone()),
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
        print_model_card(
            "Model Unchanged",
            vec![
                ("Provider", provider_display.to_string()),
                ("Default", config.default_model.clone()),
            ],
        );
    } else {
        print_model_card(
            "Model Updated",
            vec![
                ("Provider", provider_display.to_string()),
                ("Previous", previous_model),
                ("Now", config.default_model.clone()),
            ],
        );
    }

    println!("Use: tt \"your question\" to chat with this default model.\n");

    Ok(())
}

fn print_model_card(title: &str, rows: Vec<(&str, String)>) {
    if rows.is_empty() {
        return;
    }

    let label_width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);

    let formatted_rows: Vec<String> = rows
        .into_iter()
        .map(|(label, value)| {
            format!(
                "{:<label_width$} : {}",
                label,
                value,
                label_width = label_width
            )
        })
        .collect();

    let mut inner_width = formatted_rows
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(0);
    let title_width = UnicodeWidthStr::width(title);
    inner_width = inner_width.max(title_width + 2);
    let total_width = inner_width + 4;

    println!();
    println!("{}", card_top_line(title, total_width));
    for line in formatted_rows {
        let current_width = UnicodeWidthStr::width(line.as_str());
        let padding = " ".repeat(inner_width.saturating_sub(current_width));
        println!("│ {}{} │", line, padding);
    }
    println!("╰{}╯", "─".repeat(total_width.saturating_sub(2)));
    println!();
}

fn card_top_line(title: &str, total_width: usize) -> String {
    let mut line = format!("╭─ {title} ");
    let prefix_width = UnicodeWidthStr::width(line.as_str());
    let remaining = total_width.saturating_sub(prefix_width.saturating_add(1)); // reserve space for closing corner
    line.push_str(&"─".repeat(remaining));
    line.push('╮');
    line
}
