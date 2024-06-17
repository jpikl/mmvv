use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use bstr::ByteSlice;
use bstr::ByteVec;
use deunicode::deunicode_char;

pub const META: Meta = command_meta! {
    name: "ascii",
    group: Group::Mappers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples! [
    "Convert input to ASCII.":  {
        args: &[],
        input: &["Æneid", "étude", "🦀rocks!"],
        output: &["AEneid", "etude", "crab rocks!"],
    },
    "Delete non-ASCII characters from input.": {
        args: &["-d"],
        input: &["Æneid", "étude", "🦀rocks!"],
        output: &["neid", "tude", "rocks!"],
    },
];

/// Convert characters to ASCII.
#[derive(clap::Args)]
struct Args {
    /// Delete non-ASCII characters instead of converting them.
    #[arg(short, long)]
    delete: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.char_chunk_reader();
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    while let Some(chunk) = reader.read_chunk()? {
        if chunk.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            writer.write(chunk)?;
            continue;
        }

        // Copying chars to a side buffer is faster than directly writing them to buffered writer
        if args.delete {
            chunk
                .chars()
                .filter(char::is_ascii)
                .for_each(|char| buffer.push(char as u8));
        } else {
            chunk
                .chars()
                .map(deunicode_char)
                .for_each(|str| buffer.push_str(str.unwrap_or("?")));
        }

        writer.write(&buffer)?;
        buffer.clear();
    }

    Ok(())
}
