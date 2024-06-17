use crate::command::Adapter;
use crate::command::BaseArg;
use crate::command::NonEmpty;
use crate::command::OptionalArg;
use crate::command::PositionalArg;
use anyhow::format_err;
use anyhow::Result;
use rew::examples::Example;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub fn generate_summary(command: &Adapter<'_>, path: &Path) -> Result<()> {
    println!("Updating summary in {path:?}");

    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    let start_marker = "<!--[GENERATED_REFERENCE_START]-->";
    let end_marker = "<!--[GENERATED_REFERENCE_END]-->";

    let start_pos = match content.find(start_marker) {
        Some(pos) => pos + start_marker.len(),
        None => {
            return Err(format_err!(
                "'{}' did not contain '{}' marker",
                path.to_string_lossy(),
                start_marker
            ))
        }
    };

    let Some(end_pos) = content.find(end_marker) else {
        return Err(format_err!(
            "'{}' did not contain '{}' marker",
            path.to_string_lossy(),
            end_marker
        ));
    };

    let mut file = File::create(path)?;
    file.write_all(&content.as_bytes()[..start_pos])?;

    writeln!(&mut file)?;
    writeln!(&mut file)?;
    write_summary(&mut file, command)?;
    writeln!(&mut file)?;

    file.write_all(&content.as_bytes()[end_pos..])?;

    Ok(())
}

fn write_summary(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    writeln!(
        writer,
        "- [{}](./reference/{}.md)",
        command.full_name(),
        command.file_stem()
    )?;

    for subcommand in command.subcommands() {
        write_summary(writer, &subcommand)?;
    }

    Ok(())
}

pub fn generate_reference(command: &Adapter<'_>, dir: &Path) -> Result<()> {
    let path = dir.join(command.file_stem()).with_extension("md");
    println!("Generating {:?} reference to {path:?}", command.full_name());

    write_reference(&mut File::create(path)?, command)?;

    for subcommand in command.subcommands() {
        generate_reference(&subcommand, dir)?;
    }

    Ok(())
}

fn write_reference(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    writeln!(writer, "# {}", command.full_name())?;
    writeln!(writer)?;
    writeln!(writer, "{}", command.description()?)?;

    writeln!(writer)?;
    writeln!(writer, "## Usage")?;
    writeln!(writer)?;
    writeln!(writer, "```")?;
    write!(writer, "{}", command.full_name())?;

    for arg in command.synopsis_args() {
        if arg.is_required() {
            write!(writer, " <{}>", arg.name())?;
        } else {
            write!(writer, " [{}]", arg.name())?;
        }
        if arg.is_many() {
            write!(writer, "...")?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "```")?;

    for (group, subcommands) in command.grouped_subcommands() {
        writeln!(writer)?;
        writeln!(writer, "## {group}")?;

        if let Some(description) = group.description() {
            writeln!(writer)?;
            writeln!(writer, "{description}")?;
        }

        writeln!(writer)?;
        writeln!(writer, "<dl>")?;

        for subcommand in subcommands {
            write_subcommand(writer, &subcommand)?;
        }

        writeln!(writer, "</dl>")?;
    }

    if let Some(args) = command.pos_args().non_empty() {
        writeln!(writer)?;
        writeln!(writer, "## Arguments")?;
        writeln!(writer)?;
        writeln!(writer, "<dl>")?;

        for arg in args {
            write_pos_arg(writer, &arg)?;
        }

        writeln!(writer, "</dl>")?;
    }

    if let Some(args) = command.opt_args().non_empty() {
        writeln!(writer)?;
        writeln!(writer, "## Options")?;
        writeln!(writer)?;
        writeln!(writer, "<dl>")?;

        for arg in args {
            write_opt_arg(writer, &arg)?;
        }

        writeln!(writer, "</dl>")?;
    }

    if let Some(args) = command.global_opt_args().non_empty() {
        let args = args.collect::<Vec<_>>();

        writeln!(writer)?;
        writeln!(writer, "## Global options")?;

        if let Some(parent) = command.parent_with_args(&args) {
            writeln!(writer)?;
            writeln!(
                writer,
                "See [{} reference]({}.md#global-options) for list of additional global options.",
                parent.full_name(),
                parent.file_stem()
            )?;
        }

        if let Some(args) = command.own_args(&args).non_empty() {
            writeln!(writer)?;
            writeln!(writer, "<dl>")?;

            for arg in args {
                write_opt_arg(writer, arg)?;
            }

            writeln!(writer, "</dl>")?;
        };
    }

    if let Some(examples) = command.examples().non_empty() {
        writeln!(writer)?;
        writeln!(writer, "## Examples")?;

        for example in examples {
            write_example(writer, command, example)?;
        }
    }

    Ok(())
}

fn write_subcommand(writer: &mut impl Write, subcommands: &Adapter<'_>) -> Result<()> {
    writeln!(
        writer,
        "<dt><a href=\"{}.html\"><code>{}</code></a></dt>",
        subcommands.file_stem(),
        subcommands.name()
    )?;
    writeln!(writer, "<dd>{}</dd>", subcommands.short_description()?)?;
    Ok(())
}

fn write_pos_arg(writer: &mut impl Write, arg: &PositionalArg<'_>) -> Result<()> {
    write!(writer, "<dt><code>")?;

    if arg.is_required() {
        write!(writer, "&lt;{}&gt;", arg.name())?;
    } else {
        write!(writer, "[{}]", arg.name())?;
    }

    if arg.is_many() {
        write!(writer, "...")?;
    }

    writeln!(writer, "</code></dt>")?;
    write_arg(writer, arg.base())
}

fn write_opt_arg(writer: &mut impl Write, arg: &OptionalArg<'_>) -> Result<()> {
    writeln!(writer)?;
    write!(writer, "<dt><code>{}", arg.names().join(", "))?;

    for value_name in arg.value_names() {
        write!(writer, " &lt;{value_name}&gt;")?;
    }

    writeln!(writer, "</code></dt>")?;
    write_arg(writer, arg.base())
}

fn write_arg(writer: &mut impl Write, arg: &BaseArg<'_>) -> Result<()> {
    writeln!(writer, "<dd>")?;
    writeln!(writer)?; // MD markup in the first line after <dd> is not processed. Also, the space before looks better.
    writeln!(writer, "{}", arg.description()?)?;

    if let Some(possible_values) = arg.possible_values().non_empty() {
        writeln!(writer)?;
        writeln!(writer, "Possible values:")?;
        writeln!(writer)?;

        for value in possible_values {
            writeln!(writer, " - `{}` - {}", value.name(), value.description()?)?;
        }
    }

    if let Some(default_value) = arg.default_value() {
        writeln!(writer)?;
        writeln!(writer, "Default value: `{default_value}`")?;
    }

    if let Some(env_var) = arg.env_var() {
        writeln!(writer)?;
        writeln!(
            writer,
            "Can be also set using `{env_var}` environment variable."
        )?;
    }

    writeln!(writer, "</dd>")?;
    Ok(())
}

fn write_example(writer: &mut impl Write, command: &Adapter<'_>, example: &Example) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{}", example.text)?;
    writeln!(writer)?;

    writeln!(writer, "```sh")?;
    write!(writer, "{}", command.full_name())?;

    for arg in example.args {
        let arg = arg.replace('\'', "\"");

        if arg.contains(' ') || arg.contains('|') || arg.contains('\\') {
            write!(writer, " '{arg}'")?;
        } else {
            write!(writer, " {arg}")?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "```")?;

    if !example.input.is_empty() || !example.output.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "<div class=\"example-io\">")?;

        if !example.input.is_empty() {
            write_example_io(writer, "stdin", example.input, example.has_null_arg())?;
        }

        if !example.output.is_empty() {
            write_example_io(writer, "stdout", example.output, example.has_null_arg())?;
        }

        writeln!(writer, "</div>")?;
    }

    Ok(())
}

fn write_example_io(
    writer: &mut impl Write,
    title: &str,
    lines: &[&str],
    null_separator: bool,
) -> Result<()> {
    writeln!(writer, "<div class=\"example-io-stream\">")?;
    writeln!(writer, "<small><b>{title}:</b></small>")?;
    writeln!(writer, "<ul>")?;

    // We use <code> because mdbook cannot render empty `` which we need for empty IO lines.
    if null_separator {
        let line = format!("{}\\0", lines.join("\\0"));
        let line = normalize_line(&line);
        writeln!(writer, "<li><code>{line}</code></li>")?;
    } else {
        for line in lines {
            let line = normalize_line(line);
            writeln!(writer, "<li><code>{line}</code></li>")?;
        }
    }

    writeln!(writer, "</ul>")?;
    writeln!(writer, "</div>")?;
    Ok(())
}

fn normalize_line(line: &str) -> String {
    line.replace(' ', "\u{a0}") // Unbreakable spaces to prevent trimming by mdbook html renderer.
        .replace('\t', "\u{a0}\u{a0}\u{a0}\u{a0}")
}
