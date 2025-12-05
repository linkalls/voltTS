use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser, Subcommand};
use globwalk::GlobWalkerBuilder;

use crate::codegen::codegen_c;
use crate::formatter::format_program;
use crate::parser::{load_program, parse_program};

#[derive(Parser)]
#[command(name = "voltts", version, about = "VoltTS CLI (v0.1 prototype)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scaffold a minimal VoltTS project
    Init {
        /// Target directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Compile and run an entry file (native via C)
    Run {
        /// Entry file path
        #[arg(value_name = "ENTRY", default_value = "src/main.vts")]
        entry: PathBuf,
    },
    /// Execute test files discovered under the given path (discovery implemented)
    Test {
        #[arg(value_name = "PATH", default_value = "tests")]
        path: PathBuf,
    },
    /// Format VoltTS source files (round-trip formatter for supported syntax)
    Fmt {
        /// Run in check mode without modifying files
        #[arg(long, action = ArgAction::SetTrue)]
        check: bool,
        /// Path to format
        #[arg(value_name = "PATH", default_value = "src/main.vts")]
        path: PathBuf,
    },
    /// Lint VoltTS sources (parses and reports diagnostics)
    Lint {
        #[arg(value_name = "PATH", default_value = "src/main.vts")]
        path: PathBuf,
    },
    /// Build a VoltTS entry file to a C artifact and native binary
    Build {
        #[arg(value_name = "ENTRY", default_value = "src/main.vts")]
        entry: PathBuf,
        #[arg(long, value_name = "C_OUT", default_value = "dist/app.c")]
        c_out: PathBuf,
        #[arg(long, value_name = "BIN_OUT", default_value = "dist/app")]
        bin_out: PathBuf,
    },
}

pub fn handle_init(root: PathBuf) -> Result<()> {
    fs::create_dir_all(root.join("src"))
        .with_context(|| format!("failed to create src directory under {}", root.display()))?;

    let sample_path = root.join("src/main.vts");
    if !sample_path.exists() {
        fs::write(&sample_path, crate::templates::SAMPLE_MAIN)
            .with_context(|| format!("failed to write sample file at {}", sample_path.display()))?;
    }

    let support_dir = root.join("src/support");
    fs::create_dir_all(&support_dir).with_context(|| {
        format!(
            "failed to create support directory under {}",
            root.display()
        )
    })?;
    let support_helper = support_dir.join("log_helper.vts");
    if !support_helper.exists() {
        fs::write(&support_helper, crate::templates::SAMPLE_HELPER).with_context(|| {
            format!(
                "failed to write helper file at {}",
                support_helper.display()
            )
        })?;
    }

    let tests_dir = root.join("tests");
    fs::create_dir_all(&tests_dir)
        .with_context(|| format!("failed to create tests directory under {}", root.display()))?;

    println!("Initialized VoltTS workspace at {}", root.display());
    println!("  - src/main.vts (sample)");
    println!("  - tests/ (empty)");
    Ok(())
}

pub fn handle_run(entry: PathBuf) -> Result<()> {
    let bin = handle_build(
        entry.clone(),
        PathBuf::from("dist/app.c"),
        PathBuf::from("dist/app"),
    )?;
    println!("Running {}...", bin.display());
    let status = Command::new(&bin)
        .status()
        .with_context(|| format!("failed to execute {}", bin.display()))?;

    if !status.success() {
        return Err(anyhow!("program exited with status {}", status));
    }
    Ok(())
}

pub fn handle_test(path: PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!(
            "test path {} does not exist; create *.test.vts files first",
            path.display()
        ));
    }

    let walker = GlobWalkerBuilder::from_patterns(
        &path,
        &["**/*.test.vts", "**/*.spec.vts", "**/*_test.vts"],
    )
    .follow_links(true)
    .build()
    .with_context(|| format!("failed to walk test path {}", path.display()))?;

    let mut found = Vec::new();
    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            found.push(entry.path().to_path_buf());
        }
    }

    if found.is_empty() {
        println!(
            "No tests found under {} (patterns: *.test.vts, *.spec.vts, *_test.vts)",
            path.display()
        );
        return Ok(());
    }

    println!("Discovered {} test file(s):", found.len());
    for file in &found {
        println!("  - {}", file.display());
    }

    println!("[todo] Execute discovered tests with the built-in runner");
    Ok(())
}

pub fn handle_fmt(path: PathBuf, check: bool) -> Result<()> {
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let program = parse_program(&source)?;
    let formatted = format_program(&program);

    if check {
        if source == formatted {
            println!("{} is already formatted", path.display());
        } else {
            return Err(anyhow!("{} is not formatted", path.display()));
        }
    } else {
        fs::write(&path, formatted)
            .with_context(|| format!("failed to write formatted file {}", path.display()))?;
        println!("Formatted {}", path.display());
    }
    Ok(())
}

pub fn handle_lint(path: PathBuf) -> Result<()> {
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_program(&source)?;
    println!("{} linted successfully", path.display());
    Ok(())
}

pub fn handle_build(entry: PathBuf, c_out: PathBuf, bin_out: PathBuf) -> Result<PathBuf> {
    ensure_entry_exists(&entry)?;
    let program = load_program(&entry)?;
    let c_code = codegen_c(&program, &entry);

    if let Some(parent) = c_out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create output dir {}", parent.display()))?;
        }
    }
    fs::write(&c_out, &c_code)
        .with_context(|| format!("failed to write C artifact at {}", c_out.display()))?;

    let bin_parent = bin_out
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&bin_parent)
        .with_context(|| format!("failed to create binary dir {}", bin_parent.display()))?;

    let status = Command::new("cc")
        .args(["-std=c99", "-Wall", "-Werror"])
        .arg(&c_out)
        .arg("-o")
        .arg(&bin_out)
        .status()
        .with_context(|| format!("failed to invoke cc for {}", c_out.display()))?;

    if !status.success() {
        return Err(anyhow!("C compilation failed for {}", c_out.display()));
    }

    println!(
        "Generated {} and binary {}",
        c_out.display(),
        bin_out.display()
    );
    Ok(bin_out)
}

fn ensure_entry_exists(entry: &PathBuf) -> Result<()> {
    if entry.exists() {
        return Ok(());
    }

    Err(anyhow!("entry file {} does not exist", entry.display()))
}
