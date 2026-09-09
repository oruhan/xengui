use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::{cli::CargoArgs, workspace::Workspace};

const DEFAULT_RUN_PACKAGE: &str = "xengui_website";

pub fn run(workspace: &Workspace, command: &str, args: CargoArgs) -> Result<()> {
    let mut cargo = Command::new("cargo");
    cargo.arg(command).current_dir(&workspace.root);

    if let Some(package) = args.package {
        cargo.args(["--package", &package]);
    } else if command == "run" {
        cargo.args(["--package", DEFAULT_RUN_PACKAGE]);
    } else if command != "run" {
        cargo.arg("--workspace");
    }
    if args.release {
        cargo.arg("--release");
    }
    cargo.args(args.cargo_args);
    run_command(cargo, &format!("cargo {command}"))
}

fn run_command(mut command: Command, display: &str) -> Result<()> {
    println!("$ {display}");
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to start {display}"))?;
    if !status.success() {
        bail!("{display} exited with {status}");
    }
    Ok(())
}
