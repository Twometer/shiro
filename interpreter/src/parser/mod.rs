mod preproc;

use std::{fs, path::Path};

use codespan_reporting::files::SimpleFiles;
use lalrpop_util::ParseError;

use crate::{ast::Expr, diag::ShiroError, shiro::ChunkParser};

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

    pub fn open(path: &str) -> Result<Self, ShiroError> {
        let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
        let file_content = fs::read_to_string(path);
        match file_content {
            Ok(file_content) => Ok(Self {
                name: file_name.to_string(),
                content: file_content,
            }),
            Err(_) => Err(ShiroError::ModuleNotFound {
                path: path.to_string(),
            }),
        }
    }
}

pub fn parse(files: &mut SimpleFiles<String, String>, file: CodeFile) -> Result<Chunk, ShiroError> {
    let code = preproc::preprocess_code(&file.content);

    let file_id = files.add(file.name, code.clone());
    let parse_result = ChunkParser::new().parse(&code);

    return match parse_result {
        Ok(chunk) => Ok(chunk),
        Err(e) => Err(match e {
            ParseError::InvalidToken { location } => ShiroError::InvalidToken {
                file_id,
                range: location..location + 1,
                token: code.chars().nth(location).unwrap(),
            },
            ParseError::UnrecognizedEOF { location, expected } => ShiroError::UnrecognizedEOF {
                file_id,
                range: location..location + 1,
                expected,
            },
            ParseError::UnrecognizedToken { token, expected } => ShiroError::UnrecognizedToken {
                file_id,
                range: (token.0)..(token.2),
                token: token.1.to_string(),
                expected,
            },
            ParseError::ExtraToken { token } => ShiroError::ExtraToken {
                file_id,
                range: (token.0)..(token.2),
                token: token.1.to_string(),
            },
            ParseError::User { error } => ShiroError::GenericParserError(error.to_string()),
        }),
    };
}
