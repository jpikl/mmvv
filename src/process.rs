use crate::command::Group;
use crate::command::Meta;
use crate::commands::get_meta;
use crate::env::Env;
use crate::spawn::ContextItem;
use crate::spawn::SpawnWithContext;
use crate::spawn::Spawned;
use anyhow::Context;
use anyhow::Result;
use clap::crate_name;
use std::env::current_exe;
use std::process;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Stdio;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StdinMode {
    Connected,
    Disconnected,
}

pub enum Command {
    Internal {
        meta: &'static Meta,
        args: Vec<String>,
    },
    UnknownInternal {
        args: Vec<String>,
    },
    External {
        name: String,
        args: Vec<String>,
    },
}

impl Command {
    pub fn internal(meta: &'static Meta) -> Self {
        Self::Internal {
            meta,
            args: Vec::new(),
        }
    }

    pub fn detect(name: &str, args: &[String], external: bool) -> Self {
        if external {
            return Self::External {
                name: name.to_string(),
                args: args.to_vec(),
            };
        }

        if name == crate_name!() {
            if let Some((name, args)) = args.split_first() {
                if let Some(meta) = get_meta(name) {
                    return Self::Internal {
                        meta,
                        args: args.to_vec(),
                    };
                }
            }

            return Self::UnknownInternal {
                args: args.to_vec(),
            };
        }

        if let Some(meta) = get_meta(name) {
            return Self::Internal {
                meta,
                args: args.to_vec(),
            };
        }

        Self::External {
            name: name.to_string(),
            args: args.to_vec(),
        }
    }

    pub fn stdin_mode(&self) -> StdinMode {
        match self {
            Self::Internal { meta, .. } => match meta.group {
                Group::Generators => StdinMode::Disconnected,
                _ => StdinMode::Connected,
            },
            Self::UnknownInternal { .. } => StdinMode::Disconnected,
            Self::External { .. } => StdinMode::Connected,
        }
    }

    pub fn build(&self, env: &Env) -> Result<process::Command> {
        match self {
            Command::Internal { meta, args } => {
                let mut command = internal_command()?;
                command.arg(meta.name);
                command.args(args);
                command.envs(env.internal());
                Ok(command)
            }
            Command::UnknownInternal { args } => {
                let mut command = internal_command()?;
                command.args(args);
                command.envs(env.internal());
                Ok(command)
            }
            Command::External { name, args } => {
                let mut command = process::Command::new(name);
                command.args(args);
                command.envs(env.external());
                Ok(command)
            }
        }
    }
}

fn internal_command() -> Result<process::Command> {
    let program = current_exe().context("could not detect current executable")?;
    Ok(process::Command::new(program))
}

pub struct Pipeline {
    pub children: Vec<Spawned<Child>>,
    pub stdin: Option<Spawned<ChildStdin>>,
    pub stdout: Option<Spawned<ChildStdout>>,
    stdin_mode: StdinMode,
}

impl Pipeline {
    pub fn new(stdin_mode: StdinMode) -> Self {
        Self {
            children: Vec::new(),
            stdin: None,
            stdout: None,
            stdin_mode,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn add_command(
        mut self,
        mut command: process::Command,
        stdin_mode: StdinMode,
    ) -> Result<Self> {
        if stdin_mode == StdinMode::Disconnected {
            command.stdin(Stdio::null());
            self.stdin_mode = StdinMode::Disconnected; // Disconnected command => Disable stdin for the whole pipeline
        } else if let Some(stdout) = self.stdout {
            command.stdin(Stdio::from(stdout.inner)); // Connect to the previous command
        } else {
            command.stdin(Stdio::piped()); // The first command in the pipeline
        }

        command.stdout(Stdio::piped());

        let mut child = command.spawn_with_context()?;

        if self.stdin_mode == StdinMode::Disconnected {
            self.stdin = None;
        } else if self.stdin.is_none() {
            self.stdin = child.take_stdin(); // The first command in the pipeline
        }

        self.stdout = child.take_stdout();
        self.children.push(child);

        Ok(self)
    }

    pub fn add_context(&mut self, item: ContextItem) {
        for child in &mut self.children {
            child.context.add_item(item.clone());
        }

        if let Some(stdin) = &mut self.stdin {
            stdin.context.add_item(item.clone());
        }

        if let Some(stdout) = &mut self.stdout {
            stdout.context.add_item(item);
        }
    }
}
