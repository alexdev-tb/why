mod db;
mod detect;
mod output;
mod setup;
mod update;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "why",
    version,
    about = "Plain English explanations for compiler errors and CLI failures"
)]
struct Cli {
    /// Error code to look up directly (e.g. E0499, TypeError)
    error_code: Option<String>,

    /// List all known errors, optionally filtered by language
    #[arg(short, long)]
    list: Option<Option<String>>,

    /// Install shell hook into your shell config automatically
    #[arg(short, long)]
    setup: bool,

    /// Show manual shell hook setup instructions
    #[arg(long)]
    setup_manual: bool,

    /// Remove shell hook from your shell config
    #[arg(long)]
    uninstall: bool,

    /// Print shell hook script to stdout (bash or zsh)
    #[arg(short = 'H', long, value_name = "SHELL")]
    hook: Option<String>,

    /// Download the latest error database from GitHub
    #[arg(short, long)]
    update: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.setup {
        setup::install();
        return;
    }

    if cli.setup_manual {
        output::print_setup();
        return;
    }

    if cli.uninstall {
        setup::uninstall();
        return;
    }

    if let Some(shell) = cli.hook {
        output::print_hook(&shell);
        return;
    }

    if cli.update {
        update::run();
        return;
    }

    if let Some(filter) = cli.list {
        let entries = db::list(filter.as_deref());
        if entries.is_empty() {
            output::print_list_empty(filter.as_deref());
        }
        output::print_list(&entries);
        return;
    }

    let error_code = match cli.error_code {
        Some(code) => code,
        None => match detect::from_env() {
            Some(code) => code,
            None => output::print_no_error(),
        },
    };

    match db::lookup(&error_code) {
        Some(entry) => output::print_entry(&entry),
        None => output::print_not_found(&error_code),
    }
}
