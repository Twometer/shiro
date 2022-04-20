use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::ShiroValue;

#[derive(Debug)]
pub struct HeapObject {
    address: u32,
    value: HashMap<String, ShiroValue>,
    references: u32,
}

impl HeapObject {
    pub fn address(&self) -> u32 {
        return self.address;
    }
    pub fn put(&mut self, key: &String, val: ShiroValue) {
        self.value.insert(key.to_string(), val);
    }
    pub fn get(&self, key: &String) -> ShiroValue {
        self.value[key].clone()
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
        obj.borrow_mut().references += 1;
        obj
    }
    pub fn return_ref(&self, address: u32) {
        let obj = self.objects[&address].clone();
        let mut obj = obj.borrow_mut();
        if obj.references == 0 {
            panic!("References imbalanced");
        }
        obj.references -= 1;
    }
    pub fn alloc(&mut self) -> Rc<RefCell<HeapObject>> {
        let addr = self.addr_ctr;
        self.addr_ctr += 1;

        let obj = Rc::new(RefCell::new(HeapObject {
            address: addr,
            value: HashMap::new(),
            references: 0,
        }));
        self.objects.insert(addr, obj.clone());
        obj
    }
    pub fn gc(&mut self) {
        // TODO
    }
}
