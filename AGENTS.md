# Repository Guidelines

## Layout

- `src/app.rs` owns CLI parsing and delegates to `src/commands/`.
- `src/commands/` contains focused handlers (`chat`, `setup`, `model`, `config`, `version`) and shares prompt logic via `src/interaction.rs`.
- `src/client.rs` streams completions for Anthropic/OpenAI/LM Studio, `src/models.rs` discovers `/v1/models`, `src/config.rs` persists provider settings, and `src/loader.rs` + `src/version.rs` cover UX + release metadata.

## Dev Loop

- `cargo fmt` → format before every commit.
- `cargo clippy -- -D warnings` → keep lint debt at zero.
- `cargo test` (add targeted unit tests in the same module using `#[cfg(test)]` and `#[tokio::test(flavor = "current_thread")]` for async logic).
- `cargo run -- "<question>"` to smoke-test CLI paths (quote to avoid shell globbing of `?`, `*`, etc.).

## Style & Patterns

- Rust 2021, four spaces, idiomatic `snake_case` for items and `PascalCase` for types.
- Keep modules small: orchestration lives in `app.rs`, IO/side-effects in command modules, and provider-specific logic in `client.rs`/`models.rs`.
- Prefer small async helpers that stream output instead of buffering; reuse prompt helpers from `interaction.rs` instead of re-implementing dialog flows.

## Testing Expectations

- Exercise parsing, config serialization, and provider adapters with unit tests.
- Mock file/network IO by isolating pure helpers; avoid hitting Anthropic/OpenAI APIs in tests.
- For terminal rendering changes, cover formatting helpers (e.g., `format_elapsed`, response header builders) with deterministic assertions.

## Commits, PRs & Security

- Use Conventional Commits (`feat: add lm studio selector`) and keep subjects ≤72 chars.
- PRs should summarize user-visible changes, list commands run (`fmt`, `clippy`, `test`), and attach terminal screenshots for UX tweaks.
- Anthropic/OpenAI keys live only in `~/.config/tt-cli/config.json`; LM Studio stores an `api_base_override`. Never log raw secrets—use `Config::api_key_preview()` and redact LM Studio URLs when sharing output. Document any extra files written to `~/.config/tt-cli/`.
