use std::path::Path;

use crate::ast::{LogLevel, Program, Stmt};

pub fn codegen_c(program: &Program, source_path: &Path) -> String {
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
    out.push_str("static VTS_UNUSED void vts_log_error(const char *msg) { printf(\"[error] %s\\n\", msg); }\n");
    out.push_str("static VTS_UNUSED void vts_sleep_ms(unsigned long ms) { usleep(ms * 1000); }\n");
    out.push_str("static VTS_UNUSED long long vts_time_now_ms(void) { struct timeval tv; gettimeofday(&tv, NULL); return (long long)tv.tv_sec * 1000 + tv.tv_usec / 1000; }\n\n");
    out.push_str("static VTS_UNUSED char *vts_fs_read_file(const char *path) { FILE *f = fopen(path, \"rb\"); if (!f) return NULL; if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return NULL; } long size = ftell(f); if (size < 0) { fclose(f); return NULL; } if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return NULL; } char *buf = (char *)malloc((size_t)size + 1); if (!buf) { fclose(f); return NULL; } size_t read = fread(buf, 1, (size_t)size, f); buf[read] = 0; fclose(f); return buf; }\n");
    out.push_str("static VTS_UNUSED int vts_fs_write_file(const char *path, const char *contents) { const char *slash = strrchr(path, '/'); if (slash) { size_t len = (size_t)(slash - path); if (len > 0) { char *dir = (char *)malloc(len + 1); if (!dir) return -1; memcpy(dir, path, len); dir[len] = 0; struct stat st; if (stat(dir, &st) != 0) { mkdir(dir, 0755); } free(dir); } } FILE *f = fopen(path, \"wb\"); if (!f) return -1; size_t len = strlen(contents); size_t written = fwrite(contents, 1, len, f); fclose(f); return written == len ? 0 : -1; }\n\n");

    out.push_str("// --- user prototypes ---\n");
    for func in &program.functions {
        let returns_int = func
            .return_type
            .as_deref()
            .map(|rt| rt == "int")
            .unwrap_or(false)
            || func.name == "main";
        let c_return = if returns_int { "int" } else { "void" };
        out.push_str(&format!("{} {}(void);\n", c_return, func.name));
    }
    out.push_str("\n");

    for func in &program.functions {
        let mut temp_counter = 0;
        if func.name == "main" {
            out.push_str("int main(void) {\n");
            let mut saw_return = false;
            for stmt in &func.body {
                emit_stmt(
                    &mut out,
                    stmt,
                    &mut temp_counter,
                    Some(&mut saw_return),
                    true,
                    true,
                );
            }
            if !saw_return {
                out.push_str("    return 0;\n");
            }
            out.push_str("}\n\n");
        } else {
            let returns_int = func
                .return_type
                .as_deref()
                .map(|rt| rt == "int")
                .unwrap_or(false);
            let c_return = if returns_int { "int" } else { "void" };
            out.push_str(&format!("{} {}(void) {{\n", c_return, func.name));
            let mut saw_return = false;
            for stmt in &func.body {
                emit_stmt(
                    &mut out,
                    stmt,
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
            } else if func.return_type.is_none() {
                out.push_str("    return;\n");
            }
            out.push_str("}\n\n");
        }
    }

    out
}

fn emit_stmt(
    out: &mut String,
    stmt: &Stmt,
    temp_counter: &mut usize,
    saw_return: Option<&mut bool>,
    is_main: bool,
    returns_int: bool,
) {
    match stmt {
        Stmt::Await(inner) => emit_stmt(out, inner, temp_counter, saw_return, is_main, returns_int),
        Stmt::Print(text) => {
            out.push_str(&format!(
                "    printf(\"{}\\n\");\n",
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
            *temp_counter += 1;
            out.push_str(&format!(
                "    long long vts_tmp_{0} = vts_time_now_ms(); printf(\"unix epoch (ms): %lld\\n\", vts_tmp_{0});\n",
                temp_counter
            ));
        }
        Stmt::FsReadFile { path } => {
            *temp_counter += 1;
            out.push_str(&format!(
                "    char *vts_tmp_{0} = vts_fs_read_file(\"{1}\"); if (!vts_tmp_{0}) {{ fprintf(stderr, \"[fs.readFile] failed: {1}\\n\"); {2} }} else {{ printf(\"%s\\n\", vts_tmp_{0}); free(vts_tmp_{0}); }}\n",
                temp_counter,
                path.replace('"', "\\\""),
                if returns_int { "return 1;" } else if is_main { "return 1;" } else { "return;" }
            ));
        }
        Stmt::FsWriteFile { path, contents } => {
            let ret = if returns_int {
                "return 1;"
            } else if is_main {
                "return 1;"
            } else {
                "return;"
            };
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
            if let Some(flag) = saw_return {
                *flag = true;
            }
            out.push_str(&format!("    return {};\n", v));
        }
    }
}
