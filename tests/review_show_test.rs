use std::fs;

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn review_updates_date_and_show_prints_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::write(
        root.join(".context/invariants/rule.md"),
        "---\nscope: \"**\"\n---\nBody\n",
    )
    .expect("write rule");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["review", "invariants/rule"])
        .assert()
        .success();

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["show", "invariants/rule"])
        .assert()
        .success()
        .stdout(contains("reviewed:"))
        .stdout(contains("Body"));
}
