use colored::Colorize;
use std::env;
use std::fs;
use std::path::PathBuf;

const HOOK_LINE_BASH: &str = r#"eval "$(why --hook bash)""#;
const HOOK_LINE_ZSH: &str = r#"eval "$(why --hook zsh)""#;

fn detect_shell() -> Option<(&'static str, PathBuf)> {
    let shell_env = env::var("SHELL").unwrap_or_default();
    let shell_name = shell_env.rsplit('/').next().unwrap_or("");
    let home = dirs::home_dir()?;

    match shell_name {
        "zsh" => Some(("zsh", home.join(".zshrc"))),
        "bash" => {
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                Some(("bash", bashrc))
            } else {
                Some(("bash", home.join(".bash_profile")))
            }
        }
        _ => None,
    }
}

fn hook_line(shell: &str) -> &'static str {
    match shell {
        "zsh" => HOOK_LINE_ZSH,
        _ => HOOK_LINE_BASH,
    }
}

/// Automatically install the shell hook into the user's shell config.
pub fn install() {
    let (shell, config_path) = match detect_shell() {
        Some(s) => s,
        None => {
            eprintln!(
                "{} Could not detect your shell from $SHELL.",
                "error:".red().bold()
            );
            eprintln!("  Supported shells: bash, zsh");
            eprintln!(
                "  You can set it up manually — run: {} {}",
                "why".green(),
                "--setup-manual".dimmed()
            );
            std::process::exit(1);
        }
    };

    let line = hook_line(shell);
    let display_path = config_path.display();

    if let Ok(contents) = fs::read_to_string(&config_path) {
        if contents.contains(line) {
            eprintln!(
                "  {} Shell hook already installed in {}",
                "✓".green().bold(),
                display_path
            );
            return;
        }
    }

    let addition = format!("\n# why - automatic error detection\n{}\n", line);

    if let Err(e) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(addition.as_bytes())
        })
    {
        eprintln!(
            "{} Failed to write to {}: {}",
            "error:".red().bold(),
            display_path,
            e
        );
        std::process::exit(1);
    }

    eprintln!("  {} Hook added to {}", "✓".green().bold(), display_path);
    eprintln!(
        "  Restart your shell or run: {} {}",
        "source".green(),
        display_path
    );
}

/// Remove the shell hook from the user's shell config.
pub fn uninstall() {
    let (shell, config_path) = match detect_shell() {
        Some(s) => s,
        None => {
            eprintln!(
                "{} Could not detect your shell from $SHELL.",
                "error:".red().bold()
            );
            std::process::exit(1);
        }
    };

    let display_path = config_path.display();

    let contents = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "  {} No shell hook found ({})",
                "✓".green().bold(),
                display_path
            );
            return;
        }
    };

    let line = hook_line(shell);

    if !contents.contains(line) {
        eprintln!(
            "  {} No shell hook found in {}",
            "✓".green().bold(),
            display_path
        );
        return;
    }

    // Remove the hook line and its comment
    let new_contents = contents
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            trimmed != line && trimmed != "# why - automatic error detection"
        })
        .collect::<Vec<_>>()
        .join("\n");

    let new_contents = if contents.ends_with('\n') && !new_contents.ends_with('\n') {
        format!("{}\n", new_contents)
    } else {
        new_contents
    };

    if let Err(e) = fs::write(&config_path, &new_contents) {
        eprintln!(
            "{} Failed to update {}: {}",
            "error:".red().bold(),
            display_path,
            e
        );
        std::process::exit(1);
    }

    eprintln!(
        "  {} Hook removed from {}",
        "✓".green().bold(),
        display_path
    );
    eprintln!("  Restart your shell to complete removal.");
}
