use std::convert::Infallible;

use sarge::ArgumentType;

use crate::cli::{InputArgs, OutputArgs};

fn split_repeatable_values(val: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();

    let mut in_quote: Option<char> = None;
    let mut depth_brace: usize = 0;
    let mut depth_bracket: usize = 0;
    let mut depth_paren: usize = 0;

    let mut it = val.chars().peekable();
    while let Some(ch) = it.next() {
        if let Some(q) = in_quote {
            match ch {
                '\\' => {
                    buf.push(ch);
                    if let Some(next) = it.next() {
                        buf.push(next);
                    }
                }
                _ => {
                    buf.push(ch);
                    if ch == q {
                        in_quote = None;
                    }
                }
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                in_quote = Some(ch);
                buf.push(ch);
            }
            '\\' => {
                if matches!(it.peek(), Some(',')) {
                    it.next();
                    buf.push(',');
                } else {
                    buf.push('\\');
                }
            }
            '{' => {
                depth_brace = depth_brace.saturating_add(1);
                buf.push(ch);
            }
            '}' => {
                depth_brace = depth_brace.saturating_sub(1);
                buf.push(ch);
            }
            '[' => {
                depth_bracket = depth_bracket.saturating_add(1);
                buf.push(ch);
            }
            ']' => {
                depth_bracket = depth_bracket.saturating_sub(1);
                buf.push(ch);
            }
            '(' => {
                depth_paren = depth_paren.saturating_add(1);
                buf.push(ch);
            }
            ')' => {
                depth_paren = depth_paren.saturating_sub(1);
                buf.push(ch);
            }
            ',' if depth_brace == 0 && depth_bracket == 0 && depth_paren == 0 => {
                let token = buf.trim();
                if !token.is_empty() {
                    tokens.push(token.to_string());
                }
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }

    let token = buf.trim();
    if !token.is_empty() {
        tokens.push(token.to_string());
    }

    tokens
}

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
                for token in split_repeatable_values(v) {
                    inputs.push(normalize(token.trim()));
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
                for token in split_repeatable_values(v) {
                    outputs.push(normalize(token.trim()));
                }
            }
        }

        Some(Ok(OutputArgs(outputs)))
    }

    fn default_value() -> Option<Self> {
        Some(OutputArgs::default())
    }
}
