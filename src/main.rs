use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::chroot;
use tempfile::tempdir;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    let chroot_dir = temp_dir.path();
    fs::create_dir_all(chroot_dir.join("usr/local/bin/")).context("failed to create /bin")?;
    fs::create_dir_all(chroot_dir.join("dev/null")).context("failed to create /dev/null")?;
    let dest = chroot_dir.join(command.strip_prefix("/").unwrap());
    // Copy the 'docker-explorer' binary passed as 'args[3]'
    std::fs::copy(command, dest)?;

    // Execute as PID 1
    nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWPID)
        .context("Failed to unshare PID namespace")?;
    chroot(chroot_dir).context("Failed to chroot")?;

    let output = std::process::Command::new(command)
        .args(command_args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let exit_code = output.status.code().unwrap_or(1);
    std::process::exit(exit_code);
}
