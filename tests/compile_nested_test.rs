use std::fs;

use assert_cmd::Command;

#[test]
fn compile_writes_nested_agents_for_nested_scope() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::create_dir_all(root.join("apps/bundle")).expect("mkdir");

    fs::write(
        root.join(".context/invariants/root.md"),
        "---\nscope: \"**\"\n---\nGlobal rule\n",
    )
    .expect("write rule");

    fs::write(
        root.join(".context/invariants/nested.md"),
        "---\nscope: \"apps/bundle/**\"\n---\nNested rule\n",
    )
    .expect("write rule");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["compile"])
        .assert()
        .success();

    let root_agents = fs::read_to_string(root.join("AGENTS.md")).expect("read root agents");
    let nested_agents =
        fs::read_to_string(root.join("apps/bundle/AGENTS.md")).expect("read nested agents");

    assert!(root_agents.contains("Global rule"));
    assert!(!root_agents.contains("Nested rule"));

    assert!(nested_agents.contains("Nested rule"));
    assert!(!nested_agents.contains("Global rule"));
}
