use std::{cell::RefCell, collections::HashMap};

use crate::runtime::ShiroValue;

struct HeapObject {
    address: u32,
    value: HashMap<String, ShiroValue>,
}

struct Heap {
    objects: HashMap<i32, RefCell<HeapObject>>,
}

impl Heap {
    fn new() -> Heap {
        Heap {
            objects: HashMap::new(),
        }
    }
    fn deref(address: u32) {}
    fn alloc() {}
}
