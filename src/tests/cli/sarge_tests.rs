use crate::cli::{InputArgs, OutputArgs};
use sarge::ArgumentType;

#[test]
fn input_args_autodetects_path_vs_inline_content() {
    let dir = tempfile::tempdir().expect("tempdir");

    let existing = dir.path().join("in.txt");
    std::fs::write(&existing, "hello").expect("write");
    let existing = existing.to_string_lossy().to_string();

    let parsed = <InputArgs as ArgumentType>::from_value(Some(&existing))
        .expect("some")
        .expect("ok");
    assert_eq!(parsed.as_slice(), &[existing.clone()]);

    let missing = dir.path().join("missing.txt").to_string_lossy().to_string();
    let parsed = <InputArgs as ArgumentType>::from_value(Some(&missing))
        .expect("some")
        .expect("ok");
    assert_eq!(parsed.as_slice(), &[format!("={missing}")]);
}

#[test]
fn output_args_normalizes_stdout_stderr_and_forced_path() {
    let stdout = <OutputArgs as ArgumentType>::from_value(Some("stdout"))
        .expect("some")
        .expect("ok");
    assert_eq!(stdout.as_slice(), &["-".to_string()]);

    let stderr = <OutputArgs as ArgumentType>::from_value(Some("stderr"))
        .expect("some")
        .expect("ok");
    assert_eq!(stderr.as_slice(), &["stderr".to_string()]);

    let forced_path = <OutputArgs as ArgumentType>::from_value(Some("@stderr"))
        .expect("some")
        .expect("ok");
    assert_eq!(forced_path.as_slice(), &["@stderr".to_string()]);
}

#[test]
fn input_args_repeatable_split_allows_inline_json_with_commas() {
    // This string mirrors how sarge joins repeatable values: it inserts a top-level comma
    // between occurrences. The first JSON value itself contains commas.
    let merged = r#"{"a":1,"b":2},{"c":3}"#;

    let parsed = <InputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &[r#"={"a":1,"b":2}"#.to_string(), r#"={"c":3}"#.to_string()]
    );
}

#[test]
fn input_args_repeatable_split_allows_inline_yaml_flow_mapping_with_commas() {
    let merged = r#"{a:1,b:2},{c:3}"#;

    let parsed = <InputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &[r#"={a:1,b:2}"#.to_string(), r#"={c:3}"#.to_string()]
    );
}

#[test]
fn input_args_repeatable_split_allows_inline_toml_inline_table_with_commas() {
    let merged = r#"{a=1,b=2},{c=3}"#;

    let parsed = <InputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &[r#"={a=1,b=2}"#.to_string(), r#"={c=3}"#.to_string()]
    );
}

#[test]
fn input_args_repeatable_split_allows_inline_xml_with_commas_in_quoted_attrs() {
    let merged = r#"<root a="1,2"/>,<root a="3"/>"#;

    let parsed = <InputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &[
            r#"=<root a="1,2"/>"#.to_string(),
            r#"=<root a="3"/>"#.to_string()
        ]
    );
}

#[test]
fn input_args_repeatable_split_supports_escaped_commas_for_plaintext_like_tokens() {
    let merged = r#"=a\,b,=c"#;

    let parsed = <InputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &[r#"=a,b"#.to_string(), r#"=c"#.to_string()]
    );
}

#[test]
fn output_args_repeatable_split_allows_escaped_commas() {
    let merged = r#"@a\,b.txt,stderr"#;

    let parsed = <OutputArgs as ArgumentType>::from_value(Some(merged))
        .expect("some")
        .expect("ok");

    assert_eq!(
        parsed.as_slice(),
        &["@a,b.txt".to_string(), "stderr".to_string()]
    );
}
