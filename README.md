# Talk-to your command line (tt-cli)

> A lightning-fast command-line tool to chat with Claude, OpenAI, or local LM Studio models directly from your terminal. Get Codex-style answers with streaming responses and beautiful Markdown rendering.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Table of Contents

- [Talk-to your command line (tt-cli)](#talk-to-your-command-line-tt-cli)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installation](#installation)
    - [npm (recommended)](#npm-recommended)
    - [Homebrew (macOS)](#homebrew-macos)
    - [Build from source](#build-from-source)
  - [Setup](#setup)
  - [Usage](#usage)
    - [Ask a question](#ask-a-question)
    - [View current configuration](#view-current-configuration)
    - [Change model](#change-model)
    - [Reconfigure](#reconfigure)
    - [Check version](#check-version)
  - [Providers \& Model Discovery](#providers--model-discovery)
  - [Configuration](#configuration)
  - [Project Structure](#project-structure)
  - [Examples](#examples)
  - [Troubleshooting](#troubleshooting)
    - ["Please provide a question or run 'tt setup' to configure"](#please-provide-a-question-or-run-tt-setup-to-configure)
    - [API key not working](#api-key-not-working)
    - [LM Studio connection failed](#lm-studio-connection-failed)
    - [Model not available](#model-not-available)
    - [Slow responses or timeouts](#slow-responses-or-timeouts)
  - [Requirements](#requirements)
  - [Releasing](#releasing)
  - [Contributing](#contributing)
    - [Development workflow](#development-workflow)
  - [License](#license)

## Features

- **Multiple AI providers** - Works with Claude (Anthropic), OpenAI, and LM Studio (local models)
- **Streaming responses** - See answers appear live with beautifully rendered Markdown
- **Braille loader** - Smooth spinner with live timer (e.g., `⡿ tt is working (3s)`)
- **Interactive setup** - Guided configuration for provider, API keys, and model selection
- **Dynamic model discovery** - Choose from any model available via the provider's `/v1/models` endpoint
- **Local-first option** - Run completely offline with LM Studio
- **Lightweight & fast** - Built in Rust for speed and efficiency
- **Cross-platform** - Works on macOS, Linux, and Windows

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

1. Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/)

2. Clone this repository and navigate to it:

```bash
git clone https://github.com/bmkubia/tt-cli.git
cd tt-cli
```

3. Build the release binary:

```bash
cargo build --release
```

4. Install the binary to your system:

```bash
cargo install --path .
```

This installs the `tt` binary to `~/.cargo/bin/`, which should already be in your PATH.

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

### Ask a question

Type `tt` followed by your question:

```bash
tt What is the capital of France?
```

```bash
tt How do I reverse a string in Python?
```

```bash
tt "Explain quantum computing in simple terms"
```

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

### Check version

To see the current version of `tt`:

```bash
tt --version
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

## Examples

```bash
# Simple question
tt What is Rust?

# Code-related question
tt How do I read a file in Python?

# Multi-word questions (quotes optional but recommended for complex queries)
tt "What are the best practices for REST API design?"

# Ask about debugging
tt "How do I debug a segmentation fault in C++?"

# Request code examples
tt "Show me how to implement a binary search in JavaScript"

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

### "Please provide a question or run 'tt setup' to configure"

This means you haven't configured `tt` yet. Run `tt setup` to select a provider and model.

### API key not working

- For Anthropic/OpenAI: Verify your API key is valid at their respective dashboards
- Check that your key has the correct permissions and billing is set up
- Run `tt setup` again to re-enter your credentials

### LM Studio connection failed

- Ensure LM Studio is running and the server is enabled (Server tab → Start Server)
- Verify the server is listening on `http://localhost:1234` (or your custom port)
- Check that you have a model loaded in LM Studio
- If using a custom port, run `tt setup` and enter the correct base URL (e.g., `http://localhost:8080/v1`)

### Model not available

- Run `tt model` to see the current list of available models from your provider
- For LM Studio, ensure you have at least one model downloaded and loaded
- For Anthropic/OpenAI, check your account has access to the model you're trying to use

### Slow responses or timeouts

- Check your internet connection (for Anthropic/OpenAI)
- For LM Studio, verify your local machine has sufficient resources (CPU/GPU/RAM)
- Try a smaller/faster model with `tt model`

## Requirements

- Rust 1.70+ (for building)
- A provider credential: Anthropic API key, OpenAI API key, or the LM Studio desktop app with its local OpenAI-compatible server enabled

## Releasing

Maintainers can follow `docs/releasing.md` for the exact workflow (version
selection script, GitHub Actions release pipeline, npm + Homebrew publishing).

## Contributing

Contributions are welcome but please follow these guidelines:

1. **Code Style**: Run `cargo fmt` before committing
2. **Linting**: Ensure `cargo clippy -- -D warnings` passes with no warnings
3. **Tests**: Add tests for new features and run `cargo test`
4. **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format (e.g., `feat: add model caching`, `fix: handle offline mode`)

### Development workflow

```bash
# Clone and setup
git clone https://github.com/bmkubia/tt-cli.git
cd tt-cli

# Build and run locally
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

## License

MIT
