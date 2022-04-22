use std::{cmp::min, rc::Rc};

use crate::{
    ast::{AssignOpcode, Expr, Opcode, Reference},
    shiro,
};

use super::{heap::Heap, preproc::preprocess_code, scope::Scope, value::ShiroValue};

fn get_value(name: &Vec<String>, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue {
    let mut val = scope.find(&name.first().expect("Invalid identifier"));

    for p in name.iter().skip(1) {
        if let ShiroValue::HeapRef(addr) = val {
            let heap_obj = heap.deref(addr);
            val = heap_obj.borrow().get(p);
        } else {
            panic!("Cannot read property '{}' of type {}", p, val);
        }
    }

    val
}

fn set_value(name: &Vec<String>, new_val: ShiroValue, scope: Rc<Scope>, heap: &mut Heap) {
    let local_name = name.first().expect("Invalid identifier");
    if name.len() == 1 {
        scope.put(local_name, new_val);
    } else {
        let mut val = scope.find(local_name);
        let mut obj = None;

        for i in 1..name.len() {
            let p = &name[i];
            if let ShiroValue::HeapRef(addr) = val {
                let heap_obj = heap.deref(addr);
                val = heap_obj.borrow().get(p);
                obj = Some(heap_obj);
            } else {
                panic!("Cannot write property '{}' of type {}", p, val);
            }
        }

        obj.unwrap().borrow_mut().put(name.last().unwrap(), new_val);
    }
}

fn ref_to_string(r: &Reference, scope: Rc<Scope>, heap: &mut Heap) -> Vec<String> {
    match &r {
        Reference::Regular(name) => name.clone(),
        Reference::Indexed(name, idx) => {
            let mut new_name = name.clone();
            new_name.push(idx.eval(scope.clone(), heap).coerce_string());
            new_name
        }
    }
}

trait Eval {
    fn eval(self, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue;
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
            Expr::Reference(r) => {
                let ref_str = &ref_to_string(r, scope.clone(), heap);
                get_value(ref_str, scope.clone(), heap)
            }

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
            Expr::AssignOp(lhs, op, rhs) => {
                let ref_str = &ref_to_string(lhs, scope.clone(), heap);
                match op {
                    AssignOpcode::Eq => {
                        let new_val = rhs.eval(scope.clone(), heap);
                        set_value(ref_str, new_val.clone(), scope.clone(), heap);
                        new_val
                    }
                    AssignOpcode::Add => {
                        let val =
                            get_value(ref_str, scope.clone(), heap) + rhs.eval(scope.clone(), heap);
                        set_value(ref_str, val.clone(), scope.clone(), heap);
                        val
                    }
                    AssignOpcode::Sub => {
                        let val =
                            get_value(ref_str, scope.clone(), heap) - rhs.eval(scope.clone(), heap);
                        set_value(ref_str, val.clone(), scope, heap);
                        val
                    }
                    AssignOpcode::Mul => {
                        let val =
                            get_value(ref_str, scope.clone(), heap) * rhs.eval(scope.clone(), heap);
                        set_value(ref_str, val.clone(), scope, heap);
                        val
                    }
                    AssignOpcode::Div => {
                        let val =
                            get_value(ref_str, scope.clone(), heap) / rhs.eval(scope.clone(), heap);
                        set_value(ref_str, val.clone(), scope, heap);
                        val
                    }
                    AssignOpcode::Mod => {
                        let val =
                            get_value(ref_str, scope.clone(), heap) % rhs.eval(scope.clone(), heap);
                        set_value(ref_str, val.clone(), scope, heap);
                        val
                    }
                }
            }
            Expr::Fun(name, args, body) => {
                let shiro_fun = ShiroValue::Function {
                    args: args.clone(),
                    body: body.clone(),
                    scope: scope.clone(),
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
                    ShiroValue::Function {
                        args,
                        body,
                        scope: fun_scope,
                    } => {
                        let new_scope = Scope::new(Some(fun_scope.clone()));
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
            Expr::ShionArray(items) => {
                let obj = heap.alloc();
                let mut obj_mut = obj.borrow_mut();
                let addr = obj_mut.address();
                let mut idx = 0;
                for itm in items {
                    let val = itm.eval(scope.clone(), heap);
                    obj_mut.put(&idx.to_string(), val);
                    idx += 1;
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

fn eval_tree(tree: &Vec<Box<Expr>>) -> ShiroValue {
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

pub fn eval(code: &String) -> ShiroValue {
    let preprocessed = preprocess_code(code.as_str());
    match shiro::ChunkParser::new().parse(&preprocessed.as_str()) {
        Ok(ast) => eval_tree(&ast),
        Err(e) => panic!("Parser failed: {}", e),
    }
}
