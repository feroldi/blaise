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

        let category = match c {
            '(' => Category::LParen,
            ')' => Category::RParen,
            '{' => Category::LBrace,
            '}' => Category::RBrace,
            '!' => {
                if '=' == self.peek() {
                    self.consume();
                    Category::ExclamaEqual
                } else {
                    Category::Exclama
                }
            }
            '=' => {
                if '=' == self.peek() {
                    self.consume();
                    Category::EqualEqual
                } else {
                    Category::Equal
                }
            }
            '>' => {
                if '=' == self.peek() {
                    self.consume();
                    Category::GreaterEqual
                } else {
                    Category::Greater
                }
            }
            '<' => {
                if '=' == self.peek() {
                    self.consume();
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
            '\0' => Category::Eof,
            _ => Category::Unknown,
        };

        Token { category, location }
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
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::RParen,
                location: BytePos(2),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::LBrace,
                location: BytePos(4),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::RBrace,
                location: BytePos(6),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::ExclamaEqual,
                location: BytePos(8),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::EqualEqual,
                location: BytePos(11),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Equal,
                location: BytePos(14),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::GreaterEqual,
                location: BytePos(16),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Greater,
                location: BytePos(19),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::LesserEqual,
                location: BytePos(21),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Lesser,
                location: BytePos(24),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Star,
                location: BytePos(26),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Slash,
                location: BytePos(28),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Plus,
                location: BytePos(30),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Minus,
                location: BytePos(32),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Comma,
                location: BytePos(34),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Colon,
                location: BytePos(36),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Semi,
                location: BytePos(38),
            },
            scanner.next()
        );
        assert_eq!(
            Token {
                category: Category::Eof,
                location: BytePos::invalid(),
            },
            scanner.next()
        );
    }
}
