use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use crate::range::StartRange;
use anyhow::format_err;
use anyhow::Result;

pub const META: Meta = command_meta! {
    name: "seq",
    group: Group::Generators,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Print numbers from 1 to 3.": {
        args: &["1..3"],
        input: &[],
        output: &["1", "2", "3"],
    },
    "Print numbers from 1 to 5 with step 2.": {
        args: &["1..5", "2"],
        input: &[],
        output: &["1", "3", "5"],
    },
    "Print numbers from 1 to -1.": {
        args: &["1..-1"],
        input: &[],
        output: &["1", "0", "-1"],
    },
    "Print numbers from 1 to -3 with step -2.": {
        args: &["1..-3", "-2"],
        input: &[],
        output: &["1", "-1", "-3"],
    },
];

/// Print sequence of numbers as lines
#[derive(clap::Args)]
struct Args {
    /// Sequence range.
    ///
    /// Both `FROM` and `TO` are integers.
    ///
    /// `TO` may be ommited to produce an infinite sequence.
    #[arg(value_name = "FROM..[TO]", default_value_t = StartRange(1, None), allow_hyphen_values = true)]
    range: StartRange<i128>,

    /// Increment between numbers in sequence.
    ///
    /// Default value: `1` (for increasing sequence), `-1` (for decreasing sequence)
    #[arg(allow_negative_numbers = true)]
    step: Option<i128>,
}

#[allow(clippy::too_many_lines)]
fn run(context: &Context, args: &Args) -> Result<()> {
    let StartRange(first, last) = args.range;
    let step = args.step;

    let mut value = first;
    let mut writer = context.writer();
    let mut formatter = Formatter::new();

    match last {
        Some(last) if first < last => {
            let step = step.unwrap_or(1);
            while value <= last {
                writer.write_line(formatter.format(value))?;
                value += step;
            }
        }
        Some(last) if first > last => {
            let step = step.unwrap_or(-1);
            while value >= last {
                writer.write_line(formatter.format(value))?;
                value += step;
            }
        }
        Some(_) => {
            // first == last
            writer.write_line(formatter.format(value))?;
        }
        None => {
            let step = step.unwrap_or(1);
            loop {
                writer.write_line(formatter.format(value))?;
                if let Some(new_value) = value.checked_add(step) {
                    value = new_value;
                } else {
                    return Err(format_err!("number sequence reached interger limit"));
                }
            }
        }
    }

    Ok(())
}

struct Formatter {
    buffer: Vec<u8>,
}

impl Formatter {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(64), // Enough for i128 digits
        }
    }

    fn format(&mut self, value: i128) -> &[u8] {
        self.buffer.clear();
        itoap::write_to_vec(&mut self.buffer, value);
        &self.buffer
    }
}
