use std::result;
pub use syntax::errors::Error;

pub mod errors;
pub mod scanner;
pub mod parser;
pub mod source_map;

pub type Result<T> = result::Result<T, Error>;
