use std::fmt;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Identifier(String),
    Quote,
    Integer(i64)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Atom {
    List(Vec<Atom>),
    Integer(i64),
    Identifier(String),
    Quoted(Box<Atom>)
}

pub struct Parsed<T> {
    remaining: Vec<Token>,
    value: T,
    parsed_tree: Atom
}

impl Display for Atom {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &Atom::List(ref atoms) => {
                try!(fmt.write_str("( "));
                for atom in atoms {
                    try!(atom.fmt(fmt));
                    try!(fmt.write_char(' '));
                }
                try!(fmt.write_str(")"));
                Ok(())
            },
            &Atom::Identifier(ref name) => name.fmt(fmt),
            &Atom::Integer(num) => num.fmt(fmt),
            &Atom::Quoted(ref atom) => {
                try!(fmt.write_str("' "));
                atom.fmt(fmt)
            }
        }
    }
}

pub type TokenResult<T> = Result<T, &'static str>;
type ParseResult<T> = Result<Parsed<T>, &'static str>;

// Input must be formatted so that each token is separated by a space.
pub fn tokenize(program: &str) -> TokenResult<Vec<Token>> {
    let mut tokens = vec![];
    for token in program.split_whitespace() {
        let token = try!(match_token(token));
        tokens.push(token);
    }
    tokens.reverse();
    Ok(tokens)
}

pub fn parse(tokens: Vec<Token>) {
    let result = parse_atoms(tokens, false);
    match result {
        Ok(parsed) => {
            if parsed.remaining.len() > 0 {
                // Left over tokens not part of any production.
                println!("Error: unparsed tokens {:?}", parsed.remaining);
            } else {
                println!("Value: {:?}", parsed.value);
            }
        },
        Err(msg) => println!("Error: {}", msg)
    }
}

fn match_token(token: &str) -> TokenResult<Token> {
    match token {
        "(" => Ok(Token::OpenParen),
        ")" => Ok(Token::CloseParen),
        "'" => Ok(Token::Quote),
        "" => Err("empty tokens are invalid"),
        _   => match_long_token(token)
    }
}

fn match_long_token(token: &str) -> TokenResult<Token> {
    assert!(!token.is_empty());
    if token.chars().all(|c| c.is_digit(10)) {
        Ok(Token::Integer(token.parse().unwrap()))
    } else {
        Ok(Token::Identifier(token.to_string()))
    }
}

fn parse_atoms(tokens: Vec<Token>, is_quoted: bool) -> ParseResult<Vec<Atom>> {
    if tokens.len() == 0 {
        // Atoms ->
        return Ok(Parsed{
            remaining: tokens,
            value: vec![],
            parsed_tree: Atom::List(vec![])
        })
    }
    let token = tokens[tokens.len() - 1].clone();

    match token {
        Token::CloseParen => {
            // Atoms ->
            Ok(Parsed{
                remaining: tokens,
                value: vec![],
                parsed_tree: Atom::List(vec![])
            })
        },
        _ => {
            // Atoms -> Atom Atoms
            let atom = try!(parse_atom(tokens, is_quoted));
            let atom_value = if is_quoted { atom.value } else { try!(eval(atom.value)) };
            println!("eval( {} ) -> {}", atom.parsed_tree, atom_value);

            let mut atoms = try!(parse_atoms(atom.remaining, is_quoted));
            let mut atoms_tree = match atoms.parsed_tree {
                Atom::List(val) => val,
                _ => unreachable!()
            };

            Ok(Parsed{
                remaining: atoms.remaining,
                parsed_tree: Atom::List(prepend(atom.parsed_tree, &mut atoms_tree)),
                value: prepend(atom_value, &mut atoms.value)
            })
        }
    }
}

fn parse_atom(mut tokens: Vec<Token>, is_quoted: bool) -> ParseResult<Atom> {
    if tokens.len() == 0 {
        return Err("no tokens given to parse_atom");
    }
    let token = tokens[tokens.len() - 1].clone();

    match token {
        Token::Quote => {
            // Atom -> ' Atom
            tokens.pop();
            let atom = try!(parse_atom(tokens, true));
            Ok(Parsed{remaining: atom.remaining,
                      value: Atom::Quoted(Box::new(atom.value)),
                      parsed_tree: Atom::Quoted(Box::new(atom.parsed_tree))})
        },
        Token::OpenParen => {
            // Atom -> List
            let list = try!(parse_list(tokens, is_quoted));
            Ok(Parsed{remaining: list.remaining,
                      value: Atom::List(list.value),
                      parsed_tree: list.parsed_tree})
        },
        Token::Identifier(name) => {
            // Atom -> id
            tokens.pop();
            Ok(Parsed{remaining: tokens,
                      value: Atom::Identifier(name.clone()),
                      parsed_tree: Atom::Identifier(name)})
        },
        Token::Integer(number) => {
            // Atom -> int
            tokens.pop();
            Ok(Parsed{remaining: tokens,
                      value: Atom::Integer(number),
                      parsed_tree: Atom::Integer(number)})
        },
        _ => Err("unexpected token in parse_atom")
    }
}

fn parse_list(mut tokens: Vec<Token>, is_quoted: bool) -> ParseResult<Vec<Atom>> {
    // List -> ( ListBody )
    if tokens.pop() != Some(Token::OpenParen) {
        return Err("list did not start with (");
    }
    let mut body = try!(parse_list_body(tokens, is_quoted));
    if body.remaining.pop() != Some(Token::CloseParen) {
        return Err("list did not end with )")
    }
    Ok(Parsed{remaining: body.remaining,
              value: body.value,
              parsed_tree: body.parsed_tree})
}

fn parse_list_body(tokens: Vec<Token>, is_quoted: bool) -> ParseResult<Vec<Atom>> {
    // ListBody -> Atoms
    parse_atoms(tokens, is_quoted)
}

fn eval(atom: Atom) -> Result<Atom, &'static str> {
    match atom {
        Atom::Quoted(value) => Ok(*value),
        Atom::Integer(_) | Atom::Identifier(_) => Ok(atom),
        Atom::List(atoms) => {
            match atoms.split_first() {
                Some((car, cdr)) => apply(car, cdr),
                None => Err("eval() on empty list")
            }
        }
    }
}

fn apply(car: &Atom, cdr: &[Atom]) -> Result<Atom, &'static str> {
    match car {
        &Atom::Identifier(ref name) => {
            let name_ref: &str = name;
            match name_ref {
                "+" => apply_math(0, &|a, &b| a + b, cdr),
                "*" => apply_math(1, &|a, &b| a * b, cdr),
                "/" => apply_math_first(&|a, &b| a / b, cdr),
                "-" => apply_math_first(&|a, &b| a - b, cdr),
                "car" => apply_car(cdr),
                "cdr" => apply_cdr(cdr),
                "cons" => apply_cons(cdr),
                _ => Err("unknown function")
            }
        },
        _ => Err("cannot apply non-identifier")
    }
}

fn apply_math(start: i64, reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    if cdr.len() == 0 { return Err("attempted math on empty list") }
    match extract_ints(cdr) {
        Some(ints) => Ok(Atom::Integer(ints.iter().fold(start, reduce))),
        None => Err("attempted math on non-integer")
    }
}

fn apply_math_first(reduce: &Fn(i64, &i64) -> i64, cdr: &[Atom])
    -> Result<Atom, &'static str> {
    match extract_ints(cdr) {
        Some(ints) => {
            if ints.len() == 0 {
                Err("attempted math on empty list")
            } else {
                Ok(Atom::Integer(ints[1..].iter().fold(ints[0], reduce)))
            }
        }
        None => Err("attempted math on non-integer")
    }
}

fn extract_ints(cdr: &[Atom]) -> Option<Vec<i64>> {
    let mut result = Vec::new();
    for atom in cdr {
        match atom {
            &Atom::Integer(val) => result.push(val),
            _ => return None
        }
    }
    Some(result)
}

fn apply_car(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 1 { return Err("wrong number of args to car") }
    match &cdr[0] {
        &Atom::List(ref atoms) => Ok(atoms[0].clone()),
        _ => Err("invalid argument to car")
    }
}

fn apply_cdr(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 0 {
        Ok(Atom::List(cdr[1..].to_vec()))
    } else {
        Err("cdr on empty list")
    }
}

fn apply_cons(cdr: &[Atom]) -> Result<Atom, &'static str> {
    if cdr.len() != 2 { return Err("wrong number of args for cons ") }
    match &cdr[1] {
        &Atom::List(ref vals) => Ok(Atom::List(prepend(cdr[0].clone(), &mut vals.clone()))),
        _ => Err("invalid type to cons() onto")
    }
}

fn prepend<T>(item: T, items: &mut Vec<T>) -> Vec<T> {
    let mut new = vec![item];
    new.append(items);
    new
}