use rew::app;
use rew::commands::get_meta;
use rew::error::Reporter;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let app = app::build();
    let reporter = Reporter::new(&app);

    let matches = match app.try_get_matches() {
        Ok(matches) => matches,
        // Only `--help` and `--version` produce exit code 0.
        Err(error) if error.exit_code() == 0 => {
            reporter.print_help(&error);
            return ExitCode::from(0);
        }
        Err(error) => {
            reporter.print_invalid_usage(&error);
            return ExitCode::from(2);
        }
    };

    let (cmd_name, cmd_matches) = matches.subcommand().expect("command not matched");
    let cmd = get_meta(cmd_name).expect("command not found");

    match cmd.run(cmd_matches) {
        Ok(()) => ExitCode::from(0),
        // Ignore broken pipe, this is common behavior for coreutils commands.
        Err(error) if is_broken_pipe(&error) => ExitCode::from(0),
        Err(error) => {
            reporter.print_error(&error);
            ExitCode::from(1)
        }
    }
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io_err| io_err.kind() == io::ErrorKind::BrokenPipe)
    })
}
