use crate::{client::ModelClient, config::Config, loader};
use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    style::{Attribute, Color},
    terminal,
};
use futures::StreamExt;
use std::io::{Write, stdout};
use std::time::{Duration, Instant};
use termimad::{Alignment, ListItemsIndentationMode, MadSkin};

pub async fn run(question: &str) -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;

    if !config.is_configured() {
        anyhow::bail!("No configuration found. Run 'tt setup' first.");
    }

    let api_base = config.api_base();
    let client = ModelClient::new(config.provider, config.api_key.clone(), api_base);

    let mut loader_handle = Some(loader::ShimmerLoader::new("tt is working").spawn());
    let start_time = Instant::now();
    let mut header_printed = false;
    let mut renderer = ResponseRenderer::new();
    let system_prompt = build_system_prompt(&config.default_model);

    let mut stream = client
        .ask_stream(question, &config.default_model, &system_prompt)
        .await
        .context("Failed to get response from the provider")?;

    let mut accumulated_text = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(text) => {
                accumulated_text.push_str(&text);

                if !header_printed {
                    stop_loader(&mut loader_handle).await;
                    print_response_header(start_time.elapsed(), &config.default_model)
                        .context("Failed to write response header")?;
                    header_printed = true;
                }

                renderer
                    .render(&accumulated_text)
                    .context("Failed to render streamed response")?;
            }
            Err(err) => {
                stop_loader(&mut loader_handle).await;
                renderer.finish();
                return Err(err);
            }
        }
    }

    stop_loader(&mut loader_handle).await;

    if renderer.has_output() {
        renderer.finish();
    } else {
        println!("(No response received from the provider)");
    }

    Ok(())
}

async fn stop_loader(loader_handle: &mut Option<loader::LoaderHandle>) {
    if let Some(mut handle) = loader_handle.take() {
        handle.stop().await;
        let _ = execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::CurrentLine),
            cursor::MoveToColumn(0),
            cursor::Show
        );
    }
}

fn build_system_prompt(model_name: &str) -> String {
    let os_name = current_os_display_name();
    let shell_name = current_shell_display_name();
    format!(
        "You are the command line assistant `tt-cli`. You translate natural-language requests into shell commands.\n\nEnvironment:\n- OS: {os_name}\n- Shell: {shell_name}\n- Model: {model_name}\n\nRules:\n- Output commands with minimal prose.\n- Use one command per line.\n- No placeholders. Quote paths and variables safely.\n- Prefer non-destructive forms and --dry-run/-n when available."
    )
}

fn current_os_display_name() -> String {
    match std::env::consts::OS {
        "macos" => "macOS",
        "linux" => "Linux",
        "windows" => "Windows",
        "freebsd" => "FreeBSD",
        other => other,
    }
    .to_string()
}

fn current_shell_display_name() -> String {
    std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn codex_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.limit_to_ascii();

    skin.paragraph.align = Alignment::Left;
    skin.paragraph.compound_style.set_fg(Color::White);
    skin.paragraph.left_margin = 0;
    skin.paragraph.right_margin = 0;
    skin.list_items_indentation_mode = ListItemsIndentationMode::Block;

    skin.bold.set_fg(Color::White);
    skin.inline_code.set_fg(Color::DarkGrey);
    skin.inline_code.set_bg(Color::Reset);
    skin.inline_code.add_attr(Attribute::Bold);
    skin.code_block.set_fg(Color::DarkGrey);
    skin.code_block.set_bg(Color::Reset);

    let heading_palette = [
        Color::Cyan,
        Color::White,
        Color::Grey,
        Color::DarkGrey,
        Color::DarkGrey,
        Color::DarkGrey,
    ];

    for (style, color) in skin.headers.iter_mut().zip(heading_palette.iter().cycle()) {
        style.align = Alignment::Left;
        style.add_attr(Attribute::Bold);
        style.set_fg(*color);
    }

    skin.bullet.set_char('-');
    skin.bullet.set_fg(Color::White);
    skin.horizontal_rule.set_char('-');

    skin
}

struct ResponseRenderer {
    skin: MadSkin,
    rendered_lines: Vec<String>,
}

impl ResponseRenderer {
    fn new() -> Self {
        Self {
            skin: codex_skin(),
            rendered_lines: Vec::new(),
        }
    }

    fn render(&mut self, markdown: &str) -> Result<()> {
        if markdown.trim().is_empty() {
            return Ok(());
        }

        let mut rendered = format!("{}", self.skin.term_text(markdown));
        if !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        let new_lines: Vec<String> = rendered
            .split_inclusive('\n')
            .map(|line| line.to_string())
            .collect();
        let common_prefix = count_common_prefix(&self.rendered_lines, &new_lines);
        let lines_to_rewrite = self.rendered_lines.len().saturating_sub(common_prefix);

        let mut out = stdout();

        if lines_to_rewrite > 0 {
            let lines_to_move = lines_to_rewrite.min(u16::MAX as usize) as u16;
            execute!(
                out,
                cursor::MoveUp(lines_to_move),
                terminal::Clear(terminal::ClearType::FromCursorDown)
            )?;
        }

        if common_prefix == 0 {
            write!(out, "{rendered}")?;
        } else {
            for line in new_lines.iter().skip(common_prefix) {
                write!(out, "{line}")?;
            }
        }
        out.flush()?;

        self.rendered_lines = new_lines;
        Ok(())
    }

    fn finish(&mut self) {
        self.rendered_lines.clear();
    }

    fn has_output(&self) -> bool {
        !self.rendered_lines.is_empty()
    }
}

fn count_common_prefix(a: &[String], b: &[String]) -> usize {
    a.iter()
        .zip(b.iter())
        .take_while(|(left, right)| left == right)
        .count()
}

fn print_response_header(duration: Duration, model: &str) -> Result<()> {
    let (width, _) = terminal::size().unwrap_or((100, 0));
    let label = format!("Thought for {}", loader::format_elapsed(duration));
    let model_label = format!(" {model} ");

    let desired_width = width.max(20) as usize;
    let left_part = format!("─ {label} ");
    let left_len = left_part.chars().count();
    let model_len = model_label.chars().count();

    let mut line = left_part;
    let separator_len = desired_width.saturating_sub(left_len + model_len + 1);
    if separator_len > 0 {
        line.push_str(&"─".repeat(separator_len));
        line.push_str(&model_label);
        line.push('─');
    } else {
        let remaining = desired_width.saturating_sub(left_len);
        if remaining > 0 {
            line.push_str(&"─".repeat(remaining));
        }
    }

    const CODE_COLOR: &str = "\x1b[90m";
    const RESET: &str = "\x1b[0m";
    println!();
    println!("{CODE_COLOR}{line}{RESET}");
    println!();

    Ok(())
}
