use std::{cmp::min, rc::Rc};

use crate::{
    ast::{AssignOpcode, BinaryOpcode, Expr, Reference, UnaryOpcode},
    diag::ShiroError,
    parser::CodeFile,
};

use super::{heap::Heap, scope::Scope, value::ShiroValue, Runtime};

fn get_value(
    name: &Vec<ShiroValue>,
    scope: Rc<Scope>,
    heap: &mut Heap,
) -> Result<ShiroValue, ShiroError> {
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
                return Err(ShiroError::GenericRuntimeError(format!(
                    "Cannot read property `{}` from a `{}` reference",
                    p.coerce_string(),
                    val
                )));
            }
        }
    }

    Ok(val)
}

fn set_value(
    name: &Vec<ShiroValue>,
    new_val: ShiroValue,
    scope: Rc<Scope>,
    heap: &mut Heap,
) -> Result<(), ShiroError> {
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
                return Err(ShiroError::GenericRuntimeError(format!(
                    "Cannot write property `{}` to a `{}` reference",
                    p.coerce_string(),
                    val
                )));
            }
        }

        obj.unwrap()
            .borrow_mut()
            .put(name.last().unwrap().clone(), new_val)?;
    }
    Ok(())
}

fn ref_to_string(
    r: &Reference,
    scope: Rc<Scope>,
    ctx: &mut Runtime,
) -> Result<Vec<ShiroValue>, ShiroError> {
    match &r {
        Reference::Regular(name) => Ok(map_strings(name)),
        Reference::Indexed(name, idx) => {
            let mut name = map_strings(name);
            let value = idx.eval(scope.clone(), ctx)?;
            name.push(value);
            Ok(name)
        }
    }
}

fn map_strings(vec: &Vec<String>) -> Vec<ShiroValue> {
    vec.iter().map(|p| ShiroValue::String(p.clone())).collect()
}

fn load_library(path: &str, ctx: &mut Runtime) -> Result<ShiroValue, ShiroError> {
    if ctx.libs.is_native_lib(path) {
        Ok(ctx.libs.load(&path, &mut &mut ctx.heap))
    } else {
        let stdlib_path = std::env::var("SHIRO_LIB_PATH");
        let full_path = if path.starts_with('@') && stdlib_path.is_ok() {
            format!("{}/{}.shiro", &stdlib_path.unwrap(), &path)
        } else {
            format!("{}.shiro", &path)
        };
        let file = CodeFile::open(&full_path)?;
        ctx.eval_file(file)
    }
}

pub trait Eval {
    fn eval(self, scope: Rc<Scope>, ctx: &mut Runtime) -> Result<ShiroValue, ShiroError>;
}

impl Eval for &Expr {
    fn eval(self, scope: Rc<Scope>, ctx: &mut Runtime) -> Result<ShiroValue, ShiroError> {
        match self {
            Expr::Decimal(val) => Ok(ShiroValue::Decimal(*val)),
            Expr::Integer(val) => Ok(ShiroValue::Integer(*val)),
            Expr::Boolean(val) => Ok(ShiroValue::Boolean(*val)),
            Expr::String(val) => Ok(ShiroValue::String(val.to_string())),
            Expr::Let(name, value) => {
                let result = value.eval(scope.clone(), ctx)?;
                scope.put_by_str(name, result.clone(), true);
                Ok(result)
            }
            Expr::Reference(r) => {
                let ref_str = &ref_to_string(r, scope.clone(), ctx)?;
                get_value(ref_str, scope.clone(), &mut ctx.heap)
            }
            Expr::Import(path, name) => {
                let lib = load_library(path, ctx)?;
                scope.put_by_str(name, lib, true);
                Ok(ShiroValue::Null)
            }
            Expr::BinaryOp(lhs, op, rhs) => match op {
                BinaryOpcode::Add => {
                    Ok(lhs.eval(scope.clone(), ctx)? + rhs.eval(scope.clone(), ctx)?)
                }
                BinaryOpcode::Sub => {
                    Ok(lhs.eval(scope.clone(), ctx)? - rhs.eval(scope.clone(), ctx)?)
                }
                BinaryOpcode::Mul => {
                    Ok(lhs.eval(scope.clone(), ctx)? * rhs.eval(scope.clone(), ctx)?)
                }
                BinaryOpcode::Div => {
                    Ok(lhs.eval(scope.clone(), ctx)? / rhs.eval(scope.clone(), ctx)?)
                }
                BinaryOpcode::Mod => {
                    Ok(lhs.eval(scope.clone(), ctx)? % rhs.eval(scope.clone(), ctx)?)
                }
                BinaryOpcode::Lt => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? < rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::Gt => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? > rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::Lte => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? <= rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::Gte => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? >= rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::Eq => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? == rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::Neq => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)? != rhs.eval(scope.clone(), ctx)?,
                )),
                BinaryOpcode::BOr => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)?.coerce_boolean()
                        || rhs.eval(scope.clone(), ctx)?.coerce_boolean(),
                )),
                BinaryOpcode::BAnd => Ok(ShiroValue::Boolean(
                    lhs.eval(scope.clone(), ctx)?.coerce_boolean()
                        && rhs.eval(scope.clone(), ctx)?.coerce_boolean(),
                )),
            },
            Expr::AssignOp(lhs, op, rhs) => {
                let ref_str = &ref_to_string(lhs, scope.clone(), ctx)?;
                match op {
                    AssignOpcode::Eq => {
                        let new_val = rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, new_val.clone(), scope.clone(), &mut ctx.heap)?;
                        Ok(new_val)
                    }
                    AssignOpcode::Add => {
                        let val = get_value(ref_str, scope.clone(), &mut ctx.heap)?
                            + rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, val.clone(), scope.clone(), &mut ctx.heap)?;
                        Ok(val)
                    }
                    AssignOpcode::Sub => {
                        let val = get_value(ref_str, scope.clone(), &mut ctx.heap)?
                            - rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, val.clone(), scope, &mut ctx.heap)?;
                        Ok(val)
                    }
                    AssignOpcode::Mul => {
                        let val = get_value(ref_str, scope.clone(), &mut ctx.heap)?
                            * rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, val.clone(), scope, &mut ctx.heap)?;
                        Ok(val)
                    }
                    AssignOpcode::Div => {
                        let val = get_value(ref_str, scope.clone(), &mut ctx.heap)?
                            / rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, val.clone(), scope, &mut ctx.heap)?;
                        Ok(val)
                    }
                    AssignOpcode::Mod => {
                        let val = get_value(ref_str, scope.clone(), &mut ctx.heap)?
                            % rhs.eval(scope.clone(), ctx)?;
                        set_value(ref_str, val.clone(), scope, &mut ctx.heap)?;
                        Ok(val)
                    }
                }
            }
            Expr::FunctionDecl(name, args, body) => {
                let shiro_fun = ShiroValue::Function {
                    args: args.clone(),
                    body: body.clone(),
                    scope: scope.clone(),
                };
                Ok(match name {
                    Some(name) => {
                        scope.put_by_str(name, shiro_fun, true);
                        ShiroValue::Null
                    }
                    _ => shiro_fun,
                })
            }
            Expr::Invocation(name, in_args) => {
                let shiro_name = map_strings(name);
                let target = get_value(&shiro_name, scope.clone(), &mut ctx.heap)?;
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
                            let arg_val = in_args[i].eval(scope.clone(), ctx)?;
                            new_scope.put_by_str(arg_key, arg_val, true);
                        }
                        let rc = Rc::new(new_scope);
                        eval_block(&body, rc, ctx)
                    }
                    ShiroValue::NativeFunction(body) => body(in_args, scope, ctx),
                    _ => Err(ShiroError::GenericRuntimeError(format!(
                        "Cannot call reference `{}` that is of type `{}`",
                        name.join("."),
                        target
                    ))),
                }
            }
            Expr::Return(expr) => expr.eval(scope, ctx),
            Expr::For(init_expr, condition_expr, inc_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                init_expr.eval(new_scope.clone(), ctx)?;
                while condition_expr
                    .eval(new_scope.clone(), ctx)?
                    .coerce_boolean()
                {
                    eval_block(body, new_scope.clone(), ctx)?;
                    inc_expr.eval(new_scope.clone(), ctx)?;
                }
                Ok(ShiroValue::Null)
            }
            Expr::While(condition_expr, body) => {
                let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                while condition_expr
                    .eval(new_scope.clone(), ctx)?
                    .coerce_boolean()
                {
                    eval_block(body, new_scope.clone(), ctx)?;
                }
                Ok(ShiroValue::Null)
            }
            Expr::If(branches) => {
                for branch in branches {
                    let new_scope = Rc::new(Scope::new(Some(scope.clone())));
                    match &branch.condition {
                        Some(c) => {
                            if c.eval(new_scope.clone(), ctx)?.coerce_boolean() {
                                return eval_block(&branch.body, new_scope.clone(), ctx);
                            }
                        }
                        None => return eval_block(&branch.body, new_scope.clone(), ctx),
                    }
                }

                Ok(ShiroValue::Null)
            }
            Expr::ObjectDef(body) => {
                let obj = &mut ctx.heap.alloc_object();
                let mut obj = obj.borrow_mut();
                for def in body {
                    if let Expr::ObjectEntry(k, v) = def.as_ref() {
                        let v = v.eval(scope.clone(), ctx)?;
                        obj.try_insert(k, v)?;
                    } else {
                        panic!("Expected ShionDef got {:?}", def);
                    }
                }
                Ok(ShiroValue::HeapRef(obj.address()))
            }
            Expr::ArrayDef(items) => {
                let arr = &mut ctx.heap.alloc_array();
                let mut arr = arr.borrow_mut();
                for itm in items {
                    let val = itm.eval(scope.clone(), ctx)?;
                    arr.try_push(val)?;
                }
                Ok(ShiroValue::HeapRef(arr.address()))
            }
            Expr::UnaryOp(op, expr) => {
                let value = expr.eval(scope.clone(), ctx)?;
                Ok(match op {
                    UnaryOpcode::BNot => ShiroValue::Boolean(!value.coerce_boolean()),
                })
            }
            _ => Err(ShiroError::UnknownInstruction),
        }
    }
}

fn eval_block(
    block: &Vec<Box<Expr>>,
    scope: Rc<Scope>,
    ctx: &mut Runtime,
) -> Result<ShiroValue, ShiroError> {
    let mut retval = ShiroValue::Null;
    for expr in block {
        let expr = expr.as_ref();
        retval = expr.eval(scope.clone(), ctx)?;
        if matches!(expr, Expr::Return(_)) {
            break;
        }
    }
    Ok(retval)
}

impl Runtime {
    fn eval_tree(&mut self, tree: &Vec<Box<Expr>>) -> Result<ShiroValue, ShiroError> {
        let global_scope = Rc::new(Scope::new(None));
        global_scope.register_native_function("typeof", |args, scope, ctx| {
            Ok(if args.len() == 0 {
                ShiroValue::Null
            } else {
                let val = args.first().unwrap().eval(scope, ctx)?;
                ShiroValue::String(val.type_string())
            })
        });
        global_scope.register_native_function("append", |args, scope, ctx| {
            assert!(args.len() == 2);
            let dst = args[0].eval(scope.clone(), ctx)?;
            return if let ShiroValue::HeapRef(array_addr) = dst {
                let array = &mut ctx.heap.deref(array_addr);
                let mut array = array.borrow_mut();
                let value = args[1].eval(scope.clone(), ctx)?;
                array.try_push(value)?;
                Ok(ShiroValue::Null)
            } else {
                Err(ShiroError::GenericRuntimeError(format!(
                    "Cannot append to value of type {}",
                    dst.type_string()
                )))
            };
        });
        global_scope.register_native_function("len", |args, scope, ctx| {
            assert!(args.len() == 1);
            let dst = args[0].eval(scope.clone(), ctx)?;
            match &dst {
                ShiroValue::HeapRef(array_addr) => {
                    let obj = &mut ctx.heap.deref(*array_addr);
                    let obj = obj.borrow_mut();
                    Ok(ShiroValue::Integer(obj.len() as i64))
                }
                ShiroValue::String(str) => Ok(ShiroValue::Integer(str.len() as i64)),
                _ => Err(ShiroError::GenericRuntimeError(format!(
                    "Cannot retreive length of type {}",
                    dst.type_string()
                ))),
            }
        });
        global_scope.register_native_function("keys", |args, scope, ctx| {
            assert!(args.len() == 1);

            let dst = args[0].eval(scope.clone(), ctx)?;
            if let ShiroValue::HeapRef(array_addr) = dst {
                let obj = &mut ctx.heap.deref(array_addr);
                let obj = obj.borrow();

                let result_arr = &mut ctx.heap.alloc_array();
                let mut result_arr = result_arr.borrow_mut();

                for i in obj.keys()? {
                    result_arr.try_push(ShiroValue::String(i))?;
                }

                return Ok(ShiroValue::HeapRef(result_arr.address()));
            } else {
                panic!("Cannot get length of value of type {}", dst.type_string());
            }
        });
        global_scope.register_native_function("dbg", |args, scope, ctx| {
            for arg in args {
                println!("[dbg] {:?} = {:?}", arg, arg.eval(scope.clone(), ctx)?);
            }
            Ok(ShiroValue::Null)
        });

        eval_block(tree, global_scope, self)
    }

    fn eval_file(&mut self, file: CodeFile) -> Result<ShiroValue, ShiroError> {
        let ast = self.parse_file(file)?;
        self.eval_tree(&ast)
    }

    pub fn eval(&mut self, code_file: CodeFile) -> Result<ShiroValue, ShiroError> {
        let result = self.eval_file(code_file)?;
        self.heap.gc();
        Ok(result)
    }
}
