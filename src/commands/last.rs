use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::examples::Example;
use anyhow::Result;
use memchr::memchr;
use std::collections::LinkedList;
use std::io::Read;
use std::mem::replace;

pub const META: Meta = command_meta! {
    name: "last",
    group: Group::Filters,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples![
    "Print the last line.": {
        args: &[],
        input: &["first", "second", "third"],
        output: &["third"],
    },
    "Print the last two lines.": {
        args: &["2"],
        input: &["first", "second", "third"],
        output: &["second", "third"],
    },
];

/// Output last N input lines.
#[derive(clap::Args)]
struct Args {
    /// Number of lines to print.
    #[arg(default_value_t = 1)]
    count: usize,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    if args.count == 0 {
        return Ok(());
    }

    let mut reader = context.raw_reader();
    let mut buffers = LinkedList::new();
    let mut last_buf = LineBuf::new(context.zeroed_buf());
    let mut total_lines = 0;
    let separator = context.separator().as_byte();

    loop {
        let len = reader.read(last_buf.available())?;
        if len == 0 {
            if !last_buf.is_empty() {
                buffers.push_back(last_buf);
            }
            break;
        }

        total_lines += last_buf.advance(len, separator);

        if last_buf.is_full() {
            let new_buf = LineBuf::new(context.zeroed_buf());
            buffers.push_back(replace(&mut last_buf, new_buf));

            if let Some(first_buf) = buffers.front() {
                if total_lines - first_buf.lines > args.count {
                    total_lines -= first_buf.lines;
                    buffers.pop_front();
                }
            }
        }
    }

    let last_line_terminated = buffers
        .back()
        .is_some_and(|buf| buf.is_terminated_with(separator));

    if !last_line_terminated {
        total_lines += 1;
    }

    let mut writer = context.writer();

    for buf in buffers {
        let mut data = buf.used();

        if total_lines > args.count {
            while let Some(pos) = memchr(separator, data) {
                data = &data[(pos + 1)..];
                total_lines -= 1;

                if total_lines == args.count {
                    writer.write(data)?;
                    break;
                }
            }
        } else {
            writer.write(data)?;
        }
    }

    Ok(())
}

struct LineBuf {
    data: Vec<u8>,
    end: usize,
    lines: usize,
}

impl LineBuf {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            end: 0,
            lines: 0,
        }
    }

    fn used(&self) -> &[u8] {
        &self.data[..self.end]
    }

    fn available(&mut self) -> &mut [u8] {
        &mut self.data[self.end..]
    }

    fn advance(&mut self, len: usize, separator: u8) -> usize {
        let next_end = self.end + len;
        let mut area = &self.data[self.end..next_end];
        let mut lines = 0;

        while let Some(pos) = memchr(separator, area) {
            lines += 1;
            area = &area[(pos + 1)..];
        }

        self.lines += lines;
        self.end = next_end;
        lines
    }

    fn is_empty(&self) -> bool {
        self.end == 0
    }

    fn is_full(&self) -> bool {
        self.end == self.data.len()
    }

    fn is_terminated_with(&self, separator: u8) -> bool {
        self.end > 0 && self.data[self.end - 1] == separator
    }
}
