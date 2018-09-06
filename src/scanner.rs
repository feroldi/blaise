use std::str::CharIndices;
use std::iter::Peekable;
use codemap::BytePos;

#[derive(Debug, PartialEq, Eq)]
enum Category {
    Identifier,
    Number,
    StringLiteral,
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
    Eof,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
struct Token {
    category: Category,
    location: BytePos,
    lexeme: String,
}

struct Scanner<'a> {
    char_stream: Peekable<CharIndices<'a>>,
}

impl<'a> Scanner<'a> {
    fn with_str(input: &'a str) -> Scanner<'a> {
        Scanner {
            char_stream: input.char_indices().peekable(),
        }
    }

    fn peek(&mut self) -> char {
        self.char_stream.peek().map_or('\0', |&(_, c)| c)
    }

    fn consume(&mut self) -> char {
        self.char_stream.next().map_or('\0', |(_, c)| c)
    }

    fn location(&mut self) -> BytePos {
        self.char_stream
            .peek()
            .map_or(BytePos::invalid(), |&(i, _)| BytePos(i))
    }

    fn next(&mut self) -> Token {
        while self.peek().is_whitespace() {
            self.consume();
        }

        let location = self.location();
        let c = self.consume();

        let mut lexeme = String::with_capacity(8);
        lexeme.push(c);

        let category = match c {
            '(' => Category::LParen,
            ')' => Category::RParen,
            '{' => Category::LBrace,
            '}' => Category::RBrace,
            '!' => {
                if '=' == self.peek() {
                    lexeme.push(self.consume());
                    Category::ExclamaEqual
                } else {
                    Category::Exclama
                }
            }
            '=' => {
                if '=' == self.peek() {
                    lexeme.push(self.consume());
                    Category::EqualEqual
                } else {
                    Category::Equal
                }
            }
            '>' => {
                if '=' == self.peek() {
                    lexeme.push(self.consume());
                    Category::GreaterEqual
                } else {
                    Category::Greater
                }
            }
            '<' => {
                if '=' == self.peek() {
                    lexeme.push(self.consume());
                    Category::LesserEqual
                } else {
                    Category::Lesser
                }
            }
            '*' => Category::Star,
            '/' => Category::Slash,
            '+' => Category::Plus,
            '-' => Category::Minus,
            ',' => Category::Comma,
            ':' => Category::Colon,
            ';' => Category::Semi,
            _ if c.is_alphabetic() => {
                while self.peek().is_alphanumeric() {
                    lexeme.push(self.consume());
                }
                let category = match lexeme.as_ref() {
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
                    _ => Category::Identifier,
                };
                category
            }
            _ if c.is_digit(10) => {
                let mut had_error = false;
                while self.peek().is_digit(10) {
                    lexeme.push(self.consume());
                }
                if '.' == self.peek() {
                    lexeme.push(self.consume());
                    while self.peek().is_digit(10) {
                        lexeme.push(self.consume());
                    }
                }
                if 'e' == self.peek() || 'E' == self.peek() {
                    lexeme.push(self.consume());
                    if '+' == self.peek() || '-' == self.peek() {
                        lexeme.push(self.consume());
                    }
                    if self.peek().is_digit(10) {
                        lexeme.push(self.consume());
                        while self.peek().is_digit(10) {
                            lexeme.push(self.consume());
                        }
                    } else {
                        had_error = true;
                    }
                }
                if self.peek().is_alphabetic() {
                    had_error = true;
                    while self.peek().is_alphanumeric() {
                        lexeme.push(self.consume());
                    }
                }
                if !had_error {
                    Category::Number
                } else {
                    Category::Unknown
                }
            }
            '"' => {
                while '"' != self.peek() && '\0' != self.peek() {
                    lexeme.push(self.consume());
                }
                if '"' == self.peek() {
                    lexeme.push(self.consume());
                    Category::StringLiteral
                } else {
                    Category::Unknown
                }
            }
            '\0' => Category::Eof,
            _ => Category::Unknown,
        };

        Token {
            category,
            location,
            lexeme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_char_stream() {
        let mut scanner = Scanner::with_str("abc");
        assert_eq!(BytePos(0), scanner.location());
        assert_eq!('a', scanner.peek());
        assert_eq!('a', scanner.consume());
        assert_eq!(BytePos(1), scanner.location());
        assert_eq!('b', scanner.peek());
        assert_eq!('b', scanner.consume());
        assert_eq!(BytePos(2), scanner.location());
        assert_eq!('c', scanner.peek());
        assert_eq!('c', scanner.consume());
        assert_eq!(BytePos::invalid(), scanner.location());
        assert_eq!('\0', scanner.peek());
        assert_eq!('\0', scanner.consume());
    }

    #[test]
    fn punctuators() {
        let mut scanner = Scanner::with_str("( ) { } != == = >= > <= < * / + - , : ;");

        assert_eq!(
            Token {
                category: Category::LParen,
                location: BytePos(0),
                lexeme: "(".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::RParen,
                location: BytePos(2),
                lexeme: ")".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::LBrace,
                location: BytePos(4),
                lexeme: "{".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::RBrace,
                location: BytePos(6),
                lexeme: "}".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::ExclamaEqual,
                location: BytePos(8),
                lexeme: "!=".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::EqualEqual,
                location: BytePos(11),
                lexeme: "==".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Equal,
                location: BytePos(14),
                lexeme: "=".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::GreaterEqual,
                location: BytePos(16),
                lexeme: ">=".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Greater,
                location: BytePos(19),
                lexeme: ">".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::LesserEqual,
                location: BytePos(21),
                lexeme: "<=".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Lesser,
                location: BytePos(24),
                lexeme: "<".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Star,
                location: BytePos(26),
                lexeme: "*".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Slash,
                location: BytePos(28),
                lexeme: "/".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Plus,
                location: BytePos(30),
                lexeme: "+".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Minus,
                location: BytePos(32),
                lexeme: "-".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Comma,
                location: BytePos(34),
                lexeme: ",".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Colon,
                location: BytePos(36),
                lexeme: ":".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Semi,
                location: BytePos(38),
                lexeme: ";".into(),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Eof,
                location: BytePos::invalid(),
                lexeme: "\0".into(),
            },
            scanner.next()
        );
    }

    #[test]
    fn identifiers() {
        let mut scanner = Scanner::with_str("a abc abc123 123abc");

        let tok = scanner.next();
        assert_eq!(Category::Identifier, tok.category);
        assert_eq!("a", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Identifier, tok.category);
        assert_eq!("abc", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Identifier, tok.category);
        assert_eq!("abc123", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Unknown, tok.category);
        assert_eq!("123abc", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Eof, tok.category);
    }

    #[test]
    fn keywords() {
        let mut scanner = Scanner::with_str(
            "let int bool float str read readln write writeln if else while whileif",
        );

        let tok = scanner.next();
        assert_eq!(Category::Let, tok.category);
        assert_eq!("let", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Int, tok.category);
        assert_eq!("int", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Bool, tok.category);
        assert_eq!("bool", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Float, tok.category);
        assert_eq!("float", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Str, tok.category);
        assert_eq!("str", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Read, tok.category);
        assert_eq!("read", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Readln, tok.category);
        assert_eq!("readln", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Write, tok.category);
        assert_eq!("write", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Writeln, tok.category);
        assert_eq!("writeln", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::If, tok.category);
        assert_eq!("if", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Else, tok.category);
        assert_eq!("else", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::While, tok.category);
        assert_eq!("while", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Identifier, tok.category);
        assert_eq!("whileif", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Eof, tok.category);
    }

    #[test]
    fn string_literals() {
        let mut scanner = Scanner::with_str("\"\" \"foo bar 123 !!!\"");

        let tok = scanner.next();
        assert_eq!(Category::StringLiteral, tok.category);
        assert_eq!("\"\"", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::StringLiteral, tok.category);
        assert_eq!("\"foo bar 123 !!!\"", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Eof, tok.category);
    }

    #[test]
    fn nonterminating_string_literal() {
        let mut scanner = Scanner::with_str("\"abc");

        let tok = scanner.next();
        assert_eq!(Category::Unknown, tok.category);
        assert_eq!("\"abc", tok.lexeme);

        let tok = scanner.next();
        assert_eq!(Category::Eof, tok.category);
    }
}
