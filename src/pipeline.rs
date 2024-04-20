use crate::process::SpawnWithContext;
use crate::process::Spawned;
use crate::process::StdinMode;
use anyhow::Result;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

pub struct Pipeline {
    pub children: Vec<Spawned<Child>>,
    pub stdin: Option<Spawned<ChildStdin>>,
    pub stdout: Spawned<ChildStdout>,
}

impl Pipeline {
    #[must_use]
    pub fn context(mut self, context: impl Into<String>) -> Self {
        let context = context.into();

        if let Some(stdin) = &mut self.stdin {
            stdin.context.add(context.clone());
        }

        for child in &mut self.children {
            child.context.add(context.clone());
        }

        self.stdout.context.add(context);
        self
    }
}

pub struct Builder {
    children: Vec<Spawned<Child>>,
    stdin: Option<Spawned<ChildStdin>>,
    stdin_mode: StdinMode,
    stdout: Option<Spawned<ChildStdout>>,
}

impl Builder {
    pub fn new(stdin_mode: StdinMode) -> Self {
        Self {
            children: Vec::new(),
            stdin: None,
            stdin_mode,
            stdout: None,
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

    pub fn build(mut self) -> Pipeline {
        Pipeline {
            children: self.children,
            stdin: self.stdin,
            stdout: self
                .stdout
                .take()
                .expect("constructed pipeline without stdout"),
        }
    }
}
