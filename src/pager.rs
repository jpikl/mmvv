use crate::process::CommandEx;
use anstream::stream::IsTerminal;
use anyhow::Result;
use std::io::stdout;
use std::panic::resume_unwind;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use which::which;

pub struct Pager(Command);

impl Pager {
    pub fn detect() -> Option<Pager> {
        if !stdout().is_terminal() {
            return None;
        }

        // We could eventually do something more complex, such as parsing PAGER
        // env variable like `bat` does https://github.com/sharkdp/bat/issues/158,
        // but that would be an overkill for our use case.

        if let Ok(path) = which("less") {
            let mut command = Command::new(path);
            // F = Exit immediately if the text fits the entire screen.
            // I = Ignore case when searching.
            // r = Causes "raw" control characters to be displayed.
            // X = Disables sending the termcap (de)itialization.
            command.arg("-FIrX");
            return Some(Pager(command));
        }

        if let Ok(path) = which("more") {
            return Some(Pager(Command::new(path)));
        }

        None
    }

    pub fn open(
        &mut self,
        callback: impl Fn(&mut ChildStdin) -> Result<()> + Send + 'static,
    ) -> Result<()> {
        let mut pager = self.0.stdin(Stdio::piped()).spawn_with_context()?;
        let mut stdin = pager.take_stdin().expect("could not get pager stdin");

        let thread = thread::spawn(move || {
            callback(&mut stdin.inner).map_err(|err| {
                stdin
                    .context
                    .apply(err)
                    .context("failed to write to child process stdin")
            })
        });

        pager.wait()?;
        thread.join().map_err(resume_unwind)??;

        Ok(())
    }
}
