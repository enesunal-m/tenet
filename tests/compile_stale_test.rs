use std::fs;

use assert_cmd::Command;

#[test]
fn stale_command_exits_one_when_stale_rules_exist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::write(
        root.join(".context/invariants/stale.md"),
        "---\nreviewed: 2000-01-01\n---\nold\n",
    )
    .expect("write rule");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["stale"])
        .assert()
        .code(1);
}

#[test]
fn list_can_filter_by_scope() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::write(
        root.join(".context/invariants/a.md"),
        "---\nscope: \"apps/bundle/**\"\n---\nA\n",
    )
    .expect("write rule");

    let output = Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["list", "--scope", "apps/bundle/src/main.rs"])
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).expect("utf8");
    assert!(stdout.contains("invariants/a"));
}
