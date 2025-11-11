use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{env, path::Path};
use tempfile::Builder;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[default]
    Anthropic,
    OpenAi,
    OpenRouter,
    LmStudio,
}

impl ProviderKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "Anthropic (Claude)",
            ProviderKind::OpenAi => "OpenAI",
            ProviderKind::OpenRouter => "OpenRouter",
            ProviderKind::LmStudio => "LM Studio (local)",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        matches!(
            self,
            ProviderKind::Anthropic | ProviderKind::OpenAi | ProviderKind::OpenRouter
        )
    }

    pub fn default_api_base(&self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "https://api.anthropic.com/v1",
            ProviderKind::OpenAi => "https://api.openai.com/v1",
            ProviderKind::OpenRouter => "https://openrouter.ai/api/v1",
            ProviderKind::LmStudio => "http://localhost:1234/v1",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub provider: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base_override: Option<String>,
}

fn default_model() -> String {
    "claude-haiku-4-5-20251001".to_string()
}

const APP_CONFIG_DIR: &str = "tt-cli";

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderKind::default(),
            api_key: None,
            default_model: default_model(),
            api_base_override: None,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        if let Ok(custom) = env::var("TT_CONFIG_DIR") {
            let custom_path = Path::new(&custom);
            if custom_path.as_os_str().is_empty() {
                anyhow::bail!("TT_CONFIG_DIR cannot be empty");
            }
            fs::create_dir_all(custom_path).context("Could not create TT_CONFIG_DIR")?;
            return Ok(custom_path.to_path_buf());
        }

        let config_dir = dirs::config_dir().context("Could not find config directory")?;
        let app_config_dir = config_dir.join(APP_CONFIG_DIR);
        fs::create_dir_all(&app_config_dir).context("Could not create config directory")?;
        Ok(app_config_dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path).context("Could not read config file")?;
        let config: Config =
            serde_json::from_str(&contents).context("Could not parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self).context("Could not serialize config")?;
        let dir = config_path
            .parent()
            .context("Config path is missing a parent directory")?;
        fs::create_dir_all(dir).context("Could not ensure config directory exists")?;

        let mut temp_file = Builder::new()
            .prefix("config")
            .suffix(".tmp")
            .tempfile_in(dir)
            .context("Failed to create temporary config file")?;

        temp_file
            .write_all(contents.as_bytes())
            .context("Failed to write temporary config file")?;
        temp_file
            .flush()
            .context("Failed to flush temporary config file")?;

        temp_file
            .persist(&config_path)
            .map_err(|err| anyhow!("Failed to persist config file: {}", err))?;

        Ok(())
    }

    pub fn api_base(&self) -> String {
        self.api_base_override
            .clone()
            .unwrap_or_else(|| self.provider.default_api_base().to_string())
    }

    pub fn is_configured(&self) -> bool {
        let has_model = !self.default_model.trim().is_empty();
        if !has_model {
            return false;
        }

        if self.provider.requires_api_key() {
            return self
                .api_key
                .as_ref()
                .map(|key| !key.trim().is_empty())
                .unwrap_or(false);
        }

        true
    }

    pub fn api_key_preview(&self) -> Option<String> {
        let key = self.api_key.as_ref()?;

        if key.is_empty() {
            return None;
        }

        let total_chars = key.chars().count();
        if total_chars <= 11 {
            return Some(key.clone());
        }

        let prefix_len = total_chars.min(7);
        let suffix_len = total_chars.min(4);
        let prefix: String = key.chars().take(prefix_len).collect();
        let suffix: String = key
            .chars()
            .skip(total_chars.saturating_sub(suffix_len))
            .collect();

        Some(format!("{prefix}...{suffix}"))
    }
}
