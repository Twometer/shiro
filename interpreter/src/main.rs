use lalrpop_util::lalrpop_mod;

use crate::parser::CodeFile;

pub mod ast;
mod parser;
mod runtime;
mod stdlib;

lalrpop_mod!(pub shiro);

fn main() {
    let file = CodeFile::open("../examples/test_full.shiro");
    let result = runtime::eval::eval(file);
    println!("> {}", result.coerce_string());
}
