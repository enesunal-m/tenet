use std::{env, fs};

use assert_cmd::Command;

#[test]
fn pre_commit_hook_blocks_when_context_changes_without_compile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "you@example.com"])
        .assert()
        .success();
    Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Your Name"])
        .assert()
        .success();

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    Command::new("git")
        .current_dir(root)
        .args(["add", "-A"])
        .assert()
        .success();
    Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "initial"])
        .assert()
        .success();

    fs::write(
        root.join(".context/invariants/example.md"),
        "---\nscope: \"**\"\n---\nchanged\n",
    )
    .expect("write");

    Command::new("git")
        .current_dir(root)
        .args(["add", "-A"])
        .assert()
        .success();

    let tenet_bin = assert_cmd::cargo::cargo_bin("tenet");
    let bin_dir = tenet_bin.parent().expect("bin parent");
    let current_path = env::var("PATH").unwrap_or_default();
    let merged_path = format!("{}:{}", bin_dir.display(), current_path);
    Command::new("git")
        .current_dir(root)
        .env("PATH", merged_path)
        .args(["commit", "-m", "should fail"])
        .assert()
        .failure();
}
