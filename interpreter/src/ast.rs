#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Nop,
    Null,
    Let(String, Box<Expr>),
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    Reference(Reference),
    AssignOp(Reference, AssignOpcode, Box<Expr>),
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
    Invocation(Vec<String>, Vec<Box<Expr>>),
    FunctionDecl(Option<String>, Vec<String>, Vec<Box<Expr>>),
    ObjectDef(Vec<Box<Expr>>),
    ArrayDef(Vec<Box<Expr>>),
    ObjectEntry(String, Box<Expr>),
    If(Vec<Box<IfBranch>>),
    While(Box<Expr>, Vec<Box<Expr>>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Vec<Box<Expr>>),
    Return(Box<Expr>),
    Import(String, String),
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
    BOr,
    BAnd,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfBranch {
    pub condition: Option<Box<Expr>>,
    pub body: Vec<Box<Expr>>,
}
