use std::convert::Infallible;

use sarge::ArgumentType;

use crate::cli::{InputArgs, OutputArgs};

impl ArgumentType for InputArgs {
    type Error = Infallible;

    const REPEATABLE: bool = true;

    fn from_value(val: Option<&str>) -> sarge::ArgResult<Self> {
        fn normalize(token: &str) -> String {
            // Preserve explicit prefixes so callers can disambiguate.
            if token.starts_with('@') || token.starts_with('=') {
                return token.to_string();
            }

            if token == "-" || token.eq_ignore_ascii_case("stdin") {
                return "-".to_string();
            }

            // Auto-detect: existing path => file, otherwise treat as inline content.
            if std::fs::metadata(token).is_ok() {
                token.to_string()
            } else {
                format!("={token}")
            }
        }

        let mut inputs = Vec::new();
        match val {
            None => inputs.push("-".to_string()),
            Some(v) => {
                for token in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    inputs.push(normalize(token));
                }
            }
        }

        Some(Ok(InputArgs(inputs)))
    }

    fn default_value() -> Option<Self> {
        Some(InputArgs::default())
    }
}

impl ArgumentType for OutputArgs {
    type Error = Infallible;

    const REPEATABLE: bool = true;

    fn from_value(val: Option<&str>) -> sarge::ArgResult<Self> {
        fn normalize(token: &str) -> String {
            // Preserve explicit prefixes so callers can disambiguate.
            if token.starts_with('@') {
                return token.to_string();
            }

            if token == "-" || token.eq_ignore_ascii_case("stdout") {
                return "-".to_string();
            }

            if token.eq_ignore_ascii_case("stderr") {
                return "stderr".to_string();
            }

            token.to_string()
        }

        let mut outputs = Vec::new();
        match val {
            None => outputs.push("-".to_string()),
            Some(v) => {
                for token in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    outputs.push(normalize(token));
                }
            }
        }

        Some(Ok(OutputArgs(outputs)))
    }

    fn default_value() -> Option<Self> {
        Some(OutputArgs::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
