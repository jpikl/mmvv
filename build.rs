use std::process::Command;
use std::process::Stdio;

pub fn main() {
    let git_ok = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |status| status.success());

    if git_ok {
        let stdout = Command::new("git")
            .arg("log")
            .arg("--max-count=1")
            .arg("--format=%h %cd")
            .arg("--abbrev=10")
            .arg("--date=short")
            .output()
            .map(|output| String::from_utf8(output.stdout))
            .unwrap()
            .unwrap();

        println!("cargo:rustc-env=BUILD_COMMIT={stdout}");
    }
}
