mod command;
mod docs;

use anyhow::format_err;
use clap::Parser;
use clap::Subcommand;
use command::Adapter;
use docs::generate_reference;
use docs::generate_summary;
use rew::app;
use rew::commands::COMMANDS;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;

/// Execute project task.
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub task: Task,
}

#[derive(Subcommand)]
pub enum Task {
    /// Generate documentation.
    #[command()]
    Docs,
    /// Generate man pages.
    #[command()]
    Man,
}

fn main() -> anyhow::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let root_path = Path::new(&manifest_dir)
        .parent()
        .ok_or_else(|| format_err!("'{}' does not have parent", manifest_dir))?;

    let cli = Cli::parse();

    let mut app = app::build(COMMANDS);
    app.build();

    match cli.task {
        Task::Docs => {
            let docs_path = root_path.join("docs");
            let reference_path = docs_path.join("reference");
            let summary_path = docs_path.join("SUMMARY.md");

            if !reference_path.exists() {
                create_dir_all(&reference_path)?;
            }

            let adapter = Adapter::new(&app);
            generate_summary(&adapter, &summary_path)?;
            generate_reference(&adapter, &reference_path)?;
        }
        Task::Man => unimplemented!(),
    }

    Ok(())
}
