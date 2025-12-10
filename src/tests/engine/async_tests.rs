#![cfg(feature = "async")]

use std::sync::Arc;

use crate::config::{AsyncInputSpec, AsyncOutputSpec, FileExistsPolicy};
use crate::error::{AggregateError, ErrorPolicy, Stage};
use crate::io::{AsyncFileInput, AsyncFileOutput, AsyncInputProvider};
use crate::{AsyncIoEngine, FormatKind, default_async_registry};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    name: String,
    value: i32,
}

fn make_engine(
    error_policy: ErrorPolicy,
    inputs: Vec<AsyncInputSpec>,
    outputs: Vec<AsyncOutputSpec>,
) -> AsyncIoEngine {
    let registry = default_async_registry();
    AsyncIoEngine::new(registry, error_policy, inputs, outputs)
}

#[tokio::test]
async fn async_engine_read_write_file_ok() {
    let dir = tempfile::tempdir().unwrap();
    let in_path = dir.path().join("input.json");
    let out_path = dir.path().join("output.json");

    let json = r#"{"name": "a", "value": 1}"#;
    tokio::fs::write(&in_path, json).await.unwrap();

    let in_id = in_path.to_string_lossy().into_owned();
    let out_id = out_path.to_string_lossy().into_owned();

    let in_provider = Arc::new(AsyncFileInput::new(in_path.clone()));
    let input_spec = AsyncInputSpec::new(in_id.clone(), in_provider)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let out_target = Arc::new(AsyncFileOutput::new(out_path.clone()));
    let output_spec = AsyncOutputSpec::new(out_id.clone(), out_target)
        .with_format(FormatKind::Json)
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(ErrorPolicy::FastFail, vec![input_spec], vec![output_spec]);

    let values: Vec<Config> = engine.read_all().await.expect("read_all should succeed");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].name, "a");
    assert_eq!(values[0].value, 1);

    engine
        .write_all(&values)
        .await
        .expect("write_all should succeed");

    let out_bytes = tokio::fs::read(&out_path).await.unwrap();
    let decoded: Vec<Config> = serde_json::from_slice(&out_bytes).unwrap();
    assert_eq!(decoded, values);
}

#[tokio::test]
async fn async_engine_fast_fail_on_open_error() {
    #[derive(Debug)]
    struct FailingAsyncInput {
        id: String,
    }

    #[async_trait::async_trait]
    impl AsyncInputProvider for FailingAsyncInput {
        fn id(&self) -> &str {
            &self.id
        }

        async fn open(&self) -> std::io::Result<Box<dyn AsyncRead + Unpin + Send>> {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "simulated async network timeout",
            ))
        }
    }

    let src = Arc::new(FailingAsyncInput {
        id: "net://async-example".to_string(),
    });

    let input_spec = AsyncInputSpec::new("net://async-example", src)
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.json");
    let out_target = Arc::new(AsyncFileOutput::new(out_path));
    let output_spec = AsyncOutputSpec::new("out", out_target)
        .with_format(FormatKind::Json)
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(ErrorPolicy::FastFail, vec![input_spec], vec![output_spec]);

    let result: Result<Vec<Config>, AggregateError> = engine.read_all().await;
    let agg = result.expect_err("expected failure due to open error");

    assert_eq!(agg.errors.len(), 1);
    let e = &agg.errors[0];
    assert_eq!(e.stage, Stage::Open);
    assert_eq!(e.target, "net://async-example");
}

#[tokio::test]
async fn async_engine_accumulate_parse_errors() {
    let dir = tempfile::tempdir().unwrap();

    let ok_path = dir.path().join("ok.json");
    tokio::fs::write(&ok_path, r#"{"name": "ok", "value": 1}"#)
        .await
        .unwrap();

    let bad1_path = dir.path().join("bad1.json");
    tokio::fs::write(&bad1_path, "{not-json").await.unwrap();

    let bad2_path = dir.path().join("bad2.json");
    tokio::fs::write(&bad2_path, "[1,2,,]").await.unwrap();

    let ok_id = ok_path.to_string_lossy().to_string();
    let ok_spec = AsyncInputSpec::new(ok_id, Arc::new(AsyncFileInput::new(ok_path.clone())))
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let bad1_id = bad1_path.to_string_lossy().to_string();
    let bad1_spec = AsyncInputSpec::new(bad1_id, Arc::new(AsyncFileInput::new(bad1_path.clone())))
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let bad2_id = bad2_path.to_string_lossy().to_string();
    let bad2_spec = AsyncInputSpec::new(bad2_id, Arc::new(AsyncFileInput::new(bad2_path.clone())))
        .with_format(FormatKind::Json)
        .with_candidates(vec![FormatKind::Json]);

    let out_path = dir.path().join("out.json");
    let out_id = out_path.to_string_lossy().to_string();
    let out_spec = AsyncOutputSpec::new(out_id, Arc::new(AsyncFileOutput::new(out_path.clone())))
        .with_format(FormatKind::Json)
        .with_file_exists_policy(FileExistsPolicy::Overwrite);

    let engine = make_engine(
        ErrorPolicy::Accumulate,
        vec![ok_spec, bad1_spec, bad2_spec],
        vec![out_spec],
    );

    let result: Result<Vec<Config>, AggregateError> = engine.read_all().await;
    let agg = result.expect_err("expected aggregate parse errors");

    assert_eq!(agg.errors.len(), 2);
    assert!(agg.errors.iter().all(|e| e.stage == Stage::Parse));

    let targets: Vec<_> = agg.errors.iter().map(|e| e.target.as_str()).collect();
    assert!(targets.iter().any(|t| t.contains("bad1")));
    assert!(targets.iter().any(|t| t.contains("bad2")));
}
