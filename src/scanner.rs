use srcmap;
use srcmap::{BytePos, Pos, SourceMap, Span};

/// The syntactic category of a token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
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

struct TokenAndSpan {
    tok: Token,
    sp: Span,
}

struct Scanner<'a> {
    source_map: &'a SourceMap,
    src: &'a str,
    ch: Option<char>,
    pos: BytePos,
    next_pos: BytePos,
    peek_tok: Token,
    peek_span: Span,
}

impl<'a> Scanner<'a> {
    fn new(source_map: &'a SourceMap) -> Scanner {
        let mut sc = Scanner {
            source_map,
            src: source_map.src.as_ref(),
            ch: Some('\n'),
            pos: Pos::from_usize(0),
            next_pos: Pos::from_usize(0),
            peek_tok: Token::Eof,
            peek_span: srcmap::DUMMY_SPAN,
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

    fn try_next_token(&mut self) -> Result<TokenAndSpan, ()> {
        while is_whitespace(self.ch) {
            self.bump();
        }

        if self.is_eof() {
            self.peek_tok = Token::Eof;
            self.peek_span = srcmap::DUMMY_SPAN;
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

    pub fn next_token(&mut self) -> TokenAndSpan {
        self.try_next_token().unwrap()
    }

    fn scan_token(&mut self) -> Result<Token, ()> {
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

                let tok = match self.source_map.span_to_snippet(Span {
                    start: start_bytepos,
                    end: self.pos,
                }) {
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
                        return Err(());
                    }
                }

                while is_dec_digit(self.ch) {
                    self.bump();
                }

                Ok(Token::Number)
            }
            '"' => {
                self.bump();

                while !(self.ch_is('"') || self.is_eof()) {
                    self.bump();
                    if self.ch_is('\n') {
                        return Err(());
                    }
                }

                if self.is_eof() {
                    return Err(());
                }

                assert_eq!(Some('"'), self.ch);
                self.bump();

                Ok(Token::StringLiteral)
            }
            _ => {
                self.bump();
                Err(())
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
mod tests {
    use super::{Scanner, SourceMap, Token, TokenAndSpan};

    #[test]
    fn scan_punctuators_test() {
        let sm = SourceMap::new(
            "test".into(),
            "( ) { } != ! == = >= > <= < * / + - , : ;".into(),
        );
        let mut sc = Scanner::new(&sm);

        assert_eq!(Token::LParen, sc.next_token().tok);
        assert_eq!(Token::RParen, sc.next_token().tok);
        assert_eq!(Token::LBrace, sc.next_token().tok);
        assert_eq!(Token::RBrace, sc.next_token().tok);
        assert_eq!(Token::ExclamaEqual, sc.next_token().tok);
        assert_eq!(Token::Exclama, sc.next_token().tok);
        assert_eq!(Token::EqualEqual, sc.next_token().tok);
        assert_eq!(Token::Equal, sc.next_token().tok);
        assert_eq!(Token::GreaterEqual, sc.next_token().tok);
        assert_eq!(Token::Greater, sc.next_token().tok);
        assert_eq!(Token::LesserEqual, sc.next_token().tok);
        assert_eq!(Token::Lesser, sc.next_token().tok);
        assert_eq!(Token::Star, sc.next_token().tok);
        assert_eq!(Token::Slash, sc.next_token().tok);
        assert_eq!(Token::Plus, sc.next_token().tok);
        assert_eq!(Token::Minus, sc.next_token().tok);
        assert_eq!(Token::Comma, sc.next_token().tok);
        assert_eq!(Token::Colon, sc.next_token().tok);
        assert_eq!(Token::Semi, sc.next_token().tok);
        assert_eq!(Token::Eof, sc.next_token().tok);
    }

    #[test]
    fn scan_identifiers_test() {
        let sm =
            SourceMap::new("test".into(), "a abc abc123 123abc _a_".into());
        let mut sc = Scanner::new(&sm);

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("a", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc123", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("123", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("abc", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("_a_", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_keywords_test() {
        let sm = SourceMap::new(
            "test".into(),
            "let int bool float str read readln write writeln if else while whileif".into(),
        );
        let mut sc = Scanner::new(&sm);

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Let, tok);
        assert_eq!("let", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Int, tok);
        assert_eq!("int", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Bool, tok);
        assert_eq!("bool", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Float, tok);
        assert_eq!("float", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Str, tok);
        assert_eq!("str", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Read, tok);
        assert_eq!("read", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Readln, tok);
        assert_eq!("readln", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Write, tok);
        assert_eq!("write", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Writeln, tok);
        assert_eq!("writeln", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::If, tok);
        assert_eq!("if", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Else, tok);
        assert_eq!("else", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::While, tok);
        assert_eq!("while", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Identifier, tok);
        assert_eq!("whileif", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_string_literals_test() {
        let sm = SourceMap::new(
            "test".into(),
            "\"\" \"foo bar 123 !!!\"".into(),
        );
        let mut sc = Scanner::new(&sm);

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::StringLiteral, tok);
        assert_eq!("\"\"", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::StringLiteral, tok);
        assert_eq!("\"foo bar 123 !!!\"", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn nonterminating_string_literal_test() {
        let sm = SourceMap::new(
            "test".into(),
            "\"abc".into(),
        );
        let mut sc = Scanner::new(&sm);

        let tok = sc.try_next_token();
        assert!(tok.is_err());

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn invalid_newline_in_string_literal_test() {
        let sm = SourceMap::new(
            "test".into(),
            "\"abc\n\"".into(),
        );
        let mut sc = Scanner::new(&sm);

        // Scans the first string.
        let tok = sc.try_next_token();
        assert!(tok.is_err());

        // Recognizes a second string.
        let tok = sc.try_next_token();
        assert!(tok.is_err());

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn scan_numbers_test() {
        let sm = SourceMap::new(
            "test".into(),
            "0 0123 3.14 3.14e42 0e0 0E0 0e+0 0e-0 0E+0 0E-0".into(),
        );
        let mut sc = Scanner::new(&sm);

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0123", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("3.14", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("3.14e42", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e+0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0e-0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E+0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, sp } = sc.next_token();
        assert_eq!(Token::Number, tok);
        assert_eq!("0E-0", sm.span_to_snippet(sp));

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }

    #[test]
    fn missing_exponent_digits_test() {
        let sm = SourceMap::new(
            "test".into(),
            "0e".into(),
        );
        let mut sc = Scanner::new(&sm);

        let tok = sc.try_next_token();
        assert!(tok.is_err());

        let TokenAndSpan { tok, .. } = sc.next_token();
        assert_eq!(Token::Eof, tok);
    }
}
