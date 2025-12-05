#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub names: Vec<String>,
    pub module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
    pub is_async: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Print(String),
    ReturnInt(i32),
    Log { level: LogLevel, message: String },
    SleepMs(u64),
    TimeNow,
    FsReadFile { path: String },
    FsWriteFile { path: String, contents: String },
    Call(String),
    Await(Box<Stmt>),
}
