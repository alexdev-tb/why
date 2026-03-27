use colored::Colorize;

use crate::db::ErrorEntry;

pub fn print_entry(entry: &ErrorEntry) {
    let width = 60;
    let divider = "─".repeat(width);

    let badge = format!("{}/{}", entry.language, entry.tool);
    // "→ " + id + " — " + title = prefix; pad to right-align badge
    let prefix_len = 2 + entry.id.len() + 3 + entry.title.len();
    let padding = if prefix_len + badge.len() < width {
        width - prefix_len - badge.len()
    } else {
        2
    };

    println!();
    println!(
        "  {} {} {} {}{}{}",
        "→".green().bold(),
        entry.id.bold().green(),
        "—".dimmed(),
        entry.title.bold(),
        " ".repeat(padding),
        badge.dimmed()
    );
    println!("  {}", divider.dimmed());

    println!();
    for line in entry.explain.trim().lines() {
        println!("  {}", line);
    }

    println!();

    println!("  {}", "Fix:".yellow().bold());
    for line in entry.fix.trim().lines() {
        println!("  {}", line);
    }

    if let Some(ref links) = entry.links {
        println!();
        for link in links {
            println!("  {} {}", "Docs:".dimmed(), link.underline());
        }
    }

    println!();
    println!("  {}", divider.dimmed());
    println!();
}

pub fn print_list(entries: &[(String, String, String)]) {
    let mut current_lang = String::new();
    for (lang, id, title) in entries {
        if lang != &current_lang {
            if !current_lang.is_empty() {
                println!();
            }
            println!("{}", format!("  {}", lang).bold().cyan());
            current_lang = lang.clone();
        }
        println!("    {} — {}", id.green(), title);
    }
    println!();
    println!(
        "  {} errors across {} languages",
        entries.len().to_string().bold(),
        {
            let mut langs: Vec<&str> = entries.iter().map(|(l, _, _)| l.as_str()).collect();
            langs.dedup();
            langs.len()
        }
    );
}

pub fn print_setup() {
    println!();
    println!("  {}", "why — Shell Hook Setup".bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();
    println!("  Add one of the following to your shell config:");
    println!();

    println!("  {} (~/.bashrc or ~/.bash_profile)", "Bash".cyan().bold());
    println!();
    println!("    eval \"$(why --hook bash)\"");
    println!();

    println!("  {} (~/.zshrc)", "Zsh".cyan().bold());
    println!();
    println!("    eval \"$(why --hook zsh)\"");
    println!();

    println!("  After adding the hook, restart your shell or run:");
    println!("    {} ~/.bashrc  # or ~/.zshrc", "source".green());
    println!();
}

const BASH_HOOK: &str = include_str!("../shell/why.bash");
const ZSH_HOOK: &str = include_str!("../shell/why.zsh");

pub fn print_hook(shell: &str) {
    match shell.to_lowercase().as_str() {
        "bash" => print!("{}", BASH_HOOK),
        "zsh" => print!("{}", ZSH_HOOK),
        _ => {
            eprintln!(
                "{} Unknown shell '{}'. Supported: bash, zsh",
                "error:".red().bold(),
                shell
            );
            std::process::exit(1);
        }
    }
}

pub fn print_no_error() -> ! {
    eprintln!(
        "{} No error detected. Run a command that fails first, or pass an error code directly:",
        "error:".red().bold()
    );
    eprintln!("  {} E0499", "why".green());
    eprintln!();
    eprintln!(
        "To set up shell hooks, run: {} {}",
        "why".green(),
        "--setup".dimmed()
    );
    std::process::exit(1);
}

pub fn print_not_found(code: &str) -> ! {
    eprintln!(
        "{} No explanation found for '{}'",
        "sorry:".yellow().bold(),
        code
    );
    eprintln!();
    eprintln!(
        "Know this error? Contribute an explanation: {}",
        "https://github.com/alexdev-tb/why/blob/main/CONTRIBUTING.md".dimmed()
    );
    std::process::exit(1);
}

pub fn print_list_empty(filter: Option<&str>) -> ! {
    if let Some(f) = filter {
        eprintln!("No error entries found for language: {}", f);
    } else {
        eprintln!("No error entries found. Is the database installed?");
    }
    std::process::exit(1);
}
