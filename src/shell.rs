use derive_more::Display;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Formatter;
use std::path::Path;
use std::process::Command;

pub enum ShellKind {
    UnixShell,
    WinCmd,
    PowerShell,
}

impl ShellKind {
    fn command_option_name(&self) -> &str {
        match self {
            Self::UnixShell => "-c",
            Self::WinCmd => "/c",
            Self::PowerShell => "-Command",
        }
    }

    pub fn normalize_path<'a>(&self, path: &'a str) -> Cow<'a, str> {
        match self {
            // MinGW Bash from "Git from Windows" needs unix path separators.
            #[cfg(target_os = "windows")]
            Self::UnixShell => Cow::Owned(path.replace('\\', "/")),
            _ => Cow::Borrowed(path),
        }
    }
}

#[derive(Clone)]
pub struct Shell(OsString);

impl Display for Shell {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.0.to_string_lossy())
    }
}

impl Default for Shell {
    #[cfg(target_os = "windows")]
    fn default() -> Self {
        Self("cmd".into())
    }

    #[cfg(not(target_os = "windows"))]
    fn default() -> Self {
        Self("sh".into())
    }
}

impl From<&OsStr> for Shell {
    fn from(value: &OsStr) -> Self {
        Self::new(value)
    }
}

impl Shell {
    pub fn new(bin: impl Into<OsString>) -> Self {
        Self(bin.into())
    }

    pub fn bin(&self) -> &OsStr {
        &self.0
    }

    pub fn kind(&self) -> ShellKind {
        let file_stem = Path::new(&self.0)
            .file_stem()
            .unwrap_or_default()
            .to_ascii_lowercase();

        match file_stem.to_str() {
            #[cfg(target_os = "windows")]
            Some("cmd") => ShellKind::WinCmd,
            #[cfg(target_os = "windows")]
            Some("powershell") => ShellKind::PowerShell,
            // Cross-platform edition of PowerShell
            Some("pwsh") => ShellKind::PowerShell,
            _ => ShellKind::UnixShell,
        }
    }

    pub fn build_command(&self, shell_command: &str) -> Command {
        let mut command = Command::new(self.bin());
        // SHELL env is needed for two reasons:
        // 1. To propagate `SHELL` to `rew x` which could get potentially spawned inside the shell.
        // 2. To fix this weird Rust issue https://github.com/rust-lang/rust/issues/122660 which causes
        //    that a potentially wrong shell is spawned, ignoring precedence in `PATH` env var.
        command.env("SHELL", self.bin());
        command.arg(self.kind().command_option_name());
        command.arg(shell_command);
        command
    }
}
