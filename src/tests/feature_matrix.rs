use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

fn cargo_check(no_default: bool, features: &[&str], target_dir: &Path) -> Result<(), String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(target_dir).map_err(|e| format!("create target dir failed: {e}"))?;
    let mut cmd = Command::new(cargo);
    cmd.arg("check")
        .arg("--all-targets")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--quiet");
    if no_default {
        cmd.arg("--no-default-features");
    }
    if !features.is_empty() {
        let feat_str = features.to_vec().join(",");
        cmd.arg("--features").arg(feat_str);
    }
    cmd.env("CARGO_TARGET_DIR", target_dir);
    cmd.env("RUSTFLAGS", "-D warnings");

    let output = cmd
        .output()
        .map_err(|e| format!("spawn cargo check failed: {e}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "cargo check failed for no_default={no_default}, features={features:?}\nstdout:\n{stdout}\n\nstderr:\n{stderr}"
        ));
    }

    Ok(())
}

#[test]
fn feature_matrix_compiles() {
    #[derive(Clone, Debug)]
    struct Case {
        no_default: bool,
        features: Vec<&'static str>,
    }

    let cases: Vec<Case> = vec![
        // No features at all.
        Case {
            no_default: true,
            features: vec![],
        },
        // Default (currently plaintext only).
        Case {
            no_default: false,
            features: vec![],
        },
        // Single-format features.
        Case {
            no_default: true,
            features: vec!["plaintext"],
        },
        Case {
            no_default: true,
            features: vec!["sarge"],
        },
        Case {
            no_default: true,
            features: vec!["yaml"],
        },
        Case {
            no_default: true,
            features: vec!["json"],
        },
        Case {
            no_default: true,
            features: vec!["toml"],
        },
        Case {
            no_default: true,
            features: vec!["ini"],
        },
        Case {
            no_default: true,
            features: vec!["xml"],
        },
        Case {
            no_default: true,
            features: vec!["csv"],
        }, // pulls json transitively
        Case {
            no_default: true,
            features: vec!["custom"],
        }, // pulls json transitively
        // Multi-feature and umbrella sets.
        Case {
            no_default: true,
            features: vec!["plaintext", "sarge"],
        },
        Case {
            no_default: true,
            features: vec!["json", "yaml"],
        },
        Case {
            no_default: true,
            features: vec!["custom", "yaml"],
        },
        Case {
            no_default: true,
            features: vec!["async"],
        },
        Case {
            no_default: true,
            features: vec!["full"],
        },
    ];

    // 并行度：可用环境变量 FEATURE_MATRIX_JOBS 覆盖；默认取 CPU 并行度
    let jobs = std::env::var("FEATURE_MATRIX_JOBS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .min(4)
        });
    let jobs = jobs.clamp(1, cases.len().max(1));

    let queue = Arc::new(Mutex::new(VecDeque::from(cases)));
    let failures: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let base_target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("feature-matrix-unit");

    thread::scope(|s| {
        for worker_idx in 0..jobs {
            let queue = Arc::clone(&queue);
            let failures = Arc::clone(&failures);
            let worker_target_dir = base_target_dir.join(format!("worker-{worker_idx}"));

            s.spawn(move || {
                loop {
                    let case = {
                        let mut q = queue.lock().unwrap();
                        q.pop_front()
                    };
                    let Some(case) = case else {
                        break;
                    };

                    let label = format!(
                        "no_default={}, features={:?}",
                        case.no_default, case.features
                    );
                    println!("running feature matrix case: {label}");

                    if let Err(err) =
                        cargo_check(case.no_default, &case.features, &worker_target_dir)
                    {
                        failures.lock().unwrap().push(format!("{label}: {err}"));
                    }
                }
            });
        }
    });

    let failures = failures.lock().unwrap();
    assert!(
        failures.is_empty(),
        "feature matrix failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
