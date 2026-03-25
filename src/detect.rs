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

struct PatternRule {
    groups: &'static [&'static [&'static str]],
    exclude: &'static [&'static str],
    code: &'static str,
}

/// C/C++ error patterns
const C_CPP_PATTERNS: &[PatternRule] = &[
    PatternRule { groups: &[&["undeclared", "first use in this function"]], exclude: &[], code: "undeclared-identifier" },
    PatternRule { groups: &[&["multiple definition of"]], exclude: &[], code: "multiple-definition" },
    PatternRule { groups: &[&["undefined reference"]], exclude: &[], code: "undefined-reference" },
    PatternRule { groups: &[&["incompatible types when"], &["incompatible type"]], exclude: &[], code: "type-mismatch" },
    PatternRule { groups: &[&["implicit declaration of function"]], exclude: &[], code: "implicit-function-declaration" },
    PatternRule { groups: &[&["too few arguments"]], exclude: &[], code: "function-argument-mismatch" },
    PatternRule { groups: &[&["too many arguments"]], exclude: &["go"], code: "function-argument-mismatch" },
    PatternRule { groups: &[&["No such file or directory", "fatal error"]], exclude: &[], code: "missing-header" },
    PatternRule { groups: &[&["has no member named"]], exclude: &[], code: "no-member" },
    PatternRule { groups: &[&["redefinition of"]], exclude: &[], code: "redefinition" },
    PatternRule { groups: &[&["format", "expects argument of type"]], exclude: &[], code: "format-specifier" },
    PatternRule { groups: &[&["is used uninitialized"], &["may be uninitialized"]], exclude: &[], code: "uninitialized-variable" },
    PatternRule { groups: &[&["array subscript", "above array bounds"]], exclude: &[], code: "array-out-of-bounds" },
    PatternRule { groups: &[&["control reaches end of non-void function"]], exclude: &[], code: "return-type-mismatch" },
    PatternRule { groups: &[&["double free"]], exclude: &[], code: "double-free" },
    PatternRule { groups: &[&["stack smashing detected"]], exclude: &[], code: "stack-smashing" },
    PatternRule { groups: &[&["Segmentation fault"], &["segmentation fault"]], exclude: &[], code: "segmentation-fault" },
    PatternRule { groups: &[&["expected", "before"], &["expected", "';'"]], exclude: &[], code: "syntax-error" },
];

/// Go error patterns.
const GO_PATTERNS: &[PatternRule] = &[
    PatternRule { groups: &[&["undefined:"]], exclude: &[], code: "undefined" },
    PatternRule { groups: &[&["cannot use"]], exclude: &[], code: "cannot-use-type" },
    PatternRule { groups: &[&["no required module"]], exclude: &[], code: "no-required-module" },
    PatternRule { groups: &[&["syntax error"]], exclude: &[], code: "syntax-error" },
    PatternRule { groups: &[&["not enough arguments"]], exclude: &[], code: "not-enough-arguments" },
    PatternRule { groups: &[&["too many arguments"]], exclude: &[], code: "too-many-arguments" },
    PatternRule { groups: &[&["invalid operation"]], exclude: &[], code: "invalid-operation" },
    PatternRule { groups: &[&["assignment mismatch"]], exclude: &[], code: "assignment-mismatch" },
    PatternRule { groups: &[&["concurrent map"]], exclude: &[], code: "concurrent-map-write" },
    PatternRule { groups: &[&["declared but not used"]], exclude: &[], code: "unused-variable" },
    PatternRule { groups: &[&["imported and not used"]], exclude: &[], code: "unused-import" },
    PatternRule { groups: &[&["interface", "is", "not"]], exclude: &[], code: "type-assertion-failed" },
    PatternRule { groups: &[&["index out of range"]], exclude: &[], code: "index-out-of-range" },
    PatternRule { groups: &[&["assignment to entry in nil map"]], exclude: &[], code: "assignment-to-nil-map" },
    PatternRule { groups: &[&["runtime error"]], exclude: &[], code: "panic-runtime-error" },
];

fn match_line(line: &str, rule: &PatternRule) -> bool {
    let any_group_matches = rule.groups.iter().any(|group| {
        group.iter().all(|pat| line.contains(pat))
    });
    any_group_matches && rule.exclude.iter().all(|pat| !line.contains(pat))
}

fn match_patterns(stderr: &str, rules: &[PatternRule]) -> Option<String> {
    for line in stderr.lines() {
        for rule in rules {
            if match_line(line, rule) {
                return Some(rule.code.to_string());
            }
        }
    }
    None
}

pub fn extract_error_code(stderr: &str) -> Option<String> {
    let rust_re = Regex::new(r"error\[E(\d+)\]").unwrap();
    if let Some(caps) = rust_re.captures(stderr) {
        return Some(format!("E{}", &caps[1]));
    }

    // Python detection
    let python_re = Regex::new(r"(?m)^((?:\w*(?:Error|Exception|Warning)|StopIteration|KeyboardInterrupt|SystemExit|GeneratorExit|BaseException))\b").unwrap();
    if let Some(caps) = python_re.captures(stderr) {
        return Some(caps[1].to_string());
    }

    // C/C++ (checked before Go to avoid collisions)
    if let Some(code) = match_patterns(stderr, C_CPP_PATTERNS) {
        return Some(code);
    }

    // Go detection
    match_patterns(stderr, GO_PATTERNS)
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_error_codes() {
        let cases = vec![
            (r#"error[E0499]: cannot borrow `x` as mutable more than once at a time
 --> src/main.rs:4:13"#, Some("E0499")),
            ("error[E0308]: mismatched types", Some("E0308")),
        ];
        for (stderr, expected) in cases {
            assert_eq!(extract_error_code(stderr), expected.map(String::from), "failed on: {stderr}");
        }
    }

    #[test]
    fn test_extract_python_exception() {
        let stderr = "Traceback (most recent call last):\n  File \"test.py\", line 1\nTypeError: unsupported operand type";
        assert_eq!(extract_error_code(stderr), Some("TypeError".to_string()));
    }

    #[test]
    fn test_extract_c_cpp_errors() {
        let cases = vec![
            ("main.c:5:5: error: 'x' undeclared (first use in this function)\n     x = 10;", "undeclared-identifier"),
            ("/tmp/ccXXXXXXXX.o: In function `main':\nmain.c:(.text+0x18): undefined reference to `add'\ncollect2: error: ld returned 1 exit status", "undefined-reference"),
            ("main.c:6:10: error: incompatible types when assigning to type 'int *' from type 'double *'", "type-mismatch"),
            ("main.c:4:5: warning: implicit declaration of function 'printf' [-Wimplicit-function-declaration]", "implicit-function-declaration"),
            ("Segmentation fault (core dumped)", "segmentation-fault"),
            ("main.c:5:10: error: expected ';' before 'return'", "syntax-error"),
            ("main.c:8:5: error: too few arguments to function 'int add(int, int)'", "function-argument-mismatch"),
            ("main.c:2:10: fatal error: myheader.h: No such file or directory\n    2 | #include \"myheader.h\"\n      |          ^~~~~~~~~~~~\ncompilation terminated.", "missing-header"),
            ("main.c:9:20: error: 'struct point' has no member named 'z'", "no-member"),
            ("main.c:5:8: error: redefinition of 'struct point'", "redefinition"),
            ("main.c:5:14: warning: format '%d' expects argument of type 'int', but argument 2 has type 'double' [-Wformat=]", "format-specifier"),
            ("main.c:5:5: warning: 'x' is used uninitialized [-Wuninitialized]", "uninitialized-variable"),
            ("main.c:5:5: warning: array subscript 10 is above array bounds of 'int [10]' [-Warray-bounds]", "array-out-of-bounds"),
            ("main.c:6:1: warning: control reaches end of non-void function [-Wreturn-type]", "return-type-mismatch"),
            ("free(): double free detected in tcache 2\nAborted (core dumped)", "double-free"),
            ("*** stack smashing detected ***: terminated\nAborted (core dumped)", "stack-smashing"),
            ("/usr/bin/ld: /tmp/ccYYYYYY.o: in function `helper':\nutils.c:(.text+0x0): multiple definition of `helper'; /tmp/ccXXXXXX.o:main.c:(.text+0x0): first defined here", "multiple-definition"),
        ];
        for (stderr, expected) in cases {
            assert_eq!(extract_error_code(stderr), Some(expected.to_string()), "failed on: {stderr}");
        }
    }

    #[test]
    fn test_extract_go_errors() {
        let cases = vec![
            ("./test.go:4:5: undefined: fmt", "undefined"),
            ("./main.go:10:8: cannot use myInt (type int) as type int32 in assignment", "cannot-use-type"),
            ("./main.go:5:2: syntax error: unexpected x, expected }", "syntax-error"),
        ];
        for (stderr, expected) in cases {
            assert_eq!(extract_error_code(stderr), Some(expected.to_string()), "failed on: {stderr}");
        }
    }

    #[test]
    fn test_extract_no_match() {
        assert_eq!(extract_error_code("some random output"), None);
    }

    #[test]
    fn test_is_safe_to_rerun() {
        for cmd in &["cargo build", "rustc main.rs", "python3 script.py", "gcc -o main main.c",
                      "npm run build", "/usr/bin/cargo build", "git status"] {
            assert!(is_safe_to_rerun(cmd), "expected safe: {cmd}");
        }
    }

    #[test]
    fn test_is_not_safe_to_rerun() {
        for cmd in &["rm -rf /", "curl -X POST http://example.com", "docker run something",
                      "sudo anything", "ssh server", ""] {
            assert!(!is_safe_to_rerun(cmd), "expected unsafe: {cmd}");
        }
    }
}
