# Blaise

[![Build Status](https://travis-ci.com/feroldi/blaise.svg?token=FjPQjKrsmeJzM46SGssn&branch=master)](https://travis-ci.com/feroldi/blaise)

Blaise is my university's compilers class project.
The name is inspired on the mathematician Blaise Pascal's first name, given the language chosen to be compiled by Blaise is a modification of Pascal's syntax grammar.

## Building

Blaise is written in Rust, so you may use its package manager, `cargo`, to build and run it:

    cargo build
    cargo test
    cargo run <some source file>

## Syntax grammar

```ebnf
<program> ::= "program" <ident> ";" { <decl> } <stmt-list>

<decl> ::= "let" <ident> ":" <type> ";"

<type> ::= "int" | "bool" | "float" | "str"

<stmt-list> ::= <stmt> { <stmt> }

<stmt> ::= <assign-stmt>
         | <func-call-stmt>
         | <block-stmt>
         | <sel-stmt>
         | <iter-stmt>

<param-list> ::= [<expr> { "," <expr> }]

<assign-stmt> ::= <ident> "=" <expr> ";"

<func-call-stmt> ::= <ident> "(" <param-list> ")" ";"

<block-stmt> ::= "{" <stmt-list> "}"

<sel-stmt> ::= "if" <expr> <block-stmt> ["else" <block-stmt>]

<iter-stmt> ::= "while" <expr> <block-stmt>

<add-expr> ::= <add-expr> "+" <mult-expr>
             | <add-expr> "–" <mult-expr>
             | "–" <add-expr>
             | <mult-expr>

<mult-expr> ::= <mult-expr> "*" <eguality-expr>
              | <mult-expr> "/" <eq-expr>
              | <eq-expr>

<eq-expr> ::= <eq-expr> "==" <rel-expr>
            | <eq-expr> "!=" <rel-expr>
            | <relacional-expr>

<rel-expr> ::= <rel-expr> "<" <expr>
             | <rel-expr> "<=" <expr>
             | <rel-expr> ">" <expr>
             | <rel-expr> ">=" <expr>
             | <expr>

<expr> ::= <num-const>
         | <ident>
         | <str-lit>
         | "(" <expr> ")"


<num-const> ::= <int-const>
              | <float-const>
```

## Regular exprs

```
<str-lit> ::= "[^"]*"

<ident> ::= [a-zA-Z_][a-zA-Z0-9_]*

<int-const> ::= 0|([1-9][0-9]*)

<float-const> ::= [0-9]+\.[0-9]+([Ee][+-]?[0-9]+)?
```
