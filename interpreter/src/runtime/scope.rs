use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{native::NativeFunctionPtr, value::ShiroValue};

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

    pub fn get_by_val(&self, name: &ShiroValue) -> ShiroValue {
        let name_str = name.borrow_string();
        return self.get_by_str(name_str);
    }

    pub fn get_by_str(&self, name: &str) -> ShiroValue {
        if !self.vars.borrow().contains_key(name) {
            match &self.parent {
                Some(parent) => parent.get_by_str(name),
                _ => ShiroValue::Null,
            }
        } else {
            self.vars.borrow()[name].clone()
        }
    }

    pub fn put_by_str(&self, name: &str, val: ShiroValue, define: bool) {
        self.put_cascade(name.to_string(), val, define);
    }

    pub fn put_by_val(&self, name: &ShiroValue, val: ShiroValue, define: bool) {
        self.put_cascade(name.coerce_string(), val, define);
    }

    fn put_cascade(&self, name: String, val: ShiroValue, define: bool) -> bool {
        if !define && !self.vars.borrow().contains_key(&name) {
            if let Some(parent) = &self.parent {
                if parent.put_cascade(name, val, define) {
                    return true;
                }
            }
        } else {
            self.vars.borrow_mut().insert(name, val);
            return true;
        }
        false
    }

    pub fn register_native_function(&self, name: &str, ptr: NativeFunctionPtr) {
        self.vars
            .borrow_mut()
            .insert(name.to_string(), ShiroValue::NativeFunction(ptr));
    }
}
