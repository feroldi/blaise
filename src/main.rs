#![feature(universal_impl_trait, dotdoteq_in_patterns, use_nested_groups,
           fs_read_write)]

use std::env;
use std::fs;
use std::rc::Rc;

use parser::Parser;
use scanner::{Scanner, WordStream};
use source_map::{Loc, SourceFile};

pub mod ast;
pub mod errors;
pub mod scanner;
pub mod parser;
pub mod source_map;

fn main() {
    let mut args = env::args();
    args.next();
    let path = args.next().unwrap();
    let src = fs::read_string(path).unwrap();
    let file = Rc::new(SourceFile::new("test".into(), src.into()));
    let scanner = Scanner::new(file.clone());
    let handler = errors::Handler::with_emitter(move |diag| {
        let Loc { line, col } =
            file.lookup_source_location(diag.location()).unwrap();
        println!("{}:{}: error: {}", line, col.0, diag);
        true
    });
    let word_stream = WordStream::new(scanner, &handler);
    let mut parser = Parser::new(word_stream);

    match parser.parse_program() {
        Ok(program) => println!("{:#?}", program),
        Err(diag) => {
            handler.report(diag);
        }
    }
}
