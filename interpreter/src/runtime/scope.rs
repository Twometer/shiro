use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::{NativeFunctionPtr, ShiroValue};

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    vars: RefCell<HashMap<String, ShiroValue>>,
}

impl Scope {
    pub fn new(parent: Option<Rc<Scope>>) -> Scope {
        Scope {
            parent,
            vars: RefCell::new(HashMap::new()),
        }
    }

    pub fn find(&self, name: &String) -> ShiroValue {
        if !self.vars.borrow().contains_key(name) {
            match &self.parent {
                Some(parent) => parent.find(name),
                _ => ShiroValue::Null,
            }
        } else {
            self.vars.borrow()[name].clone()
        }
    }

    pub fn put(&self, name: &String, val: ShiroValue) {
        self.vars.borrow_mut().insert(name.to_string(), val);
    }

    pub fn register_native_function(&self, name: &str, ptr: NativeFunctionPtr) {
        self.put(&name.to_string(), ShiroValue::NativeFunction(ptr));
    }
}
