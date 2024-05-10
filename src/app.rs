use crate::colors::BOLD;
use crate::colors::RESET;
use crate::commands::METAS;
use crate::env;
use clap::command;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;
use clap::Args;
use clap::Command;

const REFERENCE_URL: &str = "https://jpikl.github.io/rew/reference";

pub fn build() -> Command {
    let mut app = command!()
        .version(get_version())
        .about(crate_description!().replace(". ", ".\n"))
        .after_help(get_after_help(None))
        .subcommand_required(true);

    for meta in METAS {
        let command = meta.build().after_help(get_after_help(Some(meta.name)));
        app = app.subcommand(command);
    }

    env::Args::augment_args(app.next_help_heading("Global options"))
}

fn get_version() -> String {
    let version = crate_version!();
    let commit = option_env!("BUILD_COMMIT").unwrap_or("unknown Git commit");

    format!("{version} ({commit})")
}

fn get_after_help(cmd: Option<&str>) -> String {
    let app = crate_name!();
    let file = if let Some(cmd) = cmd {
        format!("{app}-{cmd}.html")
    } else {
        format!("{app}.html")
    };
    format!("Visit {BOLD}{REFERENCE_URL}/{file}.html{RESET} for a complete reference and examples.")
}
