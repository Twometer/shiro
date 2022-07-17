use std::env;

use crate::runtime::{eval::Eval, heap::HeapObject, value::ShiroValue};

pub fn lib(obj: &mut HeapObject) {
    obj.must_insert_fun("getenv", |args, scope, ctx| {
        assert!(args.len() >= 1);

        let key = args[0].eval(scope, ctx)?.coerce_string();
        let val = env::var(&key).unwrap_or("".to_string());

        Ok(ShiroValue::String(val))
    });
}
