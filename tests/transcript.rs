use assert_cmd::crate_name;
use std::env;
use std::path::Path;
use term_transcript::svg::NamedPalette;
use term_transcript::svg::Template;
use term_transcript::svg::TemplateOptions;
use term_transcript::test::MatchKind;
use term_transcript::test::TestConfig;
use term_transcript::ExitStatus;
use term_transcript::ShellOptions;

#[test]
fn help() {
    // We want to check custom help formatting done by `Colorizer`
    transcript("help", "seq --help");
}

#[test]
fn invalid_usage() {
    transcript("invalid_usage", "--unkown");
}

#[test]
fn error() {
    // Use double quotes `""` because `''` do not work on Windows `cmd`
    transcript("error", "x \"{cat --unknown}\"");
}

#[test]
fn examples() {
    transcript("examples", "x --examples");
}

#[cfg(target_family = "unix")]
const STATUS_COMMAND: &str = "echo $?";
#[cfg(target_family = "windows")]
const STATUS_COMMAND: &str = "echo %errorlevel%";

fn transcript(snapshot: &str, args: &str) {
    let shell_options = ShellOptions::default()
        .with_cargo_path()
        .with_env("CLICOLOR_FORCE", "1")
        .with_current_dir(env!("CARGO_MANIFEST_DIR"))
        .with_status_check(STATUS_COMMAND, |output| {
            let response = output.to_plaintext().ok()?;
            response.trim().parse().ok().map(ExitStatus)
        });

    let template_options = TemplateOptions {
        palette: NamedPalette::Xterm.into(),
        ..TemplateOptions::default()
    };

    let path = Path::new("snapshots").join(snapshot).with_extension("svg");
    let input = format!("{} {args}", crate_name!());

    TestConfig::new(shell_options)
        .with_match_kind(MatchKind::Precise)
        .with_template(Template::new(template_options))
        .test(path, [input.as_ref()]);
}
