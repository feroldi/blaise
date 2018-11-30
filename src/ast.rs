#[derive(Debug, PartialEq)]
pub enum Ty {
    BoolTy,
    IntTy,
    FloatTy,
    StrTy,
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
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    While(Expr, Box<Block>),
    If(Expr, Box<Block>, Option<Box<Block>>),
    Assign(Ident, Expr),
    BlockStmt(Box<Block>),
    Call(Ident, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct Decl {
    pub ident: Ident,
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub name: Ident,
    pub decls: Vec<Decl>,
    pub stmts: Vec<Stmt>,
}

