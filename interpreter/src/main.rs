pub mod ast;
mod runtime;
mod stdlib;

use std::fs;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub shiro);

fn main() {
    let code = fs::read_to_string("../examples/simple.shiro").unwrap();
    let result = runtime::eval::eval(&code);
    dbg!(result);
}
