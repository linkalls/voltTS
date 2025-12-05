mod ast;
mod cli;
mod codegen;
mod diagnostics;
mod formatter;
mod parser;
mod templates;

use clap::Parser;

use crate::cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => cli::handle_init(path)?,
        Commands::Run { entry } => cli::handle_run(entry)?,
        Commands::Test { path } => cli::handle_test(path)?,
        Commands::Fmt { check, path } => cli::handle_fmt(path, check)?,
        Commands::Lint { path } => cli::handle_lint(path)?,
        Commands::Build {
            entry,
            c_out,
            bin_out,
        } => {
            cli::handle_build(entry, c_out, bin_out)?;
        }
    }

    Ok(())
}
