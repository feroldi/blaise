#![feature(universal_impl_trait, dotdoteq_in_patterns, use_nested_groups)]

pub mod ast;
pub mod errors;
pub mod scanner;
pub mod parser;
pub mod source_map;

fn main() {
    println!("Hello, World!");
}
