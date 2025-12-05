use crate::ast::{LogLevel, Program, Stmt};

pub fn format_program(program: &Program) -> String {
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
            Some(rt) => out.push_str(&format!("{async_prefix}fn {}(): {} {{\n", func.name, rt)),
            None => out.push_str(&format!("{async_prefix}fn {}() {{\n", func.name)),
        }
        for stmt in &func.body {
            match stmt {
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
                    }
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
                Stmt::Call(name) => out.push_str(&format!("    {}()\n", name)),
                Stmt::ReturnInt(v) => out.push_str(&format!("    return {}\n", v)),
            }
        }
        out.push_str("}\n\n");
    }

    out
}
