use std::rc::Rc;

use crate::{
    ast::Expr,
    runtime::{eval::Eval, heap::HeapObject, scope::Scope, value::ShiroValue, RunContext},
};

fn eval_contact(vec: &Vec<Box<Expr>>, scope: Rc<Scope>, ctx: &mut RunContext) -> String {
    let mut str = String::new();
    for arg in vec {
        str.push_str(arg.eval(scope.clone(), ctx).coerce_string().as_str());
        str.push(' ');
    }
    return str;
}

pub fn lib(obj: &mut HeapObject) {
    obj.try_insert_fun("hello_native", |_, _, _| {
        ShiroValue::String("hello from the other side".to_string())
    });
    obj.try_insert_fun("println", |args, scope, ctx| {
        println!("{}", eval_contact(args, scope, ctx));
        ShiroValue::Null
    });
    obj.try_insert_fun("print", |args, scope, ctx| {
        print!("{}", eval_contact(args, scope, ctx));
        ShiroValue::Null
    });
}
