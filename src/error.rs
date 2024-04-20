use crate::args::get_bin_name;
use crate::args::ENV_SPAWNED_BY;
use crate::colors::Colorizer;
use crate::colors::BOLD;
use crate::colors::BOLD_RED;
use crate::colors::RESET;
use anstream::eprint;
use anstream::eprintln;
use anstream::stdout;
use anyhow::Context as AnyhowContext;
use clap::Command;
use std::env;

#[derive(Clone)]
pub struct Context(Vec<String>);

impl Context {
    pub fn new(value: impl Into<String>) -> Self {
        Self(vec![value.into()])
    }

    pub fn add(&mut self, value: impl Into<String>) {
        self.0.push(value.into());
    }

    pub fn apply<E: Into<anyhow::Error>>(&self, error: E) -> anyhow::Error {
        let mut error = error.into();
        for context in &self.0 {
            error = error.context(context.clone());
        }
        error
    }
}

pub struct Reporter {
    app: Command,
}

impl Reporter {
    pub fn new(app: &Command) -> Self {
        Self { app: app.clone() }
    }

    pub fn print_help(&self, error: &clap::Error) {
        let mut stdout = stdout().lock();
        let help = error.render().ansi().to_string();

        let colorizer = Colorizer {
            quote_char: '`',
            quote_color: BOLD,
        };

        if let Err(error) = colorizer
            .write(&mut stdout, help)
            .context("could not write to stdout")
        {
            self.print_error(&error);
        }
    }

    pub fn print_invalid_usage(&self, error: &clap::Error) {
        let main_prefix = self.build_prefix();
        let err_prefix = "invalid usage";

        if env::var_os(ENV_SPAWNED_BY).is_some() {
            // Be brief when spawned by a parent `rew` process
            let message = error.kind().as_str().unwrap_or("unknown error");
            eprintln!("{main_prefix}: {BOLD_RED}{err_prefix}:{RESET} {message}");
        } else {
            let message = error.render().ansi().to_string();
            let message = message.replacen("error", err_prefix, 1);
            eprint!("{main_prefix}: {message}");
        };
    }

    pub fn print_error(&self, error: &anyhow::Error) {
        let prefix = self.build_prefix();
        eprintln!("{prefix}: {BOLD_RED}error:{RESET} {error}");

        for cause in error.chain().skip(1) {
            eprintln!("{prefix}: └─> {cause}");
        }
    }

    fn build_prefix(&self) -> String {
        let bin_name = get_bin_name();

        let prefix = self
            .app
            .clone()
            .ignore_errors(true)
            .try_get_matches()
            .ok()
            .and_then(|matches| matches.subcommand_name().map(ToString::to_string))
            .map(|cmd_name| format!("{} {cmd_name}", &bin_name))
            .unwrap_or(bin_name);

        if let Some(spawned_by) = env::var_os(ENV_SPAWNED_BY) {
            let spawned_by = spawned_by.to_string_lossy();
            format!("{BOLD}{prefix}{RESET} (spawned by '{BOLD}{spawned_by}{RESET}')")
        } else {
            format!("{BOLD}{prefix}{RESET}")
        }
    }
}
