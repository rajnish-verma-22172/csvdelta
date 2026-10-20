use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run(old: &str, new: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_csvdelta"))
        .arg(fixture(old))
        .arg(fixture(new))
        .args(["--key", "id"])
        .args(extra)
        .output()
        .expect("csvdelta binary should be built by Cargo")
}

fn json_output(old: &str, new: &str, expected_code: i32) -> serde_json::Value {
    let output = run(old, new, &["--format", "json"]);
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn identical_files_have_no_changes() {
    let report = json_output("identical_old.csv", "identical_new.csv", 0);
    assert!(report["added"].as_array().unwrap().is_empty());
    assert!(report["removed"].as_array().unwrap().is_empty());
    assert!(report["changed"].as_array().unwrap().is_empty());
}

#[test]
fn added_and_removed_rows_are_reported() {
    let report = json_output("rows_old.csv", "rows_new.csv", 1);
    assert_eq!(report["added"][0]["key"], "4");
    assert_eq!(report["removed"][0]["key"], "2");
}

#[test]
fn changed_cells_include_before_and_after_values() {
    let report = json_output("changed_old.csv", "changed_new.csv", 1);
    let changes = report["changed"].as_array().unwrap();
    assert_eq!(changes.len(), 3);
    assert!(changes.iter().any(|change| change["new"] == "Widget Pro"));
    assert!(changes
        .iter()
        .any(|change| change["old"] == "10" && change["new"] == "14"));
    assert!(changes
        .iter()
        .any(|change| change["old"] == "4.50" && change["new"] == "4.75"));
}

#[test]
fn column_reordering_is_not_a_row_change() {
    let report = json_output("reordered_old.csv", "reordered_new.csv", 0);
    assert!(report["changed"].as_array().unwrap().is_empty());
}

#[test]
fn duplicate_key_is_a_command_error() {
    let output = run(
        "duplicate_key.csv",
        "identical_new.csv",
        &["--format", "json"],
    );
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(
        stderr.contains("duplicate") && stderr.contains("id"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn quoted_commas_and_embedded_newlines_are_parsed_as_fields() {
    let report = json_output("quoted_old.csv", "quoted_new.csv", 1);
    let changes = report["changed"].as_array().unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["old"], "Compact widget for lab use");
    assert_eq!(changes[0]["new"], "Compact widget for field use");
}

#[test]
fn default_key_is_the_first_column_and_all_formats_are_available() {
    let default_key = Command::new(env!("CARGO_BIN_EXE_csvdelta"))
        .arg(fixture("rows_old.csv"))
        .arg(fixture("rows_new.csv"))
        .output()
        .unwrap();
    assert_eq!(default_key.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&default_key.stdout).contains("1 added, 1 removed"));

    for format in ["table", "markdown"] {
        let output = run("changed_old.csv", "changed_new.csv", &["--format", format]);
        assert_eq!(output.status.code(), Some(1));
        assert!(!output.stdout.is_empty());
    }
}

#[test]
fn missing_file_is_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_csvdelta"))
        .arg(fixture("does-not-exist.csv"))
        .arg(fixture("identical_new.csv"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("does-not-exist.csv"));
}

#[test]
fn empty_file_is_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_csvdelta"))
        .arg(fixture("empty.csv"))
        .arg(fixture("identical_new.csv"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .to_lowercase()
        .contains("empty"));
}

#[test]
fn missing_key_column_is_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_csvdelta"))
        .arg(fixture("identical_old.csv"))
        .arg(fixture("identical_new.csv"))
        .args(["--key", "sku"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("sku"));
}
