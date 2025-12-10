use std::collections::HashMap;
use std::sync::Arc;

use crate::error::ErrorPolicy;
use crate::format::{CustomFormat, FormatError, FormatKind, FormatRegistry};
use crate::io::{FileInput, FileOutput, InMemorySink, InMemorySource};
use crate::{IoEngine, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Item {
    category: String,
    value: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CategorySummary {
    category: String,
    total: i32,
    count: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Report {
    summaries: Vec<CategorySummary>,
    items: Vec<Item>,
}

fn make_items_engine(path: &std::path::Path, json_array: &str) -> IoEngine {
    let file_input = Arc::new(FileInput::new(path.to_path_buf()));
    std::fs::write(path, json_array).expect("write input file");

    let src_mem = Arc::new(InMemorySource::from_string("mem", json_array));

    let input_file = crate::config::InputSpec::new("file", file_input)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);
    let input_mem = crate::config::InputSpec::new("mem", src_mem)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let registry = default_registry();
    IoEngine::new(
        registry,
        ErrorPolicy::Accumulate,
        vec![input_file, input_mem],
        Vec::new(),
    )
}

fn make_report(items: &[Item]) -> Report {
    let mut map: HashMap<String, (i32, usize)> = HashMap::new();
    for item in items {
        let entry = map.entry(item.category.clone()).or_insert((0, 0));
        entry.0 += item.value;
        entry.1 += 1;
    }

    let mut summaries: Vec<CategorySummary> = map
        .into_iter()
        .map(|(category, (total, count))| CategorySummary {
            category,
            total,
            count,
        })
        .collect();

    summaries.sort_by(|a, b| a.category.cmp(&b.category));

    Report {
        summaries,
        items: items.to_vec(),
    }
}

#[test]
fn engine_business_report_to_markdown_json_and_custom() {
    let dir = tempfile::tempdir().expect("tempdir");
    let in_path = dir.path().join("items.json");

    let json_array = r#"[
        {"category": "A", "value": 1},
        {"category": "B", "value": 2},
        {"category": "A", "value": 3}
    ]"#;

    let engine = make_items_engine(&in_path, json_array);

    // Read items as a single Vec<Item> per input (file + memory) and flatten
    let per_input: Vec<Vec<Item>> = engine.read_all().expect("read_all items");
    let all_items: Vec<Item> = per_input.into_iter().flatten().collect();

    let report = make_report(&all_items);

    // Build a registry with a custom "bracket" format
    let mut registry = FormatRegistry::new();
    registry.register(FormatKind::Json);
    registry.register(FormatKind::Markdown);

    let bracket = CustomFormat::new("bracket", &["brk"])
        .with_deserialize(|bytes| {
            let s = String::from_utf8_lossy(bytes);
            let inner = s.trim_start_matches('[').trim_end_matches(']');
            serde_json::from_str(inner).map_err(|e| FormatError::Serde(Box::new(e)))
        })
        .with_serialize(|value| {
            let json = serde_json::to_string(value).map_err(|e| FormatError::Serde(Box::new(e)))?;
            Ok(format!("[{json}]").into_bytes())
        });

    registry.register_custom(bracket);

    // Outputs: markdown file, JSON in-memory, custom bracket in-memory
    let md_path = dir.path().join("report.md");
    let md_target = Arc::new(FileOutput::new(md_path.clone()));
    let md_spec = crate::config::OutputSpec::new("md", md_target)
        .with_format(FormatKind::Markdown)
        .with_candidates(vec![FormatKind::Markdown])
        .with_file_exists_policy(crate::config::FileExistsPolicy::Overwrite);

    let json_sink = Arc::new(InMemorySink::new("json"));
    let json_spec = crate::config::OutputSpec::new("json", json_sink.clone())
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json])
        .with_file_exists_policy(crate::config::FileExistsPolicy::Overwrite);

    let custom_sink = Arc::new(InMemorySink::new("bracket"));
    let custom_spec = crate::config::OutputSpec::new("bracket", custom_sink.clone())
        .with_format(FormatKind::Custom("bracket"))
        .with_candidates(vec![FormatKind::Custom("bracket")])
        .with_file_exists_policy(crate::config::FileExistsPolicy::Overwrite);

    let out_engine = IoEngine::new(
        registry,
        ErrorPolicy::Accumulate,
        Vec::new(),
        vec![md_spec, json_spec, custom_spec],
    );

    out_engine
        .write_one_value(&report)
        .expect("write_one_value should succeed");

    // Verify markdown report can be deserialized back to Report
    let md_bytes = std::fs::read(&md_path).expect("read markdown report");
    let md_report: Report = crate::format::deserialize(FormatKind::Markdown, &md_bytes)
        .expect("deserialize markdown report");
    assert_eq!(md_report, report);

    // Verify JSON details
    let json_bytes = json_sink.contents();
    let json_report: Report = serde_json::from_slice(&json_bytes).expect("parse json report");
    assert_eq!(json_report, report);

    // Verify custom bracket format via registry
    let bracket_bytes = custom_sink.contents();
    let roundtrip: Report = out_engine
        .registry()
        .deserialize_value(Some(&FormatKind::Custom("bracket")), &[], &bracket_bytes)
        .expect("deserialize custom bracket report");
    assert_eq!(roundtrip, report);
}
