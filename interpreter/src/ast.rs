#[derive(Debug, PartialEq)]
pub enum Expr {
    Nop,
    Null,
    Let(String, Box<Expr>),
    String(String),
    Number(f32),
    Boolean(bool),
    Reference(Vec<String>),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Assignment(Vec<String>, Box<Expr>),
    Invocation(Vec<String>, Vec<Box<Expr>>),
    ShionObject(Vec<Box<Expr>>),
    ShionDef(String, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neq,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}
