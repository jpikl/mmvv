use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use anyhow::Result;
use std::ffi::OsString;

pub const META: Meta = command_meta! {
    name: "stream",
    group: Group::Generators,
    args: Args,
    run: run,
    examples: command_examples! [],
};

/// Print arguments as lines.
#[derive(clap::Args)]
struct Args {
    /// Values to print.
    values: Vec<OsString>,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut writer = context.writer();

    for value in &args.values {
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::ffi::OsStrExt;
            writer.write_line(value.as_bytes())?;
        }
        #[cfg(target_family = "windows")]
        {
            writer.write_line(value.to_string_lossy().as_bytes())?;
        }
    }

    Ok(())
}
