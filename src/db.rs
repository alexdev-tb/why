use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

static EMBEDDED_DB: Dir = include_dir!("$CARGO_MANIFEST_DIR/db");

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ErrorEntry {
    pub id: String,
    pub tool: String,
    pub language: String,
    pub title: String,
    pub explain: String,
    pub fix: String,
    pub tags: Option<Vec<String>>,
    pub patterns: Option<Vec<Vec<String>>>,
    pub exclude: Option<Vec<String>>,
    pub example_error: Option<String>,
    pub example_code: Option<String>,
    pub links: Option<Vec<String>>,
}

/// Priority: WHY_DB env override → local cache → embedded db.
pub fn lookup(code: &str) -> Option<ErrorEntry> {
    if let Some(db_path) = env_db_dir() {
        if let Some(entry) = fs_lookup(&db_path, code) {
            return Some(entry);
        }
    }

    // Local cache (populated by `why --update`)
    if let Some(cache) = crate::update::cache_dir() {
        if cache.is_dir() {
            if let Some(entry) = fs_lookup(&cache, code) {
                return Some(entry);
            }
        }
    }

    embedded_lookup(code)
}

pub fn list(filter: Option<&str>) -> Vec<(String, String, String)> {
    if let Some(db_path) = env_db_dir() {
        let entries = fs_list(&db_path, filter);
        if !entries.is_empty() {
            return entries;
        }
    }

    if let Some(cache) = crate::update::cache_dir() {
        if cache.is_dir() {
            let entries = fs_list(&cache, filter);
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    embedded_list(filter)
}

// ── Embedded database ──────────────────────────────────────────────────

fn embedded_lookup(code: &str) -> Option<ErrorEntry> {
    let normalized = code.to_uppercase();

    for lang_dir in EMBEDDED_DB.dirs() {
        for file in lang_dir.files() {
            let stem = file.path().file_stem().and_then(|s| s.to_str())?;
            if stem.eq_ignore_ascii_case(&normalized) {
                let content = file.contents_utf8()?;
                return serde_yaml::from_str(content).ok();
            }
        }
    }

    None
}

fn embedded_list(filter: Option<&str>) -> Vec<(String, String, String)> {
    let mut entries = Vec::new();

    for lang_dir in EMBEDDED_DB.dirs() {
        let lang_name = lang_dir
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if let Some(f) = filter {
            if !lang_name.eq_ignore_ascii_case(f) {
                continue;
            }
        }

        for file in lang_dir.files() {
            let path = file.path();

            if path.file_stem().map(|s| s == "TEMPLATE").unwrap_or(false) {
                continue;
            }

            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                if let Some(content) = file.contents_utf8() {
                    if let Ok(entry) = serde_yaml::from_str::<ErrorEntry>(content) {
                        entries.push((lang_name.to_string(), entry.id, entry.title));
                    }
                }
            }
        }
    }

    entries.sort();
    entries
}

/// Load all entries that have `patterns` defined, for data-driven detection.
/// Returns (id, patterns, exclude) tuples.
pub fn load_pattern_entries() -> Vec<(String, Vec<Vec<String>>, Vec<String>)> {
    // Same priority as lookup: env override → cache → embedded
    if let Some(db_path) = env_db_dir() {
        let entries = fs_pattern_entries(&db_path);
        if !entries.is_empty() {
            return entries;
        }
    }

    if let Some(cache) = crate::update::cache_dir() {
        if cache.is_dir() {
            let entries = fs_pattern_entries(&cache);
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    embedded_pattern_entries()
}

fn embedded_pattern_entries() -> Vec<(String, Vec<Vec<String>>, Vec<String>)> {
    let mut out = Vec::new();
    for lang_dir in EMBEDDED_DB.dirs() {
        for file in lang_dir.files() {
            if let Some(content) = file.contents_utf8() {
                if let Ok(entry) = serde_yaml::from_str::<ErrorEntry>(content) {
                    if let Some(patterns) = entry.patterns {
                        let exclude = entry.exclude.unwrap_or_default();
                        out.push((entry.id, patterns, exclude));
                    }
                }
            }
        }
    }
    out
}

fn fs_pattern_entries(db: &Path) -> Vec<(String, Vec<Vec<String>>, Vec<String>)> {
    let mut out = Vec::new();
    if let Ok(languages) = fs::read_dir(db) {
        for lang_dir in languages.flatten() {
            if !lang_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            if let Ok(files) = fs::read_dir(lang_dir.path()) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                        if let Some(entry) = load_fs_entry(&path) {
                            if let Some(patterns) = entry.patterns {
                                let exclude = entry.exclude.unwrap_or_default();
                                out.push((entry.id, patterns, exclude));
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

// ── Filesystem lookup ──────────────────────────────────────────────────

fn env_db_dir() -> Option<PathBuf> {
    let p = env::var("WHY_DB").ok()?;
    let path = PathBuf::from(p);
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

fn fs_lookup(db: &Path, code: &str) -> Option<ErrorEntry> {
    let normalized = code.to_uppercase();

    if let Ok(languages) = fs::read_dir(db) {
        for lang_dir in languages.flatten() {
            if !lang_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }

            let yaml_path = lang_dir.path().join(format!("{}.yaml", normalized));
            if yaml_path.exists() {
                return load_fs_entry(&yaml_path);
            }

            let yaml_path_orig = lang_dir.path().join(format!("{}.yaml", code));
            if yaml_path_orig.exists() {
                return load_fs_entry(&yaml_path_orig);
            }
        }
    }

    None
}

fn fs_list(db: &Path, filter: Option<&str>) -> Vec<(String, String, String)> {
    let mut entries = Vec::new();

    if let Ok(languages) = fs::read_dir(db) {
        for lang_dir in languages.flatten() {
            if !lang_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }

            let lang_name = lang_dir.file_name().to_string_lossy().to_string();

            if let Some(f) = filter {
                if !lang_name.eq_ignore_ascii_case(f) {
                    continue;
                }
            }

            if let Ok(files) = fs::read_dir(lang_dir.path()) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.file_stem().map(|s| s == "TEMPLATE").unwrap_or(false) {
                        continue;
                    }
                    if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                        if let Some(entry) = load_fs_entry(&path) {
                            entries.push((lang_name.clone(), entry.id, entry.title));
                        }
                    }
                }
            }
        }
    }

    entries.sort();
    entries
}

fn load_fs_entry(path: &Path) -> Option<ErrorEntry> {
    let content = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_missing_entry() {
        assert!(lookup("E9999").is_none());
    }

    #[test]
    fn test_embedded_lookup_finds_entry() {
        let entry = embedded_lookup("E0499");
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.id, "E0499");
        assert_eq!(entry.language, "rust");
    }

    #[test]
    fn test_embedded_lookup_case_insensitive() {
        assert!(embedded_lookup("e0499").is_some());
    }

    #[test]
    fn test_embedded_list_returns_entries() {
        let entries = embedded_list(None);
        assert!(!entries.is_empty());
        assert!(entries.len() >= 30);
    }

    #[test]
    fn test_embedded_list_filter_by_language() {
        let rust_entries = embedded_list(Some("rust"));
        let unknown_entries = embedded_list(Some("cobol"));
        assert!(!rust_entries.is_empty());
        assert!(unknown_entries.is_empty());
    }
}
