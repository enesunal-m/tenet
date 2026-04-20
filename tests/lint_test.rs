use std::fs;

use assert_cmd::Command;

#[test]
fn lint_reports_expected_findings_and_exits_two_when_errors_present() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::create_dir_all(root.join(".context/random")).expect("mkdir");

    fs::write(
        root.join(".context/invariants/bad_yaml.md"),
        "---\n: nope\n---\nbody\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/abs_scope.md"),
        "---\nscope: \"/absolute\"\n---\nbody\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/secret.md"),
        "---\n---\nthis has ghp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/Bad_Name.md"),
        "---\ncustom: true\n---\nbody\n",
    )
    .expect("write");
    fs::write(root.join(".context/invariants/empty-body.md"), "---\n---\n").expect("write");
    fs::write(
        root.join(".context/invariants/bad-scope.md"),
        "---\nscope: \"[\"\n---\nbody\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/bad-date.md"),
        "---\nreviewed: tomorrow\n---\nbody\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/bad-priority.md"),
        "---\npriority: urgent\n---\nbody\n",
    )
    .expect("write");
    fs::write(
        root.join(".context/invariants/missing-dir.md"),
        "---\nscope: \"apps/missing/**\"\n---\nbody\n",
    )
    .expect("write");
    fs::write(root.join(".context/random/other.md"), "body\n").expect("write");

    let output = Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["lint"])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).expect("utf8");
    assert!(text.contains("bad-frontmatter"));
    assert!(text.contains("abs-scope"));
    assert!(text.contains("bad-scope"));
    assert!(text.contains("bad-date"));
    assert!(text.contains("bad-priority"));
    assert!(text.contains("unknown-field"));
    assert!(text.contains("empty-body"));
    assert!(text.contains("bad-filename"));
    assert!(text.contains("missing-dir"));
    assert!(text.contains("secret-github"));
    assert!(text.contains("unknown-type-dir"));
}

#[test]
fn lint_respects_configured_secret_scanning_default_and_override() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    Command::new("git")
        .current_dir(root)
        .args(["init"])
        .assert()
        .success();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::write(
        root.join(".tenetrc"),
        "[lint]\ncheck_secrets = false\ncheck_filenames = true\n",
    )
    .expect("write config");
    fs::write(
        root.join(".context/invariants/secret.md"),
        "---\n---\nthis has ghp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
    )
    .expect("write");

    let output = Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["lint"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8");
    assert!(!text.contains("secret-github"));

    let output = Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["lint", "--check-secrets"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8");
    assert!(text.contains("secret-github"));
}
