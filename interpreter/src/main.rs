pub mod ast;
mod runtime;
mod stdlib;

use std::fs;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub shiro);

fn main() {
    let code = fs::read_to_string("../examples/test_full.shiro").unwrap();
    let result = runtime::eval::eval(&code);
    println!("{}", result.coerce_string());
}
