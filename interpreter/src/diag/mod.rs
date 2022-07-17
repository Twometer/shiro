use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum ShiroError {
    ModuleNotFound,
    InvalidToken,
    UnrecognizedEOF,
    UnrecognizedToken,
    ExtraToken,
    GenericParserError,
    UnknownOperation,
    UnknownInstruction,
}

impl ShiroError {
    pub fn error_code(&self) -> String {
        match self {
            ShiroError::ModuleNotFound => "E0101",
            ShiroError::InvalidToken => "E0201",
            ShiroError::UnrecognizedEOF => "E0202",
            ShiroError::UnrecognizedToken => "E0203",
            ShiroError::ExtraToken => "E0204",
            ShiroError::GenericParserError => "E0299",
            ShiroError::UnknownOperation => "E0301",
            ShiroError::UnknownInstruction => "E0302",
        }
        .to_string()
    }

    pub fn error_message(&self) -> String {
        todo!();
    }
}

impl Error for ShiroError {}

impl Display for ShiroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
