use crate::args::BufMode;
use crate::args::GlobalArgs;
use crate::examples::Example;
use crate::io::ChunkReader;
use crate::io::LineReader;
use crate::io::Separator;
use crate::io::Writer;
use anyhow::Result;
use clap::ArgMatches;
use clap::Command;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;
use std::io::StdinLock;
use std::io::StdoutLock;

pub struct Meta {
    pub name: &'static str,
    pub group: Group,
    pub build: fn(meta: &Meta) -> Command,
    pub run: fn(meta: &Meta, &ArgMatches) -> Result<()>,
    pub examples: &'static [Example],
}

impl Meta {
    pub fn build(&self) -> Command {
        (self.build)(self)
    }

    pub fn run(&self, matches: &ArgMatches) -> Result<()> {
        (self.run)(self, matches)
    }
}

#[macro_export]
macro_rules! command_meta {
    (name: $name:literal, group: $group:expr, args: $args:ident, run: $run:ident, examples: $examples:expr,) => {
        $crate::command::Meta {
            name: $name,
            group: $group,
            build: |meta| -> clap::Command {
                use clap::Args as _;

                let mut command = clap::Command::new(meta.name);
                command = $args::augment_args(command);
                command = $crate::examples::augment_args(command);
                command
            },
            run: |meta, matches| -> anyhow::Result<()> {
                use clap::FromArgMatches;

                if $crate::examples::is_arg_set(&matches) {
                    return $crate::examples::print(&meta.name, meta.examples);
                }

                let global_args = $crate::args::GlobalArgs::from_arg_matches(matches)?;
                let context = $crate::command::Context::from(global_args);
                let args = $args::from_arg_matches(matches)?;

                $run(&context, &args)
            },
            examples: $examples,
        }
    };
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Group {
    #[default]
    General,
    Transformers,
    Mappers,
    Paths,
    Filters,
    Generators,
}

impl Display for Group {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

impl Group {
    pub fn name(&self) -> &'static str {
        match self {
            Self::General => "Commands",
            Self::Transformers => "Transform commands",
            Self::Mappers => "Mapper commands",
            Self::Paths => "Path commands",
            Self::Filters => "Filter commands",
            Self::Generators => "Generator commands",
        }
    }

    pub fn description(&self) -> Option<&'static str> {
        match self {
            Self::General => None,
            Self::Transformers => Some("Transform input text to output. May output a different number of lines than was on input."),
            Self::Mappers => Some("Transform each input line to output. Should output the same number of lines as was on input."),
            Self::Paths => Some("Just like mapper commands but expect input lines to be filesystem paths."),
            Self::Filters => Some("Output only certain input lines based on some criteria."),
            Self::Generators => Some("Generate lines, ignore standard input."),
        }
    }

    pub fn values() -> Vec<Group> {
        vec![
            Self::General,
            Self::Transformers,
            Self::Mappers,
            Self::Paths,
            Self::Filters,
            Self::Generators,
        ]
    }
}

#[derive(Clone)]
pub struct Context(GlobalArgs);

impl From<GlobalArgs> for Context {
    fn from(args: GlobalArgs) -> Self {
        Self(args)
    }
}

impl Context {
    #[allow(clippy::unused_self)]
    pub fn raw_reader(&self) -> StdinLock<'_> {
        stdin().lock()
    }

    pub fn chunk_reader(&self) -> ChunkReader<StdinLock<'_>> {
        ChunkReader::new(self.raw_reader(), self.buf_size())
    }

    pub fn line_reader(&self) -> LineReader<StdinLock<'_>> {
        self.line_reader_from(self.raw_reader())
    }

    pub fn line_reader_from<R: Read>(&self, reader: R) -> LineReader<R> {
        LineReader::new(reader, self.separator(), self.buf_size())
    }

    #[allow(clippy::unused_self)]
    pub fn raw_writer(&self) -> StdoutLock<'_> {
        stdout().lock()
    }

    pub fn writer(&self) -> Writer<StdoutLock<'_>> {
        Writer::new(
            self.raw_writer(),
            self.separator(),
            self.buf_mode().is_full(),
            self.buf_size(),
        )
    }

    pub fn zeroed_buf(&self) -> Vec<u8> {
        vec![0u8; self.buf_size()]
    }

    pub fn uninit_buf(&self) -> Vec<u8> {
        Vec::with_capacity(self.buf_size())
    }

    pub fn buf_mode(&self) -> BufMode {
        self.0.buf_mode
    }

    pub fn buf_size(&self) -> usize {
        self.0.buf_size
    }

    pub fn separator(&self) -> Separator {
        if self.0.null {
            Separator::Null
        } else {
            Separator::Newline
        }
    }
}
