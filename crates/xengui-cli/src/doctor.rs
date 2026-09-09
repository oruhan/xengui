use std::process::Command;

use anyhow::{Result, bail};

use crate::workspace::Workspace;

pub fn run(workspace: &Workspace) -> Result<()> {
    println!("XenGui workspace: {}", workspace.root.display());
    println!("Workspace packages: {}", workspace.packages().count());

    let mut failures = Vec::new();
    for tool in ["rustc", "cargo", "git"] {
        match Command::new(tool).arg("--version").output() {
            Ok(output) if output.status.success() => {
                println!(
                    "[ok] {tool}: {}",
                    String::from_utf8_lossy(&output.stdout).trim()
                );
            }
            Ok(output) => failures.push(format!("{tool} returned {}", output.status)),
            Err(error) => failures.push(format!("{tool} is unavailable: {error}")),
        }
    }

    let wasm = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == "wasm32-unknown-unknown")
        });
    println!(
        "[{}] wasm32-unknown-unknown target",
        if wasm { "ok" } else { "warn" }
    );

    let alsa = Command::new("pkg-config")
        .args(["--exists", "alsa"])
        .status()
        .is_ok_and(|status| status.success());
    println!(
        "[{}] ALSA development package (needed for xen-audio on Linux)",
        if alsa { "ok" } else { "warn" }
    );

    if failures.is_empty() {
        println!("Doctor completed without required-tool failures.");
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("[error] {failure}");
        }
        bail!("doctor found {} required-tool failure(s)", failures.len());
    }
}
