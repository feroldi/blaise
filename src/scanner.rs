use diagnostics::ReportCode;
use srcmap::{BytePos, Pos, SourceFile, Span, DUMMY_SPAN};
use std::rc::Rc;

/// The syntactic category of a token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    ExclamaEqual,
    Exclama,
    Equal,
    EqualEqual,
    GreaterEqual,
    Greater,
    LesserEqual,
    Lesser,
    Star,
    Slash,
    Plus,
    Minus,
    Comma,
    Colon,
    Semi,
    Program,
    Let,
    Int,
    Bool,
    Float,
    Str,
    Read,
    Readln,
    Write,
    Writeln,
    If,
    Else,
    While,
    Identifier,
    Number,
    StringLiteral,
    Eof,
}

/// A token and its lexeme information given by a span.
#[derive(Debug)]
pub struct TokenAndSpan {
    /// The scanned token.
    tok: Token,
    /// The lexeme information of this token.
    sp: Span,
}

/// The scanner.
///
/// This struct provides an interface to perform concurrent lexical analysis
/// over a source file. In other words, it transforms a source file (i.e. text
/// buffer) into a stream of tokens.
pub struct Scanner {
    source_file: Rc<SourceFile>,
    src: Rc<String>,
    ch: Option<char>,
    pos: BytePos,
    next_pos: BytePos,
    peek_tok: Token,
    peek_span: Span,
}

impl Scanner {
    /// Creates a scanner for a source file.
    pub fn new(source_file: Rc<SourceFile>) -> Scanner {
        let src = source_file.src.clone();
        let mut sc = Scanner {
            source_file,
            src,
            ch: Some('\n'),
            pos: Pos::from_usize(0),
            next_pos: Pos::from_usize(0),
            peek_tok: Token::Eof,
            peek_span: DUMMY_SPAN,
        };

        sc.bump();
        sc
    }

    fn ch_is(&self, c: char) -> bool {
        self.ch == Some(c)
    }

    fn is_eof(&self) -> bool {
        self.ch.is_none()
    }

    /// Advances the Scanner by one character.
    fn bump(&mut self) {
        let next_pos_idx = self.next_pos.to_usize();

        if next_pos_idx < self.src.len() {
            let next_ch = self.src[next_pos_idx..].chars().next().unwrap();
            let next_ch_len = next_ch.len_utf8();

            self.ch = Some(next_ch);
            self.pos = self.next_pos;
            self.next_pos = self.next_pos + Pos::from_usize(next_ch_len);
        } else {
            self.ch = None;
            self.pos = self.next_pos;
        }
    }

    /// Parses a token from the source file, and advances the text buffer
    /// cursor.
    ///
    /// Usage comes down to calling `next_token` in order to parse one token
    /// from the source file at a time. This function returns either a
    /// successfully scanned token, or a parsing error, which can be
    /// reported by a diagnostic handler.
    ///
    /// # Example
    ///
    /// ```
    /// use scanner::{Scanner, TokenAndSpan};
    /// use srcmap::{ByteCode, SourceMap, Span, DUMMY_SPAN};
    ///
    /// let source_file =
    ///     Rc::new(SourceFile::new("test".into(), "abc 123\n".into()));
    /// let mut scanner = Scanner::new(source_file);
    /// let tok1 = TokenAndSpan {
    ///     tok: Token::Identifier,
    ///     sp: Span {
    ///         start: BytePos(0),
    ///         end: BytePos(3),
    ///     },
    /// };
    /// let tok2 = TokenAndSpan {
    ///     tok: Token::Number,
    ///     sp: Span {
    ///         start: BytePos(3),
    ///         end: BytePos(6),
    ///     },
    /// };
    /// let tok3 = TokenAndSpan {
    ///     tok: Token::Eof,
    ///     sp: DUMMY_SPAN,
    /// };
    ///
    /// assert_eq!(Ok(tok1), scanner.next_token());
    /// assert_eq!(Ok(tok2), scanner.next_token());
    /// assert_eq!(Ok(tok3), scanner.next_token());
    /// ```
    pub fn next_token(&mut self) -> Result<TokenAndSpan, ReportCode> {
        while is_whitespace(self.ch) {
            self.bump();
        }

        if self.is_eof() {
            self.peek_tok = Token::Eof;
            self.peek_span = DUMMY_SPAN;
        } else {
            let tok_start_pos = self.pos;
            self.peek_tok = self.scan_token()?;
            let tok_end_pos = self.pos;
            self.peek_span = Span {
                start: tok_start_pos,
                end: tok_end_pos,
            };
        }

        Ok(TokenAndSpan {
            tok: self.peek_tok,
            sp: self.peek_span,
        })
    }

    fn scan_token(&mut self) -> Result<Token, ReportCode> {
        match self.ch.expect("scan_token called on EOF") {
            '(' => {
                self.bump();
                Ok(Token::LParen)
            }
            ')' => {
                self.bump();
                Ok(Token::RParen)
            }
            '{' => {
                self.bump();
                Ok(Token::LBrace)
            }
            '}' => {
                self.bump();
                Ok(Token::RBrace)
            }
            '!' => {
                self.bump();
                Ok(if self.ch_is('=') {
                    self.bump();
                    Token::ExclamaEqual
                } else {
                    Token::Exclama
                })
            }
            '=' => {
                self.bump();
                Ok(if self.ch_is('=') {
                    self.bump();
                    Token::EqualEqual
                } else {
                    Token::Equal
                })
            }
            '>' => {
                self.bump();
                Ok(if self.ch_is('=') {
                    self.bump();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                })
            }
            '<' => {
                self.bump();
                Ok(if self.ch_is('=') {
                    self.bump();
                    Token::LesserEqual
                } else {
                    Token::Lesser
                })
            }
            '*' => {
                self.bump();
                Ok(Token::Star)
            }
            '/' => {
                self.bump();
                Ok(Token::Slash)
            }
            '+' => {
                self.bump();
                Ok(Token::Plus)
            }
            '-' => {
                self.bump();
                Ok(Token::Minus)
            }
            ',' => {
                self.bump();
                Ok(Token::Comma)
            }
            ':' => {
                self.bump();
                Ok(Token::Colon)
            }
            ';' => {
                self.bump();
                Ok(Token::Semi)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_bytepos = self.pos;
                self.bump();

                while is_ident_body(self.ch) {
                    self.bump();
                }

                let tok = match self.source_file.span_to_snippet(Span {
                    start: start_bytepos,
                    end: self.pos,
                }) {
                    "program" => Token::Program,
                    "let" => Token::Let,
                    "int" => Token::Int,
                    "bool" => Token::Bool,
                    "float" => Token::Float,
                    "str" => Token::Str,
                    "read" => Token::Read,
                    "readln" => Token::Readln,
                    "write" => Token::Write,
                    "writeln" => Token::Writeln,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    _ => Token::Identifier,
                };

                Ok(tok)
            }
            '0'..='9' => {
                self.bump();

                while is_dec_digit(self.ch) {
                    self.bump();
                }

                if self.ch_is('.') {
                    self.bump();
                }

                while is_dec_digit(self.ch) {
                    self.bump();
                }

                if self.ch_is('e') || self.ch_is('E') {
                    self.bump();

                    if self.ch_is('-') || self.ch_is('+') {
                        self.bump();
                    }

                    if !is_dec_digit(self.ch) {
                        return Err(ReportCode::MissingExponentDigits {
                            exp_pos: self.pos - BytePos(1),
                        });
                    }
                }

                while is_dec_digit(self.ch) {
                    self.bump();
                }

                Ok(Token::Number)
            }
            '"' => {
                let str_start_pos = self.pos;
                self.bump();

                while !(self.ch_is('"') || self.ch_is('\n') || self.is_eof()) {
                    self.bump();
                }

                if self.ch_is('\n') || self.is_eof() {
                    return Err(ReportCode::MissingTerminatingStringMark {
                        str_start_pos,
                        eol_pos: self.pos,
                    });
                }

                assert_eq!(Some('"'), self.ch);
                self.bump();

                Ok(Token::StringLiteral)
            }
            _ => {
                let pos = self.pos;
                self.bump();
                Err(ReportCode::UnknownCharacter { pos })
            }
        }
    }
}

fn is_ident_body(c: Option<char>) -> bool {
    let c = match c {
        Some(c) => c,
        _ => return false,
    };

    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
        _ => false,
    }
}

fn is_dec_digit(c: Option<char>) -> bool {
    match c {
        Some(c) => '0' <= c && c <= '9',
        _ => false,
    }
}

fn is_whitespace(c: Option<char>) -> bool {
    c.map_or(false, |c| c.is_whitespace())
}

#[cfg(test)]
mod test {
    use super::{BytePos, Scanner, SourceFile, Token, TokenAndSpan};
    use diagnostics::ReportCode;
    use std::rc::Rc;

    fn create_scanner(src: &str) -> (Scanner, Rc<SourceFile>) {
        let file = Rc::new(SourceFile::new("test".into(), src.into()));
        let scanner = Scanner::new(file.clone());
        (scanner, file)
    }

    #[test]
    fn scan_punctuators_test() {
        let (mut sc, _) =
            create_scanner("( ) { } != ! == = >= > <= < * / + - , : ;");

        assert_eq!(Token::LParen, sc.next_token().unwrap().tok);
        assert_eq!(Token::RParen, sc.next_token().unwrap().tok);
        assert_eq!(Token::LBrace, sc.next_token().unwrap().tok);
        assert_eq!(Token::RBrace, sc.next_token().unwrap().tok);
        assert_eq!(Token::ExclamaEqual, sc.next_token().unwrap().tok);
        assert_eq!(Token::Exclama, sc.next_token().unwrap().tok);
        assert_eq!(Token::EqualEqual, sc.next_token().unwrap().tok);
        assert_eq!(Token::Equal, sc.next_token().unwrap().tok);
        assert_eq!(Token::GreaterEqual, sc.next_token().unwrap().tok);
        assert_eq!(Token::Greater, sc.next_token().unwrap().tok);
        assert_eq!(Token::LesserEqual, sc.next_token().unwrap().tok);
        assert_eq!(Token::Lesser, sc.next_token().unwrap().tok);
        assert_eq!(Token::Star, sc.next_token().unwrap().tok);
        assert_eq!(Token::Slash, sc.next_token().unwrap().tok);
        assert_eq!(Token::Plus, sc.next_token().unwrap().tok);
        assert_eq!(Token::Minus, sc.next_token().unwrap().tok);
        assert_eq!(Token::Comma, sc.next_token().unwrap().tok);
        assert_eq!(Token::Colon, sc.next_token().unwrap().tok);
        assert_eq!(Token::Semi, sc.next_token().unwrap().tok);
        assert_eq!(Token::Eof, sc.next_token().unwrap().tok);
    }

    #[test]
    fn scan_identifiers_test() {
        let (mut sc, sf) = create_scanner("a abc abc123 123abc _a_");

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("a", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc123", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("123", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("_a_", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_keywords_test() {
        let (mut sc, sf) = create_scanner(
            "program let int bool float str read readln write writeln if else \
             while whileif",
        );

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Program, tok);
        assert_eq!("program", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Let, tok);
        assert_eq!("let", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Int, tok);
        assert_eq!("int", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Bool, tok);
        assert_eq!("bool", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Float, tok);
        assert_eq!("float", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Str, tok);
        assert_eq!("str", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Read, tok);
        assert_eq!("read", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Readln, tok);
        assert_eq!("readln", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Write, tok);
        assert_eq!("write", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Writeln, tok);
        assert_eq!("writeln", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::If, tok);
        assert_eq!("if", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Else, tok);
        assert_eq!("else", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::While, tok);
        assert_eq!("while", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("whileif", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_string_literals_test() {
        let (mut sc, sf) = create_scanner("\"\" \"foo bar 123 !!!\"");

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::StringLiteral, tok);
        assert_eq!("\"\"", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::StringLiteral, tok);
        assert_eq!("\"foo bar 123 !!!\"", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn nonterminating_string_literal_test() {
        let (mut sc, _) = create_scanner("\"abc");

        let tok = sc.next_token();
        assert_eq!(
            ReportCode::MissingTerminatingStringMark {
                str_start_pos: BytePos(0),
                eol_pos: BytePos(4),
            },
            tok.unwrap_err()
        );

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn invalid_newline_in_string_literal_test() {
        let (mut sc, _) = create_scanner("\"abc\n\"");

        // Scans the first string.
        let tok = sc.next_token();
        assert!(tok.is_err());

        // Recognizes a second string.
        let tok = sc.next_token();
        assert!(tok.is_err());

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_numbers_test() {
        let (mut sc, sf) =
            create_scanner("0 0123 3.14 3.14e42 0e0 0E0 0e+0 0e-0 0E+0 0E-0");

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0123", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("3.14", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("3.14e42", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e+0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e-0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E+0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token().unwrap();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E-0", sf.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn missing_exponent_digits_test() {
        let (mut sc, _) = create_scanner("0e");

        let tok = sc.next_token();
        assert_eq!(
            ReportCode::MissingExponentDigits {
                exp_pos: BytePos(1),
            },
            tok.unwrap_err()
        );

        let TokenAndSpan { tok, .. } = sc.next_token().unwrap();
        assert_eq!(Token::Eof, tok);
    }
}
