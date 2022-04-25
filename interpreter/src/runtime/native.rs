use std::{collections::HashMap, rc::Rc};

use crate::{ast::Expr, stdlib};

use super::{
    heap::{Heap, HeapObject},
    scope::Scope,
    value::ShiroValue,
    RunContext,
};

pub type NativeFunctionPtr =
    fn(args: &Vec<Box<Expr>>, scope: Rc<Scope>, ctx: &mut RunContext) -> ShiroValue;

pub type NativeLibCreator = fn(obj: &mut HeapObject);

pub struct NativeLibProvider {
    registry: HashMap<String, NativeLibCreator>,
}

impl NativeLibProvider {
    pub fn new() -> NativeLibProvider {
        NativeLibProvider {
            registry: HashMap::new(),
        }
    }
    pub fn is_native_lib(&self, name: &str) -> bool {
        return self.registry.contains_key(name);
    }
    pub fn load(&self, name: &str, heap: &mut Heap) -> ShiroValue {
        let creator = self.registry[name];

        let obj = heap.alloc_object();
        let mut obj = obj.borrow_mut();

        creator(&mut obj);

        ShiroValue::HeapRef(obj.address())
    }
    pub fn register_lib(&mut self, name: &str, creator: NativeLibCreator) {
        self.registry.insert(name.to_string(), creator);
    }
}

impl Default for NativeLibProvider {
    fn default() -> Self {
        let mut provider = Self::new();
        provider.register_lib("@std/io", stdlib::io);
        provider.register_lib("@std/os", stdlib::os);
        provider.register_lib("@std/net", stdlib::net);
        provider.register_lib("@std/time", stdlib::time);
        provider
    }
}
