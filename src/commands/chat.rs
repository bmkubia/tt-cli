use crate::{client::ModelClient, config::Config, loader};
use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    style::{Attribute, Color},
    terminal,
};
use futures::StreamExt;
use std::io::{stdout, Write};
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
    let system_prompt = build_system_prompt();

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
                    print_response_header(start_time.elapsed())
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

fn build_system_prompt() -> String {
    let os_name = current_os_display_name();
    format!(
        "You are Claude integrated into the tt Codex-style CLI on {os_name}. \
Response requirements:\n\
- Be fast, token-efficient, and focused on actionable command-line answers unless the user explicitly opts out.\n\
- Follow Codex CLI formatting: prefer terse '-' bullets or single lines; avoid titles/headings unless explicitly requested; use inline code/backticks for commands and fenced code blocks (with language hints) for multi-line snippets.\n\
- Provide OS-aware guidance reflecting {os_name}. Mention alternatives only when critical.\n\
- Skip filler and keep context minimal. Ask clarifying questions only when essential."
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
    lines_rendered: usize,
}

impl ResponseRenderer {
    fn new() -> Self {
        Self {
            skin: codex_skin(),
            lines_rendered: 0,
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
        let line_count = rendered.matches('\n').count();

        let mut out = stdout();

        if self.lines_rendered > 0 {
            let lines_to_move = self.lines_rendered.min(u16::MAX as usize) as u16;
            execute!(
                out,
                cursor::MoveUp(lines_to_move),
                terminal::Clear(terminal::ClearType::FromCursorDown)
            )?;
        }

        write!(out, "{rendered}")?;
        out.flush()?;

        self.lines_rendered = line_count;
        Ok(())
    }

    fn finish(&mut self) {
        self.lines_rendered = 0;
    }

    fn has_output(&self) -> bool {
        self.lines_rendered > 0
    }
}

fn print_response_header(duration: Duration) -> Result<()> {
    let (width, _) = terminal::size().unwrap_or((100, 0));
    let label = format!("Thought for {}", loader::format_elapsed(duration));
    let mut line = format!("─ {label} ");
    let desired_width = width.max(20) as usize;
    let current_len = line.chars().count();
    if desired_width > current_len {
        line.push_str(&"─".repeat(desired_width - current_len));
    }

    const CODE_COLOR: &str = "\x1b[90m";
    const RESET: &str = "\x1b[0m";
    println!();
    println!("{CODE_COLOR}{line}{RESET}");
    println!();

    Ok(())
}
