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
    assert!(text.contains("secret-github"));
    assert!(text.contains("unknown-type-dir"));
}
