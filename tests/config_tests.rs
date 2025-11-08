use tempfile::TempDir;
use tt::config::{Config, ProviderKind};

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
    std::env::set_var("TT_CONFIG_DIR", temp.path());
    let path = Config::config_dir().expect("config dir");
    assert_eq!(path, temp.path());
    std::env::remove_var("TT_CONFIG_DIR");
}
