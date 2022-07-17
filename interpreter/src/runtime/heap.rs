use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::diag::ShiroError;

use super::{native::NativeFunctionPtr, value::ShiroValue};

const HEAP_DEBUG: bool = false;

#[derive(Debug)]
pub enum HeapValue {
    Object(HashMap<String, ShiroValue>),
    Array(Vec<ShiroValue>),
}

impl Display for HeapValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            HeapValue::Object(_) => f.write_str("object"),
            HeapValue::Array(_) => f.write_str("array"),
        }
    }
}

#[derive(Debug)]
pub struct HeapObject {
    address: u32,
    value: HeapValue,
}

impl HeapObject {
    pub fn address(&self) -> u32 {
        return self.address;
    }

    pub fn put(&mut self, key: ShiroValue, val: ShiroValue) -> Result<(), ShiroError> {
        match &mut self.value {
            HeapValue::Object(map) => {
                map.insert(key.coerce_string(), val);
            }
            HeapValue::Array(vec) => {
                let idx = key.coerce_integer() as usize;
                if idx < vec.len() {
                    vec[idx] = val;
                } else if idx == vec.len() {
                    vec.push(val);
                } else {
                    return Err(ShiroError::GenericRuntimeError(format!(
                        "Array index {} out of range",
                        idx
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn must_insert_fun(&mut self, key: &str, fun: NativeFunctionPtr) {
        self.try_insert_fun(key, fun)
            .expect("Failed to register native function")
    }

    pub fn try_insert_fun(&mut self, key: &str, fun: NativeFunctionPtr) -> Result<(), ShiroError> {
        self.try_insert(key, ShiroValue::NativeFunction(fun))
    }

    pub fn try_insert(&mut self, key: &str, val: ShiroValue) -> Result<(), ShiroError> {
        if let HeapValue::Object(map) = &mut self.value {
            map.insert(key.to_string(), val);
            Ok(())
        } else {
            Err(ShiroError::GenericRuntimeError(format!(
                "Cannot insert value into an `{}`",
                &self.value
            )))
        }
    }

    pub fn try_push(&mut self, val: ShiroValue) -> Result<(), ShiroError> {
        if let HeapValue::Array(vec) = &mut self.value {
            vec.push(val);
            Ok(())
        } else {
            Err(ShiroError::GenericRuntimeError(format!(
                "Cannot append value to an `{}`",
                &self.value
            )))
        }
    }

    pub fn len(&self) -> usize {
        match &self.value {
            HeapValue::Array(vec) => vec.len(),
            HeapValue::Object(map) => map.len(),
        }
    }

    pub fn keys(&self) -> Result<Vec<String>, ShiroError> {
        match &self.value {
            HeapValue::Array(_) => Err(ShiroError::GenericRuntimeError(
                "Cannot get keys of an array".to_string(),
            )),
            HeapValue::Object(map) => Ok(map.keys().map(|k| k.to_string()).collect()),
        }
    }

    pub fn get(&self, key: &ShiroValue) -> ShiroValue {
        match &self.value {
            HeapValue::Object(map) => map
                .get(&key.coerce_string())
                .unwrap_or(&ShiroValue::Null)
                .clone(),
            HeapValue::Array(vec) => {
                let idx = key.coerce_integer() as usize;
                if idx < vec.len() {
                    vec[idx].clone()
                } else {
                    ShiroValue::Null
                }
            }
        }
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

    pub fn alloc_object(&mut self) -> Rc<RefCell<HeapObject>> {
        self.alloc_heap_value(HeapValue::Object(HashMap::new()))
    }

    pub fn alloc_array(&mut self) -> Rc<RefCell<HeapObject>> {
        self.alloc_heap_value(HeapValue::Array(Vec::new()))
    }

    pub fn deref(&self, address: u32) -> Rc<RefCell<HeapObject>> {
        self.objects[&address].clone()
    }

    pub fn gc(&mut self) {
        if HEAP_DEBUG {
            println!("[gc] running cycle");
            dbg!(&self);
        }
        self.objects.retain(|addr, obj| {
            let ct = Rc::strong_count(&obj);
            if HEAP_DEBUG {
                println!("[gc] #{} has {} references", addr, ct);
            }
            ct > 1
        });
        // TODO cycle detection
    }

    fn new_addr(&mut self) -> u32 {
        let addr = self.addr_ctr;
        self.addr_ctr += 1;
        return addr;
    }

    fn alloc_heap_value(&mut self, value: HeapValue) -> Rc<RefCell<HeapObject>> {
        let address = self.new_addr();
        let obj = Rc::new(RefCell::new(HeapObject { address, value }));
        self.objects.insert(address, obj.clone());
        obj
    }
}
