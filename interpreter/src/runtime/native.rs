use std::{collections::HashMap, panic, rc::Rc};

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
        return name.chars().nth(0) == Some('@');
    }
    pub fn load(&self, name: &str, heap: &mut Heap) -> ShiroValue {
        let registered = self.registry.get(name);
        if matches!(registered, None) {
            panic!("Cannot find native library {}", name);
        }

        let obj = heap.alloc_object();
        let mut obj = obj.borrow_mut();

        registered.unwrap()(&mut obj);

        ShiroValue::HeapRef(obj.address())
    }
    pub fn register_lib(&mut self, name: &str, creator: NativeLibCreator) {
        self.registry.insert(name.to_string(), creator);
    }
}

impl Default for NativeLibProvider {
    fn default() -> Self {
        let mut provider = Self::new();
        provider.register_lib("@std/io", stdlib::io::lib);
        provider
    }
}
