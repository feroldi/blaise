use ast;
use errors::Diag;
use scanner::{Category, Word, WordStream};
use source_map::Span;
use std::collections::HashMap;
use std::result;

type Result<T> = result::Result<T, Diag>;

struct Parser<'a> {
    word_stream: WordStream<'a>,
    peek_word: Word,
    ident_table: HashMap<String, ast::Name>,
    last_name_id: u64,
}

impl<'a> Parser<'a> {
    fn new(mut word_stream: WordStream<'a>) -> Parser {
        let peek_word = word_stream.next();
        Parser {
            word_stream,
            peek_word,
            ident_table: HashMap::new(),
            last_name_id: 0,
        }
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
                    Category::Star => ast::BinOp::Add,
                    Category::Slash => ast::BinOp::Sub,
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
        use ast::{Expr, Ident, Lit};
        match self.peek_word.category {
            Category::OpenParen => {
                self.consume();
                let expr = self.parse_expr()?;
                self.expect_and_consume(Category::CloseParen)?;
                Ok(Expr::Paren(Box::new(expr)))
            }
            Category::StrLit => {
                let str_data =
                    self.get_peek_lexeme().trim_matches('"').to_owned();
                self.consume();
                Ok(Expr::Lit(Lit::StrLit(str_data)))
            }
            Category::NumConst { is_float: false } => {
                let value = self.get_peek_lexeme()
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
            Category::Ident => {
                let mut last_name_id = self.last_name_id;
                let ident = self.get_peek_lexeme().to_owned();
                let name =
                    *self.ident_table.entry(ident).or_insert_with(|| {
                        let name = ast::Name(last_name_id);
                        last_name_id += 1;
                        name
                    });
                self.last_name_id = last_name_id;
                self.consume();
                Ok(Expr::Ident(Ident { name }))
            }
            _ => unimplemented!(),
        }
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
    use source_map::SourceFile;
    use std::rc::Rc;

    fn create_parser<'a>(
        src: &str,
        handler: &'a errors::Handler,
    ) -> Parser<'a> {
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
        assert_eq!(
            Ok(ast::Expr::Lit(ast::Lit::IntLit(0))),
            parser.parse_expr()
        );
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
        let expr = ast::Expr::BinaryOp(
            ast::BinOp::Lt,
            Box::new(mk_int(0)),
            Box::new(mk_int(0)),
        );
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_equality_expr() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 == 0", &handler);
        let expr = ast::Expr::BinaryOp(
            ast::BinOp::Eq,
            Box::new(mk_int(0)),
            Box::new(mk_int(0)),
        );
        assert_eq!(Ok(expr), parser.parse_expr());
    }

    #[test]
    fn test_parse_term() {
        let handler = errors::Handler::with_ignoring_emitter();
        let mut parser = create_parser("0 * 0", &handler);
        let expr = ast::Expr::BinaryOp(
            ast::BinOp::Mult,
            Box::new(mk_int(0)),
            Box::new(mk_int(0)),
        );
        assert_eq!(Ok(expr), parser.parse_expr());
    }
}
