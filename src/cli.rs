use crate::pattern::META_CHARS;
use clap::{App, Arg, ArgMatches, OsValues};
use regex::Regex;
use termcolor::ColorChoice;

const COLOR: &str = "color";
const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";
const ESCAPE: &str = "escape";
const PATHS: &str = "paths";
const PATTERN: &str = "pattern";
const PRINT_NUL: &str = "print-nul";
const PRINT_RAW: &str = "print-raw";
const READ_NUL: &str = "read-nul";
const REGEX_FILENAME: &str = "regex-filename";
const REGEX_PATH: &str = "regex-path";

pub struct Cli<'a> {
    matches: ArgMatches<'a>,
}

impl<'a> Cli<'a> {
    pub fn new() -> Self {
        Self {
            matches: App::new(env!("CARGO_PKG_NAME"))
                .version(env!("CARGO_PKG_VERSION"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .arg(Self::pattern_arg())
                .arg(Self::paths_arg())
                .arg(Self::color_arg())
                .arg(Self::escape_arg())
                .arg(Self::print_nul_arg())
                .arg(Self::print_raw_arg())
                .arg(Self::read_nul_arg())
                .arg(Self::regex_filename_arg())
                .arg(Self::regex_path_arg())
                .get_matches(),
        }
    }

    fn pattern_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PATTERN)
            .index(1)
            .required(true)
            .validator(Self::validate_pattern)
            .value_name("PATTERN")
            .help("Output pattern")
    }

    pub fn pattern(&self) -> &str {
        self.matches.value_of(PATTERN).unwrap()
    }

    fn validate_pattern(value: String) -> Result<(), String> {
        if value.is_empty() {
            Err("Empty string".to_string())
        } else {
            Ok(())
        }
    }

    fn paths_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PATHS)
            .index(2)
            .multiple(true)
            .value_name("PATH")
            .help("Paths to process. Optional, paths are read from stdin by default")
    }

    pub fn paths(&self) -> Option<OsValues> {
        self.matches.values_of_os(PATHS)
    }

    fn color_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(COLOR)
            .long("color")
            .takes_value(true)
            .value_name("WHEN")
            .possible_values(&[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI])
            .help("Output colors")
    }

    pub fn color(&self) -> Option<ColorChoice> {
        self.matches.value_of(COLOR).map(|value| match value {
            COLOR_AUTO => ColorChoice::Auto,
            COLOR_ALWAYS => ColorChoice::Always,
            COLOR_ANSI => ColorChoice::AlwaysAnsi,
            COLOR_NEVER => ColorChoice::Never,
            _ => panic!("Unexpected {} value {}", COLOR, value),
        })
    }

    fn escape_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(ESCAPE)
            .long("escape")
            .takes_value(true)
            .value_name("CHAR")
            .validator(Self::validate_escape)
            .help("Custom escape character to use in pattern")
    }

    fn validate_escape(value: String) -> Result<(), String> {
        let chars: Vec<char> = value.chars().collect();
        if chars.len() != 1 {
            Err("Value must be a single character".to_string())
        } else if META_CHARS.contains(&chars[0]) {
            Err(format!(
                "Cannot use one of meta characters {}",
                META_CHARS
                    .iter()
                    .map(|char| format!("'{}'", char))
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        } else {
            Ok(())
        }
    }

    pub fn escape(&self) -> Option<char> {
        self.matches
            .value_of(ESCAPE)
            .map(|value| value.chars().next().expect("Validation failed"))
    }

    fn print_nul_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PRINT_NUL)
            .short("Z")
            .long("print-0")
            .conflicts_with(PRINT_RAW)
            .help("Print paths delimited by NUL, not newline")
    }

    pub fn print_nul(&self) -> bool {
        self.matches.is_present(PRINT_NUL)
    }

    fn print_raw_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(PRINT_RAW)
            .short("R")
            .long("print-raw")
            .conflicts_with(PRINT_NUL)
            .help("Print paths without any delimiter")
    }

    pub fn print_raw(&self) -> bool {
        self.matches.is_present(PRINT_RAW)
    }

    fn read_nul_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(READ_NUL)
            .short("z")
            .long("read-0")
            .help("Read paths delimited by NUL, not newline")
    }

    pub fn read_nul(&self) -> bool {
        self.matches.is_present(READ_NUL)
    }

    fn regex_filename_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(REGEX_FILENAME)
            .short("e")
            .long("regex")
            .takes_value(true)
            .value_name("EXPR")
            .validator(Self::validate_regex)
            .conflicts_with(REGEX_PATH)
            .help("Regular expression matched against filename")
    }

    pub fn regex_filename(&self) -> Option<Regex> {
        self.matches
            .value_of(REGEX_FILENAME)
            .map(|value| Regex::new(value).expect("Validation failed"))
    }

    fn regex_path_arg<'b>() -> Arg<'a, 'b> {
        Arg::with_name(REGEX_PATH)
            .short("E")
            .long("regex-full")
            .takes_value(true)
            .value_name("EXPR")
            .validator(Self::validate_regex)
            .conflicts_with(REGEX_FILENAME)
            .help("Regular expression matched against full path")
    }

    pub fn regex_path(&self) -> Option<Regex> {
        self.matches
            .value_of(REGEX_PATH)
            .map(|value| Regex::new(value).expect("Validation failed"))
    }

    fn validate_regex(value: String) -> Result<(), String> {
        if let Err(error) = Regex::new(&value) {
            Err(error.to_string())
        } else {
            Ok(())
        }
    }
}
