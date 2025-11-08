# terminal transformer – tt-cli

> Transform natural language into shell commands. A lightning fast, local-first, and highly customizable command-line tool written in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![npm](https://img.shields.io/badge/npm-@bmkubia/tt--cli-red.svg)](https://www.npmjs.com/package/@bmkubia/tt-cli)

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Examples](#examples)
- [Providers](#providers)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

### Multiple AI Providers

- LM Studio – Run models locally, completely offline
- Claude (Anthropic) – Claude-4.5 Sonnet, Haiku, 4.1 Opus
- OpenAI – GPT-5-Codex, o3, and more

### Performance

- Streaming responses with live Markdown rendering
- Built in Rust for minimal overhead
- Efficient token usage with optimized system prompts

### Developer-Focused

- Shell-aware command generation
- OS-specific guidance (macOS, Linux, Windows)
- Automatic model discovery from provider APIs
- Interactive setup with model selection

### Privacy & Control

- Local-first option with LM Studio
- Configuration stored securely in your home directory
- No telemetry or data collection

## Quick Start

```bash
# Install via npm
npm install -g @bmkubia/tt-cli

# Configure your provider
tt setup

# Start using it
tt find all python files modified today
```

## Installation

### npm (recommended)

Install the packaged binary via npm:

```bash
npm install -g @bmkubia/tt-cli
```

This installs a lightweight Node.js launcher plus all platform-specific binaries
into your global npm prefix (`~/.npm-global/bin/tt` or similar).

### Homebrew (macOS)

```bash
brew install --cask tt
```

Homebrew downloads the notarized macOS tarball from GitHub Releases.

### Build from source

### From Source

Requires Rust 1.70 or later. Install from [rustup.rs](https://rustup.rs/) if needed.

```bash
# Clone repository
git clone https://github.com/bmkubia/tt-cli.git
cd tt-cli

# Build and install
cargo install --path .
```

The `tt` binary will be installed to `~/.cargo/bin/`.

## Setup

Run `tt setup` once to configure a provider:

```bash
tt setup
```

Setup walks you through:

1. Selecting a provider (Anthropic/Claude, OpenAI, or LM Studio via its OpenAI-compatible server)
2. Supplying the required credential: an API key for Anthropic/OpenAI or the LM Studio base URL (defaults to `http://localhost:1234/v1`)
3. Picking a default model from the provider’s live `/v1/models` response (with a manual entry fallback when offline)

Your configuration is saved to `~/.config/tt-cli/config.json` (on macOS/Linux).

## Usage

### Basic Commands

Ask questions in natural language - `tt` translates them into shell commands:

```bash
tt find all markdown files modified in the last 7 days
```

```bash
tt recursively search for TODO comments in python files
```

```bash
tt show disk usage sorted by size
```

**Note:** Quote complex queries to prevent shell glob expansion of special characters like `?`, `*`, or `[]`.

## Examples

### File Operations

```bash
tt find all log files older than 30 days
tt "count lines of code in javascript files"
tt "find duplicate files by hash"
```

### Process Management

```bash
tt "show top 10 memory-consuming processes"
tt "kill all node processes"
tt "find process listening on port 3000"
```

### Git Operations

```bash
tt "show git commits from last week with author names"
tt "undo last commit but keep changes"
tt "list all branches merged into main"
```

### Network & System

```bash
tt "check if port 8080 is in use"
tt "show current IP address"
tt "monitor network bandwidth usage"
```

### Text Processing

```bash
tt "extract all email addresses from a file"
tt "find and replace text in multiple files"
tt "count unique lines in access.log"
```

### Package Management

```bash
tt "list outdated npm packages in current directory"
tt "remove unused docker images"
tt "upgrade all brew packages"
```

## Providers

### LM Studio

- **Endpoint**: `http://localhost:1234/v1` (configurable)
- **Authentication**: None (local)
- **Models**: Any models you load

### Anthropic (Claude)

- **Endpoint**: `https://api.anthropic.com/v1`
- **Authentication**: API key required
- **Models**: All `claude-*` models (Opus, Sonnet, Haiku)

### OpenAI

- **Endpoint**: `https://api.openai.com/v1`
- **Authentication**: API key required
- **Models**: GPT-4o, o1, GPT-4, GPT-3.5, and newer models

All providers use dynamic model discovery via their `/v1/models` endpoint, ensuring you always see current available models.

### View current configuration

To see your current provider, API base, masked API key (if applicable), and default model:

```bash
tt config
```

### Change model

To switch the default model without rerunning the whole setup:

```bash
tt model
```

The command calls your provider’s `/v1/models` endpoint to present the latest list (and falls back to a manual entry when offline), so it stays in sync with Anthropic, OpenAI, or LM Studio.

### Reconfigure

To change providers, swap API keys, or point to a different LM Studio base URL, run setup again:

```bash
tt setup
```

## Providers & Model Discovery

`tt setup` and `tt model` both call the provider’s authoritative `/v1/models` endpoint so you always pick from what’s actually deployed:

- **Anthropic (Claude)** &mdash; uses `https://api.anthropic.com/v1`, requires a Claude API key, and exposes every `claude-*` model returned by Anthropic.
- **OpenAI** &mdash; uses `https://api.openai.com/v1`, requires an OpenAI API key, and lists chat-capable models (`gpt-*`, `gpt-4o`, `o1`, `omni`, etc.).
- **LM Studio** &mdash; talks to your local OpenAI-compatible server (defaults to `http://localhost:1234/v1`) and surfaces whichever models LM Studio currently hosts; no API key required.

If a `/v1/models` call fails (offline LM Studio, no network), the CLI falls back to letting you type the desired model ID manually.

## Configuration

The configuration file is stored at:

- macOS/Linux: `~/.config/tt-cli/config.json`
- Windows: `%APPDATA%\tt-cli\config.json`

It records the current provider, default model, and any credentials. Example Anthropic config:

```json
{
  "provider": "anthropic",
  "default_model": "claude-haiku-4-5-20251001",
  "api_key": "sk-ant-***"
}
```

If you use LM Studio, no API key is stored and the CLI records the custom base URL instead:

```json
{
  "provider": "lm_studio",
  "default_model": "TheBloke/Mistral-7B-Instruct-v0.2-GGUF",
  "api_base_override": "http://localhost:1234/v1"
}
```

## Project Structure

- `src/app.rs` — CLI entrypoint; parses args and dispatches to command handlers.
- `src/commands/` — modular command implementations (`chat`, `setup`, `model`, `config`, `version`).
- `src/interaction.rs` — shared dialoguer prompts (provider, API key, model selection).
- `src/client.rs` & `src/models.rs` — provider integrations (streaming completions + `/v1/models` discovery).
- `src/config.rs`, `src/loader.rs`, `src/version.rs` — persisted settings, spinner UX, and semantic version metadata.

## Command Examples

```bash
# File operations
tt find all log files older than 30 days

# Process management
tt "show top 10 memory-consuming processes"

# Git operations
tt "show git commits from last week with author names"

# Network diagnostics
tt "check if port 8080 is in use"

# Disk and filesystem
tt "find directories taking more than 1GB"

# Text processing
tt "extract all email addresses from a file"

# System information
tt show current shell and version

# Package management
tt "list outdated npm packages in current directory"

# View configuration
tt config

# Change default model
tt model

# Reconfigure provider or API key
tt setup

# Check version
tt --version
```

## Troubleshooting

### Configuration Issues

#### "Please provide a question or run 'tt setup' to configure"

You haven't configured `tt` yet. Run `tt setup` to select a provider and model.

#### API key not working

- Verify your API key at your provider's dashboard (Anthropic Console, OpenAI Platform)
- Ensure billing is configured and your account is in good standing
- Check that the key has appropriate permissions
- Re-run `tt setup` to enter a new key

### Provider-Specific Issues

#### LM Studio connection failed

- Verify LM Studio is running and the server is enabled (Server tab → Start Server)
- Confirm the server is listening on `http://localhost:1234` (default port)
- Ensure you have a model loaded in LM Studio
- If using a custom port, run `tt setup` and enter the correct base URL (e.g., `http://localhost:8080/v1`)

#### Model not available

- Run `tt model` to see current available models from your provider
- For LM Studio: Download and load at least one model
- For Anthropic/OpenAI: Verify your account has access to the requested model

#### Slow responses or timeouts

- Check your internet connection (cloud providers)
- For LM Studio: Verify sufficient system resources (CPU/GPU/RAM)
- Try a smaller/faster model with `tt model`
- Check provider status pages for outages

### Shell Issues

#### Shell glob pattern errors ("no matches found")

Shells like zsh treat `?`, `*`, or `[]` as glob patterns. Always quote your questions:

```bash
# ✅ Correct
tt "can you find all files larger than 100MB?"

# ❌ Wrong - shell will try to expand the * pattern
tt can you find all files larger than 100MB?
```

## Development

### Architecture

```text
src/
├── app.rs          # CLI entrypoint and argument parsing
├── commands/       # Command implementations (chat, setup, model, config)
├── client.rs       # Provider API clients and streaming
├── models.rs       # Model discovery and listing
├── config.rs       # Configuration persistence
├── interaction.rs  # Interactive prompts
├── loader.rs       # Spinner/progress UI
└── version.rs      # Version metadata
```

### Local Development

```bash
# Clone repository
git clone https://github.com/bmkubia/tt-cli.git
cd tt-cli

# Build and run
cargo build
cargo run -- "your test question"

# Run tests
cargo test

# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Build release binary
cargo build --release
```

### Requirements

- Rust 1.70 or later
- One of: Anthropic API key, OpenAI API key, or LM Studio with local server

## Contributing

Please ensure your changes adhere to the following guidelines:

### Code Standards

- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Ensure `cargo clippy -- -D warnings` passes
- **Testing**: Add tests for new features (`cargo test`)
- **Commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/)
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation changes
  - `refactor:` for code refactoring

### Release Process

See `docs/releasing.md` for the release workflow, including version management, GitHub Actions pipeline, and npm/Homebrew publishing.

## License

MIT License - see [LICENSE](LICENSE) file for details.
