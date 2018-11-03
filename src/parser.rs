use scanner::{Scanner, Token};

#[derive(Debug)]
enum Ty {
    Bool,
    Int,
    Float,
    Str,
}

#[derive(Debug)]
enum CalTy {
    Read,
    ReadLn,
    Write,
    WriteLn,
}

#[derive(Debug)]
struct Ident {
    name: String,
}

#[derive(Debug)]
enum Lit {
    Int(u64),
    Float(f64),
    Str(String),
}

#[derive(Debug)]
enum BinOp {
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

#[derive(Debug)]
enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
enum Expr {
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnOp, Box<Expr>),
    Lit(Lit),
}

#[derive(Debug)]
struct Block {
    stmts: Vec<Box<Stmt>>,
}

#[derive(Debug)]
enum Stmt {
    While(Box<Expr>, Box<Block>),
    If(Box<Expr>, Box<Block>, Option<Box<Block>>),
    Block(Box<Block>),
    Assign(Ident, Box<Expr>),
    Call(CallTy, Vec<Ident>),
}

#[derive(Debug)]
struct Decl {
    id: Ident,
    ty: Ty,
}

#[derive(Debug)]
struct Program {
    decls: Vec<Box<Decl>>,
    stmts: Vec<Box<Stmt>>,
}

struct Parser {
    scanner: Scanner,
    peek_tok: Option<Token>,
}

impl Parser {
    fn new(scanner: Scanner) -> Parser {
        Parser {
            scanner,
        }
    }

    fn peek_is(&mut self, tok: Token) -> bool {
        match self.peek_tok {
            Some(peek_tok) => peek_tok == tok,
            None => {
                let next_tok = self.scanner.next_token();
            }
        }
    }

    fn consume(&self) -> Token {

    }
}
