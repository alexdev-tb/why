use regex::Regex;
use std::env;
use std::process::Command;

pub fn from_env() -> Option<String> {
    let exit_code = env::var("WHY_LAST_EXIT").ok()?;
    if exit_code == "0" {
        return None;
    }

    if let Some(stderr) = env::var("WHY_LAST_STDERR").ok().filter(|s| !s.is_empty()) {
        if let Some(code) = extract_error_code(&stderr) {
            return Some(code);
        }
    }

    // TODO: Replace re-run with a background daemon or shell-level stderr capture
    // (e.g. a `why-daemon` on a Unix socket, or eBPF tracing)
    let last_cmd = env::var("WHY_LAST_CMD").ok().filter(|s| !s.is_empty())?;
    let stderr = rerun_for_stderr(&last_cmd)?;
    extract_error_code(&stderr)
}

const SAFE_COMMANDS: &[&str] = &[
    "cargo", "rustc", "gcc", "g++", "clang", "clang++", "make", "cmake",
    "python", "python3", "pip", "pip3",
    "node", "npm", "npx", "yarn", "pnpm", "bun", "deno", "tsc",
    "go", "javac", "kotlinc", "scalac", "ghc", "swiftc",
    "ruby", "perl", "php", "lua", "elixir", "erlc",
    "dotnet", "csc", "mcs",
    "zig", "nim", "dart", "flutter",
    "eslint", "prettier", "clippy", "mypy", "pylint", "ruff",
    "shellcheck", "hadolint",
    "ls", "cat", "head", "tail", "grep", "find", "wc",
    "git",
];

fn is_safe_to_rerun(cmd: &str) -> bool {
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    let binary = first_word.rsplit('/').next().unwrap_or(first_word);
    SAFE_COMMANDS.iter().any(|&safe| binary == safe)
}

fn rerun_for_stderr(cmd: &str) -> Option<String> {
    if !is_safe_to_rerun(cmd) {
        return None;
    }

    eprintln!(
        "  {} Re-running `{}` to capture error...",
        "\u{2192}",
        cmd
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if stderr.is_empty() {
        None
    } else {
        Some(stderr)
    }
}

pub fn extract_error_code(stderr: &str) -> Option<String> {
    let rust_re = Regex::new(r"error\[E(\d+)\]").unwrap();
    if let Some(caps) = rust_re.captures(stderr) {
        return Some(format!("E{}", &caps[1]));
    }

    let python_re = Regex::new(r"(?m)^(\w*(?:Error|Exception|Warning))\b").unwrap();
    if let Some(caps) = python_re.captures(stderr) {
        return Some(caps[1].to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_error_code() {
        let stderr = r#"error[E0499]: cannot borrow `x` as mutable more than once at a time
 --> src/main.rs:4:13"#;
        assert_eq!(extract_error_code(stderr), Some("E0499".to_string()));
    }

    #[test]
    fn test_extract_rust_error_code_e0308() {
        let stderr = "error[E0308]: mismatched types";
        assert_eq!(extract_error_code(stderr), Some("E0308".to_string()));
    }

    #[test]
    fn test_extract_python_exception() {
        let stderr = "Traceback (most recent call last):\n  File \"test.py\", line 1\nTypeError: unsupported operand type";
        assert_eq!(extract_error_code(stderr), Some("TypeError".to_string()));
    }

    #[test]
    fn test_extract_no_match() {
        let stderr = "some random output";
        assert_eq!(extract_error_code(stderr), None);
    }

    #[test]
    fn test_is_safe_to_rerun() {
        assert!(is_safe_to_rerun("cargo build"));
        assert!(is_safe_to_rerun("rustc main.rs"));
        assert!(is_safe_to_rerun("python3 script.py"));
        assert!(is_safe_to_rerun("gcc -o main main.c"));
        assert!(is_safe_to_rerun("npm run build"));
        assert!(is_safe_to_rerun("/usr/bin/cargo build"));
        assert!(is_safe_to_rerun("git status"));
    }

    #[test]
    fn test_is_not_safe_to_rerun() {
        assert!(!is_safe_to_rerun("rm -rf /"));
        assert!(!is_safe_to_rerun("curl -X POST http://example.com"));
        assert!(!is_safe_to_rerun("docker run something"));
        assert!(!is_safe_to_rerun("sudo anything"));
        assert!(!is_safe_to_rerun("ssh server"));
        assert!(!is_safe_to_rerun(""));
    }
}
