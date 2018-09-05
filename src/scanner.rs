use std::str::CharIndices;
use std::iter::Peekable;
use codemap::BytePos;

enum Category {
    Identifier,
    Number,
    StringLiteral,
    LParen,
    RParen,
    LBrace,
    RBrace,
    ExclamaEqual,
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
}

struct Token {
    category: Category,
    location: BytePos,
}

struct Scanner<'a> {
    char_stream: Peekable<CharIndices<'a>>,
}

impl<'a> Scanner<'a> {
    fn with_input(input: &'a str) -> Scanner<'a> {
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

    fn peek_position(&mut self) -> BytePos {
        BytePos(
            self.char_stream
                .peek()
                .map_or(<usize>::max_value(), |&(i, _)| i),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_char_stream() {
        let mut scanner = Scanner::with_input("abc");
        assert_eq!(BytePos(0), scanner.peek_position());
        assert_eq!('a', scanner.peek());
        assert_eq!('a', scanner.consume());
        assert_eq!(BytePos(1), scanner.peek_position());
        assert_eq!('b', scanner.peek());
        assert_eq!('b', scanner.consume());
        assert_eq!(BytePos(2), scanner.peek_position());
        assert_eq!('c', scanner.peek());
        assert_eq!('c', scanner.consume());
        assert_eq!(BytePos(<usize>::max_value()), scanner.peek_position());
        assert_eq!('\0', scanner.peek());
        assert_eq!('\0', scanner.consume());
    }
}
