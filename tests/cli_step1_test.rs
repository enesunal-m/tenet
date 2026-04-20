use assert_cmd::Command;
use predicates::str::contains;

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
fn init_stub_reports_not_implemented() {
    let mut cmd = Command::cargo_bin("tenet").expect("binary exists");
    cmd.arg("init");

    cmd.assert().success().stdout(contains("not implemented"));
}

#[test]
fn add_stub_accepts_type_and_reports_not_implemented() {
    let mut cmd = Command::cargo_bin("tenet").expect("binary exists");
    cmd.args(["add", "invariants"]);

    cmd.assert().success().stdout(contains("not implemented"));
}
