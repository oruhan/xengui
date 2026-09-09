use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use semver::Version;
use toml_edit::{DocumentMut, Item, Table, Value, value};

use crate::{
    cli::BumpLevel,
    git::{self, ChangeLevel},
    workspace::Workspace,
};

pub fn show(workspace: &Workspace) -> Result<()> {
    let mut packages: Vec<_> = workspace.packages().collect();
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    for package in packages {
        println!("{:<24} {}", package.name, package.version);
    }
    Ok(())
}

pub fn check(workspace: &Workspace) -> Result<()> {
    let versions = workspace.package_versions();
    let mut errors = Vec::new();
    let mut dependency_count = 0;

    for package in workspace.packages() {
        for dependency in &package.dependencies {
            let Some(expected) = versions.get(dependency.name.as_str()) else {
                continue;
            };
            if dependency.path.is_none() {
                continue;
            }
            dependency_count += 1;
            if !dependency.req.matches(expected) {
                errors.push(format!(
                    "{} requires internal crate {} {}, but the package version is {}",
                    package.name, dependency.name, dependency.req, expected
                ));
            }
        }
    }

    if errors.is_empty() {
        println!(
            "Version check passed: {} workspace packages and {dependency_count} internal dependency requirements are consistent.",
            versions.len()
        );
        Ok(())
    } else {
        for error in &errors {
            eprintln!("error: {error}");
        }
        bail!(
            "version check failed with {} inconsistency(s)",
            errors.len()
        );
    }
}

pub fn set(workspace: &Workspace, requested: &str, write: bool) -> Result<()> {
    let version = Version::parse(requested)
        .with_context(|| format!("invalid SemVer version `{requested}`"))?;
    apply_version(workspace, &version, write)
}

pub fn bump(workspace: &Workspace, level: BumpLevel, write: bool) -> Result<()> {
    let current = highest_version(workspace)?;
    let next = bumped(current.clone(), level);
    println!("Baseline: highest workspace version is {current}");
    println!("Requested bump: {level:?} -> {next}");
    apply_version(workspace, &next, write)
}

pub fn suggest(workspace: &Workspace, apply: bool) -> Result<()> {
    let analysis = git::analyze(workspace)?;
    let level = match analysis.level {
        ChangeLevel::Patch => BumpLevel::Patch,
        ChangeLevel::Minor => BumpLevel::Minor,
        ChangeLevel::Major => BumpLevel::Major,
    };
    let current = highest_version(workspace)?;
    let next = bumped(current.clone(), level);

    println!(
        "Recommended bump: {}",
        format!("{level:?}").to_ascii_lowercase()
    );
    println!("Reason: {}", analysis.reason);
    println!(
        "Baseline: highest workspace version is {current}; proposed coordinated version is {next}."
    );

    if apply {
        println!("\n--apply was provided; applying the displayed recommendation.");
        apply_version(workspace, &next, true)
    } else {
        println!("\nDry run only. Re-run with --apply to write this recommendation.");
        Ok(())
    }
}

fn highest_version(workspace: &Workspace) -> Result<Version> {
    workspace
        .packages()
        .map(|package| package.version.clone())
        .max()
        .context("workspace contains no packages")
}

fn bumped(mut version: Version, level: BumpLevel) -> Version {
    version.pre = semver::Prerelease::EMPTY;
    version.build = semver::BuildMetadata::EMPTY;
    match level {
        BumpLevel::Patch => version.patch += 1,
        BumpLevel::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        BumpLevel::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
    }
    version
}

fn apply_version(workspace: &Workspace, target: &Version, write: bool) -> Result<()> {
    let package_versions = workspace.package_versions();
    let package_manifests: HashMap<PathBuf, String> = workspace
        .packages()
        .map(|package| {
            (
                package.manifest_path.clone().into_std_path_buf(),
                package.name.to_string(),
            )
        })
        .collect();

    let mut manifests: Vec<PathBuf> = package_manifests.keys().cloned().collect();
    manifests.push(workspace.root.join("Cargo.toml"));
    manifests.sort();
    manifests.dedup();

    let mut changes = Vec::new();
    for manifest in manifests {
        let source = fs::read_to_string(&manifest)
            .with_context(|| format!("failed to read {}", manifest.display()))?;
        let mut document = source
            .parse::<DocumentMut>()
            .with_context(|| format!("failed to parse {}", manifest.display()))?;

        if package_manifests.contains_key(&manifest) {
            replace_item_value(&mut document["package"]["version"], target.to_string());
        }
        update_dependency_tables(document.as_table_mut(), &package_versions, target);

        let updated = document.to_string();
        if updated != source {
            changes.push((manifest, updated));
        }
    }

    if changes.is_empty() {
        println!("Workspace is already at {target}; no manifest changes are needed.");
        return Ok(());
    }

    println!(
        "{} manifest(s) would be updated to {target}:",
        changes.len()
    );
    for (manifest, _) in &changes {
        let relative = manifest.strip_prefix(&workspace.root).unwrap_or(manifest);
        println!("  {}", relative.display());
    }

    if !write {
        println!("\nDry run only. Re-run with --write to modify these files.");
        return Ok(());
    }

    for (manifest, updated) in changes {
        fs::write(&manifest, updated)
            .with_context(|| format!("failed to write {}", manifest.display()))?;
    }
    refresh_lockfile(workspace)?;
    println!("\nUpdated workspace manifests and refreshed Cargo.lock.");
    Ok(())
}

fn update_dependency_tables(
    table: &mut Table,
    packages: &HashMap<String, Version>,
    target: &Version,
) {
    let keys: Vec<String> = table.iter().map(|(key, _)| key.to_owned()).collect();
    for key in keys {
        let Some(item) = table.get_mut(&key) else {
            continue;
        };
        if matches!(
            key.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        ) {
            update_dependencies(item, packages, target);
            continue;
        }
        match item {
            Item::Table(child) => update_dependency_tables(child, packages, target),
            Item::ArrayOfTables(children) => {
                for child in children.iter_mut() {
                    update_dependency_tables(child, packages, target);
                }
            }
            _ => {}
        }
    }
}

fn update_dependencies(item: &mut Item, packages: &HashMap<String, Version>, target: &Version) {
    let Some(table) = item.as_table_like_mut() else {
        return;
    };
    let names: Vec<String> = table.iter().map(|(name, _)| name.to_owned()).collect();
    for alias in names {
        let Some(dependency) = table.get_mut(&alias) else {
            continue;
        };
        let package_name = dependency
            .get("package")
            .and_then(Item::as_str)
            .unwrap_or(&alias);
        if !packages.contains_key(package_name) {
            continue;
        }
        if dependency.get("workspace").and_then(Item::as_bool) == Some(true) {
            continue;
        }
        if let Some(version) = dependency.as_str() {
            if version != target.to_string() {
                replace_item_value(dependency, target.to_string());
            }
        } else if let Some(inline) = dependency.as_inline_table_mut() {
            if let Some(version) = inline.get_mut("version") {
                replace_value(version, target.to_string());
            } else {
                inline.insert("version", Value::from(target.to_string()));
            }
        } else if let Some(detail) = dependency.as_table_mut() {
            if let Some(version) = detail.get_mut("version") {
                replace_item_value(version, target.to_string());
            } else {
                detail.insert("version", value(target.to_string()));
            }
        }
    }
}

fn replace_item_value(item: &mut Item, version: String) {
    if let Some(current) = item.as_value_mut() {
        replace_value(current, version);
    } else {
        *item = value(version);
    }
}

fn replace_value(current: &mut Value, version: String) {
    let decor = current.decor().clone();
    let mut replacement = Value::from(version);
    *replacement.decor_mut() = decor;
    *current = replacement;
}

fn refresh_lockfile(workspace: &Workspace) -> Result<()> {
    let status = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&workspace.root)
        .stdout(Stdio::null())
        .status()
        .context("failed to refresh Cargo.lock")?;
    if !status.success() {
        bail!("cargo metadata failed while refreshing Cargo.lock");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_updates_preserve_comments_and_workspace_inheritance() {
        let mut document = r#"[package]
name = "demo"
version = "0.2.8" # package version comment

[workspace.dependencies]
xengui = { version = "0.2.8", path = "crates/xengui" } # keep this

[dependencies]
xengui = { workspace = true }
"#
        .parse::<DocumentMut>()
        .unwrap();
        let packages = HashMap::from([("xengui".to_owned(), Version::new(0, 2, 8))]);
        update_dependency_tables(document.as_table_mut(), &packages, &Version::new(1, 0, 0));
        replace_item_value(&mut document["package"]["version"], "1.0.0".to_owned());
        let output = document.to_string();
        assert!(output.contains("version = \"1.0.0\""));
        assert!(output.contains("# keep this"));
        assert!(output.contains("# package version comment"));
        assert!(output.contains("xengui = { workspace = true }"));
    }

    #[test]
    fn major_bump_resets_minor_and_patch() {
        assert_eq!(
            bumped(Version::new(0, 2, 8), BumpLevel::Major),
            Version::new(1, 0, 0)
        );
    }
}
