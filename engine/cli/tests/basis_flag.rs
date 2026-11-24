//! The `--basis` flag of the command line: independent by default, legacy on
//! request, and an unknown word refused.

use serde_json::Value;
use std::process::Command;

fn json(args: &[&str]) -> (bool, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_nush2"))
        .arg("json")
        .args(args)
        .output()
        .expect("the nush2 binary runs");
    let value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.success(), value)
}

#[test]
fn the_default_basis_is_the_plants_own_ledger() {
    let (ok, v) = json(&[]);
    assert!(ok);
    assert_eq!(v["carbon"]["src"], "Plant");
    assert_eq!(v["emissions"]["baseline_id"], "IEAGHG_9p00");
    let (ok, explicit) = json(&["--basis", "independent"]);
    assert!(ok);
    assert_eq!(v, explicit);
}

#[test]
fn the_legacy_basis_keeps_the_anchored_ledger() {
    let (ok, v) = json(&["--basis", "legacy"]);
    assert!(ok);
    assert_eq!(v["carbon"]["src"], "Anchored");
    let a = v["emissions"]["a_net"].as_f64().unwrap();
    assert!((a - 578051.6117).abs() < 1e-3, "{a}");
    assert_eq!(v["emissions"]["ts_emission"], 0.0);
}

#[test]
fn an_unknown_basis_is_refused() {
    let (ok, _) = json(&["--basis", "other"]);
    assert!(!ok);
}
