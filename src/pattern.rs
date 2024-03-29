use crate::colors::BOLD_RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use derive_more::Display;
use derive_more::IsVariant;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::iter::Fuse;
use std::iter::Peekable;
use std::result;
use std::str::Chars;

const EXPR_START: char = '{';
const EXPR_END: char = '}';
const PIPE: char = '|';

const NO_STDIN_MARKER: char = ':';
const RAW_SHELL_MARKER: char = '#';
const EXTERN_MARKER: char = '!';

const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';

const ESCAPED_LF: char = 'n';
const ESCAPED_CR: char = 'r';
const ESCAPED_TAB: char = 't';

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, derive_more::Error, PartialEq)]
pub struct Error {
    input: String,
    kind: ErrorKind,
    position: usize,
}

#[derive(Debug, Display, PartialEq)]
pub enum ErrorKind {
    #[display("the previous {EXPR_START} was not closed")]
    UnexpectedExprStart,
    #[display("missing command before {PIPE}")]
    MissingCommandBefore,
    #[display("missing command after {PIPE}")]
    MissingCommandAfter,
    #[display("missing closing {_0}")]
    MissingClosingQuote(char),
    #[display("missing opening {EXPR_START}")]
    MissingExprStart,
    #[display("missing closing {EXPR_END}")]
    MissingExprEnd,
    #[display("empty shell command")]
    EmptyShellCommand,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            fmt,
            "pattern syntax error at position {YELLOW}{}{RESET}",
            self.position
        )?;

        writeln!(fmt)?;
        writeln!(fmt, "{}", self.input)?;

        let padding = " ".repeat(self.position);
        writeln!(fmt, "{padding}{BOLD_RED}^{RESET}")?;
        writeln!(fmt, "{padding}{BOLD_RED}{}{RESET}", self.kind)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern(Vec<Item>);

pub struct SimplePattern(Vec<SimpleItem>);

#[derive(Debug, Display, Clone, PartialEq, IsVariant)]
pub enum Item {
    Constant(String),
    Expression(Expression),
}

pub enum SimpleItem {
    Constant(String),
    Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub no_stdin: bool,
    pub value: ExpressionValue,
    pub raw_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionValue {
    Pipeline(Vec<Command>),
    RawShell(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub external: bool,
}

impl Pattern {
    pub fn parse(input: &str, escape: char) -> Result<Self> {
        Parser::new(input, escape).parse().map(Self)
    }

    pub fn items(&self) -> &Vec<Item> {
        &self.0
    }

    #[must_use]
    pub fn quote_expressions(self, quote: char) -> Self {
        let mut items = Vec::new();

        for item in self.0 {
            if item.is_expression() {
                items.push(Item::Constant(quote.to_string()));
                items.push(item);
                items.push(Item::Constant(quote.to_string()));
            } else {
                items.push(item);
            }
        }

        Self(items)
    }

    pub fn try_simplify(&self) -> Option<SimplePattern> {
        let mut simple_items = Vec::new();

        for item in self.items() {
            let simple_item = match item {
                Item::Constant(value) => SimpleItem::Constant(value.clone()),
                Item::Expression(expr) => match &expr.value {
                    ExpressionValue::Pipeline(commands) if commands.is_empty() => {
                        SimpleItem::Expression
                    }
                    _ => return None, // One or more commands or raw shell
                },
            };
            simple_items.push(simple_item);
        }

        Some(SimplePattern(simple_items))
    }
}

impl SimplePattern {
    pub fn items(&self) -> &Vec<SimpleItem> {
        &self.0
    }
}

impl Display for Pattern {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        for item in self.items() {
            write!(fmt, "{item}")?;
        }
        Ok(())
    }
}

impl Display for Expression {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{{")?;
        if self.no_stdin {
            write!(fmt, ":")?;
        }
        write!(fmt, "{}", self.value)?;
        write!(fmt, "}}")
    }
}

impl Display for ExpressionValue {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawShell(command) => write!(fmt, "#`{command}`"),
            Self::Pipeline(commands) => {
                for (i, command) in commands.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, "|")?;
                    }
                    write!(fmt, "{command}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for Command {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        if self.external {
            write!(fmt, "!")?;
        }
        write!(fmt, "`{}`", self.name)?;
        for arg in &self.args {
            write!(fmt, " `{arg}`")?;
        }
        Ok(())
    }
}

pub struct Parser<'a> {
    input: String,
    iterator: Peekable<Fuse<Chars<'a>>>,
    position: usize,
    offset: usize,
    escape: char,
}

impl Parser<'_> {
    pub fn new(input: &str, escape: char) -> Parser<'_> {
        Parser {
            input: input.into(),
            iterator: input.chars().fuse().peekable(),
            position: 0,
            offset: 0,
            escape,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Item>> {
        let mut items = Vec::new();

        while let Some(char) = self.peek() {
            let item = if char == EXPR_START {
                Item::Expression(self.parse_expression()?)
            } else {
                Item::Constant(self.parse_constant()?)
            };
            items.push(item);
        }

        Ok(items)
    }

    fn parse_constant(&mut self) -> Result<String> {
        let mut constant = String::new();

        while let Some(char) = self.peek() {
            match char {
                EXPR_START => break,
                EXPR_END => return Err(self.err(ErrorKind::MissingExprStart)),
                char if char == self.escape => {
                    let is_escapable = |char| matches!(char, EXPR_START | EXPR_END);
                    constant.push(self.parse_escape_sequence(is_escapable));
                }
                char => constant.push(self.consume(char)),
            }
        }

        Ok(constant)
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        let start_position = self.position;
        let start_offset = self.offset;

        self.consume(EXPR_START);

        let no_stdin = self.try_consume(NO_STDIN_MARKER);
        let raw_shell = self.try_consume(RAW_SHELL_MARKER);

        self.consume_whitespaces();

        let value = if raw_shell {
            ExpressionValue::RawShell(self.parse_raw_shell()?)
        } else {
            ExpressionValue::Pipeline(self.parse_pipeline()?)
        };

        if self.try_consume(EXPR_END) {
            Ok(Expression {
                no_stdin,
                value,
                raw_value: self.input[start_offset..self.offset].into(),
            })
        } else {
            Err(self.err_at(ErrorKind::MissingExprEnd, start_position))
        }
    }

    fn parse_raw_shell(&mut self) -> Result<String> {
        let mut command = String::new();

        while let Some(char) = self.peek() {
            match char {
                EXPR_START => return Err(self.err(ErrorKind::UnexpectedExprStart)),
                EXPR_END => break,
                char if char == self.escape => {
                    let is_escapable = |char| matches!(char, EXPR_START | EXPR_END);
                    command.push(self.parse_escape_sequence(is_escapable));
                }
                _ => command.push(self.consume(char)),
            }
        }

        if command.trim().is_empty() {
            Err(self.err(ErrorKind::EmptyShellCommand))
        } else {
            Ok(command)
        }
    }

    fn parse_pipeline(&mut self) -> Result<Vec<Command>> {
        let mut commands = Vec::new();
        let mut command_expected = false;

        while let Some(char) = self.peek() {
            match char {
                EXPR_START => return Err(self.err(ErrorKind::UnexpectedExprStart)),
                PIPE => return Err(self.err(ErrorKind::MissingCommandBefore)),
                EXPR_END => break,
                _ => {
                    commands.push(self.parse_command()?);
                    command_expected = self.try_consume(PIPE);

                    if command_expected {
                        self.consume_whitespaces();
                    };
                }
            }
        }

        if command_expected {
            Err(self.err(ErrorKind::MissingCommandAfter))
        } else {
            Ok(commands)
        }
    }

    fn parse_command(&mut self) -> Result<Command> {
        let external = self.try_consume(EXTERN_MARKER);
        let name = self.parse_arg()?;
        let mut args = Vec::new();

        self.consume_whitespaces();

        while let Some(char) = self.peek() {
            match char {
                EXPR_START | PIPE | EXPR_END => break,
                _ => {
                    args.push(self.parse_arg()?);
                    self.consume_whitespaces();
                }
            }
        }

        Ok(Command {
            name,
            args,
            external,
        })
    }

    fn parse_arg(&mut self) -> Result<String> {
        let mut arg = String::new();

        while let Some(char) = self.peek() {
            match char {
                EXPR_START | PIPE | EXPR_END => break,
                char if char.is_whitespace() => break,
                char @ (SINGLE_QUOTE | DOUBLE_QUOTE) => arg.push_str(&self.parse_quoted_arg(char)?),
                _ => arg.push_str(&self.parse_unquote_arg()),
            }
        }

        Ok(arg)
    }

    fn parse_quoted_arg(&mut self, quote: char) -> Result<String> {
        let start_position = self.position;
        let mut arg = String::new();

        self.consume(quote);

        while let Some(char) = self.peek() {
            if char == quote {
                self.consume(quote);
                return Ok(arg);
            } else if char == self.escape {
                arg.push(self.parse_escape_sequence(|char| char == quote));
            } else {
                arg.push(self.consume(char));
            }
        }

        Err(self.err_at(ErrorKind::MissingClosingQuote(quote), start_position))
    }

    fn parse_unquote_arg(&mut self) -> String {
        let mut arg = String::new();

        while let Some(char) = self.peek() {
            match char {
                EXPR_START | PIPE | EXPR_END | SINGLE_QUOTE | DOUBLE_QUOTE => break,
                char if char.is_whitespace() => break,
                char if char == self.escape => {
                    arg.push(self.parse_escape_sequence(|char| match char {
                        EXPR_START | PIPE | EXPR_END | SINGLE_QUOTE | DOUBLE_QUOTE => true,
                        char if char.is_whitespace() => true,
                        _ => false,
                    }));
                }
                char => arg.push(self.consume(char)),
            };
        }

        arg
    }

    fn parse_escape_sequence(&mut self, is_escapable: impl Fn(char) -> bool) -> char {
        self.consume(self.escape);

        match self.peek() {
            Some(ESCAPED_LF) => self.consume_as(ESCAPED_LF, '\n'),
            Some(ESCAPED_CR) => self.consume_as(ESCAPED_CR, '\r'),
            Some(ESCAPED_TAB) => self.consume_as(ESCAPED_TAB, '\t'),
            Some(char) if char == self.escape => self.consume(char),
            Some(char) if is_escapable(char) => self.consume(char),
            _ => self.escape, // If this is not a valid escape sequence, keep the escape character
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.iterator.peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        self.iterator.next().map(|char| {
            self.offset += char.len_utf8();
            self.position += 1;
            char
        })
    }

    fn try_consume(&mut self, expected: char) -> bool {
        if let Some(value) = self.peek() {
            if value == expected {
                self.consume(value);
                return true;
            }
        }
        false
    }

    fn consume(&mut self, expected: char) -> char {
        self.consume_as(expected, expected)
    }

    fn consume_as(&mut self, expected: char, result: char) -> char {
        match self.next() {
            Some(char) if char == expected => result,
            Some(char) => unreachable!("parser expected {:?} but got {:?}", expected, char),
            None => unreachable!("parser expected {:?} but got EOF", expected),
        }
    }

    fn consume_whitespaces(&mut self) {
        while let Some(char) = self.peek() {
            if char.is_whitespace() {
                self.consume(char);
            } else {
                break;
            }
        }
    }

    fn err(&self, kind: ErrorKind) -> Error {
        Error {
            input: self.input.clone(),
            kind,
            position: self.position,
        }
    }

    fn err_at(&self, kind: ErrorKind, position: usize) -> Error {
        Error {
            input: self.input.clone(),
            kind,
            position,
        }
    }
}

#[cfg(test)]
#[rustfmt::skip::attributes(case)]
mod tests {
    use super::*;
    use claims::assert_err;
    use claims::assert_ok;
    use rstest::rstest;
    use std::time::Duration;

    #[rstest]
    // Constants and expressions - Standalone
    #[case("",     "")]
    #[case("c1",   "c1")]
    #[case("{}",   "{}")]
    #[case("{  }", "{}")]
    // Constants and expressions - Composed
    #[case("{}c1{}",     "{}c1{}")]
    #[case("c1{}c2{}c3", "c1{}c2{}c3")]
    #[case("  c1  {  }  c2  {  }  c3  ", "  c1  {}  c2  {}  c3  ")]
    // Command with args - Short
    #[case("{n}",     "{`n`}")]
    #[case("{n a}",   "{`n` `a`}")]
    #[case("{n a b}", "{`n` `a` `b`}")]
    // Command with args - Long
    #[case("{name}",           "{`name`}")]
    #[case("{name arg}",       "{`name` `arg`}")]
    #[case("{name arg1 arg2}", "{`name` `arg1` `arg2`}")]
    // Command with args - External
    #[case("{!name}",             "{!`name`}")]
    #[case("{!name arg}",         "{!`name` `arg`}")]
    #[case("{!name arg1 arg2}",   "{!`name` `arg1` `arg2`}")]
    #[case("{!'name' arg1 arg2}", "{!`name` `arg1` `arg2`}")] // External marker
    #[case("{'!name' arg1 arg2}", "{`!name` `arg1` `arg2`}")] // ! is part for command name
    // Command pipelines
    #[case("{n1|n2}",                  "{`n1`|`n2`}")]
    #[case("{n1|n2|n3}",               "{`n1`|`n2`|`n3`}")]
    #[case("{n1|n2 a21|n3 a31 a32}",   "{`n1`|`n2` `a21`|`n3` `a31` `a32`}")]
    #[case("{!n1|n2 a21|!n3 a31 a32}", "{!`n1`|`n2` `a21`|!`n3` `a31` `a32`}")]
    // Complex patterns
    #[case(
        "c1{}c2{n1}c3{n2 a21 a22}c4{n3|n4 a41|n5 a51 a52}c5",
        "c1{}c2{`n1`}c3{`n2` `a21` `a22`}c4{`n3`|`n4` `a41`|`n5` `a51` `a52`}c5"
    )]
    #[case(
    "  c1  {}  c2  {  n1  }  c3  {  n2  a21  a22  }  c4  {  n3 |  n4  a41  |  n5  a51  a52  }  c5  ",
    "  c1  {}  c2  {`n1`}  c3  {`n2` `a21` `a22`}  c4  {`n3`|`n4` `a41`|`n5` `a51` `a52`}  c5  ",
    )]
    // Pipeline markers
    #[case("{:n1|n2}", "{:`n1`|`n2`}")]
    #[case("{: n1|n2}", "{:`n1`|`n2`}")]
    #[case("{ :n1|n2}", "{`:n1`|`n2`}")] // : is part for command name
    #[case("{ : n1|n2}", "{`:` `n1`|`n2`}")] // : is separate command
    #[case("{#n1|n2}", "{#`n1|n2`}")]
    #[case("{# n1|n2}", "{#`n1|n2`}")]
    #[case("{ #n1|n2}", "{`#n1`|`n2`}")] // # is part for command name
    #[case("{ # n1|n2}", "{`#` `n1`|`n2`}")] // # is separate command
    #[case("{:#n1|n2}", "{:#`n1|n2`}")]
    #[case("{:# n1|n2}", "{:#`n1|n2`}")]
    #[case("{ :#n1|n2}", "{`:#n1`|`n2`}")] // :# is part for command name
    #[case("{ :# n1|n2}", "{`:#` `n1`|`n2`}")] // :# is separate command
    #[case("{#:n1|n2}", "{#`:n1|n2`}")] // : is part of shell command
    #[case("{#: n1|n2}", "{#`: n1|n2`}")] // : is part of shell command
    #[case("{ #:n1|n2}", "{`#:n1`|`n2`}")] // #: is part for command name
    #[case("{ #: n1|n2}", "{`#:` `n1`|`n2`}")] // #: is separate command
    // Escaping - General
    #[case("%",  "%")] // No
    #[case("%%", "%")] // Yes
    #[case("%n", "\n")] // Yes
    #[case("%r", "\r")] // Yes
    #[case("%t", "\t")] // Yes
    // Escaping - Constants
    #[case("%}",  "}")] // Yes
    #[case("%{",  "{")] // Yes
    #[case("%'",  "%'")] // No
    #[case("%\"",  "%\"")] // No
    #[case("%|",  "%|")] // No
    #[case("%x",  "%x")] // No
    // Escaping - Unquoted arg
    #[case("{a% b}", "{`a b`}")] // Yes
    #[case("{a%'b}", "{`a'b`}")] // Yes
    #[case("{a%\"b}", "{`a\"b`}")] // Yes
    #[case("{a%|b}", "{`a|b`}")] // Yes
    #[case("{a%{b}", "{`a{b`}")] // Yes
    #[case("{a%}b}", "{`a}b`}")] // Yes
    #[case("{a%xb}", "{`a%xb`}")] // No
    // Escaping - Single quoted arg
    #[case("{'a%'b'}", "{`a'b`}")] // Yes
    #[case("{'a% b'}", "{`a% b`}")] // No
    #[case("{'a%\"b'}", "{`a%\"b`}")] // No
    #[case("{'a%|b'}", "{`a%|b`}")] // No
    #[case("{'a%{b'}", "{`a%{b`}")] // No
    #[case("{'a%}b'}", "{`a%}b`}")] // No
    #[case("{'a%xb'}", "{`a%xb`}")] // No
    // Escaped - Double quoted arg
    #[case("{\"a%\"b\"}", "{`a\"b`}")] // Yes
    #[case("{\"a% b\"}", "{`a% b`}")] // No
    #[case("{\"a%'b\"}", "{`a%'b`}")] // No
    #[case("{\"a%|b\"}", "{`a%|b`}")] // No
    #[case("{\"a%{b\"}", "{`a%{b`}")] // No
    #[case("{\"a%}b\"}", "{`a%}b`}")] // No
    #[case("{\"a%xb\"}", "{`a%xb`}")] // No
    // Escaping - Raw shell
    #[case("{#a% b}", "{#`a% b`}")] // No
    #[case("{#a%'b}", "{#`a%'b`}")] // No
    #[case("{#a%\"b}", "{#`a%\"b`}")] // No
    #[case("{#a%|b}", "{#`a%|b`}")] // No
    #[case("{#a%{b}", "{#`a{b`}")] // Yes
    #[case("{#a%}b}", "{#`a}b`}")] // Yes
    #[case("{#a%xb}", "{#`a%xb`}")] // No
    // Consecutive quoted joined args
    #[case("{a'b'\"c\"}", "{`abc`}")]
    #[case("{a\"c\"'b'}", "{`acb`}")]
    #[case("{'b'a\"c\"}", "{`bac`}")]
    #[case("{'b'\"c\"a}", "{`bca`}")]
    #[case("{\"c\"a'b'}", "{`cab`}")]
    #[case("{\"c\"'b'a}", "{`cba`}")]
    #[timeout(Duration::from_secs(1))] // To protect against possible infinite loops
    fn parse(#[case] input: &str, #[case] normalized: &str) {
        let pattern = assert_ok!(Pattern::parse(input, '%'));
        assert_eq!(pattern.to_string(), normalized);
    }

    #[rstest]
    #[case("{{",    1, ErrorKind::UnexpectedExprStart)]
    #[case("{a{",   2, ErrorKind::UnexpectedExprStart)] // Different condition than the one before
    #[case("{#a{",  3, ErrorKind::UnexpectedExprStart)] // Different condition for raw shell
    #[case("{|",    1, ErrorKind::MissingCommandBefore)]
    #[case("{a|",   3, ErrorKind::MissingCommandAfter)]
    #[case("{a|}",  3, ErrorKind::MissingCommandAfter)] // Different condition than the one before
    #[case("{'a",   1, ErrorKind::MissingClosingQuote('\''))]
    #[case("{\"a",  1, ErrorKind::MissingClosingQuote('"'))]
    #[case("}",     0, ErrorKind::MissingExprStart)]
    #[case("{",     0, ErrorKind::MissingExprEnd)]
    #[case("{#}",   2, ErrorKind::EmptyShellCommand)]
    #[case("{# }",  3, ErrorKind::EmptyShellCommand)]
    #[timeout(Duration::from_secs(1))] // To protect against possible infinite loops
    fn parse_err(#[case] input: &str, #[case] position: usize, #[case] kind: ErrorKind) {
        let error = assert_err!(Pattern::parse(input, '%'));
        let expected_error = Error {
            input: input.into(),
            position,
            kind,
        };
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case(0, "{n1}")]
    #[case(1, "{n2 a21}")]
    #[case(3, "{n3 a31 a32}")]
    fn raw_expr(#[case] position: usize, #[case] value: &str) {
        let pattern = assert_ok!(Pattern::parse("{n1}{n2 a21}_{n3 a31 a32}", '%'));

        match pattern.items().get(position) {
            Some(Item::Expression(expr)) => assert_eq!(expr.raw_value, value),
            _ => panic!("no expression at position {position}"),
        }
    }

    #[rstest]
    #[case("", "")]
    #[case("{}", "'{}'")]
    #[case("{}{}", "'{}''{}'")]
    #[case(" {} {} ", " '{}' '{}' ")]
    #[case(" {n1} {n2 a21 | n3 a31 a32} ", " '{`n1`}' '{`n2` `a21`|`n3` `a31` `a32`}' ")]
    fn quote(#[case] input: &str, #[case] normalized: &str) {
        let pattern = assert_ok!(Pattern::parse(input, '%'));
        assert_eq!(pattern.quote_expressions('\'').to_string(), normalized);
    }

    #[test]
    fn err_display() {
        let error = Error {
            input: "abc}".into(),
            position: 3,
            kind: ErrorKind::MissingExprStart,
        };
        let expected_result = format!(
            "pattern syntax error at position {YELLOW}3{RESET}\n\
             \n\
             abc}}\n   \
             {BOLD_RED}^{RESET}\n   \
             {BOLD_RED}missing opening {{{RESET}\n",
        );
        assert_eq!(error.to_string(), expected_result);
    }
}
