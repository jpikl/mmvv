use super::get_meta;
use crate::args::get_spawned_by;
use crate::args::ENV_BUF_MODE;
use crate::args::ENV_BUF_SIZE;
use crate::args::ENV_NULL;
use crate::args::ENV_SPAWNED_BY;
use crate::colors::RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::commands::cat;
use crate::examples;
use crate::io::LineReader;
use crate::pattern;
use crate::pattern::Expression;
use crate::pattern::ExpressionValue;
use crate::pattern::Item;
use crate::pattern::Pattern;
use crate::pattern::SimpleItem;
use crate::pattern::SimplePattern;
use crate::shell::Shell;
use crate::stdbuf::StdBuf;
use anyhow::Context as AnyhowContext;
use anyhow::Error;
use anyhow::Result;
use bstr::ByteVec;
use clap::crate_name;
use clap::ArgAction;
use std::env;
use std::env::current_exe;
use std::io;
use std::io::Write;
use std::panic::resume_unwind;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::result;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
    examples: examples! [
        "Empty expression `{}` is replaced by input line.": {
            args: &["Hello {}"],
            input: &["first", "second", "third"],
            output: &["Hello first", "Hello second", "Hello third"],
        },
        "Expressions can call other `rew` commands to process the input.\n\n\
         Here, we call the `rew upper` command which converts text to uppercase.": {
            args: &["Hello {upper}"],
            input: &["first", "second", "third"],
            output: &["Hello FIRST", "Hello SECOND", "Hello THIRD"],
        },
        "Expressions can also call any external command.\n\n\
         Let's remove all `aeiou` characters from text using `tr`.": {
            args: &["Hello {tr -d aeiou}"],
            input: &["first", "second", "third"],
            output: &["Hello frst", "Hello scnd", "Hello thrd"],
        },
        "Multiple commands can be joined into a pipeline.": {
            args: &["Hello {tr -d aeiou | upper}"],
            input: &["first", "second", "third"],
            output: &["Hello FRST", "Hello SCND", "Hello THRD"],
        },
        "Multiple expressions are run in parallel and their output is combined. \
         The excution runs until one of the expressions no longer produces any output.": {
            args: &["{seq}. {tr -d aeiou | upper}"],
            input: &["first", "second", "third"],
            output: &["1. FRST", "2. SCND", "3. THRD"],
        },
        "Arguments containing whitepaces must be wrapped in single `''` or double quotes `\"\"`.\n\n\
         Here, we replace `aeiou` characters with space `' '`.": {
            args: &["Hello {tr aeiou ' ' | upper}"],
            input: &["first", "second", "third"],
            output: &["Hello F RST", "Hello S C ND", "Hello TH RD"],
        },
        "The `!` marker denotes an external command.\n\n\
        Let's call the standard `seq` command instead of the built-in `rew seq`.": {
           args: &["{!seq 1 3}. {}"],
           input: &["first", "second", "third"],
           output: &["1. first", "2. second", "3. third"],
       },
       "The `#` marker denotes \"raw shell expression\". \
        Everything after it will be interpreted by the current shell.\n\n\
        For example, the following expression is equivalent to `{sh -c 'printf \"%s\\n\" a b c'}`": {
            args: &["{# printf '%s\\n' a b c}. {}"],
            input: &["first", "second", "third"],
            output: &["a. first", "b. second", "c. third"],
        },
        "A specific shell for `{# ...}` can be set using the `-s, --shell` option or the `SHELL` environment variable.": {
            args: &["-s", "bash", "{# for((i=0;i<3;i++)); do echo $i; done}. {}"],
            input: &["first", "second", "third"],
            output: &["0. first", "1. second", "2. third"],
        },
       "The `:` marker is a hint that an expression does not consume stdin. \
        Without it, the overall execution might get stuck forever due to blocked IO calls.\n\n\
        Only external commands need `:` to be explicitely specified. \
        For built-in commands, `:` is detected automatically.": {
            args: &["{seq 1..3} {: !seq 1 3} {:# echo 1; echo 2; echo 3}"],
            input: &[],
            output: &["1 1 1", "2 2 2", "3 3 3"],
        },
        "Backslash `\\` can be used to escape special characters": {
            args: &["\\{ \"{}\": {seq} \\}"],
            input: &["first", "second", "third"],
            output: &["{ \"first\": 1 }", "{ \"second\": 2 }", "{ \"third\": 3 }"],
        },
        "A custom escape character can be set using the `-e, --escape` option.": {
            args: &["-e%", "%{ \"{}\": {seq} %}"],
            input: &["first", "second", "third"],
            output: &["{ \"first\": 1 }", "{ \"second\": 2 }", "{ \"third\": 3 }"],
        },
        "Certain special characters like `|` must be escaped only within a specific context.": {
            args: &["| {echo \"|\"} {echo \\|}"],
            input: &[],
            output: &["| | |"],
        },
        "Escape character can be also used to produce line feed `\\n`, carriage return `\\r` or tab `\\t`.": {
            args: &["{seq}:\\n\\t{}"],
            input: &["first", "second", "third"],
            output: &["1:", "\tfirst", "2:", "\tsecond", "3:", "\tthird"],
        },
        "You can enable automatic expression quoting using `-q, --quote` flag.": {
            args: &["-q", "mv {} {lower | tr ' ' '_'}"],
            input: &["IMG 1.jpg", "IMG 2.jpg"],
            output: &["mv 'IMG 1.jpg' 'img_1.jpg'", "mv 'IMG 2.jpg' 'img_2.jpg'"],
        },
        "Double the `-q, --quote` to use double quotes instead of single quotes.": {
            args: &["-qq", "mv {} {lower | tr ' ' '_'}"],
            input: &["IMG 1.jpg", "IMG 2.jpg"],
            output: &["mv \"IMG 1.jpg\" \"img_1.jpg\"", "mv \"IMG 2.jpg\" \"img_2.jpg\""],
        },
        "All global options `-0, --null`, `--buf-size` and `--buf-mode` are propagated to rew subcommands. \
         Do not forget configure NUL separator manually for any external commands.": {
            args: &["--null", "{upper | sed --null-data 's/^.//g'}"],
            input: &["aa", "bb", "cc"],
            output: &["A", "B", "C"],
        },
    ],
};

/// Compose parallel shell pipelines using a pattern.
#[derive(clap::Args)]
struct Args {
    /// Output pattern(s).
    ///
    /// A pattern describes how to transform each input line into output.
    /// Multiple patterns are joined together, using space `' '` as a delimiter.
    ///
    /// See examples (`--examples` option) for more details.
    #[arg(required = true)]
    pattern: Vec<String>,

    /// Escape character for the pattern.
    #[arg(short, long, value_name = "CHAR", default_value_t = '\\')]
    escape: char,

    /// Shell used to evaluate `{# ...}` expressions.
    ///
    /// Default value: `cmd` on Windows, `sh` everywhere else.
    #[arg(short, long, env = "SHELL")]
    shell: Option<String>,

    /// Wrap output of every pattern expression in quotes
    ///
    /// Use the flag once for single quotes `''` or twice for double quotes `""`.
    #[clap(short, long, action = ArgAction::Count)]
    pub quote: u8,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let raw_pattern = args.pattern.join(" ");
    let mut pattern = Pattern::parse(&raw_pattern, args.escape)?;

    if args.quote > 0 {
        let quote = if args.quote > 1 { '"' } else { '\'' };
        pattern = pattern.quote_expressions(quote);
    }

    if let Some(pattern) = pattern.try_simplify() {
        return eval_simple_pattern(context, &pattern);
    }

    eval_pattern(context, &pattern, args.shell.as_deref())
}

fn eval_simple_pattern(context: &Context, pattern: &SimplePattern) -> Result<()> {
    let mut reader = context.line_reader();
    let mut writer = context.writer();

    while let Some(line) = reader.read_line()? {
        for item in pattern.items() {
            match item {
                SimpleItem::Constant(value) => writer.write(value.as_bytes())?,
                SimpleItem::Expression => writer.write(line)?,
            }
        }
        writer.write_separator()?;
    }

    Ok(())
}

fn eval_pattern(context: &Context, pattern: &Pattern, shell: Option<&str>) -> Result<()> {
    let mut builder = CommandBuilder::new(context, shell);
    let mut children = Vec::new();
    let mut items = Vec::new();
    let mut stdins = Vec::new();

    // Build and spawn child process pipelines.
    for item in pattern.items() {
        match &item {
            Item::Constant(value) => items.push(EvalItem::Constant(value.clone())),
            Item::Expression(ref expression) => {
                let pipeline = builder.build_expression(expression)?;

                for child in pipeline.children {
                    children.push(child);
                }

                items.push(EvalItem::Reader(
                    pipeline
                        .stdout
                        .map(|stdout| context.line_reader_from(stdout)),
                ));

                if pipeline.stdin.is_some() {
                    stdins.push(pipeline.stdin);
                }
            }
        }
    }

    // "reader" thread which forwards main process stdin to every child process.
    let thread_context = context.clone();
    let thread: JoinHandle<Result<()>> = thread::spawn(move || {
        if stdins.iter().all(Option::is_none) {
            return Ok(()); // None of the child proceses use stdin.
        }

        let mut reader = thread_context.byte_chunk_reader();

        while let Some(chunk) = reader.read_chunk()? {
            for stdin in &mut stdins {
                if let Some(writer) = stdin {
                    if !writer.write_all(chunk)? {
                        // Could not write to child process stdin because it ended.
                        // Do not end the whole thread yet, keep writing to the other running child processes.
                        stdin.take();
                    }
                }
            }

            if stdins.iter().all(Option::is_none) {
                break; // Stdin of every child process was closed.
            }
        }

        Ok(())
    });

    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    // Combine output from stdout of every child process.
    'outer: loop {
        for item in &mut items {
            match item {
                EvalItem::Constant(value) => buffer.push_str(value),
                EvalItem::Reader(reader) => {
                    if let Some(line) = reader.read_line()? {
                        buffer.push_str(line);
                    } else {
                        break 'outer; // Quit as soon as one of child processes ends.
                    }
                }
            }
        }
        writer.write_line(&buffer)?;
        buffer.clear();
    }

    let mut all_finished = true;

    // Make sure all child processes are terminated.
    // This will cause the "reader" thread to end by detecting "broken pipe" errors everywhere.
    for child in &mut children {
        if !child.try_wait()? {
            all_finished = false;
        }
    }

    if !all_finished {
        // Give the remaining child processes some extra time to finish.
        // Needed especialy in case program exists with error on Windows.
        thread::sleep(Duration::from_millis(100));

        // Just kill the ones which did not terminate on their own.
        for child in &mut children {
            if !child.try_wait()? {
                child.kill()?;
            }
        }
    }

    // Try to wait for the "reader" thread to finish (non-blockingly).
    if thread.is_finished() {
        thread.join().map_err(resume_unwind)??;
    }

    // At this moment, the "reader" thread is blocked on read from stdin.
    // There is no way how to interrupt it, so we just let the thread die alongside the main process.
    // Reimplementing this with async Rust is probably not worth the effort, because:
    // 1. It only happens during interactive usage when stdin is TTY.
    // 2. And all process pipelines must contain at least one process which does not read from stdin.
    Ok(())
}

enum EvalItem {
    Constant(String),
    Reader(Eval<LineReader<ChildStdout>>),
}

struct Eval<T> {
    inner: T,
    context: EvalContext,
}

impl<T> Eval<T> {
    fn map<R>(self, map: impl FnOnce(T) -> R) -> Eval<R> {
        Eval {
            inner: map(self.inner),
            context: self.context,
        }
    }

    fn clone_as<R>(&self, inner: R) -> Eval<R> {
        Eval {
            inner,
            context: self.context.clone(),
        }
    }
}

#[derive(Clone)]
struct EvalContext {
    raw_command: Option<String>,
    raw_expr: Option<String>,
}

impl EvalContext {
    fn from_command(command: &Command) -> Self {
        Self {
            raw_command: Some(format!("{command:?}")),
            raw_expr: None,
        }
    }

    fn from_expr(expr: &Expression) -> Self {
        Self {
            raw_command: None,
            raw_expr: Some(expr.raw_value.clone()),
        }
    }

    fn merge(&mut self, other: &EvalContext) {
        if other.raw_command.is_some() {
            self.raw_command = other.raw_command.clone();
        }
        if other.raw_expr.is_some() {
            self.raw_expr = other.raw_expr.clone();
        }
    }

    fn apply(&self, err: impl Into<anyhow::Error>, message: &str) -> anyhow::Error {
        let mut err = err.into();

        if let Some(raw_command) = &self.raw_command {
            err = err.context(format!("{message} command {YELLOW}{raw_command}{RESET}"));
        }
        if let Some(raw_expr) = &self.raw_expr {
            err = err.context(format!("{message} expression {YELLOW}{raw_expr}{RESET}"));
        }

        err
    }
}

trait WithEvalContext<T> {
    fn with_eval_context(self, context: &EvalContext, message: &str) -> Result<T>;
}

impl<T, E: Into<anyhow::Error>> WithEvalContext<T> for result::Result<T, E> {
    fn with_eval_context(self, context: &EvalContext, message: &str) -> Result<T> {
        self.map_err(|err| context.apply(err, message))
    }
}

trait SpawnWithContext {
    fn spawn_with_context(&mut self) -> Result<Eval<Child>>;
}

impl SpawnWithContext for Command {
    fn spawn_with_context(&mut self) -> Result<Eval<Child>> {
        let context = EvalContext::from_command(self);
        let inner = self
            .spawn()
            .with_eval_context(&context, "could not spawn process for")?;
        Ok(Eval { inner, context })
    }
}

struct EvalPipeline {
    stdin: Option<Eval<ChildStdin>>,
    stdout: Eval<ChildStdout>,
    children: Vec<Eval<Child>>,
}

impl EvalPipeline {
    fn merge_context(&mut self, context: &EvalContext) {
        if let Some(stdin) = &mut self.stdin {
            stdin.context.merge(context);
        }

        for child in &mut self.children {
            child.context.merge(context);
        }

        self.stdout.context.merge(context);
    }
}

impl Eval<ChildStdin> {
    fn write_all(&mut self, buf: &[u8]) -> Result<bool> {
        match self.inner.write_all(buf) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(false),
            Err(err) => Err(self.context.apply(err, "could not write to")),
        }
    }
}

impl Eval<LineReader<ChildStdout>> {
    fn read_line(&mut self) -> Result<Option<&[u8]>> {
        self.inner
            .read_line()
            .with_eval_context(&self.context, "could not read from")
    }
}

impl Eval<Child> {
    fn take_stdin(&mut self) -> Option<Eval<ChildStdin>> {
        self.inner.stdin.take().map(|stdin| self.clone_as(stdin))
    }

    fn take_stdout(&mut self) -> Option<Eval<ChildStdout>> {
        self.inner.stdout.take().map(|stdout| self.clone_as(stdout))
    }

    fn try_wait(&mut self) -> Result<bool> {
        let result = match self.inner.try_wait() {
            Ok(None) => Ok(false),
            Ok(Some(status)) if status.success() => Ok(true),
            Ok(Some(status)) => Err(Error::msg(format!(
                "child process exited with code {RED}{}{RESET}",
                status.code().unwrap_or_default(),
            ))),
            Err(err) => Err(err.into()),
        };
        result.with_eval_context(&self.context, "failed execution of")
    }

    fn kill(&mut self) -> Result<()> {
        self.inner
            .kill()
            .with_eval_context(&self.context, "could not kill")
    }
}

struct CommandBuilder<'a> {
    context: &'a Context,
    shell: Shell,
    stdbuf: StdBuf,
}

impl<'a> CommandBuilder<'a> {
    fn new(context: &'a Context, shell: Option<&'a str>) -> Self {
        Self {
            context,
            shell: shell.map(Shell::new).unwrap_or_default(),
            stdbuf: StdBuf::default(),
        }
    }

    fn build_expression(&mut self, expr: &Expression) -> Result<EvalPipeline> {
        let context = EvalContext::from_expr(expr);

        let result = match &expr.value {
            ExpressionValue::RawShell(command) => self.build_raw_shell(command, expr.no_stdin),
            ExpressionValue::Pipeline(commands) => self.build_pipeline(commands, expr.no_stdin),
        };

        result
            .with_eval_context(&context, "could not initialize")
            .map(|mut pipeline| {
                pipeline.merge_context(&context);
                pipeline
            })
    }

    fn build_raw_shell(&self, shell_command: &str, no_stdin: bool) -> Result<EvalPipeline> {
        let mut command = self.shell.build_command(shell_command);
        command.stdout(Stdio::piped());

        if no_stdin {
            command.stdin(Stdio::null());
        } else {
            command.stdin(Stdio::piped());
        }

        let mut child = command.spawn_with_context()?;

        let stdin = child.take_stdin();
        let stdout = child
            .take_stdout()
            .expect("raw shell child process should have stdout");

        Ok(EvalPipeline {
            stdin,
            stdout,
            children: vec![child],
        })
    }

    fn build_pipeline(
        &mut self,
        commands: &[pattern::Command],
        mut no_stdin: bool,
    ) -> Result<EvalPipeline> {
        let mut children = Vec::new();
        let mut stdin = None;
        let mut stdout: Option<Eval<ChildStdout>> = None;

        for params in commands {
            let (mut command, group) = self.build_command(params)?;

            if group == Group::Generators {
                command.stdin(Stdio::null());
                no_stdin = true;
            } else if let Some(stdout) = stdout {
                command.stdin(Stdio::from(stdout.inner));
            } else {
                command.stdin(Stdio::piped());
            }

            command.stdout(Stdio::piped());

            let mut child = command.spawn_with_context()?;

            if no_stdin {
                stdin = None;
            } else if stdin.is_none() {
                stdin = child.take_stdin(); // The first process in pipeline
            }

            stdout = child.take_stdout();
            children.push(child);
        }

        if stdout.is_none() {
            let mut command = self.default_internal_command()?;

            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn_with_context()?;

            stdin = child.take_stdin();
            stdout = child.take_stdout();
        }

        let stdin = stdin.take();
        let stdout = stdout
            .take()
            .expect("pipeline child process should have stdout");

        Ok(EvalPipeline {
            stdin,
            stdout,
            children,
        })
    }

    fn build_command(&mut self, params: &pattern::Command) -> Result<(Command, Group)> {
        let pattern::Command {
            name,
            args,
            external,
        } = params;

        if !external {
            if let Some(meta) = get_meta(name) {
                let command = self.build_internal_command(Some(name), args)?;
                return Ok((command, meta.group));
            }

            if name == crate_name!() {
                if let Some((name, args)) = args.split_first() {
                    if let Some(meta) = get_meta(name) {
                        let command = self.build_internal_command(Some(name), args)?;
                        return Ok((command, meta.group));
                    }
                }

                let command = self.build_internal_command(None, args)?;
                return Ok((command, Group::Transformers));
            }
        }

        let mut command = Command::new(name);
        command.args(args);

        if self.context.buf_mode().is_line() {
            command.envs(self.stdbuf.line_buf_envs()); // libc based programs
            command.env("PYTHONUNBUFFERED", "1"); // Python programs
        }

        Ok((command, Group::Transformers))
    }

    fn build_internal_command(&self, name: Option<&str>, args: &[String]) -> Result<Command> {
        let program = current_exe().context("could not detect current executable")?;
        let mut command = Command::new(program);

        command.env(ENV_NULL, self.context.separator().is_null().to_string());
        command.env(ENV_BUF_MODE, self.context.buf_mode().to_string());
        command.env(ENV_BUF_SIZE, self.context.buf_size().to_string());
        command.env(ENV_SPAWNED_BY, get_spawned_by(META.name));

        if let Some(name) = name {
            command.arg(name);
        }

        command.args(args);
        Ok(command)
    }

    fn default_internal_command(&self) -> Result<Command> {
        self.build_internal_command(Some(cat::META.name), &[])
    }
}
