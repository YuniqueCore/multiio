//! Tests for IoEngine::read_stream and AsyncIoEngine::read_stream_async.

use std::sync::Arc;

use crate::config::{InputSpec, OutputSpec};
use crate::error::{ErrorPolicy, Stage};
use crate::io::InMemorySource;
use crate::{FormatKind, IoEngine, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct StreamConfig {
    name: String,
    value: i32,
}

fn make_sync_engine(inputs: Vec<InputSpec>) -> IoEngine {
    let registry = default_registry();
    let outputs: Vec<OutputSpec> = Vec::new();
    IoEngine::new(registry, ErrorPolicy::Accumulate, inputs, outputs)
}

#[test]
fn sync_read_stream_returns_per_input_results() {
    // One valid JSON input and two invalid ones
    let ok_src = Arc::new(InMemorySource::from_string(
        "ok",
        r#"{"name": "ok", "value": 1}"#,
    ));
    let bad1_src = Arc::new(InMemorySource::from_string("bad1", "{not-json"));
    let bad2_src = Arc::new(InMemorySource::from_string("bad2", "[1,2,,]"));

    let mk_spec = |raw: &str, src: Arc<InMemorySource>| {
        InputSpec::new(raw, src)
            .with_format(FormatKind::Json)
            .with_candidates(vec![FormatKind::Json])
    };

    let engine = make_sync_engine(vec![
        mk_spec("ok", ok_src),
        mk_spec("bad1", bad1_src),
        mk_spec("bad2", bad2_src),
    ]);

    let mut iter = engine.read_stream::<StreamConfig>();

    // First item should be Ok
    let first = iter.next().expect("one result");
    let v = first.expect("first should be Ok");
    assert_eq!(v.name, "ok");

    // Next two should be parse errors
    for expected_target in ["bad1", "bad2"] {
        let res = iter.next().expect("more results");
        let e = res.expect_err("expected error for bad input");
        assert_eq!(e.stage, Stage::Parse);
        assert_eq!(e.target, expected_target);
    }

    assert!(iter.next().is_none());
}

#[cfg(feature = "async")]
mod async_stream {
    use super::*;
    use std::sync::Arc;

    use futures::StreamExt;

    use crate::config::AsyncInputSpec;
    use crate::format::{CustomFormat, FormatError, FormatRegistry};
    use crate::io::AsyncFileInput;
    use crate::{AsyncIoEngine, default_async_registry};

    #[tokio::test]
    async fn async_read_stream_async_returns_per_input_results() {
        let dir = tempfile::tempdir().unwrap();

        // Valid JSON file
        let ok_path = dir.path().join("ok.json");
        tokio::fs::write(&ok_path, r#"{"name": "ok", "value": 1}"#)
            .await
            .unwrap();

        // Invalid JSON files
        let bad1_path = dir.path().join("bad1.json");
        tokio::fs::write(&bad1_path, "{not-json").await.unwrap();

        let bad2_path = dir.path().join("bad2.json");
        tokio::fs::write(&bad2_path, "[1,2,,]").await.unwrap();

        let mk_spec = |path: &std::path::Path| {
            let id = path.to_string_lossy().to_string();
            AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.to_path_buf())))
                .with_format(FormatKind::Json)
                .with_candidates(vec![FormatKind::Json])
        };

        let inputs = vec![mk_spec(&ok_path), mk_spec(&bad1_path), mk_spec(&bad2_path)];

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, inputs, outputs);

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_stream_async::<StreamConfig>(2).collect().await;

        assert_eq!(results.len(), 3);

        // We don't know the exact ordering due to concurrency, so partition Ok/Err
        let oks: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
        let errs: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();

        assert_eq!(oks.len(), 1);
        assert_eq!(oks[0].name, "ok");

        assert_eq!(errs.len(), 2);
        for e in errs {
            assert_eq!(e.stage, Stage::Parse);
            // targets should contain bad1/bad2 paths
            assert!(e.target.contains("bad1") || e.target.contains("bad2"));
        }
    }

    #[tokio::test]
    async fn async_read_records_async_streams_json_rows() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("rows.jsonl");
        let jsonl = "{\"name\":\"foo\",\"value\":1}\n{\"name\":\"bar\",\"value\":2}\n";
        tokio::fs::write(&path, jsonl).await.unwrap();

        let id = path.to_string_lossy().to_string();
        let spec = AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.clone())))
            .with_format(FormatKind::Json)
            .with_candidates(vec![FormatKind::Json]);

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], outputs);

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_records_async::<StreamConfig>(1).collect().await;

        assert_eq!(results.len(), 2);

        let rows: Vec<StreamConfig> = results
            .into_iter()
            .map(|r| r.expect("expected Ok rows"))
            .collect();

        assert_eq!(rows[0].name, "foo");
        assert_eq!(rows[0].value, 1);
        assert_eq!(rows[1].name, "bar");
        assert_eq!(rows[1].value, 2);
    }

    #[tokio::test]
    #[cfg(feature = "csv")]
    async fn async_read_records_async_streams_csv_rows() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("rows.csv");
        let csv = "name,value\nfoo,1\nbar,2\n";
        tokio::fs::write(&path, csv).await.unwrap();

        let id = path.to_string_lossy().to_string();
        let spec = AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.clone())))
            .with_format(FormatKind::Csv)
            .with_candidates(vec![FormatKind::Csv]);

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], outputs);

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_records_async::<StreamConfig>(1).collect().await;

        assert_eq!(results.len(), 2);

        let rows: Vec<StreamConfig> = results
            .into_iter()
            .map(|r| r.expect("expected Ok rows"))
            .collect();

        assert_eq!(rows[0].name, "foo");
        assert_eq!(rows[0].value, 1);
        assert_eq!(rows[1].name, "bar");
        assert_eq!(rows[1].value, 2);
    }

    #[tokio::test]
    #[cfg(feature = "plaintext")]
    async fn async_read_records_async_streams_plaintext_lines() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("lines.txt");
        let content = "alpha\nbeta\n";
        tokio::fs::write(&path, content).await.unwrap();

        let id = path.to_string_lossy().to_string();
        let spec = AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.clone())))
            .with_format(FormatKind::Plaintext)
            .with_candidates(vec![FormatKind::Plaintext]);

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], outputs);

        let results: Vec<Result<String, crate::error::SingleIoError>> =
            engine.read_records_async::<String>(1).collect().await;

        assert_eq!(results.len(), 2);

        let lines: Vec<String> = results
            .into_iter()
            .map(|r| r.expect("expected Ok lines"))
            .collect();

        assert_eq!(lines, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[tokio::test]
    #[cfg(feature = "yaml")]
    async fn async_read_records_async_streams_yaml_documents() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("docs.yaml");
        let yaml = "---\nname: foo\nvalue: 1\n---\nname: bar\nvalue: 2\n";
        tokio::fs::write(&path, yaml).await.unwrap();

        let id = path.to_string_lossy().to_string();
        let spec = AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.clone())))
            .with_format(FormatKind::Yaml)
            .with_candidates(vec![FormatKind::Yaml]);

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, vec![spec], outputs);

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_records_async::<StreamConfig>(1).collect().await;

        assert_eq!(results.len(), 2);

        let rows: Vec<StreamConfig> = results
            .into_iter()
            .map(|r| r.expect("expected Ok documents"))
            .collect();

        assert_eq!(rows[0].name, "foo");
        assert_eq!(rows[0].value, 1);
        assert_eq!(rows[1].name, "bar");
        assert_eq!(rows[1].value, 2);
    }

    #[tokio::test]
    async fn async_read_records_async_with_concurrency_gt_one() {
        let dir = tempfile::tempdir().unwrap();

        let mk_spec = |name: &str| async {
            let path = dir.path().join(format!("{name}.jsonl"));
            let jsonl = format!("{{\"name\":\"{name}\",\"value\":1}}\n");
            tokio::fs::write(&path, jsonl).await.unwrap();

            let id = path.to_string_lossy().to_string();
            AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path)))
                .with_format(FormatKind::Json)
                .with_candidates(vec![FormatKind::Json])
        };

        let a = mk_spec("a").await;
        let b = mk_spec("b").await;
        let c = mk_spec("c").await;

        let inputs = vec![a, b, c];

        let registry = default_async_registry();
        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new(registry, ErrorPolicy::Accumulate, inputs, outputs);

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_records_async::<StreamConfig>(4).collect().await;

        assert_eq!(results.len(), 3);

        let mut names: Vec<String> = results
            .into_iter()
            .map(|r| r.expect("expected Ok rows").name)
            .collect();
        names.sort();

        assert_eq!(
            names,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[tokio::test]
    async fn async_read_records_async_streams_custom_ndjson_via_bridge() {
        let dir = tempfile::tempdir().unwrap();

        let path = dir.path().join("rows.ndjson");
        let jsonl = "{\"name\":\"foo\",\"value\":1}\n{\"name\":\"bar\",\"value\":2}\n";
        tokio::fs::write(&path, jsonl).await.unwrap();

        // Set up a sync registry with a custom NDJSON-like streaming format.
        let mut sync_registry = FormatRegistry::new();

        let fmt = CustomFormat::new("ndjson", &["ndjson"])
            .with_deserialize(|bytes| {
                // Fallback non-streaming handler: parse a single JSON value
                serde_json::from_slice(bytes).map_err(|e| FormatError::Serde(Box::new(e)))
            })
            .with_serialize(|value| {
                serde_json::to_vec(value).map_err(|e| FormatError::Serde(Box::new(e)))
            })
            .with_stream_deserialize(|reader| {
                use std::io::{BufRead, BufReader};

                let buf = BufReader::new(reader);
                let iter = buf.lines().map(|line_res| {
                    let line = line_res.map_err(FormatError::Io)?;
                    let value: serde_json::Value =
                        serde_json::from_str(&line).map_err(|e| FormatError::Serde(Box::new(e)))?;
                    Ok(value)
                });

                Box::new(iter) as Box<dyn Iterator<Item = Result<serde_json::Value, FormatError>>>
            });

        sync_registry.register_custom(fmt);

        // Async registry must know about the custom format kind for resolution.
        let mut async_registry = default_async_registry();
        async_registry.register(FormatKind::Custom("ndjson"));

        let id = path.to_string_lossy().to_string();
        let spec = AsyncInputSpec::new(id, Arc::new(AsyncFileInput::new(path.clone())))
            .with_format(FormatKind::Custom("ndjson"))
            .with_candidates(vec![FormatKind::Custom("ndjson")]);

        let outputs: Vec<crate::config::AsyncOutputSpec> = Vec::new();
        let engine = AsyncIoEngine::new_with_sync_registry(
            async_registry,
            sync_registry,
            ErrorPolicy::Accumulate,
            vec![spec],
            outputs,
        );

        let results: Vec<Result<StreamConfig, crate::error::SingleIoError>> =
            engine.read_records_async::<StreamConfig>(1).collect().await;

        assert_eq!(results.len(), 2);

        let rows: Vec<StreamConfig> = results
            .into_iter()
            .map(|r| r.expect("expected Ok rows"))
            .collect();

        assert_eq!(rows[0].name, "foo");
        assert_eq!(rows[0].value, 1);
        assert_eq!(rows[1].name, "bar");
        assert_eq!(rows[1].value, 2);
    }
}
