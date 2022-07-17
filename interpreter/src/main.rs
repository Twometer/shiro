use std::env;

use diag::ShiroError;
use lalrpop_util::lalrpop_mod;
use runtime::{value::ShiroValue, Runtime};

use crate::parser::CodeFile;

pub mod ast;
mod diag;
mod parser;
mod runtime;
mod stdlib;

lalrpop_mod!(pub shiro);

fn run_file(rt: &mut Runtime, path: &str) -> Result<ShiroValue, ShiroError> {
    let file = CodeFile::open(path)?;
    rt.eval(file)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("shiro: No input file specified");
        return;
    }

    let mut rt = Runtime::new();
    let result = run_file(&mut rt, &args[1]);

    match result {
        Ok(result) => match result {
            ShiroValue::Null => {}
            _ => println!("\n{}", result.coerce_string()),
        },
        Err(error) => rt.report_error(error),
    }
}
