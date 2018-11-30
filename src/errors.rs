use scanner::{Category, Word};
use source_map::{BytePos, Span};
use std::fmt;

/// A `Diag` value gathers enough information about some error in the
/// parsing process. It is used by the diagnostics system to report good
/// quality error messages.
#[derive(Debug, PartialEq)]
pub enum Diag {
    TooManyErrors,
    InvalidDigit {
        invalid_span: Span,
    },
    /// Numeric literals with no digits after an exponent.
    MissingExponentDigits {
        exp_pos: BytePos,
    },
    /// String literals missing a terminating quotation mark.
    MissingTerminatingStringMark {
        str_start_pos: BytePos,
        eol_pos: BytePos,
    },
    /// Unknown character in the source code.
    UnknownCharacter {
        pos: BytePos,
    },
    UnexpectedEndOfFile,
    ExpectedWord { expected: Category, got: Word },
}

pub struct Handler {
    emitter: Box<Fn(Diag) -> bool>,
}

impl Handler {
    fn with_emitter<E>(emitter: E) -> Handler
    where
        E: Fn(Diag) -> bool + 'static,
    {
        Handler {
            emitter: Box::new(emitter),
        }
    }

    pub fn with_ignoring_emitter() -> Handler {
        Handler {
            emitter: Box::new(|_| true),
        }
    }

    pub fn report(&self, diag: Diag) -> bool {
        (self.emitter)(diag)
    }
}

impl fmt::Display for Diag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Diag::TooManyErrors => write!(f, "too many errors"),
            Diag::InvalidDigit { .. } => write!(f, "invalid digit"),
            Diag::MissingExponentDigits { .. } => {
                write!(f, "missing exponent digits for decimal literal")
            }
            Diag::MissingTerminatingStringMark { .. } => write!(
                f,
                "missing terminating quotation mark for string literal"
            ),
            Diag::UnknownCharacter { .. } => write!(f, "unknown character"),
            Diag::UnexpectedEndOfFile => {
                write!(f, "unexpected end of file")
            }
            Diag::ExpectedWord { expected, got } => {
                write!(f, "expected '{}', got '{}'", expected, got.category)
            }
        }
    }
}
