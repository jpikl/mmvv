use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use std::io::copy;
use std::io::Read;

pub const META: Meta = command_meta! {
    name: "loop",
    group: Group::Transformers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Repeat all input two times.": {
        args: &["2"],
        input: &["first", "second"],
        output: &["first", "second", "first", "second"],
    },
];

/// Repeatedly output all captured input.
#[derive(clap::Args)]
struct Args {
    /// How many times do the repetition (default: forever).
    #[arg()]
    count: Option<u128>,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let count = args.count;

    if count == Some(0) {
        return Ok(());
    }

    if count == Some(1) {
        // Avoid buffering the whole input if there is only one output iteration
        copy(&mut context.raw_reader(), &mut context.raw_writer())?;
        return Ok(());
    }

    let buf_size = context.buf_size();
    let mut reader = context.raw_reader();
    let mut writer = context.writer();
    let mut buffer = context.zeroed_buf();
    let mut end = 0;

    loop {
        let len = reader.read(&mut buffer[end..])?;
        if len == 0 {
            break;
        }

        // Write the first output iteration as we read the input
        let next_end = end + len;
        writer.write(&buffer[end..next_end])?;
        end = next_end;

        if buffer.len() - end < buf_size / 2 {
            buffer.resize(buffer.len() + buf_size, 0);
        }
    }

    if let Some(mut count) = count {
        // We already did first output iteration during reading phase
        while count > 1 {
            writer.write(&buffer[..end])?;
            count -= 1;
        }
        return Ok(());
    }

    loop {
        writer.write(&buffer[..end])?;
    }
}
