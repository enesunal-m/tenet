use assert_cmd::Command;
use predicates::str::contains;
use std::fs;

#[test]
fn help_displays_subcommands() {
    let mut cmd = Command::cargo_bin("tenet").expect("binary exists");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(contains("init"))
        .stdout(contains("compile"))
        .stdout(contains("lint"));
}

#[test]
fn version_subcommand_prints_semver() {
    let mut cmd = Command::cargo_bin("tenet").expect("binary exists");
    cmd.arg("version");

    cmd.assert().success().stdout(contains("tenet 0.1.0"));
}

#[test]
fn init_scaffolds_context_without_hook() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tenet")
        .expect("binary exists")
        .current_dir(root)
        .args(["init", "--no-hook"])
        .assert()
        .success()
        .stdout(contains("initialized tenet"));

    assert!(root.join(".context/invariants/example.md").exists());
    assert!(root.join(".tenetrc").exists());
    assert!(root.join("AGENTS.md").exists());
}

#[test]
fn add_accepts_type_and_required_flags() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");

    Command::cargo_bin("tenet")
        .expect("binary exists")
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
            "CLI smoke",
        ])
        .assert()
        .success()
        .stdout(contains("invariants/cli-smoke"));

    assert!(root.join(".context/invariants/cli-smoke.md").exists());
}
