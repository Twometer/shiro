pub mod ast;
mod preproc;
mod runtime;

use crate::preproc::preprocess_code;
use lalrpop_util::lalrpop_mod;
use std::fs;

lalrpop_mod!(pub shiro);

fn main() {
    let code = fs::read_to_string("../lang/simple.shiro").unwrap();
    let preprocessed = preprocess_code(code.as_str());
    match shiro::ChunkParser::new().parse(&preprocessed.as_str()) {
        Ok(ast) => {
            let retval = runtime::evaluate(&ast);
            dbg!(&retval);
        }
        Err(e) => eprintln!("{}", e),
    }
}
