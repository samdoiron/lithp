mod util;
mod atom;
mod eval;
mod tokenizer;
mod parser;

use tokenizer::tokenize;
use parser::Parser;
use std::io::{self, Read};

fn main() {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program)
        .expect("failed to read from stdin");
    let tokens = tokenize(&program)
        .expect("invalid tokens given");
    Parser::new(tokens).parse();
}