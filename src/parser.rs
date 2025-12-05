use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{Function, Import, LogLevel, Program, Stmt};
use crate::diagnostics::{VoltError, VoltResult};

pub fn parse_program(source: &str) -> VoltResult<Program> {
    let mut imports = Vec::new();
    let mut functions = Vec::new();
    let mut lines = source.lines().enumerate().peekable();

    while let Some((idx, raw_line)) = lines.next() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("import ") {
            imports.push(parse_import(trimmed, line_no)?);
            continue;
        }

        if trimmed.starts_with("export fn")
            || trimmed.starts_with("fn")
            || trimmed.starts_with("export async fn")
            || trimmed.starts_with("async fn")
        {
            let signature = trimmed.strip_prefix("export ").unwrap_or(trimmed);
            let (name, return_type, is_async) = parse_signature(signature, line_no)?;

            // consume until '{'
            if !signature.contains('{') {
                while let Some((_, next_line)) = lines.next() {
                    if next_line.contains('{') {
                        break;
                    }
                }
            }

            let mut body = Vec::new();
            for (body_idx, body_line) in &mut lines {
                let body_line_no = body_idx + 1;
                let body_trimmed = body_line.trim();
                if body_trimmed.starts_with('}') {
                    break;
                }
                if body_trimmed.is_empty() || body_trimmed.starts_with("//") {
                    continue;
                }
                body.push(parse_stmt(body_trimmed, body_line_no)?);
            }

            functions.push(Function {
                name,
                return_type,
                body,
                is_async,
            });
        }
    }

    if functions.is_empty() {
        return Err(VoltError::parse("no functions found", 1, 1));
    }

    Ok(Program { imports, functions })
}

pub fn load_program(entry: &Path) -> VoltResult<Program> {
    let mut visited = HashSet::new();
    load_program_recursive(entry, &mut visited)
}

fn load_program_recursive(path: &Path, visited: &mut HashSet<PathBuf>) -> VoltResult<Program> {
    let abs = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(abs.clone()) {
        return Ok(Program {
            imports: Vec::new(),
            functions: Vec::new(),
        });
    }

    let source = fs::read_to_string(&abs)
        .map_err(|e| VoltError::io(format!("failed to read {}: {e}", abs.display())))?;
    let mut program = parse_program(&source)?;

    let base_dir = abs
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut extra_functions = Vec::new();
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

fn parse_import(line: &str, line_no: usize) -> VoltResult<Import> {
    let without_suffix = line.trim().trim_end_matches(';').trim();
    let without_prefix = without_suffix
        .strip_prefix("import")
        .ok_or_else(|| VoltError::parse(format!("invalid import syntax: {line}"), line_no, 1))?
        .trim();

    let (names_part, rest) = without_prefix.split_once('}').ok_or_else(|| {
        VoltError::parse(
            format!("import must include a closing brace ('}}'): {line}"),
            line_no,
            1,
        )
    })?;
    let names_block = names_part.strip_prefix('{').ok_or_else(|| {
        VoltError::parse(format!("import must start with '{{': {line}"), line_no, 1)
    })?;
    let names: Vec<String> = names_block
        .split(',')
        .map(|n| n.trim())
        .filter(|n| !n.is_empty())
        .map(|n| n.to_string())
        .collect();

    if names.is_empty() {
        return Err(VoltError::parse(
            format!("import must list at least one name: {line}"),
            line_no,
            1,
        ));
    }

    let module = rest
        .trim()
        .strip_prefix("from")
        .ok_or_else(|| VoltError::parse(format!("import missing 'from': {line}"), line_no, 1))?
        .trim()
        .trim_matches('"')
        .to_string();

    if module.is_empty() {
        return Err(VoltError::parse(
            format!("import module path is empty: {line}"),
            line_no,
            1,
        ));
    }

    Ok(Import { names, module })
}

fn parse_signature(signature: &str, line_no: usize) -> VoltResult<(String, Option<String>, bool)> {
    let mut without_prefix = signature.trim_start_matches("export").trim();
    let mut is_async = false;
    if without_prefix.starts_with("async") {
        is_async = true;
        without_prefix = without_prefix.trim_start_matches("async").trim();
    }
    without_prefix = without_prefix.trim_start_matches("fn").trim();

    let name_and_rest: Vec<&str> = without_prefix.splitn(2, '(').collect();
    if name_and_rest.len() < 2 {
        return Err(VoltError::parse(
            format!("invalid function signature: {signature}"),
            line_no,
            1,
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
            Some(rt.to_string())
        }
    } else {
        None
    };

    Ok((name, return_type, is_async))
}

fn parse_stmt(line: &str, line_no: usize) -> VoltResult<Stmt> {
    let trimmed = line.trim().trim_end_matches(';');
    if let Some(rest) = trimmed.strip_prefix("await ") {
        let inner = parse_stmt_core(rest, line_no)?;
        return Ok(Stmt::Await(Box::new(inner)));
    }

    parse_stmt_core(trimmed, line_no)
}

fn parse_stmt_core(trimmed: &str, line_no: usize) -> VoltResult<Stmt> {
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
            .ok_or_else(|| VoltError::parse(format!("invalid log call: {trimmed}"), line_no, 1))?;
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
                return Err(VoltError::parse(
                    format!("unsupported log level '{level}'; use log.info/log.warn/log.error"),
                    line_no,
                    1,
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
        let (path, contents) = inner
            .split_once(',')
            .ok_or_else(|| VoltError::parse("fs.writeFile expects path, contents", line_no, 1))?;
        return Ok(Stmt::FsWriteFile {
            path: path
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .replace('\"', "\""),
            contents: contents
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .replace('\"', "\""),
        });
    }

    if let Some(rest) = trimmed.strip_prefix("time.sleep(") {
        let inner = rest.trim_end_matches(')');
        let ms: u64 = inner.trim().parse().map_err(|_| {
            VoltError::parse(
                format!("time.sleep expects integer milliseconds: {inner}"),
                line_no,
                1,
            )
        })?;
        return Ok(Stmt::SleepMs(ms));
    }

    if trimmed == "time.now()" {
        return Ok(Stmt::TimeNow);
    }

    if let Some(rest) = trimmed.strip_prefix("return ") {
        let val: i32 = rest.trim().parse().map_err(|_| {
            VoltError::parse(
                format!("only int return values supported for now: {rest}"),
                line_no,
                1,
            )
        })?;
        return Ok(Stmt::ReturnInt(val));
    }

    if let Some(rest) = trimmed.strip_prefix("await ") {
        let inner = parse_stmt_core(rest, line_no)?;
        return Ok(Stmt::Await(Box::new(inner)));
    }

    if trimmed.ends_with("()") {
        return Ok(Stmt::Call(trimmed.trim_end_matches("()").to_string()));
    }

    Err(VoltError::parse(
        format!("unsupported statement: {trimmed}"),
        line_no,
        1,
    ))
}
