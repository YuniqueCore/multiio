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
