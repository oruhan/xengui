use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};

use crate::cli::CratesCommand;

pub struct Workspace {
    pub root: PathBuf,
    pub metadata: Metadata,
}

impl Workspace {
    pub fn discover() -> Result<Self> {
        let metadata = MetadataCommand::new().no_deps().exec().context(
            "could not discover a Cargo workspace; run xengui inside the XenGui workspace",
        )?;
        let root = metadata.workspace_root.clone().into_std_path_buf();
        Ok(Self { root, metadata })
    }

    pub fn packages(&self) -> impl Iterator<Item = &Package> {
        self.metadata.workspace_packages().into_iter()
    }

    pub fn package_versions(&self) -> HashMap<String, semver::Version> {
        self.packages()
            .map(|package| (package.name.to_string(), package.version.clone()))
            .collect()
    }

    pub fn print_crates(&self, command: CratesCommand) -> Result<()> {
        match command {
            CratesCommand::List => {
                let mut packages: Vec<_> = self.packages().collect();
                packages.sort_by(|left, right| left.name.cmp(&right.name));
                println!("{:<24} {:<12} PATH", "PACKAGE", "VERSION");
                for package in packages {
                    let manifest = package.manifest_path.as_std_path();
                    let relative = manifest.strip_prefix(&self.root).unwrap_or(manifest);
                    println!(
                        "{:<24} {:<12} {}",
                        package.name,
                        package.version,
                        relative.display()
                    );
                }
                Ok(())
            }
        }
    }
}
