use crate::runtime::{heap::HeapObject, value::ShiroValue};

pub fn lib(obj: &mut HeapObject) {
    obj.try_insert_fun("hello_native", |_, _, _, _| {
        ShiroValue::String("hello from the other side".to_string())
    });
}
