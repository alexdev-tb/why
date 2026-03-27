use regex::Regex;
use std::env;

pub fn from_env() -> Option<String> {
    let exit_code = env::var("WHY_LAST_EXIT").ok()?;
    if exit_code == "0" {
        return None;
    }

    let stderr = env::var("WHY_LAST_STDERR").ok().filter(|s| !s.is_empty())?;
    extract_error_code(&stderr)
}

/// Match stderr against all DB entries that have `patterns` defined.
fn match_from_db(stderr: &str) -> Option<String> {
    let entries = crate::db::load_pattern_entries();

    for line in stderr.lines() {
        for (id, patterns, exclude) in &entries {
            let any_group = patterns
                .iter()
                .any(|group| group.iter().all(|pat| line.contains(pat.as_str())));
            let no_exclusion = exclude.iter().all(|pat| !line.contains(pat.as_str()));

            if any_group && no_exclusion {
                return Some(id.clone());
            }
        }
    }
    None
}

pub fn extract_error_code(stderr: &str) -> Option<String> {
    // Rust: structured error codes
    let rust_re = Regex::new(r"error\[E(\d+)\]").unwrap();
    if let Some(caps) = rust_re.captures(stderr) {
        return Some(format!("E{}", &caps[1]));
    }

    // Python: exception class names
    let python_re = Regex::new(r"(?m)^((?:\w*(?:Error|Exception|Warning)|StopIteration|KeyboardInterrupt|SystemExit|GeneratorExit|BaseException))\b").unwrap();
    if let Some(caps) = python_re.captures(stderr) {
        return Some(caps[1].to_string());
    }

    // Everything else: data-driven pattern matching from YAML entries
    match_from_db(stderr)
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_error_codes() {
        let cases = vec![
            (
                r#"error[E0499]: cannot borrow `x` as mutable more than once at a time
 --> src/main.rs:4:13"#,
                Some("E0499"),
            ),
            ("error[E0308]: mismatched types", Some("E0308")),
        ];
        for (stderr, expected) in cases {
            assert_eq!(
                extract_error_code(stderr),
                expected.map(String::from),
                "failed on: {stderr}"
            );
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
            assert_eq!(
                extract_error_code(stderr),
                Some(expected.to_string()),
                "failed on: {stderr}"
            );
        }
    }

    #[test]
    fn test_extract_go_errors() {
        let cases = vec![
            ("./test.go:4:5: undefined: fmt", "undefined"),
            (
                "./main.go:10:8: cannot use myInt (type int) as type int32 in assignment",
                "cannot-use-type",
            ),
            (
                "./main.go:5:2: syntax error: unexpected x, expected }",
                "syntax-error",
            ),
        ];
        for (stderr, expected) in cases {
            assert_eq!(
                extract_error_code(stderr),
                Some(expected.to_string()),
                "failed on: {stderr}"
            );
        }
    }

    #[test]
    fn test_extract_no_match() {
        assert_eq!(extract_error_code("some random output"), None);
    }
}
