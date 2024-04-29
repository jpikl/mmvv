use super::get_meta;
use crate::args::get_spawned_by;
use crate::args::ENV_BUF_MODE;
use crate::args::ENV_BUF_SIZE;
use crate::args::ENV_NULL;
use crate::args::ENV_SPAWNED_BY;
use crate::colors::RESET;
use crate::colors::YELLOW;
use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::commands::cat;
use crate::examples::Example;
use crate::io::LineReader;
use crate::pattern;
use crate::pattern::Expression;
use crate::pattern::ExpressionValue;
use crate::pattern::Item;
use crate::pattern::Pattern;
use crate::pattern::SimpleItem;
use crate::pattern::SimplePattern;
use crate::process::Pipeline;
use crate::process::StdinMode;
use crate::shell::Shell;
use crate::spawn::Spawned;
use crate::stdbuf::StdBuf;
use anyhow::Context as _;
use anyhow::Result;
use bstr::ByteVec;
use clap::crate_name;
use clap::ArgAction;
use std::env;
use std::env::current_exe;
use std::panic::resume_unwind;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
    examples: EXAMPLES,
};

const EXAMPLES: &[Example] = command_examples! [
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
];

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
    #[arg(short, long, env = "SHELL", default_value_t = Shell::default(), hide_default_value = true)]
    shell: Shell,

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
        eval_simple_pattern(context, &pattern)
    } else {
        eval_pattern(context, &pattern, &args.shell)
    }
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

fn eval_pattern(context: &Context, pattern: &Pattern, shell: &Shell) -> Result<()> {
    let mut stdbuf = StdBuf::default();
    let mut children = Vec::new();
    let mut consumers = Vec::new();
    let mut producers = Vec::new();

    for item in pattern.items() {
        match &item {
            Item::Constant(value) => producers.push(Producer::Constant(value.clone())),
            Item::Expression(ref expr) => {
                let pipeline = build_pipeline(context, &mut stdbuf, shell, expr)?;

                for child in pipeline.children {
                    children.push(child);
                }

                if let Some(stdout) = pipeline.stdout {
                    let reader = stdout.map(|inner| context.line_reader_from(inner));
                    producers.push(Producer::Child(reader));
                }

                if pipeline.stdin.is_some() {
                    consumers.push(pipeline.stdin);
                }
            }
        }
    }

    // Helper thead forwards main process stdin to every child process.
    let thread_context = context.clone();
    let thread = thread::spawn(move || forward_input(&thread_context, consumers));

    // Main thread collects output from stdout of every child process.
    collect_output(context, producers)?;
    wait_children(children)?;

    if thread.is_finished() {
        // Join the thread only if it actually endeded.
        // Otherwise, this would be stucked forewer!
        thread.join().map_err(resume_unwind)?
    } else {
        // The helper thread is blocked on read from stdin.
        // There is no way how to interrupt it, so we just let the thread die alongside the main process.
        // Reimplementing this with async Rust is probably not worth the effort, because:
        // 1. It only happens during interactive usage when stdin is TTY.
        // 2. And all process pipelines must contain at least one process which does not read from stdin.
        Ok(())
    }
}

enum Producer {
    Constant(String),
    Child(Spawned<LineReader<ChildStdout>>),
}

fn forward_input(context: &Context, mut stdins: Vec<Option<Spawned<ChildStdin>>>) -> Result<()> {
    if stdins.iter().all(Option::is_none) {
        return Ok(()); // None of the child proceses use stdin.
    }

    let mut reader = context.byte_chunk_reader();

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
}

fn collect_output(context: &Context, mut producers: Vec<Producer>) -> Result<()> {
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    // Combine output from stdout of every child process.
    loop {
        for producer in &mut producers {
            match producer {
                Producer::Constant(value) => buffer.push_str(value),
                Producer::Child(reader) => {
                    if let Some(line) = reader.read_line()? {
                        buffer.push_str(line);
                    } else {
                        return Ok(()); // Quit as soon as one of child processes ends.
                    }
                }
            }
        }
        writer.write_line(&buffer)?;
        buffer.clear();
    }
}

fn wait_children(mut children: Vec<Spawned<Child>>) -> Result<()> {
    let mut all_finished = true;

    // Make sure all child processes are terminated.
    // This will cause the "reader" thread to end by detecting "broken pipe" errors everywhere.
    for child in &mut children {
        if !child.try_wait()? {
            all_finished = false;
        }
    }

    if all_finished {
        return Ok(());
    }

    // Give the remaining child processes some extra time to finish.
    // Needed especialy in case program exists with error on Windows.
    thread::sleep(Duration::from_millis(100));

    // Just kill the ones which did not terminate on their own.
    for child in &mut children {
        if !child.try_wait()? {
            child.kill()?;
        }
    }

    Ok(())
}

fn build_pipeline(
    context: &Context,
    stdbuf: &mut StdBuf,
    shell: &Shell,
    expr: &Expression,
) -> Result<Pipeline> {
    let raw_expr = format!("{YELLOW}{}{RESET}", expr.raw_value);

    match build_pipeline_internal(context, stdbuf, shell, expr) {
        Ok(pipeline) => Ok(pipeline.context(format!("expression: {raw_expr}"))),
        Err(err) => Err(err.context(format!("failed to initialize expression {raw_expr}"))),
    }
}

fn build_pipeline_internal(
    context: &Context,
    stdbuf: &mut StdBuf,
    shell: &Shell,
    expr: &Expression,
) -> Result<Pipeline> {
    let mut pipeline = Pipeline::new(expr.stdin_mode);

    match &expr.value {
        ExpressionValue::RawShell(command) => {
            let command = shell.build_command(command);
            pipeline = pipeline.command(command, expr.stdin_mode)?;
        }
        ExpressionValue::Pipeline(commands) => {
            for command in commands {
                let (command, stdin_mode) = build_command(context, stdbuf, command)?;
                pipeline = pipeline.command(command, stdin_mode)?;
            }
            if pipeline.is_empty() {
                let command = build_default_command(context)?;
                pipeline = pipeline.command(command, StdinMode::Connected)?;
            }
        }
    };

    Ok(pipeline)
}

fn build_command(
    context: &Context,
    stdbuf: &mut StdBuf,
    params: &pattern::Command,
) -> Result<(Command, StdinMode)> {
    let pattern::Command {
        name,
        args,
        external,
    } = params;

    if !external {
        if let Some(meta) = get_meta(name) {
            let command = build_internal_command(context, Some(name), args)?;
            return Ok((command, meta.group.stdin_mode()));
        }

        if name == crate_name!() {
            if let Some((name, args)) = args.split_first() {
                if let Some(meta) = get_meta(name) {
                    let command = build_internal_command(context, Some(name), args)?;
                    return Ok((command, meta.group.stdin_mode()));
                }
            }

            let command = build_internal_command(context, None, args)?;
            return Ok((command, StdinMode::Connected));
        }
    }

    let mut command = Command::new(name);
    command.args(args);

    if context.buf_mode().is_line() {
        command.envs(stdbuf.line_buf_envs()); // libc based programs
        command.env("PYTHONUNBUFFERED", "1"); // Python programs
    }

    Ok((command, StdinMode::Connected))
}

fn build_internal_command(
    context: &Context,
    name: Option<&str>,
    args: &[String],
) -> Result<Command> {
    let program = current_exe().context("could not detect current executable")?;
    let mut command = Command::new(program);

    command.env(ENV_NULL, context.separator().is_null().to_string());
    command.env(ENV_BUF_MODE, context.buf_mode().to_string());
    command.env(ENV_BUF_SIZE, context.buf_size().to_string());
    command.env(ENV_SPAWNED_BY, get_spawned_by(META.name));

    if let Some(name) = name {
        command.arg(name);
    }

    command.args(args);
    Ok(command)
}

fn build_default_command(context: &Context) -> Result<Command> {
    build_internal_command(context, Some(cat::META.name), &[])
}
