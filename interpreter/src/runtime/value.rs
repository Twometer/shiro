use std::{
    cmp::Ordering::{Equal, Greater, Less},
    ops::{Add, Div, Mul, Rem, Sub},
    rc::Rc,
    str::FromStr,
};

use crate::ast::Expr;

use super::{native::NativeFunctionPtr, scope::Scope};

#[derive(Clone)]
pub enum ShiroValue {
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Char(char),
    Function {
        args: Vec<String>,
        body: Vec<Box<Expr>>,
        scope: Rc<Scope>,
    },
    NativeFunction(NativeFunctionPtr),
    Null,
    HeapRef(u32),
}

impl std::fmt::Display for ShiroValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiroValue::Char(_) => write!(f, "Char"),
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
            Self::Char(arg0) => f.debug_tuple("Char").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Integer(arg0) => f.debug_tuple("Integer").field(arg0).finish(),
            Self::Decimal(arg0) => f.debug_tuple("Decimal").field(arg0).finish(),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Function { args, body, .. } => f
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
    pub fn coerce_integer(&self) -> i64 {
        match self {
            ShiroValue::String(s) => i64::from_str(s.as_str()).unwrap(),
            ShiroValue::Decimal(d) => *d as i64,
            ShiroValue::Integer(d) => *d,
            ShiroValue::Boolean(d) => {
                if *d {
                    1
                } else {
                    0
                }
            }
            ShiroValue::Char(c) => *c as i64,
            _ => 0,
        }
    }

    pub fn coerce_char(&self) -> char {
        match self {
            ShiroValue::Char(c) => *c,
            _ => '\0',
        }
    }

    pub fn borrow_string(&self) -> &String {
        if let ShiroValue::String(s) = self {
            return s;
        } else {
            panic!("Cannot borrow string of non-string value");
        }
    }

    pub fn type_string(&self) -> String {
        match self {
            ShiroValue::String(_) => "string",
            ShiroValue::Char(_) => "char",
            ShiroValue::Decimal(_) => "decimal",
            ShiroValue::Integer(_) => "integer",
            ShiroValue::Boolean(_) => "boolean",
            ShiroValue::Function { .. } => "function",
            ShiroValue::NativeFunction { .. } => "function",
            ShiroValue::HeapRef(_) => "object",
            ShiroValue::Null => "null",
        }
        .to_string()
    }

    pub fn coerce_string(&self) -> String {
        match self {
            ShiroValue::String(s) => s.to_string(),
            ShiroValue::Decimal(v) => v.to_string(),
            ShiroValue::Integer(v) => v.to_string(),
            ShiroValue::Boolean(v) => v.to_string(),
            ShiroValue::Char(c) => c.to_string(),
            ShiroValue::Function { .. } => "[function]".to_string(),
            ShiroValue::NativeFunction { .. } => "[native function]".to_string(),
            ShiroValue::HeapRef(_) => "[object]".to_string(),
            _ => "null".to_string(),
        }
    }

    pub fn coerce_decimal(&self) -> f64 {
        match self {
            ShiroValue::String(s) => f64::from_str(s.as_str()).unwrap(),
            ShiroValue::Decimal(d) => *d,
            ShiroValue::Integer(d) => *d as f64,
            ShiroValue::Char(c) => (*c as i32) as f64,
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

    pub fn coerce_boolean(&self) -> bool {
        match self {
            ShiroValue::String(s) => !s.is_empty(),
            ShiroValue::Decimal(d) => *d != 0.0,
            ShiroValue::Integer(i) => *i != 0,
            ShiroValue::Boolean(b) => *b,
            ShiroValue::Function { .. } => true,
            ShiroValue::NativeFunction { .. } => true,
            ShiroValue::HeapRef(_) => true,
            ShiroValue::Char(c) => *c != '\0',
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
            ShiroValue::Char(c) => ShiroValue::String(c.to_string() + &rhs.coerce_string()),
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
            ShiroValue::Char(c) => *c as i64 == other.coerce_integer(),
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
            ShiroValue::Char(c) => c.partial_cmp(&other.coerce_char()),
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
