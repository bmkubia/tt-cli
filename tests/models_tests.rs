use tt::config::ProviderKind;
use tt::models::normalize_models;

#[test]
fn normalize_models_dedups_and_sorts() {
    let models = vec![
        "gpt-4o-mini".to_string(),
        "claude-3-opus".to_string(),
        "gpt-4o-mini".to_string(),
    ];
    let normalized = normalize_models(models, ProviderKind::OpenRouter);
    assert_eq!(
        normalized,
        vec!["claude-3-opus".to_string(), "gpt-4o-mini".to_string()]
    );
}

#[test]
fn openai_normalize_filters_to_chat_models() {
    let models = vec![
        "text-embedding-3-small".to_string(),
        "gpt-4o-mini".to_string(),
        "o1-mini".to_string(),
        "text-search-babbage-doc".to_string(),
    ];
    let normalized = normalize_models(models, ProviderKind::OpenAi);
    assert_eq!(
        normalized,
        vec!["gpt-4o-mini".to_string(), "o1-mini".to_string()]
    );
}
