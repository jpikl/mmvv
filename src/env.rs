use crate::args::GlobalArgs;
use crate::stdbuf::StdBuf;
use clap::crate_name;
use std::env;

// Publicly known env variables:
pub const ENV_NULL: &str = "REW_NULL";
pub const ENV_BUF_MODE: &str = "REW_BUF_MODE";
pub const ENV_BUF_SIZE: &str = "REW_BUF_SIZE";

// Internal env variables:
//
// When `rew` is spawned as a child of some parent `rew` process,
// it recieves the parent's name through this environment variable.
pub const ENV_SPAWNED_BY: &str = "_REW_SPAWNED_BY";

pub struct Env {
    pub command: &'static str,
    pub args: GlobalArgs,
    pub stdbuf: StdBuf,
}

impl Env {
    pub fn new(command: &'static str, args: GlobalArgs) -> Self {
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

    pub fn external(&mut self) -> Vec<(String, String)> {
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
