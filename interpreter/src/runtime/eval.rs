use std::{cmp::min, rc::Rc};

use crate::{
    ast::{AssignOpcode, BinaryOpcode, Expr, Reference, UnaryOpcode},
    parser::CodeFile,
};

use super::{heap::Heap, native::NativeLibProvider, scope::Scope, value::ShiroValue, RunContext};

fn get_value(name: &Vec<ShiroValue>, scope: Rc<Scope>, heap: &mut Heap) -> ShiroValue {
    let mut val = scope.get_by_val(&name.first().expect("Invalid identifier"));

    for p in name.iter().skip(1) {
        match &val {
            ShiroValue::HeapRef(addr) => {
                let heap_obj = heap.deref(*addr);
                val = heap_obj.borrow().get(p);
            }
            ShiroValue::String(str) => {
                let chr = str.chars().nth(p.coerce_integer() as usize);
                match chr {
                    Some(chr) => val = ShiroValue::Char(chr),
                    None => val = ShiroValue::Null,
                }
            }
            _ => {
                panic!(
                    "Cannot read property '{}' of type {}",
                    p.coerce_string(),
                    val
                );
            }
        }
    }

    val
}

fn set_value(name: &Vec<ShiroValue>, new_val: ShiroValue, scope: Rc<Scope>, heap: &mut Heap) {
    let local_name = name.first().expect("Invalid identifier");
    if name.len() == 1 {
        scope.put_by_val(local_name, new_val, false);
    } else {
        let mut val = scope.get_by_val(local_name);
        let mut obj = None;

        for i in 1..name.len() {
            let p = &name[i];
            if let ShiroValue::HeapRef(addr) = val {
                let heap_obj = heap.deref(addr);
                val = heap_obj.borrow().get(p);
                obj = Some(heap_obj);
            } else {
                panic!(
                    "Cannot write property '{}' of type {}",
                    p.coerce_string(),
                    val
                );
            }
        }

        obj.unwrap()
            .borrow_mut()
            .put(name.last().unwrap().clone(), new_val);
    }
}

fn ref_to_string(r: &Reference, scope: Rc<Scope>, ctx: &mut RunContext) -> Vec<ShiroValue> {
    match &r {
        Reference::Regular(name) => map_strings(name),
        Reference::Indexed(name, idx) => {
            let mut name = map_strings(name);
            name.push(idx.eval(scope.clone(), ctx));
            name
        }
    }
}

fn map_strings(vec: &Vec<String>) -> Vec<ShiroValue> {
    vec.iter().map(|p| ShiroValue::String(p.clone())).collect()
}

fn load_library(path: &str, ctx: &mut RunContext) -> ShiroValue {
    if ctx.libs.is_native_lib(path) {
        ctx.libs.load(&path, ctx.heap)
    } else {
        let stdlib_path = std::env::var("SHIRO_LIB_PATH").expect("Shiro library path not set");
        let full_path = if path.starts_with('@') {
            format!("{}/{}.shiro", &stdlib_path, &path)
        } else {
            format!("{}.shiro", &path)
        };
        let file = CodeFile::open(&full_path);
        eval_file(ctx, file)
    }
}

pub trait Eval {
    fn eval(self, scope: Rc<Scope>, ctx: &mut RunContext) -> ShiroValue;
}

impl Eval for &Expr {
    fn eval(self, scope: Rc<Scope>, ctx: &mut RunContext) -> ShiroValue {
        match self {
            Expr::Decimal(val) => ShiroValue::Decimal(*val),
            Expr::Integer(val) => ShiroValue::Integer(*val),
            Expr::Boolean(val) => ShiroValue::Boolean(*val),
            Expr::String(val) => ShiroValue::String(val.to_string()),
            Expr::Let(name, value) => {
                let result = value.eval(scope.clone(), ctx);
                scope.put_by_str(name, result.clone(), true);
                result
            }
            Expr::Reference(r) => {
                let ref_str = &ref_to_string(r, scope.clone(), ctx);
                get_value(ref_str, scope.clone(), ctx.heap)
            }
            Expr::Import(path, name) => {
                let lib = load_library(path, ctx);
                scope.put_by_str(name, lib, true);
                ShiroValue::Null
            }
            Expr::BinaryOp(lhs, op, rhs) => match op {
                BinaryOpcode::Add => lhs.eval(scope.clone(), ctx) + rhs.eval(scope.clone(), ctx),
                BinaryOpcode::Sub => lhs.eval(scope.clone(), ctx) - rhs.eval(scope.clone(), ctx),
                BinaryOpcode::Mul => lhs.eval(scope.clone(), ctx) * rhs.eval(scope.clone(), ctx),
                BinaryOpcode::Div => lhs.eval(scope.clone(), ctx) / rhs.eval(scope.clone(), ctx),
                BinaryOpcode::Mod => lhs.eval(scope.clone(), ctx) % rhs.eval(scope.clone(), ctx),
                BinaryOpcode::Lt => {
                    ShiroValue::Boolean(lhs.eval(scope.clone(), ctx) < rhs.eval(scope.clone(), ctx))
                }
                BinaryOpcode::Gt => {
                    ShiroValue::Boolean(lhs.eval(scope.clone(), ctx) > rhs.eval(scope.clone(), ctx))
                }
                BinaryOpcode::Lte => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx) <= rhs.eval(scope.clone(), ctx),
                ),
                BinaryOpcode::Gte => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx) >= rhs.eval(scope.clone(), ctx),
                ),
                BinaryOpcode::Eq => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx) == rhs.eval(scope.clone(), ctx),
                ),
                BinaryOpcode::Neq => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx) != rhs.eval(scope.clone(), ctx),
                ),
                BinaryOpcode::BOr => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx).coerce_boolean()
                        || rhs.eval(scope.clone(), ctx).coerce_boolean(),
                ),
                BinaryOpcode::BAnd => ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx).coerce_boolean()
                        && rhs.eval(scope.clone(), ctx).coerce_boolean(),
                ),
            },
            Expr::AssignOp(lhs, op, rhs) => {
                let ref_str = &ref_to_string(lhs, scope.clone(), ctx);
                match op {
                    AssignOpcode::Eq => {
                        let new_val = rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, new_val.clone(), scope.clone(), ctx.heap);
                        new_val
                    }
                    AssignOpcode::Add => {
                        let val = get_value(ref_str, scope.clone(), ctx.heap)
                            + rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, val.clone(), scope.clone(), ctx.heap);
                        val
                    }
                    AssignOpcode::Sub => {
                        let val = get_value(ref_str, scope.clone(), ctx.heap)
                            - rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, val.clone(), scope, ctx.heap);
                        val
                    }
                    AssignOpcode::Mul => {
                        let val = get_value(ref_str, scope.clone(), ctx.heap)
                            * rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, val.clone(), scope, ctx.heap);
                        val
                    }
                    AssignOpcode::Div => {
                        let val = get_value(ref_str, scope.clone(), ctx.heap)
                            / rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, val.clone(), scope, ctx.heap);
                        val
                    }
                    AssignOpcode::Mod => {
                        let val = get_value(ref_str, scope.clone(), ctx.heap)
                            % rhs.eval(scope.clone(), ctx);
                        set_value(ref_str, val.clone(), scope, ctx.heap);
                        val
                    }
                }
            }
            Expr::FunctionDecl(name, args, body) => {
                let shiro_fun = ShiroValue::Function {
                    args: args.clone(),
                    body: body.clone(),
                    scope: scope.clone(),
                };
                match name {
                    Some(name) => {
                        scope.put_by_str(name, shiro_fun, true);
                        ShiroValue::Null
                    }
                    _ => shiro_fun,
                }
            }
            Expr::Invocation(name, in_args) => {
                let name = &map_strings(name);
                let target = get_value(name, scope.clone(), ctx.heap);
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
                            let arg_val = in_args[i].eval(scope.clone(), ctx);
                            new_scope.put_by_str(arg_key, arg_val, true);
                        }
                        let rc = Rc::new(new_scope);
                        eval_block(&body, rc, ctx)
                    }
                    ShiroValue::NativeFunction(body) => body(in_args, scope, ctx),
                    _ => {
                        panic!(
                            "Cannot call non-function reference {:?} of type {}",
                            name, target
                        );
                    }
                }
            }
            Expr::Return(expr) => expr.eval(scope, ctx),
            Expr::For(init_expr, condition_expr, inc_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                init_expr.eval(new_scope.clone(), ctx);
                while condition_expr.eval(new_scope.clone(), ctx).coerce_boolean() {
                    eval_block(body, new_scope.clone(), ctx);
                    inc_expr.eval(new_scope.clone(), ctx);
                }
                ShiroValue::Null
            }
            Expr::While(condition_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                while condition_expr.eval(new_scope.clone(), ctx).coerce_boolean() {
                    eval_block(body, new_scope.clone(), ctx);
                }
                ShiroValue::Null
            }
            Expr::If(branches) => {
                for branch in branches {
                    let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                    match &branch.condition {
                        Some(c) => {
                            if c.eval(new_scope.clone(), ctx).coerce_boolean() {
                                return eval_block(&branch.body, new_scope.clone(), ctx);
                            }
                        }
                        None => return eval_block(&branch.body, new_scope.clone(), ctx),
                    }
                }

                ShiroValue::Null
            }
            Expr::ObjectDef(body) => {
                let obj = ctx.heap.alloc_object();
                let mut obj = obj.borrow_mut();
                for def in body {
                    if let Expr::ObjectEntry(k, v) = def.as_ref() {
                        let v = v.eval(scope.clone(), ctx);
                        obj.try_insert(k, v);
                    } else {
                        panic!("Expected ShionDef got {:?}", def);
                    }
                }
                ShiroValue::HeapRef(obj.address())
            }
            Expr::ArrayDef(items) => {
                let arr = ctx.heap.alloc_array();
                let mut arr = arr.borrow_mut();
                for itm in items {
                    let val = itm.eval(scope.clone(), ctx);
                    arr.try_push(val);
                }
                ShiroValue::HeapRef(arr.address())
            }
            Expr::UnaryOp(op, expr) => {
                let value = expr.eval(scope.clone(), ctx);
                match op {
                    UnaryOpcode::BNot => ShiroValue::Boolean(!value.coerce_boolean()),
                    //_ => panic!("Unknown operation {:?}", &op),
                }
            }
            _ => panic!("Unknown instruction {:?}", &self),
        }
    }
}

fn eval_block(block: &Vec<Box<Expr>>, scope: Rc<Scope>, ctx: &mut RunContext) -> ShiroValue {
    let mut retval = ShiroValue::Null;
    for expr in block {
        let expr = expr.as_ref();
        retval = expr.eval(scope.clone(), ctx);
        if matches!(expr, Expr::Return(_)) {
            break;
        }
    }
    retval
}

fn eval_tree(tree: &Vec<Box<Expr>>, ctx: &mut RunContext) -> ShiroValue {
    let global_scope = Rc::new(Scope::new(None));
    global_scope.register_native_function("typeof", |args, scope, ctx| {
        if args.len() == 0 {
            ShiroValue::Null
        } else {
            let val = args.first().unwrap().eval(scope, ctx);
            ShiroValue::String(val.type_string())
        }
    });
    global_scope.register_native_function("append", |args, scope, ctx| {
        assert!(args.len() == 2);
        let dst = args[0].eval(scope.clone(), ctx);
        if let ShiroValue::HeapRef(array_addr) = dst {
            let array = ctx.heap.deref(array_addr);
            let mut array = array.borrow_mut();
            let value = args[1].eval(scope.clone(), ctx);
            array.try_push(value);
        } else {
            panic!("Cannot append to value of type {}", dst.type_string());
        }
        ShiroValue::Null
    });
    global_scope.register_native_function("len", |args, scope, ctx| {
        assert!(args.len() == 1);
        let dst = args[0].eval(scope.clone(), ctx);
        match &dst {
            ShiroValue::HeapRef(array_addr) => {
                let obj = ctx.heap.deref(*array_addr);
                let obj = obj.borrow_mut();
                return ShiroValue::Integer(obj.len() as i64);
            }
            ShiroValue::String(str) => {
                return ShiroValue::Integer(str.len() as i64);
            }
            _ => panic!("Cannot get length of type {}", dst.type_string()),
        }
    });
    global_scope.register_native_function("keys", |args, scope, ctx| {
        assert!(args.len() == 1);

        let dst = args[0].eval(scope.clone(), ctx);
        if let ShiroValue::HeapRef(array_addr) = dst {
            let obj = ctx.heap.deref(array_addr);
            let obj = obj.borrow();

            let result_arr = ctx.heap.alloc_array();
            let mut result_arr = result_arr.borrow_mut();

            for i in obj.keys() {
                result_arr.try_push(ShiroValue::String(i));
            }

            return ShiroValue::HeapRef(result_arr.address());
        } else {
            panic!("Cannot get length of value of type {}", dst.type_string());
        }
    });
    global_scope.register_native_function("dbg", |args, scope, ctx| {
        for arg in args {
            println!("[dbg] {:?} = {:?}", arg, arg.eval(scope.clone(), ctx));
        }
        ShiroValue::Null
    });

    eval_block(tree, global_scope, ctx)
}

fn eval_file(ctx: &mut RunContext, file: CodeFile) -> ShiroValue {
    match ctx.parser.parse(file) {
        Ok(ast) => eval_tree(&ast, ctx),
        Err(diag) => {
            ctx.parser.report_diag(diag);
            ShiroValue::Null
        }
    }
}

pub fn eval(code_file: CodeFile) -> ShiroValue {
    let mut heap = Heap::new();
    let libs = NativeLibProvider::default();

    let mut ctx = RunContext::new(&mut heap, &libs);
    let result = eval_file(&mut ctx, code_file);

    heap.gc();
    result
}
