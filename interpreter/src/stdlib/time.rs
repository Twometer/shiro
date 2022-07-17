use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtime::{heap::HeapObject, value::ShiroValue};

pub fn lib(obj: &mut HeapObject) {
    obj.try_insert_fun("millis", |_, _, _| {
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Get out of your time machine.");
        ShiroValue::Integer(unix_time.as_millis() as i64)
    })
}
