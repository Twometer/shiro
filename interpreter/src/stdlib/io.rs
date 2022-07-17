use std::rc::Rc;

use crate::{
    ast::Expr,
    diag::ShiroError,
    runtime::{eval::Eval, heap::HeapObject, scope::Scope, value::ShiroValue, Runtime},
};

fn eval_contact(
    vec: &Vec<Box<Expr>>,
    scope: Rc<Scope>,
    ctx: &mut Runtime,
) -> Result<String, ShiroError> {
    let mut str = String::new();
    for arg in vec {
        str.push_str(arg.eval(scope.clone(), ctx)?.coerce_string().as_str());
        str.push(' ');
    }
    return Ok(str);
}

pub fn lib(obj: &mut HeapObject) {
    obj.must_insert_fun("hello_native", |_, _, _| {
        Ok(ShiroValue::String("hello from the other side".to_string()))
    });
    obj.must_insert_fun("println", |args, scope, ctx| {
        println!("{}", eval_contact(args, scope, ctx)?);
        Ok(ShiroValue::Null)
    });
    obj.must_insert_fun("print", |args, scope, ctx| {
        print!("{}", eval_contact(args, scope, ctx)?);
        Ok(ShiroValue::Null)
    });
}
