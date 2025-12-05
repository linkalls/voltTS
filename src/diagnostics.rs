use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[allow(dead_code)]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VoltError {
    #[error("parse error: {message} ({location})")]
    Parse {
        message: String,
        location: SourceLocation,
    },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("build error: {message}")]
    Build { message: String },
    #[error("codegen error: {message}")]
    Codegen { message: String },
}

pub type VoltResult<T> = Result<T, VoltError>;

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, col {}", self.line, self.column)
    }
}

#[allow(dead_code)]
impl VoltError {
    pub fn parse(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::Parse {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    pub fn build(message: impl Into<String>) -> Self {
        Self::Build {
            message: message.into(),
        }
    }

    pub fn codegen(message: impl Into<String>) -> Self {
        Self::Codegen {
            message: message.into(),
        }
    }
}
