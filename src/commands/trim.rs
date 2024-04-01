use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use bstr::ByteSlice;

pub const META: Meta = command_meta! {
    name: "trim",
    group: Group::Mappers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Trim whitespaces from both sides each line.": {
        args: &[],
        input: &["  spaces around  ", "  spaces before", "spaces after   "],
        output: &["spaces around", "spaces before", "spaces after"],
    },
    "Trim whitespaces from start of each line.": {
        args: &["-s"],
        input: &["  spaces around  ", "  spaces before", "spaces after   "],
        output: &["spaces around  ", "spaces before", "spaces after   "],
    },
    "Trim whitespaces from end of each line.": {
        args: &["-e"],
        input: &["  spaces around  ", "  spaces before", "spaces after   "],
        output: &["  spaces around", "  spaces before", "spaces after"],
    },
];

/// Trim whitespaces from each line.
///
/// By default, both the beginning and the end are trimmed.
#[derive(clap::Args)]
struct Args {
    /// Trim the beginning.
    #[arg(short, long)]
    start: bool,

    /// Trim the end.
    #[arg(short, long)]
    end: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.line_reader();
    let mut writer = context.writer();

    while let Some(line) = reader.read_line()? {
        let result = match (args.start, args.end) {
            (true, true) | (false, false) => line.trim(),
            (true, false) => line.trim_start(),
            (false, true) => line.trim_end(),
        };
        writer.write_line(result)?;
    }

    Ok(())
}
