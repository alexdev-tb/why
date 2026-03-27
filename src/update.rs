use colored::Colorize;
use flate2::read::GzDecoder;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;

const DB_URL: &str = "https://github.com/alexdev-tb/why/archive/refs/heads/db.tar.gz";

pub fn cache_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("why").join("db"))
}

pub fn run() {
    let dest = match cache_dir() {
        Some(d) => d,
        None => {
            eprintln!(
                "{} Could not determine data directory for your platform.",
                "error:".red().bold()
            );
            std::process::exit(1);
        }
    };

    eprintln!(
        "  {} Downloading latest error database...",
        "→".green().bold()
    );

    let response = match ureq::get(DB_URL).call() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} Failed to download database: {}",
                "error:".red().bold(),
                e
            );
            std::process::exit(1);
        }
    };

    let reader = response.into_reader();
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    if let Err(e) = extract_db(&mut archive, &dest) {
        eprintln!(
            "{} Failed to extract database: {}",
            "error:".red().bold(),
            e
        );
        std::process::exit(1);
    }

    let count = count_yaml_files(&dest);
    eprintln!(
        "  {} Updated! {} error entries installed to {}",
        "✓".green().bold(),
        count.to_string().bold(),
        dest.display()
    );
}

/// Extract the db/ subtree from a GitHub tarball into `dest`.
fn extract_db<R: io::Read>(archive: &mut Archive<R>, dest: &Path) -> io::Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        let relative: PathBuf = path.components().skip(1).collect();

        if relative.as_os_str().is_empty() {
            continue;
        }

        let out_path = dest.join(&relative);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

/// Count .yaml files (excluding TEMPLATE) in a directory tree.
fn count_yaml_files(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                count += count_yaml_files(&entry.path());
            } else if entry
                .path()
                .extension()
                .map(|e| e == "yaml")
                .unwrap_or(false)
                && entry
                    .path()
                    .file_stem()
                    .map(|s| s != "TEMPLATE")
                    .unwrap_or(true)
            {
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    /// Build a .tar.gz mimicking a GitHub branch archive (why-db/ prefix).
    fn build_test_tarball() -> Vec<u8> {
        let gz_buf = Vec::new();
        let encoder = GzEncoder::new(gz_buf, Compression::default());
        let mut builder = tar::Builder::new(encoder);

        let yaml_content = b"id: E0499\ntool: rustc\nlanguage: rust\ntitle: Test\nexplain: Test explanation\nfix: Test fix\n";
        let mut header = tar::Header::new_gnu();
        header.set_size(yaml_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "why-db/rust/E0499.yaml", &yaml_content[..])
            .unwrap();

        let yaml2 = b"id: TypeError\ntool: python\nlanguage: python\ntitle: Type Error\nexplain: Wrong type\nfix: Fix type\n";
        let mut header2 = tar::Header::new_gnu();
        header2.set_size(yaml2.len() as u64);
        header2.set_mode(0o644);
        header2.set_cksum();
        builder
            .append_data(&mut header2, "why-db/python/TypeError.yaml", &yaml2[..])
            .unwrap();

        let template =
            b"id: TEMPLATE\ntool: rustc\nlanguage: rust\ntitle: Template\nexplain: ...\nfix: ...\n";
        let mut header3 = tar::Header::new_gnu();
        header3.set_size(template.len() as u64);
        header3.set_mode(0o644);
        header3.set_cksum();
        builder
            .append_data(&mut header3, "why-db/rust/TEMPLATE.yaml", &template[..])
            .unwrap();

        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn test_extract_db_and_count() {
        let tarball = build_test_tarball();

        let dest = std::env::temp_dir().join("why_test_extract_db");
        let _ = fs::remove_dir_all(&dest);

        let decoder = GzDecoder::new(&tarball[..]);
        let mut archive = Archive::new(decoder);

        extract_db(&mut archive, &dest).expect("extract_db should succeed");

        assert!(dest.join("rust/E0499.yaml").exists());
        assert!(dest.join("python/TypeError.yaml").exists());
        assert!(dest.join("rust/TEMPLATE.yaml").exists());

        assert_eq!(count_yaml_files(&dest), 2);

        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn test_cache_dir_returns_some() {
        let dir = cache_dir();
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir.ends_with("why/db"));
    }
}
