use std::fmt;
use syntax::{self, scanner::Scanner};

#[derive(Debug)]
pub enum ParseError {
    ExpectedClosingDelim,
}

impl From<ParseError> for syntax::Error {
    fn from(err: ParseError) -> syntax::Error {
        syntax::Error::Parse(err)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::ExpectedClosingDelim => {
                write!(f, "expected closing delimiter")
            }
        }
    }
}

struct Parser {
    scanner: Scanner,
}

impl Parser {
    fn new(scanner: Scanner) -> Parser {
        Parser {
            scanner,
        }
    }
}
