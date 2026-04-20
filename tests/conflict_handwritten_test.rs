use std::fs;

use assert_cmd::Command;

#[test]
fn compile_refuses_to_overwrite_handwritten_agents() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    fs::create_dir_all(root.join(".context/invariants")).expect("mkdir");
    fs::create_dir_all(root.join("apps/bundle")).expect("mkdir");

    fs::write(
        root.join(".context/invariants/nested.md"),
        "---\nscope: \"apps/bundle/**\"\n---\nNested rule\n",
    )
    .expect("write rule");

    fs::write(
        root.join("apps/bundle/AGENTS.md"),
        "# Hand written\nDo not overwrite\n",
    )
    .expect("write handwritten");

    Command::cargo_bin("tenet")
        .expect("bin")
        .current_dir(root)
        .args(["compile"])
        .assert()
        .failure();
}
