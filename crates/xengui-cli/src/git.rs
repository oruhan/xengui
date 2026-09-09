use std::{ collections::BTreeSet, fs, process::Command };

use anyhow::{ Context, Result, bail };

use crate::{ cli::CommitCommand, workspace::Workspace };

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeLevel {
    Patch,
    Minor,
    Major,
}

pub struct ChangeAnalysis {
    pub level: ChangeLevel,
    pub reason: String,
    pub files: Vec<String>,
    pub added_public: usize,
}

pub fn analyze(workspace: &Workspace) -> Result<ChangeAnalysis> {
    ensure_git(workspace)?;
    let status = git_output(workspace, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if status.trim().is_empty() {
        bail!("there are no staged or unstaged Git changes to analyze");
    }

    let mut files = Vec::new();
    let mut untracked = Vec::new();
    for line in status.lines() {
        if line.len() < 4 {
            continue;
        }
        let path = line[3..]
            .split(" -> ")
            .last()
            .unwrap_or(&line[3..])
            .to_owned();
        if line.starts_with("??") {
            untracked.push(path.clone());
        }
        files.push(path);
    }

    let mut diff = git_output(
        workspace,
        &["diff", "--no-ext-diff", "--find-renames", "HEAD"]
    ).or_else(|_| git_output(workspace, &["diff", "--no-ext-diff", "--find-renames"]))?;
    for path in &untracked {
        let full_path = workspace.root.join(path);
        if
            full_path.extension().is_some_and(|extension| extension == "rs") &&
            let Ok(contents) = fs::read_to_string(full_path)
        {
            diff.push_str(&format!("\ndiff --git a/{path} b/{path}\n+++ b/{path}\n"));
            for line in contents.lines() {
                diff.push('+');
                diff.push_str(line);
                diff.push('\n');
            }
        }
    }

    let lower = diff.to_ascii_lowercase();
    let explicit_breaking = lower.lines().any(|line| {
        let changed = line
            .strip_prefix('+')
            .or_else(|| line.strip_prefix('-'))
            .unwrap_or(line)
            .trim_start();
        changed.starts_with("breaking change:") || changed.starts_with("breaking-change:")
    });
    let (added_public, removed_public) = public_api_changes(&diff);
    let removed_feature = removed_cargo_feature(&diff);
    let added_feature = added_cargo_feature(&diff);

    let added_workspace_crate = untracked
        .iter()
        .any(|path| path.starts_with("crates/") && path.ends_with("/Cargo.toml"));

    let (level, reason) = if explicit_breaking || removed_public > 0 || removed_feature {
        let detail = match (explicit_breaking, removed_public > 0) {
            (true, _) => "the changes explicitly declare a breaking change".to_owned(),
            (false, true) => {
                format!(
                    "{removed_public} removed or changed public API declaration(s) may break callers"
                )
            }
            (false, false) => "a Cargo feature appears to have been removed".to_owned(),
        };
        (ChangeLevel::Major, detail)
    } else if added_public > 0 || added_feature || added_workspace_crate {
        let detail = match (added_public > 0, added_workspace_crate) {
            (true, _) => {
                format!("{added_public} backwards-compatible public API declaration(s) were added")
            }
            (false, true) => "a backwards-compatible workspace crate was added".to_owned(),
            (false, false) => {
                "a backwards-compatible Cargo feature appears to have been added".to_owned()
            }
        };
        (ChangeLevel::Minor, detail)
    } else {
        (
            ChangeLevel::Patch,
            "changes appear limited to fixes, tests, documentation, tooling, or internal implementation".to_owned(),
        )
    };

    files.sort();
    files.dedup();
    Ok(ChangeAnalysis {
        level,
        reason,
        files,
        added_public,
    })
}

pub fn commit_suggest(workspace: &Workspace, command: CommitCommand) -> Result<()> {
    match command {
        CommitCommand::Suggest => {
            let analysis = analyze(workspace)?;
            let scope = infer_scope(&analysis.files);
            let kind = infer_commit_type(&analysis);
            let bang = if analysis.level == ChangeLevel::Major { "!" } else { "" };
            let subject = infer_subject(&analysis, kind);
            println!("Suggested commit:\n");
            println!("{kind}({scope}){bang}: {subject}");
            println!("\nReason: {}", analysis.reason);
            println!("Files analyzed: {}", analysis.files.len());
            Ok(())
        }
    }
}

fn ensure_git(workspace: &Workspace) -> Result<()> {
    let inside = git_output(workspace, &["rev-parse", "--is-inside-work-tree"])?;
    if inside.trim() != "true" {
        bail!("workspace is not inside a Git work tree");
    }
    Ok(())
}

fn git_output(workspace: &Workspace, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(&workspace.root)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn public_api_changes(diff: &str) -> (usize, usize) {
    let mut public_crate_source = false;
    let mut public_container = false;
    let mut added = 0;
    let mut removed = 0;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            public_crate_source =
                path.starts_with("crates/") &&
                path.contains("/src/") &&
                !path.starts_with("crates/xengui-cli/") &&
                !path.ends_with("/main.rs");
            public_container = false;
        }
        if line.starts_with("@@") {
            public_container = contains_public_container(line);
        } else if public_crate_source && contains_public_container(line) {
            public_container = true;
        }
        if
            public_crate_source &&
            (is_public_change(line, '+') ||
                (public_container && is_likely_container_member(line, '+')))
        {
            added += 1;
        }
        if
            public_crate_source &&
            (is_public_change(line, '-') ||
                (public_container && is_likely_container_member(line, '-')))
        {
            removed += 1;
        }
    }
    (added, removed)
}

fn contains_public_container(line: &str) -> bool {
    ["pub enum ", "pub trait ", "pub struct "].iter().any(|token| line.contains(token))
}

fn is_likely_container_member(line: &str, prefix: char) -> bool {
    if !line.starts_with(prefix) || line.starts_with("+++") || line.starts_with("---") {
        return false;
    }
    let code = line[1..].trim();
    !code.is_empty() &&
        !code.starts_with("pub ") &&
        !code.starts_with("//") &&
        !code.starts_with("#") &&
        !matches!(code, "{" | "}" | "}," | ";") &&
        (code.starts_with("fn ") ||
            code.contains(':') ||
            code.chars().next().is_some_and(char::is_uppercase))
}

fn is_public_change(line: &str, prefix: char) -> bool {
    if !line.starts_with(prefix) || line.starts_with("+++") || line.starts_with("---") {
        return false;
    }
    let code = line[1..].trim_start();
    [
        "pub fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub type ",
        "pub const ",
        "pub static ",
        "pub mod ",
        "pub use ",
    ]
        .iter()
        .any(|token| code.starts_with(token))
}

fn added_cargo_feature(diff: &str) -> bool {
    cargo_feature_change(diff, '+')
}

fn removed_cargo_feature(diff: &str) -> bool {
    cargo_feature_change(diff, '-')
}

fn cargo_feature_change(diff: &str, prefix: char) -> bool {
    let mut cargo_file = false;
    let mut features_context = false;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/").or_else(|| line.strip_prefix("--- a/")) {
            cargo_file = path.ends_with("Cargo.toml");
            features_context = false;
        }
        if cargo_file && line.contains("[features]") {
            features_context = true;
            continue;
        }
        if cargo_file && line.starts_with("@@") {
            features_context = false;
        }
        if
            cargo_file &&
            features_context &&
            line.starts_with(prefix) &&
            !line.starts_with("+++") &&
            !line.starts_with("---")
        {
            let value = line[1..].trim();
            if value.contains('=') && !value.starts_with('#') {
                return true;
            }
        }
    }
    false
}

fn infer_scope(files: &[String]) -> String {
    let mut scopes = BTreeSet::new();
    for file in files {
        let parts: Vec<_> = file.split('/').collect();
        let scope = match parts.as_slice() {
            ["crates", name, ..] | ["apps", name, ..] | ["examples", name, ..] => *name,
            ["docs", ..] => "docs",
            [".github", ..] => "ci",
            ["scripts", ..] => "tooling",
            _ => "workspace",
        };
        scopes.insert(scope);
    }
    if scopes.len() == 1 {
        scopes.into_iter().next().unwrap_or("workspace").to_owned()
    } else if scopes.contains("xengui-cli") {
        "cli".to_owned()
    } else {
        "workspace".to_owned()
    }
}

#[allow(unused_parens)]
fn infer_commit_type(analysis: &ChangeAnalysis) -> &'static str {
    if analysis.level == ChangeLevel::Major || analysis.level == ChangeLevel::Minor {
        "feat"
    } else if analysis.files.iter().all(|path| path.ends_with(".md")) {
        "docs"
    } else if analysis.files.iter().all(|path| (path.contains("test") || path.ends_with(".snap"))) {
        "test"
    } else if
        analysis.files
            .iter()
            .all(|path| (path.ends_with("Cargo.toml") || path.ends_with("Cargo.lock")))
    {
        "build"
    } else {
        "chore"
    }
}

fn infer_subject(analysis: &ChangeAnalysis, kind: &str) -> &'static str {
    if analysis.files.iter().any(|path| path.starts_with("crates/xengui-cli/")) {
        "add workspace development CLI"
    } else if analysis.level == ChangeLevel::Major {
        "update public API"
    } else if analysis.added_public > 0 {
        "extend public API"
    } else {
        match kind {
            "docs" => "update documentation",
            "test" => "update test coverage",
            "build" => "update workspace configuration",
            _ => "update workspace internals",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_publishable_library_source_counts_as_public_api() {
        let diff =
            "+++ b/crates/xengui-cli/src/cli.rs\n+pub struct InternalCli;\n+++ b/crates/xengui/src/lib.rs\n+pub struct NewWidget;\n-pub fn old_api() {}\n";
        assert_eq!(public_api_changes(diff), (1, 1));
    }

    #[test]
    fn detects_public_enum_variant_changes() {
        let diff =
            "+++ b/crates/xengui/src/lib.rs\n@@ -1,3 +1,3 @@ pub enum Mode {\n-    Old,\n+    New,\n }\n";
        assert_eq!(public_api_changes(diff), (1, 1));
    }

    #[test]
    fn detects_cargo_feature_changes() {
        let diff =
            "+++ b/crates/xengui/Cargo.toml\n@@ -1,3 +1,4 @@\n [features]\n+accessibility = []\n";
        assert!(added_cargo_feature(diff));
        assert!(!removed_cargo_feature(diff));
    }
}
