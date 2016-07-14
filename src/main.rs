mod util;
mod atom;
mod scope;
mod eval;
mod tokenizer;
mod parser;

use tokenizer::tokenize;
use parser::Parser;
use eval::eval;
use std::io::{self, Read};

fn main() {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program).unwrap();
    let tokens = tokenize(&program)
        .expect("invalid tokens given");
    let result = match Parser::new(tokens).parse() {
        Ok(ast) => eval(ast),
        Err(msg) => {
            println!("Syntax Error: {}", msg);
            return;
        }
    };
    match result {
        Err(msg) => println!("Evaluation Error: {}", msg),
        Ok(result) => println!("{}", result)
    }
}
