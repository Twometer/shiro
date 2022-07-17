use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

use crate::{
    diag::ShiroError,
    parser::{parse, Chunk, CodeFile},
};

use self::{heap::Heap, native::NativeLibProvider};

pub mod eval;
pub mod heap;
mod native;
pub mod scope;
pub mod value;

pub struct Runtime {
    pub heap: Heap,
    pub libs: NativeLibProvider,
    files: SimpleFiles<String, String>,
    diag_stream: StandardStream,
    diag_config: Config,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            heap: Heap::new(),
            libs: NativeLibProvider::default(),
            files: SimpleFiles::new(),
            diag_stream: StandardStream::stderr(ColorChoice::Auto),
            diag_config: codespan_reporting::term::Config::default(),
        }
    }

    pub fn report_error(&self, error: ShiroError) {
        term::emit(
            &mut self.diag_stream.lock(),
            &self.diag_config,
            &self.files,
            &error.into(),
        )
        .expect("Failed to print diagnostics");
    }

    fn parse_file(&mut self, file: CodeFile) -> Result<Chunk, ShiroError> {
        parse(&mut self.files, file)
    }
}
