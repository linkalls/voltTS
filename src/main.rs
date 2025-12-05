use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser, Subcommand};
use globwalk::GlobWalkerBuilder;

#[derive(Parser)]
#[command(name = "voltts", version, about = "VoltTS CLI (v0.1 prototype)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => handle_init(path)?,
        Commands::Run { entry } => handle_run(entry)?,
        Commands::Test { path } => handle_test(path)?,
        Commands::Fmt { check, path } => handle_fmt(path, check)?,
        Commands::Lint { path } => handle_lint(path)?,
        Commands::Build {
            entry,
            c_out,
            bin_out,
        } => {
            handle_build(entry, c_out, bin_out)?;
        }
    }

    Ok(())
}

fn stmt_contains_return(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::ReturnInt(_) => true,
        Stmt::If {
            condition: _,
            then_branch,
            else_branch,
        } => then_branch.iter().any(stmt_contains_return)
            || else_branch.iter().any(stmt_contains_return),
        Stmt::While { body, .. } => body.iter().any(stmt_contains_return),
        Stmt::ForRange { body, .. } => body.iter().any(stmt_contains_return),
        Stmt::Await(inner) => stmt_contains_return(inner),
        _ => false,
    }
}

fn handle_init(root: PathBuf) -> Result<()> {
    fs::create_dir_all(root.join("src"))
        .with_context(|| format!("failed to create src directory under {}", root.display()))?;

    let sample_path = root.join("src/main.vts");
    if !sample_path.exists() {
        fs::write(&sample_path, SAMPLE_MAIN)
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
        fs::write(&support_helper, SAMPLE_HELPER).with_context(|| {
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

fn handle_run(entry: PathBuf) -> Result<()> {
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

fn handle_test(path: PathBuf) -> Result<()> {
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

fn handle_fmt(path: PathBuf, check: bool) -> Result<()> {
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let program = parse_program(&source)?;
    type_check(&program)?;
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

fn handle_lint(path: PathBuf) -> Result<()> {
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let program = parse_program(&source)?;
    type_check(&program)?;
    println!("{} linted successfully", path.display());
    Ok(())
}

fn handle_build(entry: PathBuf, c_out: PathBuf, bin_out: PathBuf) -> Result<PathBuf> {
    ensure_entry_exists(&entry)?;
    let program = load_program(&entry)?;
    type_check(&program)?;
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

// --- Frontend (very small subset) ---
#[derive(Debug, Clone)]
struct Import {
    names: Vec<String>,
    module: String,
}

#[derive(Debug, Clone)]
struct Program {
    imports: Vec<Import>,
    functions: Vec<Function>,
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    return_type: Option<TypeAnnotation>,
    body: Vec<Spanned<Stmt>>,
    is_async: bool,
    #[allow(dead_code)]
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeAnnotation {
    Int,
    String,
    Void,
    Unknown(String),
}

impl TypeAnnotation {
    fn as_str(&self) -> &str {
        match self {
            TypeAnnotation::Int => "int",
            TypeAnnotation::String => "string",
            TypeAnnotation::Void => "void",
            TypeAnnotation::Unknown(raw) => raw.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
struct Spanned<T> {
    value: T,
    line: usize,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
enum Stmt {
    Print(String),
    ReturnInt(i32),
    Log { level: LogLevel, message: String },
    SleepMs(u64),
    TimeNow,
    FsReadFile { path: String },
    FsWriteFile { path: String, contents: String },
    Call(String),
    Await(Box<Stmt>),
    If {
        condition: Condition,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        condition: Condition,
        body: Vec<Stmt>,
    },
    ForRange {
        var: String,
        start: i32,
        end: i32,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
enum Condition {
    BoolLiteral(bool),
}

fn parse_program(source: &str) -> Result<Program> {
    let mut lines = source.lines().enumerate().peekable();
    let mut imports = Vec::new();
    let mut functions = Vec::new();

    while let Some((line_number, line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("import ") {
            imports.push(parse_import(trimmed, line_number + 1)?);
            continue;
        }

        if trimmed.starts_with("export fn")
            || trimmed.starts_with("fn")
            || trimmed.starts_with("export async fn")
            || trimmed.starts_with("async fn")
        {
            let signature = trimmed.strip_prefix("export ").unwrap_or(trimmed);
            let (name, return_type, is_async) = parse_signature(signature, line_number + 1)?;

            let mut body = Vec::new();
            // consume until '{'
            if !signature.contains('{') {
                while let Some((_, next)) = lines.next() {
                    if next.contains('{') {
                        break;
                    }
                }
            }

            for (body_idx, body_line) in &mut lines {
                let body_trimmed = body_line.trim();
                if body_trimmed.starts_with('}') {
                    break;
                }
                if body_trimmed.is_empty() || body_trimmed.starts_with("//") {
                    continue;
                }
                body.push(Spanned {
                    value: parse_stmt(body_trimmed)?,
                    line: body_idx + 1,
                });
            }

            functions.push(Function {
                name,
                return_type,
                body,
                is_async,
                line: line_number + 1,
            });
        }
    }

    if functions.is_empty() {
        return Err(anyhow!("no functions found"));
    }

    Ok(Program { imports, functions })
}

fn type_check(program: &Program) -> Result<()> {
    for func in &program.functions {
        if func.name == "main" {
            match func.return_type {
                Some(TypeAnnotation::Int) | Some(TypeAnnotation::Void) => {}
                Some(ref other) => {
                    return Err(anyhow!(
                        "行 {}: main は戻り値型として int もしくは void を明示してください (指定: {})",
                        func.line,
                        other.as_str()
                    ))
                }
                None => {
                    return Err(anyhow!(
                        "行 {}: main は戻り値型として int もしくは void を明示してください",
                        func.line
                    ))
                }
            }
        }

        let declared = func
            .return_type
            .as_ref()
            .cloned()
            .unwrap_or(TypeAnnotation::Void);
        let expected = declared;

        let mut saw_return = false;
        for stmt in &func.body {
            if stmt_contains_return(&stmt.value) {
                saw_return = true;
                if !matches!(expected, TypeAnnotation::Int) {
                    return Err(anyhow!(
                        "行 {}: 関数 {} は {} を返すべきなのに、int を返しています",
                        stmt.line,
                        func.name,
                        expected.as_str()
                    ));
                }
            }
        }

        if func.name != "main"
            && matches!(expected, TypeAnnotation::Int)
            && !saw_return
        {
            return Err(anyhow!(
                "行 {}: 関数 {} は int を返す必要がありますが、return が見つかりません",
                func.line, func.name
            ));
        }
    }

    Ok(())
}

fn load_program(entry: &PathBuf) -> Result<Program> {
    let mut visited = HashSet::new();
    load_program_recursive(entry, &mut visited)
}

fn load_program_recursive(path: &PathBuf, visited: &mut HashSet<PathBuf>) -> Result<Program> {
    let abs = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    if !visited.insert(abs.clone()) {
        return Ok(Program {
            imports: Vec::new(),
            functions: Vec::new(),
        });
    }

    let source =
        fs::read_to_string(&abs).with_context(|| format!("failed to read {}", abs.display()))?;
    let mut program = parse_program(&source)?;

    let mut extra_functions = Vec::new();
    let base_dir = abs
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    for import in &program.imports {
        if import.module.starts_with("./") || import.module.starts_with("../") {
            let mut resolved = base_dir.join(&import.module);
            if resolved.extension().is_none() {
                resolved.set_extension("vts");
            }
            let nested = load_program_recursive(&resolved, visited)?;
            extra_functions.extend(nested.functions);
        }
    }

    program.functions.extend(extra_functions);
    Ok(program)
}

fn parse_import(line: &str, line_number: usize) -> Result<Import> {
    let without_suffix = line.trim().trim_end_matches(';').trim();
    let without_prefix = without_suffix
        .strip_prefix("import")
        .ok_or_else(|| anyhow!("invalid import syntax at line {}: {}", line_number, line))?
        .trim();

    let (names_part, rest) = without_prefix.split_once('}').ok_or_else(|| {
        anyhow!(
            "import must include a closing brace ('}}') at line {}: {}",
            line_number,
            line
        )
    })?;
    let names_block = names_part.strip_prefix('{').ok_or_else(|| {
        anyhow!(
            "import must start with '{{' at line {}: {}",
            line_number,
            line
        )
    })?;
    let names: Vec<String> = names_block
        .split(',')
        .map(|n| n.trim())
        .filter(|n| !n.is_empty())
        .map(|n| n.to_string())
        .collect();

    if names.is_empty() {
        return Err(anyhow!(
            "import must list at least one name at line {}: {}",
            line_number,
            line
        ));
    }

    let module = rest
        .trim()
        .strip_prefix("from")
        .ok_or_else(|| anyhow!("import missing 'from' at line {}: {}", line_number, line))?
        .trim()
        .trim_matches('"')
        .to_string();

    if module.is_empty() {
        return Err(anyhow!(
            "import module path is empty at line {}: {}",
            line_number,
            line
        ));
    }

    Ok(Import { names, module })
}

fn parse_signature(
    signature: &str,
    line_number: usize,
) -> Result<(String, Option<TypeAnnotation>, bool)> {
    // signature like: fn main(): int {
    let mut without_prefix = signature.trim_start_matches("export").trim();
    let mut is_async = false;
    if without_prefix.starts_with("async") {
        is_async = true;
        without_prefix = without_prefix.trim_start_matches("async").trim();
    }
    without_prefix = without_prefix.trim_start_matches("fn").trim();

    let name_and_rest: Vec<&str> = without_prefix.splitn(2, '(').collect();
    if name_and_rest.len() < 2 {
        return Err(anyhow!(
            "invalid function signature at line {}: {}",
            line_number,
            signature
        ));
    }
    let name = name_and_rest[0].trim().to_string();

    let after_params = name_and_rest[1];
    let return_type = if let Some(idx) = after_params.find(':') {
        let rt = after_params[idx + 1..].trim();
        let rt = rt.trim_end_matches('{').trim();
        if rt.is_empty() {
            None
        } else {
            Some(parse_type_annotation(rt, line_number))
        }
    } else {
        None
    };

    Ok((name, return_type, is_async))
}

fn parse_type_annotation(raw: &str, line_number: usize) -> TypeAnnotation {
    match raw {
        "int" => TypeAnnotation::Int,
        "string" => TypeAnnotation::String,
        "void" => TypeAnnotation::Void,
        other => {
            eprintln!(
                "warning: 未知の型 '{}' (行 {}) を見つけました。Unknown type will be treated as opaque.",
                other, line_number
            );
            TypeAnnotation::Unknown(other.to_string())
        }
    }
}

fn parse_stmt(line: &str) -> Result<Stmt> {
    let trimmed = line.trim().trim_end_matches(';');
    if let Some(rest) = trimmed.strip_prefix("await ") {
        let inner = parse_stmt_core(rest)?;
        return Ok(Stmt::Await(Box::new(inner)));
    }

    parse_stmt_core(trimmed)
}

fn parse_stmt_core(trimmed: &str) -> Result<Stmt> {
    if let Some(rest) = trimmed.strip_prefix("if ") {
        let (cond_raw, branches_raw) = rest
            .split_once('{')
            .ok_or_else(|| anyhow!("if 文は {{ が必要です: {}", trimmed))?;
        let condition = parse_condition(cond_raw.trim())?;
        let (then_raw, else_raw) = branches_raw
            .rsplit_once("} else {")
            .map(|(then_part, else_part)| (then_part, Some(else_part)))
            .unwrap_or((branches_raw, None));
        let then_branch = parse_inline_block(then_raw.trim_end_matches('}').trim())?;
        let else_branch = if let Some(raw) = else_raw {
            parse_inline_block(raw.trim_end_matches('}').trim())?
        } else {
            Vec::new()
        };

        return Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        });
    }

    if let Some(rest) = trimmed.strip_prefix("while ") {
        let (cond_raw, body_raw) = rest
            .split_once('{')
            .ok_or_else(|| anyhow!("while 文は {{ が必要です: {}", trimmed))?;
        let condition = parse_condition(cond_raw.trim())?;
        let body = parse_inline_block(body_raw.trim_end_matches('}').trim())?;
        return Ok(Stmt::While { condition, body });
    }

    if let Some(rest) = trimmed.strip_prefix("for ") {
        let (head, body_raw) = rest
            .split_once('{')
            .ok_or_else(|| anyhow!("for 文は {{ が必要です: {}", trimmed))?;
        let (var_part, range_part) = head
            .trim()
            .split_once(" in ")
            .ok_or_else(|| anyhow!("for 文は 'for <var> in <start>..<end>' 形式です: {}", head))?;
        let var = var_part.trim().to_string();
        let (start_raw, end_raw) = range_part
            .split_once("..")
            .ok_or_else(|| anyhow!("for の range は start..end 形式です: {}", range_part))?;
        let start = start_raw
            .trim()
            .parse::<i32>()
            .context("for range start は整数である必要があります")?;
        let end = end_raw
            .trim()
            .parse::<i32>()
            .context("for range end は整数である必要があります")?;
        let body = parse_inline_block(body_raw.trim_end_matches('}').trim())?;
        return Ok(Stmt::ForRange {
            var,
            start,
            end,
            body,
        });
    }

    if trimmed.starts_with("print(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("print(").trim_end_matches(')');
        let text = inner
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace('\"', "\"");
        return Ok(Stmt::Print(text));
    }

    if trimmed.starts_with("log.") {
        let (level, rest) = trimmed
            .split_once('(')
            .ok_or_else(|| anyhow!("invalid log call: {}", trimmed))?;
        let level = level.trim_end_matches('.');
        let message = rest
            .trim_end_matches(')')
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace('\"', "\"");
        let level = match level {
            "log.info" => LogLevel::Info,
            "log.warn" => LogLevel::Warn,
            "log.error" => LogLevel::Error,
            _ => {
                return Err(anyhow!(
                    "unsupported log level '{}'; use log.info/log.warn/log.error",
                    level
                ));
            }
        };

        return Ok(Stmt::Log { level, message });
    }

    if let Some(rest) = trimmed.strip_prefix("fs.readFile(") {
        let inner = rest.trim_end_matches(')');
        let path = inner
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace('\"', "\"");
        return Ok(Stmt::FsReadFile { path });
    }

    if let Some(rest) = trimmed.strip_prefix("fs.writeFile(") {
        let inner = rest.trim_end_matches(')');
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(anyhow!("fs.writeFile expects path and contents"));
        }
        let path = parts[0]
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace('\"', "\"");
        let contents = parts[1]
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .replace('\"', "\"");
        return Ok(Stmt::FsWriteFile { path, contents });
    }

    if let Some(rest) = trimmed.strip_prefix("time.sleep(") {
        let value = rest
            .trim_end_matches(')')
            .trim()
            .parse::<u64>()
            .context("expected integer milliseconds for time.sleep")?;
        return Ok(Stmt::SleepMs(value));
    }

    if trimmed == "time.now()" {
        return Ok(Stmt::TimeNow);
    }

    if let Some(rest) = trimmed.strip_prefix("return ") {
        let value = rest
            .trim()
            .parse::<i32>()
            .context("expected integer return value")?;
        return Ok(Stmt::ReturnInt(value));
    }

    if trimmed.ends_with("()") {
        let name = trimmed.trim_end_matches("()").trim();
        if !name.is_empty() {
            return Ok(Stmt::Call(name.to_string()));
        }
    }

    Err(anyhow!("unsupported statement: {}", trimmed))
}

fn parse_condition(raw: &str) -> Result<Condition> {
    match raw.trim() {
        "true" => Ok(Condition::BoolLiteral(true)),
        "false" => Ok(Condition::BoolLiteral(false)),
        other => Err(anyhow!("条件式がサポートされていません: {}", other)),
    }
}

fn parse_inline_block(raw: &str) -> Result<Vec<Stmt>> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmts = Vec::new();
    for part in raw.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        stmts.push(parse_stmt_core(trimmed)?);
    }
    Ok(stmts)
}

fn emit_stmt(
    out: &mut String,
    stmt: &Stmt,
    temp_counter: &mut usize,
    saw_return: Option<&mut bool>,
    in_main: bool,
    returns_int: bool,
) {
    let mut saw_return = saw_return;
    match stmt {
        Stmt::Await(inner) => emit_stmt(out, inner, temp_counter, saw_return, in_main, returns_int),
        Stmt::Print(text) => {
            out.push_str(&format!(
                "    printf(\"%s\\n\", \"{}\");\n",
                text.replace('"', "\\\"")
            ));
        }
        Stmt::Log { level, message } => {
            let call = match level {
                LogLevel::Info => "vts_log_info",
                LogLevel::Warn => "vts_log_warn",
                LogLevel::Error => "vts_log_error",
            };
            out.push_str(&format!(
                "    {}(\"{}\");\n",
                call,
                message.replace('"', "\\\"")
            ));
        }
        Stmt::SleepMs(ms) => {
            out.push_str(&format!("    vts_sleep_ms({});\n", ms));
        }
        Stmt::TimeNow => {
            out.push_str("    printf(\"%lld\\n\", vts_time_now_ms());\n");
        }
        Stmt::FsReadFile { path } => {
            let tmp = format!("vts_tmp{}", temp_counter);
            *temp_counter += 1;
            out.push_str(&format!(
                "    char *{} = vts_fs_read_file(\"{}\");\n",
                tmp,
                path.replace('"', "\\\"")
            ));
            let ret = if returns_int { "return 1;" } else { "return;" };
            out.push_str(&format!(
                "    if ({0}) {{ printf(\"%s\\n\", {0}); free({0}); }} else {{ fprintf(stderr, \"[fs.readFile] failed: {1}\\n\"); {2} }}\n",
                tmp,
                path.replace('"', "\\\"")
                ,
                ret
            ));
        }
        Stmt::FsWriteFile { path, contents } => {
            let ret = if returns_int { "return 1;" } else { "return;" };
            out.push_str(&format!(
                "    if (vts_fs_write_file(\"{}\", \"{}\") != 0) {{ fprintf(stderr, \"[fs.writeFile] failed: {}\\n\"); {3} }}\n",
                path.replace('"', "\\\""),
                contents.replace('"', "\\\""),
                path.replace('"', "\\\""),
                ret
            ));
        }
        Stmt::Call(name) => {
            out.push_str(&format!("    {}();\n", name));
        }
        Stmt::ReturnInt(v) => {
            if let Some(flag) = saw_return.as_deref_mut() {
                *flag = true;
            }
            out.push_str(&format!("    return {};\n", v));
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            out.push_str(&format!("    if ({}) {{\n", emit_condition(condition)));
            for inner in then_branch {
                emit_stmt(
                    out,
                    inner,
                    temp_counter,
                    saw_return.as_deref_mut(),
                    in_main,
                    returns_int,
                );
            }
            out.push_str("    }\n");
            if !else_branch.is_empty() {
                out.push_str("    else {\n");
                for inner in else_branch {
                    emit_stmt(
                        out,
                        inner,
                        temp_counter,
                        saw_return.as_deref_mut(),
                        in_main,
                        returns_int,
                    );
                }
                out.push_str("    }\n");
            }
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("    while ({}) {{\n", emit_condition(condition)));
            for inner in body {
                emit_stmt(
                    out,
                    inner,
                    temp_counter,
                    saw_return.as_deref_mut(),
                    in_main,
                    returns_int,
                );
            }
            out.push_str("    }\n");
        }
        Stmt::ForRange {
            var,
            start,
            end,
            body,
        } => {
            out.push_str(&format!(
                "    for (int {} = {}; {} < {}; {}++) {{\n",
                var, start, var, end, var
            ));
            for inner in body {
                emit_stmt(
                    out,
                    inner,
                    temp_counter,
                    saw_return.as_deref_mut(),
                    in_main,
                    returns_int,
                );
            }
            out.push_str("    }\n");
        }
    }
}

fn emit_condition(condition: &Condition) -> String {
    match condition {
        Condition::BoolLiteral(true) => "1".to_string(),
        Condition::BoolLiteral(false) => "0".to_string(),
    }
}

fn codegen_c(program: &Program, source_path: &Path) -> String {
    let mut out = String::new();
    out.push_str("// VoltTS v0.1 generated C (prototype)\n");
    out.push_str(&format!("// Source: {}\n", source_path.display()));
    out.push_str("#define _XOPEN_SOURCE 700\n");
    out.push_str("#include <stdio.h>\n");
    out.push_str("#include <stdlib.h>\n");
    out.push_str("#include <string.h>\n");
    out.push_str("#include <sys/time.h>\n");
    out.push_str("#include <sys/stat.h>\n");
    out.push_str("#include <unistd.h>\n\n");

    out.push_str("// forward declaration for usleep on some libc variants\n");
    out.push_str("int usleep(unsigned int);\n\n");

    out.push_str("#if defined(__GNUC__) || defined(__clang__)\n#define VTS_UNUSED __attribute__((unused))\n#else\n#define VTS_UNUSED\n#endif\n\n");

    out.push_str("// --- standard runtime (prototype) ---\n");
    out.push_str(
        "static VTS_UNUSED void vts_log_info(const char *msg) { printf(\"[info] %s\\n\", msg); }\n",
    );
    out.push_str(
        "static VTS_UNUSED void vts_log_warn(const char *msg) { printf(\"[warn] %s\\n\", msg); }\n",
    );
    out.push_str(
        "static VTS_UNUSED void vts_log_error(const char *msg) { printf(\"[error] %s\\n\", msg); }\n",
    );
    out.push_str("static VTS_UNUSED void vts_sleep_ms(unsigned long ms) { usleep(ms * 1000); }\n");
    out.push_str(
        "static VTS_UNUSED long long vts_time_now_ms(void) { struct timeval tv; gettimeofday(&tv, NULL); return (long long)tv.tv_sec * 1000 + tv.tv_usec / 1000; }\n\n",
    );
    out.push_str("static VTS_UNUSED char *vts_fs_read_file(const char *path) { FILE *f = fopen(path, \"rb\"); if (!f) return NULL; if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return NULL; } long size = ftell(f); if (size < 0) { fclose(f); return NULL; } if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return NULL; } char *buf = (char *)malloc((size_t)size + 1); if (!buf) { fclose(f); return NULL; } size_t read = fread(buf, 1, (size_t)size, f); buf[read] = 0; fclose(f); return buf; }\n");
    out.push_str("static VTS_UNUSED int vts_fs_write_file(const char *path, const char *contents) { const char *slash = strrchr(path, '/'); if (slash) { size_t len = (size_t)(slash - path); if (len > 0) { char *dir = (char *)malloc(len + 1); if (!dir) return -1; memcpy(dir, path, len); dir[len] = 0; struct stat st; if (stat(dir, &st) != 0) { mkdir(dir, 0755); } free(dir); } } FILE *f = fopen(path, \"wb\"); if (!f) return -1; size_t len = strlen(contents); size_t written = fwrite(contents, 1, len, f); fclose(f); return written == len ? 0 : -1; }\n\n");

    out.push_str("// --- user prototypes ---\n");
    for func in &program.functions {
        let is_main = func.name == "main";
        let returns_int = is_main || matches!(func.return_type, Some(TypeAnnotation::Int));
        let c_return = if returns_int { "int" } else { "void" };
        out.push_str(&format!("{} {}(void);\n", c_return, func.name));
    }
    out.push_str("\n");

    for func in &program.functions {
        let mut temp_counter = 0;
        let is_main = func.name == "main";
        let returns_int = is_main || matches!(func.return_type, Some(TypeAnnotation::Int));
        if is_main {
            out.push_str(&format!("{} main(void) {{\n", if returns_int { "int" } else { "void" }));
            let mut saw_return = false;
            for stmt in &func.body {
                emit_stmt(
                    &mut out,
                    &stmt.value,
                    &mut temp_counter,
                    Some(&mut saw_return),
                    returns_int,
                    returns_int,
                );
            }
            if returns_int && !saw_return {
                out.push_str("    return 0;\n");
            }
            out.push_str("}\n\n");
        } else {
            let c_return = if returns_int { "int" } else { "void" };
            out.push_str(&format!("{} {}(void) {{\n", c_return, func.name));
            let mut saw_return = false;
            for stmt in &func.body {
                emit_stmt(
                    &mut out,
                    &stmt.value,
                    &mut temp_counter,
                    if returns_int {
                        Some(&mut saw_return)
                    } else {
                        None
                    },
                    false,
                    returns_int,
                );
            }
            if returns_int && !saw_return {
                out.push_str("    return 0;\n");
            } else if matches!(func.return_type, Some(TypeAnnotation::Void) | None) {
                out.push_str("    return;\n");
            }
            out.push_str("}\n\n");
        }
    }

    out
}

fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for import in &program.imports {
        out.push_str(&format!(
            "import {{ {} }} from \"{}\"\n",
            import.names.join(", "),
            import.module
        ));
    }

    if !program.imports.is_empty() {
        out.push('\n');
    }

    for func in &program.functions {
        let async_prefix = if func.is_async { "async " } else { "" };
        match &func.return_type {
            Some(rt) => out.push_str(&format!(
                "{async_prefix}fn {}(): {} {{\n",
                func.name,
                rt.as_str()
            )),
            None => out.push_str(&format!("{async_prefix}fn {}() {{\n", func.name)),
        }
        for stmt in &func.body {
            match &stmt.value {
                Stmt::Await(inner) => {
                    out.push_str("    await ");
                    match **inner {
                        Stmt::Print(ref text) => out.push_str(&format!("print(\"{}\")\n", text)),
                        Stmt::Log { level, ref message } => {
                            let level = match level {
                                LogLevel::Info => "info",
                                LogLevel::Warn => "warn",
                                LogLevel::Error => "error",
                            };
                            out.push_str(&format!("log.{}(\"{}\")\n", level, message));
                        }
                        Stmt::SleepMs(ms) => out.push_str(&format!("time.sleep({})\n", ms)),
                        Stmt::TimeNow => out.push_str("time.now()\n"),
                        Stmt::FsReadFile { ref path } => {
                            out.push_str(&format!("fs.readFile(\"{}\")\n", path));
                        }
                        Stmt::FsWriteFile {
                            ref path,
                            ref contents,
                        } => {
                            out.push_str(&format!(
                                "fs.writeFile(\"{}\", \"{}\")\n",
                                path, contents
                            ));
                        }
                        Stmt::Call(ref name) => {
                            out.push_str(&format!("{}()\n", name));
                        }
                        Stmt::ReturnInt(v) => out.push_str(&format!("return {}\n", v)),
                        Stmt::Await(_) => unreachable!("nested await handled earlier"),
                        Stmt::If { .. } | Stmt::While { .. } | Stmt::ForRange { .. } => {
                            out.push_str("/* unsupported await nesting */\n")
                        }
                    }
                }
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    out.push_str(&format!(
                        "    if {} {{ {} }}",
                        format_condition(condition),
                        format_block_inline(then_branch)
                    ));
                    if !else_branch.is_empty() {
                        out.push_str(" else { ");
                        out.push_str(&format_block_inline(else_branch));
                        out.push_str(" }");
                    }
                    out.push('\n');
                }
                Stmt::While { condition, body } => {
                    out.push_str(&format!(
                        "    while {} {{ {} }}\n",
                        format_condition(condition),
                        format_block_inline(body)
                    ));
                }
                Stmt::ForRange {
                    var,
                    start,
                    end,
                    body,
                } => {
                    out.push_str(&format!(
                        "    for {} in {}..{} {{ {} }}\n",
                        var,
                        start,
                        end,
                        format_block_inline(body)
                    ));
                }
                Stmt::Print(text) => out.push_str(&format!("    print(\"{}\")\n", text)),
                Stmt::Log { level, message } => {
                    let level = match level {
                        LogLevel::Info => "info",
                        LogLevel::Warn => "warn",
                        LogLevel::Error => "error",
                    };
                    out.push_str(&format!("    log.{}(\"{}\")\n", level, message));
                }
                Stmt::SleepMs(ms) => out.push_str(&format!("    time.sleep({})\n", ms)),
                Stmt::TimeNow => out.push_str("    time.now()\n"),
                Stmt::FsReadFile { path } => {
                    out.push_str(&format!("    fs.readFile(\"{}\")\n", path));
                }
                Stmt::FsWriteFile { path, contents } => {
                    out.push_str(&format!(
                        "    fs.writeFile(\"{}\", \"{}\")\n",
                        path, contents
                    ));
                }
                Stmt::Call(name) => {
                    out.push_str(&format!("    {}()\n", name));
                }
                Stmt::ReturnInt(v) => out.push_str(&format!("    return {}\n", v)),
            }
        }
        out.push_str("}\n");
    }
    out
}
const SAMPLE_MAIN: &str = r#"// VoltTS v0.1 sample
// Goal:
//   - TS-like readability
//   - small language surface
//   - native-first via C output
//   - explicit nullability
//   - Result-based error handling

import { fs, log, time } from "std"
import { logHelper } from "./support/log_helper.vts"

export async fn main() {
    log.info("booting VoltTS prototype (async demo)")
    print("unix epoch (ms):")
    await time.now()
    log.warn("demo sleep (await)")
    await time.sleep(20)
    await logHelper()
    log.info("writing sample file")
    await fs.writeFile("sample.txt", "hello from VoltTS async")
    log.info("reading sample file")
    await fs.readFile("sample.txt")
    log.info("demo done")
    log.error("demo complete")
}
"#;

fn format_condition(cond: &Condition) -> String {
    match cond {
        Condition::BoolLiteral(true) => "true".to_string(),
        Condition::BoolLiteral(false) => "false".to_string(),
    }
}

fn format_block_inline(stmts: &[Stmt]) -> String {
    stmts
        .iter()
        .map(|s| match s {
            Stmt::Print(text) => format!("print(\"{}\")", text),
            Stmt::Log { level, message } => {
                let level = match level {
                    LogLevel::Info => "info",
                    LogLevel::Warn => "warn",
                    LogLevel::Error => "error",
                };
                format!("log.{}(\"{}\")", level, message)
            }
            Stmt::SleepMs(ms) => format!("time.sleep({})", ms),
            Stmt::TimeNow => "time.now()".to_string(),
            Stmt::FsReadFile { path } => format!("fs.readFile(\"{}\")", path),
            Stmt::FsWriteFile { path, contents } => {
                format!("fs.writeFile(\"{}\", \"{}\")", path, contents)
            }
            Stmt::Call(name) => format!("{}()", name),
            Stmt::ReturnInt(v) => format!("return {}", v),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut rendered = format!(
                    "if {} {{ {} }}",
                    format_condition(condition),
                    format_block_inline(then_branch)
                );
                if !else_branch.is_empty() {
                    rendered.push_str(" else { ");
                    rendered.push_str(&format_block_inline(else_branch));
                    rendered.push_str(" }");
                }
                rendered
            }
            Stmt::While { condition, body } => format!(
                "while {} {{ {} }}",
                format_condition(condition),
                format_block_inline(body)
            ),
            Stmt::ForRange {
                var,
                start,
                end,
                body,
            } => format!(
                "for {} in {}..{} {{ {} }}",
                var,
                start,
                end,
                format_block_inline(body)
            ),
            Stmt::Await(inner) => format!("await {}", format_block_inline(&[*inner.clone()])),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

const SAMPLE_HELPER: &str = r#"import { log, time } from "std"

export async fn logHelper() {
    log.info("helper start")
    await time.sleep(10)
    log.warn("helper end")
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_function_requires_explicit_return() {
        let src = r#"
fn foo(): int {
    log.info("missing return")
}
"#;
        let program = parse_program(src).expect("parse program");
        let err = type_check(&program).expect_err("type check should fail");
        assert!(
            err.to_string().contains("return"),
            "expected missing return diagnostic, got: {err}"
        );
    }

    #[test]
    fn main_requires_annotation() {
        let src = r#"
fn main() {
    print("ok")
}
"#;
        let program = parse_program(src).expect("parse program");
        let err = type_check(&program).expect_err("main without type should fail");
        assert!(err
            .to_string()
            .contains("main は戻り値型として int もしくは void を明示してください"));
    }

    #[test]
    fn main_with_int_annotation_can_omit_return() {
        let src = r#"
fn main(): int {
    print("ok")
}
"#;
        let program = parse_program(src).expect("parse program");
        type_check(&program).expect("main with int annotation can omit return");
    }

    #[test]
    fn main_with_void_annotation_compiles() {
        let src = r#"
fn main(): void {
    print("ok")
}
"#;
        let program = parse_program(src).expect("parse program");
        type_check(&program).expect("main with void annotation is allowed");
    }
}
