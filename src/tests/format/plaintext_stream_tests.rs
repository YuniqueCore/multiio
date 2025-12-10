use std::io::Cursor;

use crate::format::{FormatKind, default_registry, deserialize_plaintext_stream};

#[test]
fn plaintext_stream_yields_lines_as_strings() {
    let input = "alpha\nbeta\n\ngamma\n";
    let iter = deserialize_plaintext_stream::<String, _>(Cursor::new(input.as_bytes()));

    let lines: Vec<String> = iter.collect::<Result<_, _>>().expect("lines should parse");

    assert_eq!(
        lines,
        vec![
            "alpha".to_string(),
            "beta".to_string(),
            "".to_string(),
            "gamma".to_string(),
        ]
    );
}

#[test]
fn plaintext_stream_via_registry_yields_lines() {
    let input = "one\ntwo\n";
    let registry = default_registry();

    let iter = registry
        .stream_deserialize_into::<String>(
            Some(&FormatKind::Plaintext),
            &[],
            Box::new(Cursor::new(input.as_bytes())),
        )
        .expect("plaintext streaming should be supported");

    let lines: Vec<String> = iter.collect::<Result<_, _>>().expect("lines should parse");

    assert_eq!(lines, vec!["one".to_string(), "two".to_string()]);
}

#[test]
fn plaintext_stream_empty_input_yields_no_items() {
    let input = "";
    let iter = deserialize_plaintext_stream::<String, _>(Cursor::new(input.as_bytes()));

    let lines: Vec<String> = iter.collect::<Result<_, _>>().expect("lines should parse");

    assert!(lines.is_empty());
}

#[test]
fn plaintext_stream_via_registry_empty_input_yields_no_items() {
    let input = "";
    let registry = default_registry();

    let iter = registry
        .stream_deserialize_into::<String>(
            Some(&FormatKind::Plaintext),
            &[],
            Box::new(Cursor::new(input.as_bytes())),
        )
        .expect("plaintext streaming should be supported");

    let lines: Vec<String> = iter.collect::<Result<_, _>>().expect("lines should parse");

    assert!(lines.is_empty());
}
