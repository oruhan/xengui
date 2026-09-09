mod cargo;
mod cli;
mod doctor;
mod git;
mod release;
mod version;
mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, VersionCommand};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace = workspace::Workspace::discover()?;

    match cli.command {
        Command::Run(args) => cargo::run(&workspace, "run", args),
        Command::Build(args) => cargo::run(&workspace, "build", args),
        Command::Check(args) => cargo::run(&workspace, "check", args),
        Command::Test(args) => cargo::run(&workspace, "test", args),
        Command::Crates(args) => workspace.print_crates(args.command),
        Command::Doctor => doctor::run(&workspace),
        Command::Version(args) => match args.command {
            VersionCommand::Show => version::show(&workspace),
            VersionCommand::Check => version::check(&workspace),
            VersionCommand::Set(args) => version::set(&workspace, &args.version, args.write),
            VersionCommand::Bump(args) => version::bump(&workspace, args.level, args.write),
            VersionCommand::Suggest(args) => version::suggest(&workspace, args.apply),
        },
        Command::Commit(args) => git::commit_suggest(&workspace, args.command),
        Command::Release(args) => release::run(&workspace, args.command),
    }
}
