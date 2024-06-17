use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use std::io::copy;

pub const META: Meta = command_meta! {
    name: "cat",
    group: Group::Mappers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Copy input to output.": {
        args: &[],
        input: &["first", "second", "third"],
        output: &["first", "second", "third"],
    },
];

/// Copy all input to output
///
/// Mostly useful for benchmarking raw IO throughput of rew.
#[derive(clap::Args)]
struct Args {
    /// Process data as lines.
    ///
    /// Will normalize newlines to LF as a side effect.
    #[arg(short, long, conflicts_with_all = ["chars", "bytes"])]
    lines: bool,

    /// Process data as character chunks.
    #[arg(short, long, conflicts_with_all = ["lines", "bytes"])]
    chars: bool,

    /// Process data as byte chunks.
    #[arg(short, long, conflicts_with_all = ["lines", "chars"])]
    bytes: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    if args.lines {
        let mut reader = context.line_reader();
        let mut writer = context.writer();

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if args.chars {
        let mut reader = context.char_chunk_reader();
        let mut writer = context.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else if args.bytes {
        let mut reader = context.byte_chunk_reader();
        let mut writer = context.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else {
        let mut reader = context.raw_reader();
        let mut writer = context.raw_writer();

        copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
