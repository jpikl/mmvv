use assert_cmd::crate_name;
use assert_cmd::Command;
use std::env;

#[test]
fn examples() {
    for meta in rew::commands::METAS {
        for example in meta.examples {
            // Such examples require coreutils with NUL separator support
            // which are not available on macOS by default.
            if cfg!(target_os = "macos") && example.has_null_arg() {
                continue;
            }

            println!("[{}] {}", meta.name, first_line(example.text));

            Command::cargo_bin(crate_name!())
                .unwrap()
                .env("SHELL", "sh") // Examples expect UNIX shell
                .arg(meta.name)
                .args(example.args)
                .write_stdin(join_lines(example.input, example.has_null_arg()))
                .assert()
                .success()
                .stdout(join_lines(example.output, example.has_null_arg()))
                .stderr("");
        }
    }
}

fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or_default()
}

fn join_lines(lines: &[&str], null_separator: bool) -> String {
    let separator = if null_separator { "\0" } else { "\n" };
    format!("{}{separator}", lines.join(separator))
}
