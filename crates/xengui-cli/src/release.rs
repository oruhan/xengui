use std::process::{Command, Stdio};

use anyhow::{Result, bail};

use crate::{cli::ReleaseCommand, version, workspace::Workspace};

pub fn run(workspace: &Workspace, command: ReleaseCommand) -> Result<()> {
    match command {
        ReleaseCommand::Check => check(workspace),
    }
}

fn check(workspace: &Workspace) -> Result<()> {
    let mut failures = Vec::new();

    print!("[check] workspace and internal dependency versions ... ");
    match version::check(workspace) {
        Ok(()) => println!("ok"),
        Err(error) => {
            println!("FAILED");
            failures.push(error.to_string());
        }
    }

    validate_publish_metadata(workspace, &mut failures);
    run_step(
        workspace,
        "formatting",
        "cargo",
        &["fmt", "--all", "--check"],
        &mut failures,
    );
    run_step(
        workspace,
        "workspace check",
        "cargo",
        &["check", "--workspace"],
        &mut failures,
    );
    run_step(
        workspace,
        "workspace tests",
        "cargo",
        &["test", "--workspace"],
        &mut failures,
    );

    for package in publishable_crates(workspace) {
        let manifest = package.manifest_path.as_str();
        run_step(
            workspace,
            &format!("package contents for {}", package.name),
            "cargo",
            &[
                "package",
                "--list",
                "--allow-dirty",
                "--manifest-path",
                manifest,
            ],
            &mut failures,
        );
    }

    if failures.is_empty() {
        println!("\nRelease check passed.");
        Ok(())
    } else {
        eprintln!("\nRelease check failed:");
        for failure in &failures {
            eprintln!("  - {failure}");
        }
        bail!("{} release-readiness check(s) failed", failures.len());
    }
}

fn publishable_crates(workspace: &Workspace) -> Vec<&cargo_metadata::Package> {
    workspace
        .packages()
        .filter(|package| {
            let relative = package
                .manifest_path
                .as_std_path()
                .strip_prefix(&workspace.root)
                .ok();
            let in_crates = relative.is_some_and(|path| path.starts_with("crates"));
            let publishing_enabled = !package.publish.as_ref().is_some_and(Vec::is_empty);
            in_crates && publishing_enabled
        })
        .collect()
}

fn validate_publish_metadata(workspace: &Workspace, failures: &mut Vec<String>) {
    print!("[check] publish metadata ... ");
    let before = failures.len();
    for package in publishable_crates(workspace) {
        if package.description.as_deref().is_none_or(str::is_empty) {
            failures.push(format!("{} has no package description", package.name));
        }
        if package.license.as_deref().is_none_or(str::is_empty) && package.license_file.is_none() {
            failures.push(format!("{} has no license or license-file", package.name));
        }
        if package.repository.as_deref().is_none_or(str::is_empty) {
            failures.push(format!("{} has no repository URL", package.name));
        }
        if package.targets.is_empty() {
            failures.push(format!("{} has no Cargo targets", package.name));
        }
    }
    println!(
        "{}",
        if failures.len() == before {
            "ok"
        } else {
            "FAILED"
        }
    );
}

fn run_step(
    workspace: &Workspace,
    label: &str,
    program: &str,
    args: &[&str],
    failures: &mut Vec<String>,
) {
    print!("[check] {label} ... ");
    let result = Command::new(program)
        .args(args)
        .current_dir(&workspace.root)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status();
    match result {
        Ok(status) if status.success() => println!("ok"),
        Ok(status) => {
            println!("FAILED");
            failures.push(format!("{label} exited with {status}"));
        }
        Err(error) => {
            println!("FAILED");
            failures.push(format!("could not start {label}: {error}"));
        }
    }
}
