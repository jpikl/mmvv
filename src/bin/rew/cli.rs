use crate::counter;
use clap::{crate_name, crate_version, AppSettings, ArgSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::help::highlight_static;
use common::run::Options;
use indoc::indoc;
use regex::Regex;
use std::path::PathBuf;
use termcolor::ColorChoice;

const INPUT_HEADING: Option<&str> = Some("INPUT OPTIONS");
const OUTPUT_HEADING: Option<&str> = Some("OUTPUT OPTIONS");
const PROCESSING_HEADING: Option<&str> = Some("PROCESSING OPTIONS");
const PATTERN_HEADING: Option<&str> = Some("PATTERN OPTIONS");
const HELP_HEADING: Option<&str> = Some("HELP OPTIONS");

#[derive(Debug, Clap)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    override_usage = "rew [OPTIONS] [pattern] [--] [values]...",
    after_help = highlight_static("Use `-h` for short descriptions and `--help` for more details."),
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::DontCollapseArgsInUsage),
)]
/// Rewrite FS paths according to a pattern
pub struct Cli {
    /// Output pattern
    #[clap(
        setting(ArgSettings::AllowEmptyValues),
        long_about = highlight_static(indoc!{"
            Output pattern

            If not provided, input values are directly written to stdout.

            Use `--explain` flag to print explanation of a given pattern.
            Use `--help-pattern` flag to print pattern syntax reference.
            Use `--help-filters` flag to print filter reference.
        "})
    )]
    pub pattern: Option<String>,

    /// Input values (read from stdin by default)
    #[clap(value_name = "value", setting(ArgSettings::AllowEmptyValues))]
    pub values: Vec<String>,

    /// Read values delimited by a specific character, not newline
    #[clap(
        short = 'd',
        long,
        value_name = "char",
        conflicts_with_all = &["read-nul", "read-raw"],
        parse(try_from_str = parse_single_byte_char),
        help_heading = INPUT_HEADING,
    )]
    pub read: Option<u8>,

    /// Read values delimited by NUL, not newline
    #[clap(
        short = 'z',
        long,
        conflicts_with_all = &["read-raw", "read"],
        help_heading = INPUT_HEADING)
    ]
    pub read_nul: bool,

    /// Read the whole input into memory as a single value
    #[clap(
        short = 'r',
        long,
        conflicts_with_all = &["read-nul", "read"],
        help_heading = INPUT_HEADING
    )]
    pub read_raw: bool,

    /// Print results delimited by a specific string, not newline
    #[clap(
        short = 'D',
        long,
        value_name = "string",
        conflicts_with_all = &["print-nul", "print-raw"],
        help_heading = OUTPUT_HEADING,
    )]
    pub print: Option<String>,

    /// Print results delimited by NUL, not newline
    #[clap(
        short = 'Z',
        long,
        conflicts_with_all = &["print-raw", "print"],
        help_heading = OUTPUT_HEADING
    )]
    pub print_nul: bool,

    /// Print results without a delimiter
    #[clap(
        short = 'R',
        long,
        conflicts_with_all = &["print-nul", "print"],
        help_heading = OUTPUT_HEADING
    )]
    pub print_raw: bool,

    /// Do not print final delimiter at the end of output
    #[clap(short = 'T', long, help_heading = OUTPUT_HEADING)]
    pub no_trailing_delimiter: bool,

    /// Enable diff output mode
    #[clap(
        short = 'b',
        long,
        conflicts_with = "pretty",
        help_heading = OUTPUT_HEADING,
        long_about = highlight_static(indoc!{"
            Enable diff output mode

            Respects `--print*` flags/options.
            Ignores `--no-trailing-delimiter` flag.
            Prints machine-readable transformations as results:
           
                <input_value_1
                >output_value_1
                <input_value_2
                >output_value_2
                ...
                <input_value_N
                >output_value_N
           
            Such output can be processed by accompanying `mvb` and `cpb` utilities to perform bulk move/copy.
        "}),
    )]
    pub diff: bool,

    /// Enable pretty output mode
    #[clap(
        short = 'p',
        long,
        conflicts_with = "diff",
        help_heading = OUTPUT_HEADING,
        long_about = highlight_static(indoc!{"
            Enable pretty output mode

            Ignores `--print*` flags/options.
            Ignores `--no-trailing-delimiter` flag.
            Prints human-readable transformations as results:

                input_value_1 -> output_value_1
                input_value_2 -> output_value_2
                ...
                input_value_N -> output_value_N
        "}),
    )]
    pub pretty: bool,

    /// When to use colors
    #[clap(
        long,
        value_name = "when",
        possible_values = COLOR_VALUES,
        parse(try_from_str = parse_color),
        help_heading = OUTPUT_HEADING
    )]
    pub color: Option<ColorChoice>,

    /// Regular expression matched against each input value
    #[clap(
        short = 'e',
        long,
        value_name = "regex",
        conflicts_with = "regex-filename",
        help_heading = PROCESSING_HEADING,
    )]
    pub regex: Option<Regex>,

    /// Regular expression matched against 'filename component' of each input value
    #[clap(
        short = 'E',
        long,
        value_name = "regex",
        conflicts_with = "regex",
        help_heading = PROCESSING_HEADING
    )]
    pub regex_filename: Option<Regex>,

    /// Local counter configuration
    ///
    /// init - Initial value.
    /// step - Value increment (default: 1)
    #[clap(
        short = 'c',
        long,
        value_name = "init[:step]",
        help_heading = PROCESSING_HEADING,
        verbatim_doc_comment,
    )]
    pub local_counter: Option<counter::Config>,

    /// Global counter configuration
    ///
    /// init - Initial value.
    /// step - Value increment (default: 1).
    #[clap(
        short = 'C',
        long,
        value_name = "init[:step]",
        help_heading = PROCESSING_HEADING,
        verbatim_doc_comment,
    )]
    pub global_counter: Option<counter::Config>,

    /// Directory against which to resolve relative/absolute paths
    #[clap(short = 'w', long, value_name = "path", help_heading = PROCESSING_HEADING)]
    pub working_directory: Option<PathBuf>,

    /// Continue processing after an error, fail at end
    #[clap(short = 's', long, help_heading = PROCESSING_HEADING)]
    pub fail_at_end: bool,

    /// Print explanation of a given pattern
    #[clap(long, requires = "pattern", help_heading = PATTERN_HEADING)]
    pub explain: bool,

    /// Custom escape character to use in pattern
    #[clap(long, value_name = "char", help_heading = PATTERN_HEADING)]
    pub escape: Option<char>,

    /// Print help information
    #[clap(short = 'h', long, help_heading = HELP_HEADING)]
    pub help: bool,

    /// Print pattern syntax reference
    #[clap(long, help_heading = HELP_HEADING)]
    pub help_pattern: bool,

    /// Print filter reference
    #[clap(long, help_heading = HELP_HEADING)]
    pub help_filters: bool,

    /// Print version information
    #[clap(long, help_heading = HELP_HEADING)]
    pub version: bool,
}

impl Options for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

pub fn parse_single_byte_char(string: &str) -> Result<u8, &'static str> {
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
    use super::*;
    use claim::*;

    #[test]
    fn default() {
        assert_ok!(Cli::try_parse_from(&["rew"]));
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["rew", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    mod parse_single_byte_char {
        use super::*;

        #[test]
        fn single_byte() {
            assert_eq!(parse_single_byte_char("a"), Ok(b'a'));
        }

        #[test]
        fn multi_byte() {
            assert_eq!(
                parse_single_byte_char("á"),
                Err("multi-byte characters are not supported",)
            );
        }

        #[test]
        fn multi_char() {
            assert_eq!(
                parse_single_byte_char("aa"),
                Err("value must be a single character")
            );
        }
    }
}
