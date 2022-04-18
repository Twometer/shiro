#[derive(Debug, PartialEq)]
pub enum Expr {
    Nop,
    Null,
    Let(String, Box<Expr>),
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Reference(Vec<String>),
    IndexedReference(Vec<String>, Box<Expr>),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Assignment(Vec<String>, Box<Expr>),
    Invocation(Vec<String>, Vec<Box<Expr>>),
    ShionObject(Vec<Box<Expr>>),
    ShionArray(Vec<Box<Expr>>),
    ShionDef(String, Box<Expr>),
    If(Vec<Box<IfBranch>>),
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

#[derive(Debug, PartialEq)]
pub struct IfBranch {
    pub condition: Option<Box<Expr>>,
    pub body: Vec<Box<Expr>>,
}
