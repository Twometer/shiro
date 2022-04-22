#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Nop,
    Null,
    Let(String, Box<Expr>),
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Reference(Reference),
    AssignOp(Reference, AssignOpcode, Box<Expr>),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Invocation(Vec<String>, Vec<Box<Expr>>),
    ShionObject(Vec<Box<Expr>>),
    ShionArray(Vec<Box<Expr>>),
    ShionDef(String, Box<Expr>),
    If(Vec<Box<IfBranch>>),
    While(Box<Expr>, Vec<Box<Expr>>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Vec<Box<Expr>>),
    Fun(Option<String>, Vec<String>, Vec<Box<Expr>>),
    Return(Box<Expr>),
    Use(Vec<String>, String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Reference {
    Regular(Vec<String>),
    Indexed(Vec<String>, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOpcode {
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct IfBranch {
    pub condition: Option<Box<Expr>>,
    pub body: Vec<Box<Expr>>,
}
