use assert_cmd::Command;
use predicates::prelude::*;

fn why_cmd() -> Command {
    let mut cmd = Command::cargo_bin("why").unwrap();
    // Clear env vars that could interfere with tests
    cmd.env_remove("WHY_LAST_EXIT");
    cmd.env_remove("WHY_LAST_STDERR");
    cmd.env_remove("WHY_DB");
    cmd
}

// ── Direct lookup ────────────────────────────────────────────────────

#[test]
fn lookup_rust_error() {
    why_cmd()
        .arg("E0499")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0499")
                .and(predicate::str::contains("Multiple mutable borrows"))
                .and(predicate::str::contains("Fix:")),
        );
}

#[test]
fn lookup_python_error() {
    why_cmd()
        .arg("TypeError")
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeError"));
}

#[test]
fn lookup_unknown_error() {
    why_cmd()
        .arg("E9999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No explanation found"));
}

#[test]
fn lookup_case_insensitive() {
    why_cmd()
        .arg("e0499")
        .assert()
        .success()
        .stdout(predicate::str::contains("E0499"));
}

// ── Env-based detection ──────────────────────────────────────────────

#[test]
fn env_detection_rust_error() {
    why_cmd()
        .env("WHY_LAST_EXIT", "1")
        .env("WHY_LAST_STDERR", "error[E0308]: mismatched types")
        .assert()
        .success()
        .stdout(predicate::str::contains("E0308"));
}

#[test]
fn env_detection_exit_zero_means_no_error() {
    why_cmd()
        .env("WHY_LAST_EXIT", "0")
        .env("WHY_LAST_STDERR", "error[E0308]: mismatched types")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No error detected"));
}

#[test]
fn env_detection_empty_stderr() {
    why_cmd()
        .env("WHY_LAST_EXIT", "1")
        .env("WHY_LAST_STDERR", "")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No error detected"));
}

// ── Pattern-based detection via env ──────────────────────────────────

#[test]
fn env_detection_git_merge_conflict() {
    why_cmd()
        .env("WHY_LAST_EXIT", "1")
        .env(
            "WHY_LAST_STDERR",
            "CONFLICT (content): Merge conflict in src/main.rs\nAutomatic merge failed; fix conflicts and then commit the result.",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("merge-conflict").or(predicate::str::contains("Merge conflict")));
}

#[test]
fn env_detection_c_undeclared_identifier() {
    why_cmd()
        .env("WHY_LAST_EXIT", "1")
        .env(
            "WHY_LAST_STDERR",
            "main.c:5:5: error: 'x' undeclared (first use in this function)",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("undeclared"));
}

// ── List mode ────────────────────────────────────────────────────────

#[test]
fn list_all_errors() {
    why_cmd()
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("errors across"));
}

#[test]
fn list_filter_by_language() {
    why_cmd()
        .args(["--list", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("E0499"));
}

#[test]
fn list_unknown_language() {
    why_cmd()
        .args(["--list", "cobol"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No error entries found"));
}

// ── Hook output ──────────────────────────────────────────────────────

#[test]
fn hook_bash() {
    why_cmd()
        .args(["--hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WHY_LAST_EXIT"));
}

#[test]
fn hook_zsh() {
    why_cmd()
        .args(["--hook", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WHY_LAST_EXIT"));
}

#[test]
fn hook_unknown_shell() {
    why_cmd()
        .args(["--hook", "fish"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown shell"));
}

// ── Version and help ─────────────────────────────────────────────────

#[test]
fn version_flag() {
    why_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("why"));
}

#[test]
fn help_flag() {
    why_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Plain English explanations"));
}
