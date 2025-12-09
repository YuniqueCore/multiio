//! Tests for InputSpec and OutputSpec.

use crate::FormatKind;
use crate::config::{FileExistsPolicy, InputSpec, OutputSpec};
use crate::io::{InMemorySink, InMemorySource};
use std::sync::Arc;

#[test]
fn input_spec_with_format_and_candidates() {
    let src = Arc::new(InMemorySource::from_string("id", "{}"));
    let spec = InputSpec::new("raw", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json, FormatKind::Yaml]);

    assert_eq!(spec.raw, "raw");
    assert_eq!(spec.explicit_format, Some(FormatKind::Json));
    assert_eq!(spec.format_candidates.len(), 2);
}

#[test]
fn output_spec_with_policy_and_format() {
    let sink = Arc::new(InMemorySink::new("id"));
    let spec = OutputSpec::new("raw", sink)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json])
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    assert_eq!(spec.raw, "raw");
    assert_eq!(spec.explicit_format, Some(FormatKind::Json));
    assert_eq!(spec.file_exists_policy, FileExistsPolicy::Overwrite);
}
