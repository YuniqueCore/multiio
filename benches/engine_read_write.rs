use std::sync::Arc;

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use multiio::{ErrorPolicy, InMemorySink, InMemorySource, IoEngine, default_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    value: i32,
}

fn make_engine(num_inputs: usize) -> IoEngine {
    let registry = default_registry();

    let json = r#"{"name": "a", "value": 1}"#;

    let mut inputs = Vec::with_capacity(num_inputs);
    for i in 0..num_inputs {
        let id = format!("in-{i}");
        let src = Arc::new(InMemorySource::from_string(id.clone(), json));
        let spec = multiio::InputSpec::new(id, src)
            .with_format(multiio::FormatKind::Json)
            .with_candidates(vec![multiio::FormatKind::Json]);
        inputs.push(spec);
    }

    let sink = Arc::new(InMemorySink::new("out"));
    let output = multiio::OutputSpec::new("out", sink)
        .with_format(multiio::FormatKind::Json)
        .with_candidates(vec![multiio::FormatKind::Json])
        .with_file_exists_policy(multiio::FileExistsPolicy::Overwrite);

    IoEngine::new(registry, ErrorPolicy::Accumulate, inputs, vec![output])
}

fn bench_engine_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_read_write_inmemory");

    for &n in &[1usize, 4, 16, 64] {
        group.bench_function(format!("read_write_{n}"), |b| {
            b.iter_batched(
                || make_engine(n),
                |engine| {
                    let values: Vec<Config> = engine.read_all().expect("read_all");
                    black_box(&values);
                    let _ = engine.write_all(&values);
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, bench_engine_read_write);
criterion_main!(benches);
