use ast;
use errors::Diag;
use scanner::{Category, Word, WordStream};
use source_map::Span;
use std::collections::HashMap;
use std::result;

type Result<T> = result::Result<T, Diag>;

pub struct Parser<'a> {
    word_stream: WordStream<'a>,
    peek_word: Word,
    ident_table: HashMap<String, ast::Name>,
    last_name_id: u64,
}

impl<'a> Parser<'a> {
    pub fn new(mut word_stream: WordStream<'a>) -> Parser {
        let peek_word = word_stream.next();
        Parser {
            word_stream,
            peek_word,
            ident_table: HashMap::new(),
            last_name_id: 0,
        }
    }

    fn is_start_of_statement(&self) -> bool {
        match self.peek_word.category {
            Category::Ident | Category::If | Category::While | Category::OpenCurly => true,
            _ => false,
        }
    }

    fn parse_block_stmt(&mut self) -> Result<ast::Stmt> {
        assert_eq!(Category::OpenCurly, self.peek_word.category);
        let block = self.parse_block()?;
        Ok(ast::Stmt::BlockStmt(Box::new(block)))
    }

    fn parse_command(&mut self) -> Result<ast::Stmt> {
        let stmt = match self.peek_word.category {
            Category::Ident => self.parse_assignment()?,
            Category::If => self.parse_selection()?,
            Category::While => self.parse_repetition()?,
            Category::OpenCurly => self.parse_block_stmt()?,
            _ => panic!("has to be the start of an statement!"),
        };
        Ok(stmt)
    }

    pub fn parse_program(&mut self) -> Result<ast::Program> {
        self.expect_and_consume(Category::Program)?;
        let prog_name = self.parse_ident()?;
        self.expect_and_consume(Category::Semi)?;

        let mut decls = vec![];
        let mut stmts = vec![];

        while self.peek_word.category == Category::Let {
            decls.push(self.parse_decl()?);
        }

        while self.is_start_of_statement() {
            stmts.push(self.parse_command()?);
        }

        Ok(ast::Program {
            name: prog_name,
            decls,
            stmts,
        })
    }

    fn parse_decl(&mut self) -> Result<ast::Decl> {
        assert_eq!(Category::Let, self.peek_word.category);
        self.expect_and_consume(Category::Let)?;
        let ident = self.parse_ident()?;
        self.expect_and_consume(Category::Colon)?;
        let ty = self.parse_ty()?;
        self.expect_and_consume(Category::Semi)?;
        Ok(ast::Decl { ident, ty })
    }

    fn parse_ty(&mut self) -> Result<ast::Ty> {
        let ty_word = self.expect_one_of_and_consume(&[
            Category::Bool,
            Category::Int,
            Category::Float,
            Category::Str,
        ])?;

        let ty = match ty_word.category {
            Category::Bool => ast::Ty::BoolTy,
            Category::Int => ast::Ty::IntTy,
            Category::Float => ast::Ty::FloatTy,
            Category::Str => ast::Ty::StrTy,
            _ => panic!("has to be a type!"),
        };

        Ok(ty)
    }

    fn parse_block(&mut self) -> Result<ast::Block> {
        self.expect_and_consume(Category::OpenCurly)?;
        let mut commands = vec![self.parse_command()?];
        while self.is_start_of_statement() {
            commands.push(self.parse_command()?);
        }
        self.expect_and_consume(Category::CloseCurly)?;
        Ok(ast::Block { stmts: commands })
    }

    fn parse_call(&mut self, func_id: ast::Ident) -> Result<ast::Stmt> {
        self.expect_and_consume(Category::OpenParen)?;
        let mut args = vec![];
        while self.peek_word.category != Category::CloseParen {
            args.push(self.parse_expr()?);
            if self.peek_word.category == Category::CloseParen {
                break;
            }
            self.expect_and_consume(Category::Comma)?;
        }
        self.expect_and_consume(Category::CloseParen)?;
        self.expect_and_consume(Category::Semi)?;
        Ok(ast::Stmt::Call(func_id, args))
    }

    fn parse_assignment(&mut self) -> Result<ast::Stmt> {
        assert_eq!(Category::Ident, self.peek_word.category);
        let ident = self.parse_ident()?;
        if self.peek_word.category == Category::OpenParen {
            return self.parse_call(ident);
        }
        self.expect_and_consume(Category::Eq)?;
        let expr = self.parse_expr()?;
        self.expect_and_consume(Category::Semi)?;
        Ok(ast::Stmt::Assign(ident, expr))
    }

    fn parse_selection(&mut self) -> Result<ast::Stmt> {
        assert_eq!(Category::If, self.peek_word.category);
        self.consume();
        let cond_expr = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.peek_word.category == Category::Else {
            self.consume();
            let else_block = self.parse_block()?;
            Some(Box::new(else_block))
        } else {
            None
        };

        Ok(ast::Stmt::If(cond_expr, Box::new(then_block), else_block))
    }

    fn parse_repetition(&mut self) -> Result<ast::Stmt> {
        assert_eq!(Category::While, self.peek_word.category);
        self.consume();
        let cond_expr = self.parse_expr()?;
        let block = self.parse_block()?;
        Ok(ast::Stmt::While(cond_expr, Box::new(block)))
    }

    fn parse_expr(&mut self) -> Result<ast::Expr> {
        use ast::Expr;
        let lhs_expr = self.parse_term()?;
        match self.peek_word.category {
            Category::Plus | Category::Minus => {
                let expr_cat = self.peek_word.category;
                self.consume();
                let rhs_expr = self.parse_expr()?;
                let expr_op = match expr_cat {
                    Category::Plus => ast::BinOp::Add,
                    Category::Minus => ast::BinOp::Sub,
                    _ => panic!("has to be an additive operator!"),
                };
                Ok(Expr::BinaryOp(
                    expr_op,
                    Box::new(lhs_expr),
                    Box::new(rhs_expr),
                ))
            }
            _ => Ok(lhs_expr),
        }
    }

    fn parse_term(&mut self) -> Result<ast::Expr> {
        use ast::Expr;
        let lhs_expr = self.parse_equality_expr()?;
        match self.peek_word.category {
            Category::Star | Category::Slash => {
                let term_cat = self.peek_word.category;
                self.consume();
                let rhs_expr = self.parse_term()?;
                let term_op = match term_cat {
                    Category::Star => ast::BinOp::Mult,
                    Category::Slash => ast::BinOp::Div,
                    _ => panic!("has to be a multiplicative operator!"),
                };
                Ok(Expr::BinaryOp(
                    term_op,
                    Box::new(lhs_expr),
                    Box::new(rhs_expr),
                ))
            }
            _ => Ok(lhs_expr),
        }
    }

    fn parse_equality_expr(&mut self) -> Result<ast::Expr> {
        use ast::Expr;
        let lhs_expr = self.parse_relational_expr()?;
        match self.peek_word.category {
            Category::EqEq | Category::Ne => {
                let eq_cat = self.peek_word.category;
                self.consume();
                let rhs_expr = self.parse_equality_expr()?;
                let eq_op = match eq_cat {
                    Category::EqEq => ast::BinOp::Eq,
                    Category::Ne => ast::BinOp::Ne,
                    _ => panic!("has to be an equality operator!"),
                };
                Ok(Expr::BinaryOp(
                    eq_op,
                    Box::new(lhs_expr),
                    Box::new(rhs_expr),
                ))
            }
            _ => Ok(lhs_expr),
        }
    }

    fn parse_relational_expr(&mut self) -> Result<ast::Expr> {
        use ast::Expr;
        let lhs_expr = self.parse_factor()?;
        match self.peek_word.category {
            Category::Lt | Category::Le | Category::Gt | Category::Ge => {
                let rel_cat = self.peek_word.category;
                self.consume();
                let rhs_expr = self.parse_relational_expr()?;
                let rel_op = match rel_cat {
                    Category::Lt => ast::BinOp::Lt,
                    Category::Le => ast::BinOp::Le,
                    Category::Gt => ast::BinOp::Gt,
                    Category::Ge => ast::BinOp::Ge,
                    _ => panic!("has to be a relational operator!"),
                };
                Ok(Expr::BinaryOp(
                    rel_op,
                    Box::new(lhs_expr),
                    Box::new(rhs_expr),
                ))
            }
            _ => Ok(lhs_expr),
        }
    }

    fn parse_factor(&mut self) -> Result<ast::Expr> {
        use ast::{Expr, Lit};
        match self.peek_word.category {
            Category::OpenParen => {
                self.consume();
                let expr = self.parse_expr()?;
                self.expect_and_consume(Category::CloseParen)?;
                Ok(Expr::Paren(Box::new(expr)))
            }
            Category::StrLit => {
                let str_data = self.get_peek_lexeme().trim_matches('"').to_owned();
                self.consume();
                Ok(Expr::Lit(Lit::StrLit(str_data)))
            }
            Category::NumConst { is_float: false } => {
                let value = self
                    .get_peek_lexeme()
                    .chars()
                    .flat_map(|c| c.to_digit(10))
                    .fold(0u64, |acc, val| acc * 10 + val as u64);
                self.consume();
                Ok(Expr::Lit(Lit::IntLit(value)))
            }
            Category::NumConst { is_float: true } => {
                let value = self.get_peek_lexeme().parse::<f64>().unwrap();
                self.consume();
                Ok(Expr::Lit(Lit::FloatLit(value)))
            }
            Category::Ident => Ok(Expr::Ident(self.parse_ident()?)),
            _ => unimplemented!(),
        }
    }

    fn register_name(&mut self, ident: String) -> ast::Name {
        let mut last_name_id = self.last_name_id;
        let name = *self.ident_table.entry(ident).or_insert_with(|| {
            let name = ast::Name(last_name_id);
            last_name_id += 1;
            name
        });
        self.last_name_id = last_name_id;
        name
    }

    fn parse_ident(&mut self) -> Result<ast::Ident> {
        let ident = self.expect_and_consume(Category::Ident)?;
        let id_lexeme = self.get_lexeme(ident.lexeme).to_owned();
        let name = self.register_name(id_lexeme);
        Ok(ast::Ident { name })
    }

    fn consume(&mut self) -> Word {
        let ate_word = self.peek_word;
        self.peek_word = self.word_stream.next();
        ate_word
    }

    fn expect_and_consume(&mut self, category: Category) -> Result<Word> {
        if self.peek_word.category == category {
            Ok(self.consume())
        } else {
            Err(Diag::ExpectedWord {
                expected: category,
                got: self.peek_word,
            })
        }
    }

    fn expect_one_of_and_consume(&mut self, categories: &[Category]) -> Result<Word> {
        if categories.contains(&self.peek_word.category) {
            Ok(self.consume())
        } else {
            Err(Diag::ExpectedOneOf {
                expected: categories.to_owned(),
                got: self.peek_word,
            })
        }
    }

    fn get_lexeme(&self, sp: Span) -> &str {
        self.word_stream.scanner.source_file.span_to_snippet(sp)
    }

    fn get_peek_lexeme(&self) -> &str {
        self.get_lexeme(self.peek_word.lexeme)
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use ast;
    use errors;
    use scanner::{Category, Scanner, Word, WordStream};
    use source_map::{BytePos, SourceFile, Span};
    use std::rc::Rc;

    fn create_parser<'a>(src: &str, handler: &'a errors::Handler) -> Parser<'a> {
        let file = Rc::new(SourceFile::new("test".into(), src.into()));
        let scanner = Scanner::new(file);
        let word_stream = WordStream::new(scanner, handler);
        Parser::new(word_stream)
    }

    fn mk_int(v: u64) -> ast::Expr {
        ast::Expr::Lit(ast::Lit::IntLit(v))
    }

    #[test]
    fn test_unexpected_end_of_file() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("", &handler);

        assert_eq!(Word::eof(), parser.consume());
    }

    #[test]
    fn test_parse_str_lit() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("\"abc 123!\"", &handler);
        assert_eq!(
            Ok(ast::Expr::Lit(ast::Lit::StrLit("abc 123!".into()))),
            parser.parse_expr()
        );
    }

    #[test]
    fn test_parse_num_const() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 123 3.14 42e3", &handler);
        assert_eq!(Ok(ast::Expr::Lit(ast::Lit::IntLit(0))), parser.parse_expr());
        assert_eq!(
            Ok(ast::Expr::Lit(ast::Lit::IntLit(123))),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Lit(ast::Lit::FloatLit(3.14))),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Lit(ast::Lit::FloatLit(42000.0))),
            parser.parse_expr()
        );
    }

    #[test]
    fn test_parse_ident() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("aaa bbb aaa ccc bbb aaa a", &handler);
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(0) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(1) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(0) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(2) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(1) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(0) })),
            parser.parse_expr()
        );
        assert_eq!(
            Ok(ast::Expr::Ident(ast::Ident { name: ast::Name(3) })),
            parser.parse_expr()
        );
    }

    #[test]
    fn test_parse_paren() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("(((0)))", &handler);
        let lit = ast::Expr::Lit(ast::Lit::IntLit(0));
        let paren1 = ast::Expr::Paren(Box::new(lit));
        let paren2 = ast::Expr::Paren(Box::new(paren1));
        let paren3 = ast::Expr::Paren(Box::new(paren2));

        assert_eq!(Ok(paren3), parser.parse_expr());
    }

    #[test]
    fn test_parse_unbalanced_paren() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("(((0))", &handler);
        let diag = errors::Diag::ExpectedWord {
            expected: Category::CloseParen,
            got: Word::eof(),
        };
        assert_eq!(Err(diag), parser.parse_expr());
    }

    #[test]
    fn test_parse_relational_expr() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 < 0", &handler);
        let expr = ast::Expr::BinaryOp(ast::BinOp::Lt, Box::new(mk_int(0)), Box::new(mk_int(0)));
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_equality_expr() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 == 0", &handler);
        let expr = ast::Expr::BinaryOp(ast::BinOp::Eq, Box::new(mk_int(0)), Box::new(mk_int(0)));
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_term() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 * 0", &handler);
        let expr = ast::Expr::BinaryOp(ast::BinOp::Mult, Box::new(mk_int(0)), Box::new(mk_int(0)));
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_additive() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 + 0", &handler);
        let expr = ast::Expr::BinaryOp(ast::BinOp::Add, Box::new(mk_int(0)), Box::new(mk_int(0)));
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_selection() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("if 1 { x = 0; } else { x = 1; }", &handler);

        let stmt = ast::Stmt::If(
            mk_int(1),
            Box::new(ast::Block {
                stmts: vec![ast::Stmt::Assign(
                    ast::Ident { name: ast::Name(0) },
                    mk_int(0),
                )],
            }),
            Some(Box::new(ast::Block {
                stmts: vec![ast::Stmt::Assign(
                    ast::Ident { name: ast::Name(0) },
                    mk_int(1),
                )],
            })),
        );

        assert_eq!(Ok(stmt), parser.parse_selection());
    }

    #[test]
    fn test_parse_selection_without_else() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("if 1 { x = 0; }", &handler);

        let stmt = ast::Stmt::If(
            mk_int(1),
            Box::new(ast::Block {
                stmts: vec![ast::Stmt::Assign(
                    ast::Ident { name: ast::Name(0) },
                    mk_int(0),
                )],
            }),
            None,
        );

        assert_eq!(Ok(stmt), parser.parse_selection());
    }

    #[test]
    fn test_parse_repetition() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("while 1 { x = 0; }", &handler);

        let stmt = ast::Stmt::While(
            mk_int(1),
            Box::new(ast::Block {
                stmts: vec![ast::Stmt::Assign(
                    ast::Ident { name: ast::Name(0) },
                    mk_int(0),
                )],
            }),
        );

        assert_eq!(Ok(stmt), parser.parse_repetition());
    }

    #[test]
    fn test_parse_block() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("{ x = 0; y = 1; x = 2; }", &handler);

        let stmt = ast::Stmt::BlockStmt(Box::new(ast::Block {
            stmts: vec![
                ast::Stmt::Assign(ast::Ident { name: ast::Name(0) }, mk_int(0)),
                ast::Stmt::Assign(ast::Ident { name: ast::Name(1) }, mk_int(1)),
                ast::Stmt::Assign(ast::Ident { name: ast::Name(0) }, mk_int(2)),
            ],
        }));

        assert_eq!(Ok(stmt), parser.parse_block_stmt());
    }

    #[test]
    fn test_parse_decl() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("let i: int;", &handler);

        let decl = ast::Decl {
            ident: ast::Ident { name: ast::Name(0) },
            ty: ast::Ty::IntTy,
        };

        assert_eq!(Ok(decl), parser.parse_decl());
    }

    #[test]
    fn test_parse_program() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("program a; let i: int; i = 42;", &handler);

        let prog = ast::Program {
            name: ast::Ident { name: ast::Name(0) },
            decls: vec![ast::Decl {
                ident: ast::Ident { name: ast::Name(1) },
                ty: ast::Ty::IntTy,
            }],
            stmts: vec![ast::Stmt::Assign(
                ast::Ident { name: ast::Name(1) },
                mk_int(42),
            )],
        };

        assert_eq!(Ok(prog), parser.parse_program());
    }

    #[test]
    fn test_parse_program_missing_semi() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("program a let i: int; i = 42;", &handler);

        let diag = errors::Diag::ExpectedWord {
            expected: Category::Semi,
            got: Word {
                category: Category::Let,
                lexeme: Span {
                    start: BytePos(10),
                    end: BytePos(13),
                },
            },
        };

        assert_eq!(Err(diag), parser.parse_program());
    }
}
