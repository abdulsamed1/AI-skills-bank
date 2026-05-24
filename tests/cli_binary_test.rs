use assert_cmd::Command;
use predicates::str::contains;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has parent")
        .to_path_buf()
}

#[test]
fn test_help_includes_release_gate() {
    let mut cmd = Command::cargo_bin("skills-bank").expect("binary exists");
    cmd.current_dir(repo_root());
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("release-gate"));
}

#[test]
fn test_unknown_command_fails() {
    let mut cmd = Command::cargo_bin("skills-bank").expect("binary exists");
    cmd.current_dir(repo_root());
    cmd.arg("definitely-not-a-command");
    cmd.assert()
        .failure()
        .stderr(contains("Unknown command"));
}

#[test]
fn test_doctor_command_executes() {
    let mut cmd = Command::cargo_bin("skills-bank").expect("binary exists");
    cmd.current_dir(repo_root());
    cmd.arg("doctor");
    cmd.assert().success();
}
