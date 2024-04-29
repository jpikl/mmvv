use crate::spawn::SpawnWithContext;
use crate::spawn::Spawned;
use anyhow::Result;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StdinMode {
    Connected,
    Disconnected,
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

    pub fn command(mut self, mut command: Command, stdin_mode: StdinMode) -> Result<Self> {
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

    #[must_use]
    pub fn context(mut self, context: impl Into<String>) -> Self {
        let context = context.into();

        for child in &mut self.children {
            child.context.add(context.clone());
        }

        if let Some(stdin) = &mut self.stdin {
            stdin.context.add(context.clone());
        }

        if let Some(stdout) = &mut self.stdout {
            stdout.context.add(context);
        }

        self
    }
}
