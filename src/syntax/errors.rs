use syntax::{scanner, parser};
use std::{fmt, io};

/// An `Error` value gathers enough information about some error in the
/// parsing process. It is used by the diagnostics system to report good
/// quality error messages.
#[derive(Debug)]
pub enum Error {
    TooManyErrors,
    Io(io::Error),
    Scan(scanner::ScanError),
    Parse(parser::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TooManyErrors => write!(f, "too many errors"),
            Error::Io(ref err) => err.fmt(f),
            Error::Scan(ref err) => err.fmt(f),
            Error::Parse(ref err) => err.fmt(f),
        }
    }
}
