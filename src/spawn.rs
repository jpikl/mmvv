use crate::colors::RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use crate::io::LineReader;
use anyhow::Error;
use anyhow::Result;
use std::env;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::ExitStatus;

// TODO: key/value items Context(Vec<(&'static str, String)>)
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

pub struct Spawned<T> {
    pub inner: T,
    pub context: Context,
}

impl<T> Spawned<T> {
    pub fn new(inner: T, context: Context) -> Self {
        Self { inner, context }
    }

    pub fn map<V>(self, mapper: impl Fn(T) -> V) -> Spawned<V> {
        Spawned::new(mapper(self.inner), self.context.clone())
    }

    pub fn clone_context<V>(&self, inner: V) -> Spawned<V> {
        Spawned::new(inner, self.context.clone())
    }
}

impl Spawned<ChildStdin> {
    pub fn write_all(&mut self, buf: &[u8]) -> Result<bool> {
        match self.inner.write_all(buf) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(false),
            Err(err) => Err(self
                .context
                .apply(err)
                .context("failed to write to child process stdin")),
        }
    }
}

impl Spawned<LineReader<ChildStdout>> {
    pub fn read_line(&mut self) -> Result<Option<&[u8]>> {
        self.inner.read_line().map_err(|err| {
            self.context
                .apply(err)
                .context("failed to read from child process stdout")
        })
    }
}

impl Spawned<Child> {
    pub fn take_stdin(&mut self) -> Option<Spawned<ChildStdin>> {
        self.inner
            .stdin
            .take()
            .map(|stdin| self.clone_context(stdin))
    }

    pub fn take_stdout(&mut self) -> Option<Spawned<ChildStdout>> {
        self.inner
            .stdout
            .take()
            .map(|stdout| self.clone_context(stdout))
    }

    pub fn wait(&mut self) -> Result<()> {
        match self.inner.wait() {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(self.wait_context(exit_error(status))),
            Err(err) => Err(self.wait_context(err)),
        }
    }

    pub fn try_wait(&mut self) -> Result<bool> {
        match self.inner.try_wait() {
            Ok(None) => Ok(false),
            Ok(Some(status)) if status.success() => Ok(true),
            Ok(Some(status)) => Err(self.wait_context(exit_error(status))),
            Err(err) => Err(self.wait_context(err)),
        }
    }

    fn wait_context(&self, error: impl Into<Error>) -> Error {
        self.context
            .apply(error)
            .context("child proces execution failed")
    }

    pub fn kill(&mut self) -> Result<()> {
        self.inner.kill().map_err(|err| {
            self.context
                .apply(err)
                .context("failed to kill child process")
        })
    }
}

fn exit_error(status: ExitStatus) -> Error {
    let message = match status.code() {
        Some(code) => format!("child process exited with code {RED}{code}{RESET}"),
        None => "child process was terminated by a signal".to_owned(),
    };
    Error::msg(message)
}

pub trait SpawnWithContext {
    fn spawn_with_context(&mut self) -> Result<Spawned<Child>>;
    fn context(&self) -> Context;
}

impl SpawnWithContext for Command {
    fn spawn_with_context(&mut self) -> Result<Spawned<Child>> {
        match self.spawn() {
            Ok(child) => Ok(Spawned::new(child, self.context())),
            Err(err) => Err(self
                .context()
                .apply(err)
                .context("failed to spawn child process")),
        }
    }

    fn context(&self) -> Context {
        if let Ok(command) = format_command(self) {
            let mut context = Context::new(command);
            if self.get_envs().count() > 0 {
                if let Ok(env) = format_env(self) {
                    context.add(env);
                }
            }
            context
        } else {
            Context::new(format!("command: {YELLOW}{self:?}{RESET}"))
        }
    }
}

fn format_command(command: &Command) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    write!(&mut output, "command: {YELLOW}")?;

    let program = if cfg!(debug_assertions) && env::var_os("NEXTEST").is_some() {
        // We want to obfuscate program path to make "transcript" tests reproducible.
        Path::new(command.get_program())
            .file_stem()
            .unwrap_or_default()
    } else {
        command.get_program()
    };

    write!(&mut output, "{program:?}")?;

    for arg in command.get_args() {
        write!(&mut output, " {arg:?}")?;
    }

    write!(&mut output, "{RESET}")?;
    Ok(output)
}

fn format_env(command: &Command) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    write!(&mut output, "environment: {YELLOW}")?;

    for (key, val) in command.get_envs() {
        let key = key.to_string_lossy();
        let val = val.unwrap_or_default();
        write!(&mut output, "{key}={val:?} ",)?;
    }

    write!(&mut output, "{RESET}")?;
    Ok(output)
}
