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
}
