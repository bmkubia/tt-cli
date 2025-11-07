# Talk-to CLI (tt-cli)

```bash
   __  __             ___
  / /_/ /_      _____/ (_)
 / __/ __/_____/ ___/ / /
/ /_/ /_/_____/ /__/ / /  
\__/\__/      \___/_/_/
```

A fast command-line tool to talk to Claude, OpenAI, or local LM Studio models from your terminal, tailored for Codex-style answers.

## Features

- Ask questions to Claude, OpenAI, or LM Studio (OpenAI-compatible) models from the command line
- __Streaming responses__ - see Markdown appear live, rendered exactly like Codex CLI
- __Braille loader__ - smooth spinner with live timer (`⡿ tt is working (3s)`)
- Interactive setup for picking a provider, API key (when required), and model
- Choose from any model returned by the provider’s authoritative `/v1/models` endpoint
- Lightweight and fast

## Installation

### npm (recommended)

Install the packaged binary via npm:

```bash
npm install -g tt-cli
```

This installs a lightweight Node.js launcher plus all platform-specific binaries
into your global npm prefix (`~/.npm-global/bin/tt` or similar).

### Homebrew

Once the cask propagates you can simply:

```bash
brew install --cask tt
```

Homebrew downloads the notarized macOS tarball from GitHub Releases.

### Build from source

1. Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/)

2. Clone this repository and navigate to it:

```bash
cd ask-cli
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

# Multi-word questions (quotes optional)
tt "What are the best practices for REST API design?"

# View configuration
tt config

# Change default model
tt model

# Reconfigure
tt setup
```

## Requirements

- Rust 1.70+ (for building)
- A provider credential: Anthropic API key, OpenAI API key, or the LM Studio desktop app with its local OpenAI-compatible server enabled

## Releasing

Maintainers can follow `docs/releasing.md` for the exact workflow (version
selection script, GitHub Actions release pipeline, npm + Homebrew publishing).

## License

MIT
