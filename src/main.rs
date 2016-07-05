mod util;
mod atom;
mod scope;
mod eval;
mod tokenizer;
mod parser;

use tokenizer::tokenize;
use parser::Parser;
use eval::Eval;
use std::io::{self, Read};

fn main() {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program)
        .expect("failed to read from stdin");
    let tokens = tokenize(&program)
        .expect("invalid tokens given");
    let result = match Parser::new(tokens).parse() {
        Ok(ast) => Eval::new().eval_atoms(ast),
        other => other
    };
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(message) => println!("{}", message)
    }
}