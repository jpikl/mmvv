use anyhow::Result;
use bstr::ByteSlice;
use std::process::Command;
use std::process::Stdio;

#[derive(Default)]
pub struct StdBuf {
    line_buf_env: Option<Vec<(String, String)>>,
}

impl StdBuf {
    pub fn line_buf_env(&mut self) -> Vec<(String, String)> {
        self.line_buf_env
            .get_or_insert_with(|| Self::detect_env(&["-oL"]).unwrap_or_default())
            .clone()
    }

    // This is probably the least invasive way how to force output line buffering on external commands (which use libc).
    // 1. Run `stdbuf [args] env` and extract relevant environment variables from its output.
    // 2. Run the external command with those same enviroment variables (as `stdbuf` would do).
    //
    // Alternatively we could:
    // a) Run external commands directly under `stdbuf`. This produces rather confusing error messages
    //    for the user when something goes wrong.
    // b) Re-run rew itself under `stdbuf`. This causes issues on Windows where some commands are not able
    //    to find the libstdbuf shared library because they receive it as `C:/path/to/libstdbuf.dll`
    //    instead of `/path/to/libstdbuf.dll`.
    fn detect_env(args: &[&str]) -> Result<Vec<(String, String)>> {
        // If `stdbuf` is not found, the caller will silently ignore the error.
        // If `env` is not found, the error will be print to the stderr (by `stdbuf`).
        // The second scenario is very unlikely because `stdbuf` and `env` are usually installed together.
        let output = Command::new("stdbuf")
            .args(args)
            .arg("env")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        let envs = output
            .stdout
            .split_str("\n")
            .filter_map(|line| line.split_once_str("="))
            .filter(|(key, _)| {
                is_preload_key(key) || key.starts_with_str("_STDBUF_" /* has I,O,E suffixes */)
            })
            .filter_map(|(key, val)| {
                if let (Ok(key), Ok(val)) = (key.to_str(), val.to_str()) {
                    Some((key.to_owned(), val.to_owned()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(envs)
    }
}

#[cfg(not(target_os = "macos"))]
fn is_preload_key(key: &[u8]) -> bool {
    key == b"LD_PRELOAD"
}

#[cfg(target_os = "macos")]
fn is_preload_key(key: &[u8]) -> bool {
    key == b"DYLD_INSERT_LIBRARIES" || key == b"DYLD_FORCE_FLAT_NAMESPACE"
}
