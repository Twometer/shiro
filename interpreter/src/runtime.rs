use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

use crate::ast::{AssignOpcode, Expr, Opcode};

#[derive(Clone, Debug)]
enum ShiroValue {
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Null,
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
            _ => "null".to_string(),
        }
    }
}

impl Add for ShiroValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self.coerce_integer();
        let rhs = rhs.coerce_integer();
        ShiroValue::Integer(lhs + rhs)
    }
}

impl Sub for ShiroValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Div for ShiroValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Mul for ShiroValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.coerce_integer();
        let rhs = rhs.coerce_integer();
        ShiroValue::Integer(lhs * rhs)
    }
}

#[derive(Debug)]
struct Scope {
    vars: HashMap<String, ShiroValue>,
}

impl Scope {
    // TODO: Traverse scope upwards
    fn find(&self, name: &Vec<String>) -> &ShiroValue {
        let name = name.last().expect("Invalid identifier");
        if !self.vars.contains_key(name) {
            &ShiroValue::Null
        } else {
            &self.vars[name]
        }
    }
    fn put(&mut self, name: &String, val: ShiroValue) {
        self.vars.insert(name.to_string(), val);
    }
}

trait Eval {
    fn eval(self, scope: &mut Scope) -> ShiroValue;
}

impl Eval for &mut Expr {
    fn eval(self, scope: &mut Scope) -> ShiroValue {
        match self {
            Expr::Decimal(val) => ShiroValue::Decimal(*val),
            Expr::Integer(val) => ShiroValue::Integer(*val),
            Expr::Boolean(val) => ShiroValue::Boolean(*val),
            Expr::String(val) => ShiroValue::String(val.to_string()),
            Expr::Let(name, value) => {
                let result = value.eval(scope);
                scope.put(name, result.clone());
                result
            }
            Expr::Reference(name) => scope.find(name).clone(),
            Expr::Op(lhs, op, rhs) => match op {
                Opcode::Add => lhs.eval(scope) + rhs.eval(scope),
                Opcode::Sub => lhs.eval(scope) - rhs.eval(scope),
                Opcode::Mul => lhs.eval(scope) * rhs.eval(scope),
                Opcode::Div => lhs.eval(scope) / rhs.eval(scope),
                _ => ShiroValue::Null,
            },
            Expr::AssignOp(lhs, op, rhs) => match op {
                AssignOpcode::Eq => {
                    let name = lhs.last().unwrap();
                    let new_val = rhs.eval(scope);
                    scope.vars.insert(name.to_string(), new_val.clone());
                    new_val
                }
                _ => ShiroValue::Null,
            },
            Expr::Invocation(name, args) => {
                // TODO actual function invocations
                let name = name.last().unwrap();
                if name == "print" {
                    let expr = args.first_mut().unwrap();
                    let expr = &mut **expr;
                    let value = expr.eval(scope).coerce_string();
                    println!("{}", value);
                }
                ShiroValue::Null
            }
            _ => ShiroValue::Null,
        }
    }
}

pub fn evaluate(tree: &mut Vec<Box<Expr>>) {
    let mut scope = Scope {
        vars: HashMap::new(),
    };

    for box_expr in tree {
        let expr = &mut **box_expr;
        expr.eval(&mut scope);
    }
}
