use std::fmt;
use std::rc::Rc;
use syntax;
use syntax::source_map::{BytePos, Pos, SourceFile, Span, DUMMY_SPAN};

#[derive(Debug)]
pub enum ScanError {
    /// Numeric literals with no digits after an exponent.
    MissingExponentDigits { exp_pos: BytePos },
    /// String literals missing a terminating quotation mark.
    MissingTerminatingStringMark {
        str_start_pos: BytePos,
        eol_pos: BytePos,
    },
    /// Unknown character in the source code.
    UnknownCharacter { pos: BytePos },
}

impl From<ScanError> for syntax::Error {
    fn from(err: ScanError) -> syntax::Error {
        syntax::Error::Scan(err)
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScanError::MissingExponentDigits { .. } => {
                write!(f, "missing exponent digits for decimal literal")
            }
            ScanError::MissingTerminatingStringMark { .. } => write!(
                f,
                "missing terminating quotation mark for string literal"
            ),
            ScanError::UnknownCharacter { .. } => {
                write!(f, "unknown character")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delim {
    Paren,
    Curly,
}

/// The syntactic category of a word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Category {
    OpenDelim(Delim),
    CloseDelim(Delim),
    Ne,
    Eq,
    EqEq,
    Ge,
    Gt,
    Le,
    Lt,
    Star,
    Slash,
    Plus,
    Minus,
    Not,
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
    Ident,
    Num,
    StrLit,
    Eof,
}

/// A word and its lexeme information given by a span.
#[derive(Debug)]
pub struct Word {
    /// The word's category.
    pub category: Category,
    /// The word's text representation.
    pub lexeme: Span,
}

impl Word {
    pub fn eof() -> Word {
        Word {
            category: Category::Eof,
            lexeme: DUMMY_SPAN,
        }
    }
}

/// The scanner.
///
/// This struct provides an interface to perform concurrent lexical analysis
/// over a source file. In other words, it transforms a source file (i.e. text
/// buffer) into a stream of words.
pub struct Scanner {
    source_file: Rc<SourceFile>,
    src: Rc<String>,
    peek_ch: Option<char>,
    pos: BytePos,
    next_pos: BytePos,
}

impl Scanner {
    /// Creates a scanner for a source file.
    pub fn new(source_file: Rc<SourceFile>) -> Scanner {
        let src = source_file.src.clone();
        let mut sc = Scanner {
            source_file,
            src,
            peek_ch: Some('\n'),
            pos: BytePos(0),
            next_pos: BytePos(0),
        };

        sc.bump();
        sc
    }

    fn ch_is(&self, c: char) -> bool {
        self.peek_ch == Some(c)
    }

    fn is_eof(&self) -> bool {
        self.peek_ch.is_none()
    }

    /// Advances the Scanner by one character.
    fn bump(&mut self) {
        let next_pos_idx = self.next_pos.to_usize();

        if next_pos_idx < self.src.len() {
            let next_ch = self.src[next_pos_idx..].chars().next().unwrap();
            let next_ch_len = next_ch.len_utf8();

            self.peek_ch = Some(next_ch);
            self.pos = self.next_pos;
            self.next_pos = self.next_pos + Pos::from_usize(next_ch_len);
        } else {
            self.peek_ch = None;
            self.pos = self.next_pos;
        }
    }

    /// Parses a word from the source file, and advances the text buffer
    /// cursor.
    ///
    /// Usage comes down to calling `next_word` in order to parse one word
    /// from the source file at a time. This function returns either a
    /// successfully scanned word, or a parsing error, which can be
    /// reported by a diagnostic handler.
    pub fn next_word(&mut self) -> Result<Word, ScanError> {
        while is_whitespace(self.peek_ch) {
            self.bump();
        }

        if self.is_eof() {
            Ok(Word::eof())
        } else {
            self.scan_word()
        }
    }

    fn scan_ident(&mut self) -> Result<Word, ScanError> {
        let id_start_pos = self.pos;
        self.bump();

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

        while is_ident_body(self.peek_ch) {
            self.bump();
        }

        let lexeme = Span {
            start: id_start_pos,
            end: self.pos,
        };

        let category = match self.source_file.span_to_snippet(lexeme) {
            "program" => Category::Program,
            "let" => Category::Let,
            "int" => Category::Int,
            "bool" => Category::Bool,
            "float" => Category::Float,
            "str" => Category::Str,
            "read" => Category::Read,
            "readln" => Category::Readln,
            "write" => Category::Write,
            "writeln" => Category::Writeln,
            "if" => Category::If,
            "else" => Category::Else,
            "while" => Category::While,
            _ => Category::Ident,
        };

        Ok(Word { category, lexeme })
    }

    fn scan_number(&mut self) -> Result<Word, ScanError> {
        let num_start_pos = self.pos;
        self.bump();

        fn is_dec_digit(c: Option<char>) -> bool {
            match c {
                Some(c) => '0' <= c && c <= '9',
                _ => false,
            }
        }

        while is_dec_digit(self.peek_ch) {
            self.bump();
        }

        if self.ch_is('.') {
            self.bump();
        }

        while is_dec_digit(self.peek_ch) {
            self.bump();
        }

        if self.ch_is('e') || self.ch_is('E') {
            let exponent_pos = self.pos;
            self.bump();

            if self.ch_is('-') || self.ch_is('+') {
                self.bump();
            }

            if !is_dec_digit(self.peek_ch) {
                return Err(ScanError::MissingExponentDigits {
                    exp_pos: exponent_pos,
                });
            }
        }

        while is_dec_digit(self.peek_ch) {
            self.bump();
        }

        Ok(Word {
            category: Category::Num,
            lexeme: Span {
                start: num_start_pos,
                end: self.pos,
            },
        })
    }

    fn scan_string_literal(&mut self) -> Result<Word, ScanError> {
        assert_eq!(Some('"'), self.peek_ch);
        let str_start_pos = self.pos;
        self.bump();

        while !(self.ch_is('"') || self.ch_is('\n') || self.is_eof()) {
            self.bump();
        }

        if self.ch_is('\n') || self.is_eof() {
            return Err(ScanError::MissingTerminatingStringMark {
                str_start_pos,
                eol_pos: self.pos,
            });
        }

        assert_eq!(Some('"'), self.peek_ch);
        self.bump();

        Ok(Word {
            category: Category::StrLit,
            lexeme: Span {
                start: str_start_pos,
                end: self.pos,
            },
        })
    }

    fn scan_word(&mut self) -> Result<Word, ScanError> {
        assert!(self.peek_ch.is_some());
        let start_pos = self.pos;

        let category = match self.peek_ch.unwrap() {
            '(' => {
                self.bump();
                Category::OpenDelim(Delim::Paren)
            }
            ')' => {
                self.bump();
                Category::CloseDelim(Delim::Paren)
            }
            '{' => {
                self.bump();
                Category::OpenDelim(Delim::Curly)
            }
            '}' => {
                self.bump();
                Category::CloseDelim(Delim::Curly)
            }
            '!' => {
                self.bump();
                if self.ch_is('=') {
                    self.bump();
                    Category::Ne
                } else {
                    Category::Not
                }
            }
            '=' => {
                self.bump();
                if self.ch_is('=') {
                    self.bump();
                    Category::EqEq
                } else {
                    Category::Eq
                }
            }
            '>' => {
                self.bump();
                if self.ch_is('=') {
                    self.bump();
                    Category::Ge
                } else {
                    Category::Gt
                }
            }
            '<' => {
                self.bump();
                if self.ch_is('=') {
                    self.bump();
                    Category::Le
                } else {
                    Category::Lt
                }
            }
            '*' => {
                self.bump();
                Category::Star
            }
            '/' => {
                self.bump();
                Category::Slash
            }
            '+' => {
                self.bump();
                Category::Plus
            }
            '-' => {
                self.bump();
                Category::Minus
            }
            ',' => {
                self.bump();
                Category::Comma
            }
            ':' => {
                self.bump();
                Category::Colon
            }
            ';' => {
                self.bump();
                Category::Semi
            }
            'a'..='z' | 'A'..='Z' | '_' => return self.scan_ident(),
            '0'..='9' => return self.scan_number(),
            '"' => return self.scan_string_literal(),
            _ => {
                let pos = self.pos;
                self.bump();
                return Err(ScanError::UnknownCharacter { pos });
            }
        };

        Ok(Word {
            category,
            lexeme: Span {
                start: start_pos,
                end: self.pos,
            },
        })
    }
}

fn is_whitespace(c: Option<char>) -> bool {
    c.map_or(false, |c| c.is_whitespace())
}

#[cfg(test)]
mod test {
    use super::{BytePos, Category, Delim, ScanError, Scanner, SourceFile, Word};
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

        assert_eq!(
            Category::OpenDelim(Delim::Paren),
            sc.next_word().unwrap().category
        );
        assert_eq!(
            Category::CloseDelim(Delim::Paren),
            sc.next_word().unwrap().category
        );
        assert_eq!(
            Category::OpenDelim(Delim::Curly),
            sc.next_word().unwrap().category
        );
        assert_eq!(
            Category::CloseDelim(Delim::Curly),
            sc.next_word().unwrap().category
        );
        assert_eq!(Category::Ne, sc.next_word().unwrap().category);
        assert_eq!(Category::Not, sc.next_word().unwrap().category);
        assert_eq!(Category::EqEq, sc.next_word().unwrap().category);
        assert_eq!(Category::Eq, sc.next_word().unwrap().category);
        assert_eq!(Category::Ge, sc.next_word().unwrap().category);
        assert_eq!(Category::Gt, sc.next_word().unwrap().category);
        assert_eq!(Category::Le, sc.next_word().unwrap().category);
        assert_eq!(Category::Lt, sc.next_word().unwrap().category);
        assert_eq!(Category::Star, sc.next_word().unwrap().category);
        assert_eq!(Category::Slash, sc.next_word().unwrap().category);
        assert_eq!(Category::Plus, sc.next_word().unwrap().category);
        assert_eq!(Category::Minus, sc.next_word().unwrap().category);
        assert_eq!(Category::Comma, sc.next_word().unwrap().category);
        assert_eq!(Category::Colon, sc.next_word().unwrap().category);
        assert_eq!(Category::Semi, sc.next_word().unwrap().category);
        assert_eq!(Category::Eof, sc.next_word().unwrap().category);
    }

    #[test]
    fn scan_identifiers_test() {
        let (mut sc, sf) = create_scanner("a abc abc123 123abc _a_");

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("a", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("abc", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("abc123", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("123", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("abc", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("_a_", sf.span_to_snippet(lexeme));

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn scan_keywords_test() {
        let (mut sc, sf) = create_scanner(
            "program let int bool float str read readln write writeln if else \
             while whileif",
        );

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Program, category);
        assert_eq!("program", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Let, category);
        assert_eq!("let", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Int, category);
        assert_eq!("int", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Bool, category);
        assert_eq!("bool", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Float, category);
        assert_eq!("float", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Str, category);
        assert_eq!("str", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Read, category);
        assert_eq!("read", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Readln, category);
        assert_eq!("readln", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Write, category);
        assert_eq!("write", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Writeln, category);
        assert_eq!("writeln", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::If, category);
        assert_eq!("if", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Else, category);
        assert_eq!("else", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::While, category);
        assert_eq!("while", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Ident, category);
        assert_eq!("whileif", sf.span_to_snippet(lexeme));

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn scan_string_literals_test() {
        let (mut sc, sf) = create_scanner("\"\" \"foo bar 123 !!!\"");

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::StrLit, category);
        assert_eq!("\"\"", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::StrLit, category);
        assert_eq!("\"foo bar 123 !!!\"", sf.span_to_snippet(lexeme));

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn nonterminating_string_literal_test() {
        let (mut sc, _) = create_scanner("\"abc");

        let word = sc.next_word();
        assert!(match word {
            Err(ScanError::MissingTerminatingStringMark {
                str_start_pos: BytePos(0),
                eol_pos: BytePos(4),
            }) => true,
            _ => false,
        });

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn invalid_newline_in_string_literal_test() {
        let (mut sc, _) = create_scanner("\"abc\n\"");

        // Scans the first string.
        let word = sc.next_word();

        assert!(match word {
            Err(ScanError::MissingTerminatingStringMark {
                str_start_pos: BytePos(0),
                eol_pos: BytePos(4),
            }) => true,
            _ => false,
        });

        // Recognizes a second string.
        let word = sc.next_word();
        assert!(word.is_err());

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn scan_numbers_test() {
        let (mut sc, sf) =
            create_scanner("0 0123 3.14 3.14e42 0e0 0E0 0e+0 0e-0 0E+0 0E-0");

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0123", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("3.14", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("3.14e42", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0e0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0E0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0e+0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0e-0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0E+0", sf.span_to_snippet(lexeme));

        let Word { category, lexeme } = sc.next_word().unwrap();
        assert_eq!(Category::Num, category);
        assert_eq!("0E-0", sf.span_to_snippet(lexeme));

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }

    #[test]
    fn missing_exponent_digits_test() {
        let (mut sc, _) = create_scanner("0e");

        let word = sc.next_word();
        assert!(match word {
            Err(ScanError::MissingExponentDigits {
                exp_pos: BytePos(1),
            }) => true,
            _ => false,
        });

        let Word { category, .. } = sc.next_word().unwrap();
        assert_eq!(Category::Eof, category);
    }
}
