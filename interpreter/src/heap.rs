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
        self.objects[&address].clone()
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
    fn gc(&mut self) {
        // TODO
    }
}
