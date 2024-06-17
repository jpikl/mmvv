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

#[derive(Clone)]
pub struct ContextItem {
    pub name: &'static str,
    pub value: String,
}

#[derive(Clone, Default)]
pub struct Context {
    items: Vec<ContextItem>,
}

impl Context {
    pub fn add_item(&mut self, item: ContextItem) {
        self.items.push(item);
    }

    pub fn apply_to_err<E: Into<Error>>(&self, error: E) -> Error {
        let mut error = error.into();
        for item in &self.items {
            error = error.context(format!("{}: {YELLOW}{}{RESET}", item.name, item.value));
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

    pub fn split<V>(&self, inner: V) -> Spawned<V> {
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
                .apply_to_err(err)
                .context("failed to write to child process stdin")),
        }
    }
}

impl Spawned<LineReader<ChildStdout>> {
    pub fn read_line(&mut self) -> Result<Option<&[u8]>> {
        self.inner.read_line().map_err(|err| {
            self.context
                .apply_to_err(err)
                .context("failed to read from child process stdout")
        })
    }
}

impl Spawned<Child> {
    pub fn take_stdin(&mut self) -> Option<Spawned<ChildStdin>> {
        self.inner.stdin.take().map(|stdin| self.split(stdin))
    }

    pub fn take_stdout(&mut self) -> Option<Spawned<ChildStdout>> {
        self.inner.stdout.take().map(|stdout| self.split(stdout))
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
            .apply_to_err(error)
            .context("child process execution failed")
    }

    pub fn kill(&mut self) -> Result<()> {
        self.inner.kill().map_err(|err| {
            self.context
                .apply_to_err(err)
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
                .apply_to_err(err)
                .context("failed to spawn child process")),
        }
    }

    fn context(&self) -> Context {
        let mut context = Context::default();

        if let Ok(item) = command_context(self) {
            context.add_item(item);

            if self.get_envs().count() > 0 {
                if let Ok(item) = env_context(self) {
                    context.add_item(item);
                }
            }
        } else {
            context.add_item(ContextItem {
                name: "command",
                value: format!("{self:?}"),
            });
        }

        context
    }
}

fn command_context(command: &Command) -> Result<ContextItem> {
    use std::fmt::Write;
    let mut writer = String::new();

    let program = if cfg!(debug_assertions) && env::var_os("NEXTEST").is_some() {
        // We want to obfuscate program path to make "transcript" tests reproducible.
        Path::new(command.get_program())
            .file_stem()
            .unwrap_or_default()
    } else {
        command.get_program()
    };

    write!(&mut writer, "{program:?}")?;

    for arg in command.get_args() {
        write!(&mut writer, " {arg:?}")?;
    }

    Ok(ContextItem {
        name: "command",
        value: writer,
    })
}

fn env_context(command: &Command) -> Result<ContextItem> {
    use std::fmt::Write;
    let mut writer = String::new();

    for (key, val) in command.get_envs() {
        let key = key.to_string_lossy();
        let val = val.unwrap_or_default();
        write!(&mut writer, "{key}={val:?} ",)?;
    }

    Ok(ContextItem {
        name: "environment",
        value: writer,
    })
}
