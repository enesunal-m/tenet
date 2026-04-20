use std::fs;

use assert_cmd::Command;

#[test]
fn migrate_non_interactive_creates_context_files_and_keeps_source() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::write(
        root.join("AGENTS.md"),
        "# Project\n\n## Auth\nUse sessions.\n\n## Errors\nHandle retries.\n\n## Terms\nJWT means token.\n",
    )
    .expect("write source");

    fs::write(
        root.join("mapping.toml"),
        "[sections.Auth]\ntype = \"invariants\"\nscope = \"**\"\n\n[sections.Errors]\ntype = \"gotchas\"\nscope = \"apps/**\"\n\n[sections.Terms]\ntype = \"glossary\"\nscope = \"**\"\n",
    )
    .expect("write mapping");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args([
            "migrate",
            "--from",
            "AGENTS.md",
            "--yes",
            "--mapping",
            "mapping.toml",
        ])
        .assert()
        .success();

    assert!(root.join(".context/invariants/auth.md").exists());
    assert!(root.join(".context/gotchas/errors.md").exists());
    assert!(root.join(".context/glossary/terms.md").exists());
    assert!(root.join("AGENTS.md").exists());
}
