use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};

#[derive(Debug)]
pub enum ShiroError {
    ModuleNotFound {
        path: String,
    },
    InvalidToken {
        file_id: usize,
        range: Range<usize>,
        token: char,
    },
    UnrecognizedEOF {
        file_id: usize,
        range: Range<usize>,
        expected: Vec<String>,
    },
    UnrecognizedToken {
        file_id: usize,
        range: Range<usize>,
        token: String,
        expected: Vec<String>,
    },
    ExtraToken {
        file_id: usize,
        range: Range<usize>,
        token: String,
    },
    GenericParserError(String),
    UnknownInstruction,
    GenericRuntimeError(String),
}

impl ShiroError {
    pub fn error_code(&self) -> String {
        match self {
            ShiroError::ModuleNotFound { .. } => "E0101",
            ShiroError::InvalidToken { .. } => "E0201",
            ShiroError::UnrecognizedEOF { .. } => "E0202",
            ShiroError::UnrecognizedToken { .. } => "E0203",
            ShiroError::ExtraToken { .. } => "E0204",
            ShiroError::GenericParserError(_) => "E0299",
            ShiroError::UnknownInstruction => "E0301",
            ShiroError::GenericRuntimeError(_) => "E0399",
        }
        .to_string()
    }
}

impl Into<Diagnostic<usize>> for ShiroError {
    fn into(self) -> Diagnostic<usize> {
        let diag = Diagnostic::error().with_code(self.error_code());

        match self {
            ShiroError::ModuleNotFound { path } => {
                diag.with_message(format!("Module at `{}` not found", path))
            }
            ShiroError::InvalidToken {
                file_id,
                range,
                token,
            } => diag
                .with_message(format!("Invalid token `{}`", token))
                .with_labels(vec![Label::primary(file_id, range)]),
            ShiroError::UnrecognizedEOF {
                file_id,
                range,
                expected,
            } => diag
                .with_message("Unexpected EOF")
                .with_labels(vec![Label::primary(file_id, range)])
                .with_notes(vec![format!("Expected one of: {}", expected.join(", "))]),
            ShiroError::UnrecognizedToken {
                file_id,
                range,
                token,
                expected,
            } => diag
                .with_message(format!("Unexpected token `{}`", token))
                .with_labels(vec![Label::primary(file_id, range)])
                .with_notes(vec![format!("Expected one of: {}", expected.join(", "))]),
            ShiroError::ExtraToken {
                file_id,
                range,
                token,
            } => diag
                .with_message(format!("Unexpected token `{}`", token))
                .with_labels(vec![Label::primary(file_id, range)]),
            ShiroError::GenericParserError(err) => {
                diag.with_message(format!("Parser error: {}", err))
            }
            ShiroError::UnknownInstruction => diag.with_message("Unknown instruction"),
            ShiroError::GenericRuntimeError(err) => {
                diag.with_message(format!("Runtime error: {}", err))
            }
        }
    }
}
