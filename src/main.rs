use anyhow::{Context, Result};
use std::io;
use std::io::Write;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if output.status.success() {
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        io::stdout().write_all(stdout.as_bytes())?;
        io::stderr().write_all(stderr.as_bytes())?;
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        let stderr = std::str::from_utf8(&output.stderr)?;
        io::stderr().write_all(stderr.as_bytes())?;
        std::process::exit(exit_code);
    }

    Ok(())
}
