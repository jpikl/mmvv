use crate::stdbuf::StdBuf;
use clap::crate_name;
use clap::ValueEnum;
use derive_more::Display;
use derive_more::IsVariant;
use std::env;
use std::io::stdout;
use std::io::IsTerminal;

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library https://github.com/Freaky/rust-linereader.
const DEFAULT_BUF_SIZE: usize = 32 * 1024;

// Publicly known env variables:
pub const ENV_NULL: &str = "REW_NULL";
pub const ENV_BUF_MODE: &str = "REW_BUF_MODE";
pub const ENV_BUF_SIZE: &str = "REW_BUF_SIZE";

// Internal env variables:
//
// When `rew` is spawned as a child of some parent `rew` process,
// it recieves the parent's name through this environment variable.
pub const ENV_SPAWNED_BY: &str = "_REW_SPAWNED_BY";

#[derive(Clone, Copy, ValueEnum, Display, Debug, IsVariant, PartialEq, Eq)]
pub enum BufMode {
    /// Writes to stdout after a line was processed or when the output buffer is full.
    /// Enabled by default when stdout is TTY (for interactive usage).
    #[display("line")]
    Line,
    /// Writes to stdout only when the output buffer is full.
    /// Enabled by default when stdout is not TTY (for maximal throughput).
    #[display("full")]
    Full,
}

impl Default for BufMode {
    fn default() -> Self {
        if stdout().is_terminal() {
            Self::Line
        } else {
            Self::Full
        }
    }
}

#[derive(clap::Args, Default, Debug, Clone, Eq, PartialEq)]
pub struct Args {
    /// Line delimiter is NUL, not newline.
    #[arg(global = true, short = '0', long, env = ENV_NULL)]
    pub null: bool,

    /// Output buffering mode.
    #[arg(
        global = true,
        long,
        value_name = "MODE",
        env = ENV_BUF_MODE,
        default_value_t = BufMode::default(),
        hide_default_value = true,
    )]
    pub buf_mode: BufMode,

    /// Size of a buffer used for IO operations.
    ///
    /// Smaller values will reduce memory consumption but could negatively affect througput.
    ///
    /// Larger values will increase memory consumption but may improve troughput in some cases.
    ///
    /// Certain commands (which can only operate with whole lines) won't be able to fetch
    /// a line bigger than this limit and will abort their execution instead.
    #[arg(
        global = true,
        long,
        value_name = "BYTES",
        env = ENV_BUF_SIZE,
        default_value_t = DEFAULT_BUF_SIZE,
    )]
    pub buf_size: usize,
}

pub struct Env {
    pub command: &'static str,
    pub args: Args,
    pub stdbuf: StdBuf,
}

impl Env {
    pub fn new(command: &'static str, args: Args) -> Self {
        Self {
            command,
            args,
            stdbuf: StdBuf::default(),
        }
    }

    pub fn internal(&self) -> Vec<(String, String)> {
        vec![
            (ENV_NULL.to_owned(), self.args.null.to_string()),
            (ENV_BUF_MODE.to_owned(), self.args.buf_mode.to_string()),
            (ENV_BUF_SIZE.to_owned(), self.args.buf_size.to_string()),
            (ENV_SPAWNED_BY.to_owned(), get_spawned_by(self.command)),
        ]
    }

    pub fn external(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        if self.args.buf_mode.is_line() {
            env.extend(self.stdbuf.line_buf_env()); // libc based programs
            env.push(("PYTHONUNBUFFERED".to_owned(), "1".to_owned())); // Python programs
        }

        env
    }
}

pub fn get_spawned_by(cmd_name: &str) -> String {
    format!("{} {cmd_name}", get_bin_name())
}

pub fn get_bin_name() -> String {
    if let Some(spawned_by) = env::var_os(ENV_SPAWNED_BY) {
        let mut spawned_by = spawned_by.to_string_lossy().to_string();

        if let Some(bin_name_len) = spawned_by.rfind(' ') {
            spawned_by.truncate(bin_name_len); // Trim rew subcommand name
        }

        // Parent `rew` process `argv[0]`
        return spawned_by;
    }

    if let Some(bin_name) = env::args_os().next() {
        // Current `rew` process `argv[0]`
        return bin_name.to_string_lossy().to_string();
    }

    // exec syscall did not receive `argv0`?
    crate_name!().to_owned()
}
