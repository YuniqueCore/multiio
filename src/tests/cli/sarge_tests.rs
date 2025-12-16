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
