use std::fs;

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn add_creates_rule_non_interactively() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .env("EDITOR", "true")
        .args([
            "add",
            "invariants",
            "--scope",
            "**",
            "--owner",
            "alice",
            "--priority",
            "normal",
            "--title",
            "Test rule",
        ])
        .assert()
        .success()
        .stdout(contains("invariants/test-rule"));

    let path = root.join(".context/invariants/test-rule.md");
    let content = fs::read_to_string(path).expect("read rule");
    assert!(content.contains("scope: \"**\""));
    assert!(content.contains("owner: alice"));
}

#[test]
fn add_with_invalid_priority_exits_one() {
    let mut cmd = Command::cargo_bin("tenet").expect("bin");
    cmd.args([
        "add",
        "invariants",
        "--scope",
        "**",
        "--owner",
        "alice",
        "--priority",
        "urgent",
        "--title",
        "Bad",
    ]);
    cmd.assert().code(1);
}
