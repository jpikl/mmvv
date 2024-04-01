use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use memchr::memchr;
use std::result;

pub const META: Meta = command_meta! {
    name: "split",
    group: Group::Transformers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Split input into lines on comma.": {
        args: &[","],
        input: &["first,second,third"],
        output: &["first", "second", "third"],
    },
    "Split input into lines on comma (process trailing comma).": {
        args: &[","],
        input: &["first,second,third,"],
        output: &["first", "second", "third", ""],
    },
    "Split input into lines on comma (ignore trailing comma).": {
        args: &["-t", ","],
        input: &["first,second,third,"],
        output: &["first", "second", "third"],
    },
];

/// Split input into lines using a separator.
#[derive(clap::Args)]
struct Args {
    /// Separator (single byte character).
    #[arg(value_parser = parse_separator)]
    separator: u8,

    /// Ignore trailing separator at the end of input.
    #[arg(short = 't', long)]
    ignore_trailing: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.byte_chunk_reader();
    let mut writer = context.writer();

    let input_separator = args.separator;
    let output_separator = context.separator().as_byte();
    let trim_output_separator = context.separator().trim_fn();

    let mut ending_separator_written = false;
    let mut start_next_chunk_separated = false;

    while let Some(mut chunk) = reader.read_chunk()? {
        if start_next_chunk_separated {
            writer.write(&[output_separator])?;
            ending_separator_written = true;
        }

        let trimmed_chunk_len = trim_output_separator(chunk).len();
        if trimmed_chunk_len < chunk.len() {
            // Write the trimmed separator once we know there is more data
            chunk = &mut chunk[..trimmed_chunk_len];
            start_next_chunk_separated = true;
        } else {
            start_next_chunk_separated = false;
        }

        if trimmed_chunk_len > 0 {
            let mut remainder = &mut chunk[..];

            while let Some(pos) = memchr(input_separator, remainder) {
                remainder[pos] = output_separator;
                remainder = &mut remainder[(pos + 1)..];
            }

            writer.write(chunk)?;
            ending_separator_written = chunk[trimmed_chunk_len - 1] == output_separator;
        }
    }

    if !ending_separator_written || !args.ignore_trailing {
        writer.write(&[output_separator])?;
    }

    Ok(())
}

fn parse_separator(string: &str) -> result::Result<u8, &'static str> {
    if string.chars().count() != 1 {
        Err("value must be a single character")
    } else if string.len() != 1 {
        Err("multi-byte characters are not supported")
    } else {
        Ok(string.as_bytes()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::parse_separator as parse;
    use claims::assert_err_eq;
    use claims::assert_ok_eq;

    #[test]
    fn parse_separator() {
        assert_ok_eq!(parse("a"), b'a');
        assert_err_eq!(parse(""), "value must be a single character");
        assert_err_eq!(parse("ab"), "value must be a single character");
        assert_err_eq!(parse("á"), "multi-byte characters are not supported");
    }
}
