#[derive(Debug, PartialEq)]
pub enum Ty {
    BoolTy,
    IntTy,
    FloatTy,
    StrTy,
}

#[derive(Debug, PartialEq)]
pub enum CallKind {
    Read,
    ReadLn,
    Write,
    WriteLn,
}

/// A Name references an identifier in the identifier table.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Name(pub u64);

#[derive(Debug, PartialEq)]
pub struct Ident {
    pub name: Name,
}

#[derive(Debug, PartialEq)]
pub enum Lit {
    IntLit(u64),
    FloatLit(f64),
    StrLit(String),
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnOp, Box<Expr>),
    Lit(Lit),
    Ident(Ident),
    Paren(Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    stmts: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    While(Box<Expr>, Box<Block>),
    If(Box<Expr>, Box<Block>, Option<Box<Block>>),
    Block(Box<Block>),
    Assign(Ident, Box<Expr>),
    Call(CallKind, Vec<Ident>),
}

#[derive(Debug, PartialEq)]
pub struct Decl {
    id: Ident,
    ty: Ty,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    decls: Vec<Box<Decl>>,
    stmts: Vec<Box<Stmt>>,
}

