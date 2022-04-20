use std::{
    cell::RefCell,
    cmp::min,
    cmp::Ordering::{Equal, Greater, Less},
    collections::HashMap,
    ops::{Add, Div, Mul, Rem, Sub},
    panic,
    rc::Rc,
    str::FromStr,
};

use crate::ast::{AssignOpcode, Expr, Opcode};
use crate::heap::Heap;

type NativeFunctionPtr = fn(scope: Rc<Scope>, heap: &mut Heap, args: &Vec<Box<Expr>>) -> ShiroValue;

#[derive(Clone)]
pub enum ShiroValue {
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Function {
        args: Vec<String>,
        body: Vec<Box<Expr>>,
    },
    NativeFunction(NativeFunctionPtr),
    Null,
    HeapRef(u32),
}

impl std::fmt::Display for ShiroValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiroValue::String(_) => write!(f, "String"),
            ShiroValue::Integer(_) => write!(f, "Integer"),
            ShiroValue::Decimal(_) => write!(f, "Decimal"),
            ShiroValue::Boolean(_) => write!(f, "Boolean"),
            ShiroValue::Function { .. } => write!(f, "Function"),
            ShiroValue::NativeFunction(_) => write!(f, "Function"),
            ShiroValue::Null => write!(f, "Null"),
            ShiroValue::HeapRef(_) => write!(f, "Object"),
        }
    }
}

impl std::fmt::Debug for ShiroValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Integer(arg0) => f.debug_tuple("Integer").field(arg0).finish(),
            Self::Decimal(arg0) => f.debug_tuple("Decimal").field(arg0).finish(),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Function { args, body } => f
                .debug_struct("Function")
                .field("args", args)
                .field("body", body)
                .finish(),
            Self::NativeFunction(_) => write!(f, "NativeFunction"),
            Self::HeapRef(addr) => f.debug_tuple("HeapRef").field(addr).finish(),
            Self::Null => write!(f, "Null"),
        }
    }
}

impl ShiroValue {
    fn coerce_integer(&self) -> i32 {
        match self {
            ShiroValue::String(s) => i32::from_str(s.as_str()).unwrap(),
            ShiroValue::Decimal(d) => *d as i32,
            ShiroValue::Integer(d) => *d,
            ShiroValue::Boolean(d) => {
                if *d {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn coerce_string(&self) -> String {
        match self {
            ShiroValue::String(s) => s.to_string(),
            ShiroValue::Decimal(v) => v.to_string(),
            ShiroValue::Integer(v) => v.to_string(),
            ShiroValue::Boolean(v) => v.to_string(),
            ShiroValue::Function { .. } => "[function]".to_string(),
            ShiroValue::NativeFunction { .. } => "[native function]".to_string(),
            ShiroValue::HeapRef(_) => "[object]".to_string(),
            _ => "null".to_string(),
        }
    }

    fn coerce_decimal(&self) -> f32 {
        match self {
            ShiroValue::String(s) => f32::from_str(s.as_str()).unwrap(),
            ShiroValue::Decimal(d) => *d,
            ShiroValue::Integer(d) => *d as f32,
            ShiroValue::Boolean(d) => {
                if *d {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn coerce_boolean(&self) -> bool {
        match self {
            ShiroValue::String(s) => !s.is_empty(),
            ShiroValue::Decimal(d) => *d != 0.0,
            ShiroValue::Integer(i) => *i != 0,
            ShiroValue::Boolean(b) => *b,
            ShiroValue::Function { .. } => true,
            ShiroValue::NativeFunction { .. } => true,
            ShiroValue::HeapRef(_) => true,

            _ => false,
        }
    }
}

impl Add for ShiroValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match &self {
            ShiroValue::String(str) => ShiroValue::String(str.to_owned() + &rhs.coerce_string()),
            ShiroValue::Integer(i) => ShiroValue::Integer(*i + rhs.coerce_integer()),
            ShiroValue::Boolean(_) => {
                ShiroValue::Integer(self.coerce_integer() + rhs.coerce_integer())
            }
            ShiroValue::Decimal(d) => ShiroValue::Decimal(*d + rhs.coerce_decimal()),
            _ => ShiroValue::Null,
        }
    }
}

impl Sub for ShiroValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match &self {
            ShiroValue::Integer(i) => ShiroValue::Integer(*i - rhs.coerce_integer()),
            ShiroValue::Boolean(_) => {
                ShiroValue::Integer(self.coerce_integer() - rhs.coerce_integer())
            }
            ShiroValue::Decimal(d) => ShiroValue::Decimal(*d - rhs.coerce_decimal()),
            _ => ShiroValue::Null,
        }
    }
}

impl Div for ShiroValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match &self {
            ShiroValue::Integer(i) => ShiroValue::Integer(*i / rhs.coerce_integer()),
            ShiroValue::Boolean(_) => {
                ShiroValue::Integer(self.coerce_integer() / rhs.coerce_integer())
            }
            ShiroValue::Decimal(d) => ShiroValue::Decimal(*d / rhs.coerce_decimal()),
            _ => ShiroValue::Null,
        }
    }
}

impl Mul for ShiroValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match &self {
            ShiroValue::Integer(i) => ShiroValue::Integer(*i * rhs.coerce_integer()),
            ShiroValue::Boolean(_) => {
                ShiroValue::Integer(self.coerce_integer() * rhs.coerce_integer())
            }
            ShiroValue::Decimal(d) => ShiroValue::Decimal(*d * rhs.coerce_decimal()),
            _ => ShiroValue::Null,
        }
    }
}

impl Rem for ShiroValue {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match &self {
            ShiroValue::Integer(i) => ShiroValue::Integer(*i % rhs.coerce_integer()),
            ShiroValue::Decimal(d) => ShiroValue::Decimal(*d % rhs.coerce_decimal()),
            ShiroValue::Boolean(_) => ShiroValue::Integer(self.coerce_integer()),
            _ => ShiroValue::Null,
        }
    }
}

impl PartialEq for ShiroValue {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            ShiroValue::String(str) => *str == other.coerce_string(),
            ShiroValue::Integer(i) => *i == other.coerce_integer(),
            ShiroValue::Boolean(b) => *b == other.coerce_boolean(),
            ShiroValue::Decimal(d) => *d == other.coerce_decimal(),
            // TODO: function equality
            _ => false,
        }
    }
}

impl PartialOrd for ShiroValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match &self {
            ShiroValue::String(str) => Some(str.cmp(&other.coerce_string())),
            ShiroValue::Integer(i) => Some(i.cmp(&other.coerce_integer())),
            ShiroValue::Boolean(b) => Some(b.cmp(&other.coerce_boolean())),
            ShiroValue::Decimal(d) => d.partial_cmp(&other.coerce_decimal()),
            _ => None,
        }
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Less))
    }

    fn le(&self, other: &Self) -> bool {
        // Pattern `Some(Less | Eq)` optimizes worse than negating `None | Some(Greater)`.
        // FIXME: The root cause was fixed upstream in LLVM with:
        // https://github.com/llvm/llvm-project/commit/9bad7de9a3fb844f1ca2965f35d0c2a3d1e11775
        // Revert this workaround once support for LLVM 12 gets dropped.
        !matches!(self.partial_cmp(other), None | Some(Greater))
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater | Equal))
    }
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<Scope>>,
    vars: RefCell<HashMap<String, ShiroValue>>,
}

impl Drop for Scope {
    fn drop(&mut self) {
        // todo release all heaprefs
    }
}

impl Scope {
    fn new(parent: Option<Rc<Scope>>) -> Scope {
        Scope {
            parent,
            vars: RefCell::new(HashMap::new()),
        }
    }
    fn find(&self, name: &String) -> ShiroValue {
        if !self.vars.borrow().contains_key(name) {
            match &self.parent {
                Some(parent) => parent.find(name),
                _ => ShiroValue::Null,
            }
        } else {
            self.vars.borrow()[name].clone()
        }
    }
    fn put(&self, name: &String, val: ShiroValue) {
        self.vars.borrow_mut().insert(name.to_string(), val);
    }
    fn register_native_function(&self, name: &str, ptr: NativeFunctionPtr) {
        self.put(&name.to_string(), ShiroValue::NativeFunction(ptr));
    }
}

trait Eval {
    fn eval(self, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue;
}

fn get_value(name: &Vec<String>, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue {
    let mut val = scope.find(&name.first().expect("Invalid identifier"));

    for p in name.iter().skip(1) {
        if let ShiroValue::HeapRef(addr) = val {
            let heap_obj = heap.deref(addr);
            val = heap_obj.borrow().get(p);
        } else {
            panic!(
                "Cannot access property '{}' of type {}",
                name.get(1).unwrap(),
                val
            );
        }
    }

    val
}

fn set_value(name: &Vec<String>, val: ShiroValue, scope: Rc<Scope>, heap: &mut Heap) {
    if name.len() == 1 {
        scope.put(name.first().unwrap(), val);
    } else {
        let val = scope.find(&name.first().expect("Invalid identifier"));
    }
    // TODO
}

impl Eval for &Expr {
    fn eval(self, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue {
        match self {
            Expr::Decimal(val) => ShiroValue::Decimal(*val),
            Expr::Integer(val) => ShiroValue::Integer(*val),
            Expr::Boolean(val) => ShiroValue::Boolean(*val),
            Expr::String(val) => ShiroValue::String(val.to_string()),
            Expr::Let(name, value) => {
                let result = value.eval(scope.clone(), heap);
                scope.put(name, result.clone());
                result
            }
            Expr::Reference(name) => get_value(name, scope, heap).clone(),
            Expr::Op(lhs, op, rhs) => match op {
                Opcode::Add => lhs.eval(scope.clone(), heap) + rhs.eval(scope.clone(), heap),
                Opcode::Sub => lhs.eval(scope.clone(), heap) - rhs.eval(scope.clone(), heap),
                Opcode::Mul => lhs.eval(scope.clone(), heap) * rhs.eval(scope.clone(), heap),
                Opcode::Div => lhs.eval(scope.clone(), heap) / rhs.eval(scope.clone(), heap),
                Opcode::Mod => lhs.eval(scope.clone(), heap) % rhs.eval(scope.clone(), heap),
                Opcode::Lt => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) < rhs.eval(scope.clone(), heap),
                ),
                Opcode::Gt => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) > rhs.eval(scope.clone(), heap),
                ),
                Opcode::Lte => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) <= rhs.eval(scope.clone(), heap),
                ),
                Opcode::Gte => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) >= rhs.eval(scope.clone(), heap),
                ),
                Opcode::Eq => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) == rhs.eval(scope.clone(), heap),
                ),
                Opcode::Neq => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), heap) != rhs.eval(scope.clone(), heap),
                ),
            },
            Expr::AssignOp(lhs, op, rhs) => match op {
                AssignOpcode::Eq => {
                    let name = lhs.last().unwrap();
                    let new_val = rhs.eval(scope.clone(), heap);
                    scope.put(name, new_val.clone());
                    new_val
                }
                AssignOpcode::Add => {
                    let val = get_value(lhs, scope.clone(), heap) + rhs.eval(scope.clone(), heap);
                    set_value(lhs, val.clone(), scope.clone(), heap);
                    val
                }
                AssignOpcode::Sub => {
                    let val = get_value(lhs, scope.clone(), heap) - rhs.eval(scope.clone(), heap);
                    set_value(lhs, val.clone(), scope, heap);
                    val
                }
            },
            Expr::Fun(name, args, body) => {
                let shiro_fun = ShiroValue::Function {
                    args: args.clone(),
                    body: body.clone(),
                };
                match name {
                    Some(name) => {
                        scope.put(name, shiro_fun);
                        ShiroValue::Null
                    }
                    _ => shiro_fun,
                }
            }
            Expr::Invocation(name, in_args) => {
                let target = get_value(name, scope.clone(), heap);
                match target {
                    ShiroValue::Function { args, body } => {
                        let new_scope = Scope::new(Some(scope.clone()));
                        let matching_arg_num = min(in_args.len(), args.len());
                        for i in 0..matching_arg_num {
                            let arg_key = &args[i];
                            let arg_val = in_args[i].eval(scope.clone(), heap);
                            new_scope.put(arg_key, arg_val);
                        }
                        let rc = Rc::new(new_scope);
                        eval_block(&body, rc, heap)
                    }
                    ShiroValue::NativeFunction(body) => body(scope, heap, in_args),
                    _ => {
                        panic!(
                            "Cannot call non-function reference {:?} of type {}",
                            name, target
                        );
                    }
                }
            }
            Expr::Return(expr) => expr.eval(scope, heap),
            Expr::For(init_expr, condition_expr, inc_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                init_expr.eval(new_scope.clone(), heap);
                while condition_expr
                    .eval(new_scope.clone(), heap)
                    .coerce_boolean()
                {
                    eval_block(body, new_scope.clone(), heap);
                    inc_expr.eval(new_scope.clone(), heap);
                }
                ShiroValue::Null
            }
            Expr::While(condition_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                while condition_expr
                    .eval(new_scope.clone(), heap)
                    .coerce_boolean()
                {
                    eval_block(body, new_scope.clone(), heap);
                }
                ShiroValue::Null
            }
            Expr::If(branches) => {
                for branch in branches {
                    let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                    match &branch.condition {
                        Some(c) => {
                            if c.eval(new_scope.clone(), heap).coerce_boolean() {
                                return eval_block(&branch.body, new_scope.clone(), heap);
                            }
                        }
                        None => return eval_block(&branch.body, new_scope.clone(), heap),
                    }
                }

                ShiroValue::Null
            }
            Expr::ShionObject(body) => {
                let obj = heap.alloc();
                let mut obj_mut = obj.borrow_mut();
                let addr = obj_mut.address();
                for def in body {
                    if let Expr::ShionDef(k, v) = def.as_ref() {
                        let v = v.eval(scope.clone(), heap);
                        obj_mut.put(k, v);
                    } else {
                        panic!("Expected ShionDef got {:?}", def);
                    }
                }
                ShiroValue::HeapRef(addr)
            }
            _ => ShiroValue::Null,
        }
    }
}

fn eval_block(block: &Vec<Box<Expr>>, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue {
    let mut retval = ShiroValue::Null;
    for expr in block {
        let expr = expr.as_ref();
        retval = expr.eval(scope.clone(), heap);
        if matches!(expr, Expr::Return(_)) {
            break;
        }
    }
    retval
}

pub fn evaluate(tree: &Vec<Box<Expr>>) -> ShiroValue {
    let global_scope = Rc::new(Scope::new(None));
    global_scope.register_native_function("print", |scope, heap, args| {
        let mut str = String::new();
        for arg in args {
            str.push_str(arg.eval(scope.clone(), heap).coerce_string().as_str());
            str.push(' ');
        }
        println!("{}", str);
        ShiroValue::Null
    });

    let mut heap = Heap::new();
    let r = eval_block(tree, global_scope, &mut heap);
    heap.gc();
    dbg!(&heap);
    r
}
