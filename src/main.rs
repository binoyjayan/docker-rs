use anyhow::Context;
use std::env;
use std::fs;
use std::os::unix::fs::chroot;
use tempfile::tempdir;

mod docker;

use docker::pull_image;

const DOCKER_AUTH: &str = "auth.docker.io";
const DOCKER_REGISTRY: &str = "registry.docker.io";
const DOCKER_HUB: &str = "registry.hub.docker.com";

fn main() -> anyhow::Result<()> {
    let docker_auth = env::var("DOCKER_AUTH").unwrap_or(DOCKER_AUTH.to_string());
    let docker_registry = env::var("DOCKER_REGISTRY").unwrap_or(DOCKER_REGISTRY.to_string());
    let docker_hub = env::var("DOCKER_HUB").unwrap_or(DOCKER_HUB.to_string());

    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let temp_dir = tempdir().context("Failed to create temporary directory")?;
    let chroot_dir = temp_dir.path();
    let chroot_dir_str = chroot_dir
        .to_str()
        .context("Failed to convert path to string")?;

    pull_image(
        &docker_auth,
        &docker_registry,
        &docker_hub,
        "alpine:latest",
        chroot_dir_str,
    )?;

    fs::create_dir_all(chroot_dir.join("usr/local/bin/")).context("failed to create /bin")?;
    fs::create_dir_all(chroot_dir.join("dev/null")).context("failed to create /dev/null")?;
    let dest = chroot_dir.join(
        command
            .strip_prefix("/")
            .ok_or(anyhow::anyhow!("Failed to strip prefix"))?,
    );

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
