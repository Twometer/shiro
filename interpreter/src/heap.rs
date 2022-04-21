use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::ShiroValue;

#[derive(Debug)]
pub struct HeapObject {
    address: u32,
    value: HashMap<String, ShiroValue>,
}

impl HeapObject {
    pub fn address(&self) -> u32 {
        return self.address;
    }
    pub fn put(&mut self, key: &String, val: ShiroValue) {
        self.value.insert(key.to_string(), val);
    }
    pub fn get(&self, key: &String) -> ShiroValue {
        self.value.get(key).unwrap_or(&ShiroValue::Null).clone()
    }
}

#[derive(Debug)]
pub struct Heap {
    objects: HashMap<u32, Rc<RefCell<HeapObject>>>,
    addr_ctr: u32,
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            objects: HashMap::new(),
            addr_ctr: 1,
        }
    }
    pub fn deref(&self, address: u32) -> Rc<RefCell<HeapObject>> {
        let obj = self.objects[&address].clone();
        obj
    }
    pub fn alloc(&mut self) -> Rc<RefCell<HeapObject>> {
        let addr = self.addr_ctr;
        self.addr_ctr += 1;

        let obj = Rc::new(RefCell::new(HeapObject {
            address: addr,
            value: HashMap::new(),
        }));
        self.objects.insert(addr, obj.clone());
        obj
    }
    pub fn gc(&mut self) {
        self.objects.retain(|addr, obj| {
            let ct = Rc::strong_count(&obj);
            println!("[gc] #{} has {} references", addr, ct);
            ct > 1
        });
        // TODO cycle detection
    }
}
