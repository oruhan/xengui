use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "xengui", version, about = "XenGui workspace development tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run a workspace binary or example.
    Run(CargoArgs),
    /// Build workspace packages.
    Build(CargoArgs),
    /// Check workspace packages without producing binaries.
    Check(CargoArgs),
    /// Run workspace tests.
    Test(CargoArgs),
    /// Inspect workspace crates.
    Crates(CratesArgs),
    /// Diagnose the local XenGui development environment.
    Doctor,
    /// Inspect and manage workspace versions.
    Version(VersionArgs),
    /// Suggest Git commit metadata.
    Commit(CommitArgs),
    /// Validate release readiness.
    Release(ReleaseArgs),
}

#[derive(Debug, Args)]
pub struct CargoArgs {
    /// Run Cargo for a specific package.
    #[arg(short, long)]
    pub package: Option<String>,
    /// Use the release profile.
    #[arg(long)]
    pub release: bool,
    /// Additional arguments passed directly to Cargo.
    #[arg(last = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CratesArgs {
    #[command(subcommand)]
    pub command: CratesCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CratesCommand {
    /// List all workspace packages discovered by Cargo.
    List,
}

#[derive(Debug, Args)]
pub struct VersionArgs {
    #[command(subcommand)]
    pub command: VersionCommand,
}

#[derive(Debug, Subcommand)]
pub enum VersionCommand {
    /// Show every workspace package version.
    Show,
    /// Validate package versions and internal dependency requirements.
    Check,
    /// Set every workspace package to VERSION (dry-run by default).
    Set(SetVersionArgs),
    /// Bump every workspace package from the highest current version (dry-run by default).
    Bump(BumpVersionArgs),
    /// Recommend a SemVer bump from current Git changes.
    Suggest(SuggestVersionArgs),
}

#[derive(Debug, Args)]
pub struct SetVersionArgs {
    pub version: String,
    /// Write changes to Cargo.toml files. Without this flag, only a plan is printed.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Args)]
pub struct BumpVersionArgs {
    pub level: BumpLevel,
    /// Write changes to Cargo.toml files. Without this flag, only a plan is printed.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Args)]
pub struct SuggestVersionArgs {
    /// Apply the recommendation after displaying its level and reasoning.
    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BumpLevel {
    Patch,
    Minor,
    Major,
}

#[derive(Debug, Args)]
pub struct CommitArgs {
    #[command(subcommand)]
    pub command: CommitCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CommitCommand {
    /// Suggest a concise Conventional Commit message from all local changes.
    Suggest,
}

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    pub command: ReleaseCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReleaseCommand {
    /// Run version, formatting, build, test, and package-readiness checks.
    Check,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_mutations_are_opt_in() {
        let cli = Cli::try_parse_from(["xengui", "version", "set", "1.2.3"]).unwrap();
        let Command::Version(version) = cli.command else {
            panic!("expected version command");
        };
        let VersionCommand::Set(set) = version.command else {
            panic!("expected set command");
        };
        assert!(!set.write);

        let cli = Cli::try_parse_from(["xengui", "version", "suggest"]).unwrap();
        let Command::Version(version) = cli.command else {
            panic!("expected version command");
        };
        let VersionCommand::Suggest(suggest) = version.command else {
            panic!("expected suggest command");
        };
        assert!(!suggest.apply);
    }
}
