mod preproc;

use std::{fs, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use lalrpop_util::ParseError;

use crate::{ast::Expr, shiro::ChunkParser};

pub type Chunk = Vec<Box<Expr>>;

pub struct CodeFile {
    name: String,
    content: String,
}

#[allow(dead_code)]
impl CodeFile {
    pub fn new(name: &str, content: &str) -> Self {
        Self {
            name: name.to_string(),
            content: content.to_string(),
        }
    }

    pub fn open(path: &str) -> Self {
        let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
        let file_content = fs::read_to_string(path)
            .expect(format!("Failed to open code file at {}", path).as_str());
        Self {
            name: file_name.to_string(),
            content: file_content,
        }
    }
}

pub struct ShiroParser {
    files: SimpleFiles<String, String>,
    diag_stream: StandardStream,
    diag_config: Config,
}

impl ShiroParser {
    pub fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            diag_stream: StandardStream::stderr(ColorChoice::Auto),
            diag_config: codespan_reporting::term::Config::default(),
        }
    }

    pub fn parse(&mut self, file: CodeFile) -> Result<Chunk, Diagnostic<usize>> {
        let code = preproc::preprocess_code(&file.content);

        let file_id = self.files.add(file.name, code.clone());
        let parse_result = ChunkParser::new().parse(&code);

        return match parse_result {
            Ok(chunk) => Ok(chunk),
            Err(e) => Err(match e {
                ParseError::InvalidToken { location } => Diagnostic::error()
                    .with_message("Encountered an invalid token")
                    .with_labels(vec![Label::primary(file_id, location..location + 1)
                        .with_message(format!(
                            "Unexpected `{}`",
                            code.chars().nth(location).unwrap()
                        ))])
                    .with_code("E0101"),

                ParseError::UnrecognizedEOF { location, expected } => Diagnostic::error()
                    .with_message("Reached EOF unexpectedly")
                    .with_labels(vec![Label::primary(file_id, location..location + 1)
                        .with_message("Unexpected EOF")])
                    .with_notes(vec![format!(
                        "Expected one of these: {}",
                        expected.join(", ")
                    )])
                    .with_code("E0102"),

                ParseError::UnrecognizedToken { token, expected } => Diagnostic::error()
                    .with_message("Encountered an unrecognized token")
                    .with_labels(vec![Label::primary(file_id, token.0..token.2)
                        .with_message(format!("Unexpected `{}`", token.1))])
                    .with_notes(vec![format!(
                        "Expected one of these: {}",
                        expected.join(", ")
                    )])
                    .with_code("E0103"),

                ParseError::ExtraToken { token } => Diagnostic::error()
                    .with_message("Encountered an extra token")
                    .with_labels(vec![Label::primary(file_id, token.0..token.2)
                        .with_message(format!("Unexpected `{}`", token.1))])
                    .with_code("E0104"),

                ParseError::User { error } => Diagnostic::error()
                    .with_message("User defined parser error")
                    .with_notes(vec![error.to_string()])
                    .with_code("E0199"),
            }),
        };
    }

    pub fn report_diag(&self, diag: Diagnostic<usize>) {
        term::emit(
            &mut self.diag_stream.lock(),
            &self.diag_config,
            &self.files,
            &diag,
        )
        .expect("Failed to print diagnostics");
    }
}
