//! Compilation tests for representative feature combinations.
//!
//! These tests spawn `cargo check --tests` on this crate with different feature
//! sets to ensure optional-dependency gating stays correct.

use std::env;
use std::process::Command;

fn cargo_check(no_default: bool, features: &[&str]) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let target_dir = tempfile::tempdir().expect("tempdir for feature matrix");

    let mut cmd = Command::new(cargo);
    cmd.arg("check")
        .arg("--tests")
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

    cmd.env("CARGO_TARGET_DIR", target_dir.path());

    let status = cmd.status().expect("run cargo check");
    assert!(
        status.success(),
        "cargo check failed for no_default={no_default}, features={features:?}"
    );
}

#[test]
fn feature_matrix_compiles() {
    // No features at all.
    cargo_check(true, &[]);

    // Default (currently plaintext only).
    cargo_check(false, &[]);

    // Single-format features.
    cargo_check(true, &["plaintext"]);
    cargo_check(true, &["yaml"]);
    cargo_check(true, &["json"]);
    cargo_check(true, &["toml"]);
    cargo_check(true, &["ini"]);
    cargo_check(true, &["xml"]);
    cargo_check(true, &["csv"]); // pulls json transitively

    // Multi-feature and umbrella sets.
    cargo_check(true, &["json", "yaml"]);
    cargo_check(true, &["async"]);
    cargo_check(true, &["full"]);
}
