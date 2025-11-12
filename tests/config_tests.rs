use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tempfile::TempDir;
use tt::config::{Config, ProviderKind};

struct EnvGuard {
    _guard: MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set_path(path: &Path) -> Self {
        let guard = env_mutex().lock().unwrap();
        unsafe {
            env::set_var("TT_CONFIG_DIR", path);
        }
        Self { _guard: guard }
    }

    fn set_raw(value: &str) -> Self {
        let guard = env_mutex().lock().unwrap();
        unsafe {
            env::set_var("TT_CONFIG_DIR", value);
        }
        Self { _guard: guard }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            env::remove_var("TT_CONFIG_DIR");
        }
    }
}

fn env_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn base_config(provider: ProviderKind) -> Config {
    Config {
        provider,
        api_key: None,
        default_model: "test-model".into(),
        api_base_override: None,
    }
}

#[test]
fn anthropic_requires_api_key() {
    let mut cfg = base_config(ProviderKind::Anthropic);
    assert!(!cfg.is_configured(), "API key should be required");

    cfg.api_key = Some("sk-ant-123".into());
    assert!(cfg.is_configured());
}

#[test]
fn lmstudio_only_needs_model() {
    let cfg = base_config(ProviderKind::LmStudio);
    assert!(cfg.is_configured());
}

#[test]
fn api_key_preview_masks_values() {
    let mut cfg = base_config(ProviderKind::OpenAi);
    cfg.api_key = Some("abcdefghijklmnop".into());
    assert_eq!(cfg.api_key_preview().as_deref(), Some("abcdefg...mnop"));
}

#[test]
fn api_base_uses_override_when_present() {
    let mut cfg = base_config(ProviderKind::LmStudio);
    cfg.api_base_override = Some("http://example.test/v1".into());
    assert_eq!(cfg.api_base(), "http://example.test/v1");
}

#[test]
fn config_dir_respects_tt_config_dir_env() {
    let temp = TempDir::new().unwrap();
    let _guard = EnvGuard::set_path(temp.path());
    let path = Config::config_dir().expect("config dir");
    assert_eq!(path, temp.path());
}

#[test]
fn config_dir_rejects_empty_env_value() {
    let _guard = EnvGuard::set_raw("");
    assert!(Config::config_dir().is_err());
}

#[test]
fn save_is_atomic_and_respects_custom_dir() {
    let temp = TempDir::new().unwrap();
    let _guard = EnvGuard::set_path(temp.path());

    let mut cfg = base_config(ProviderKind::Anthropic);
    cfg.api_key = Some("sk-ant-xyz".into());
    cfg.save().expect("save config");

    let config_path = temp.path().join("config.json");
    let contents = fs::read_to_string(&config_path).expect("config contents");
    let parsed: Config = serde_json::from_str(&contents).expect("valid json");
    assert_eq!(parsed.api_key.as_deref(), Some("sk-ant-xyz"));
}
