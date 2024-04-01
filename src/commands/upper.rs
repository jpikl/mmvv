use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use bstr::ByteSlice;

pub const META: Meta = command_meta! {
    name: "upper",
    group: Group::Mappers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Convert characters to uppercase.": {
        args: &[],
        input: &["hello world", "Hello World", "HELLO WORLD"],
        output: &["HELLO WORLD", "HELLO WORLD", "HELLO WORLD"],
    },
];

/// Convert characters to uppercase.
#[derive(clap::Args)]
struct Args;

fn run(context: &Context, _args: &Args) -> Result<()> {
    let mut reader = context.char_chunk_reader();
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    while let Some(chunk) = reader.read_chunk()? {
        if chunk.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            chunk.make_ascii_uppercase();
            writer.write(chunk)?;
        } else {
            buffer.clear();
            chunk.to_uppercase_into(&mut buffer);
            writer.write(&buffer)?;
        }
    }

    Ok(())
}
