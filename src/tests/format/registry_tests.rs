//! Tests for FormatRegistry edge cases.

use crate::format::{CustomFormat, FormatError, FormatKind, FormatRegistry};

#[test]
fn resolve_explicit_unknown_format_returns_unknown_error() {
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);

    let err = registry
        .resolve(Some(&FormatKind::Yaml), &[])
        .expect_err("expected UnknownFormat for Yaml when only Json is registered");

    match err {
        FormatError::UnknownFormat(kind) => assert_eq!(kind, FormatKind::Yaml),
        other => panic!("expected UnknownFormat, got: {other:?}"),
    }
}

#[test]
fn resolve_with_candidates_but_no_match_returns_no_format_matched() {
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);

    let err = registry
        .resolve(None, &[FormatKind::Yaml, FormatKind::Plaintext])
        .expect_err("expected NoFormatMatched when no candidates are registered");

    assert!(matches!(err, FormatError::NoFormatMatched));
}

#[test]
fn kind_for_extension_prefers_builtin_and_custom() {
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);

    // Built-in json extension
    assert_eq!(registry.kind_for_extension("json"), Some(FormatKind::Json));

    // Register a custom format with its own extensions
    let custom = CustomFormat::new("toml", &["toml"]).with_deserialize(|bytes| {
        // Very naive TOML handler: treat as JSON for test purposes
        serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
    });

    registry.register_custom(custom);

    // kind_for_extension should now recognize the custom ext
    assert_eq!(
        registry.kind_for_extension("toml"),
        Some(FormatKind::Custom("toml"))
    );
}

#[test]
fn kind_for_extension_unknown_returns_none() {
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);

    assert_eq!(registry.kind_for_extension("unknown_ext"), None);
}

#[test]
fn resolve_uses_first_registered_candidate() {
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);

    // Yaml is not registered; Json is. Should pick Json as first registered candidate.
    let kind = registry
        .resolve(None, &[FormatKind::Yaml, FormatKind::Json])
        .expect("expected Json to be selected as first registered candidate");

    assert_eq!(kind, FormatKind::Json);
}

#[test]
fn deserialize_value_with_missing_custom_format_returns_unknown() {
    let registry = FormatRegistry::new();
    let data = b"{}";
    let kind = FormatKind::Custom("missing-format");

    let err: FormatError = registry
        .deserialize_value::<serde_json::Value>(Some(&kind), &[], data)
        .expect_err("expected UnknownFormat for missing custom format");

    match err {
        FormatError::UnknownFormat(k) => assert_eq!(k, kind),
        other => panic!("expected UnknownFormat, got: {other:?}"),
    }
}
