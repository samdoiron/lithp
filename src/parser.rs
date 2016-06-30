use std::fmt;
use std::fmt::{Display, Formatter, Write};
use tokenizer::Token;
use atom::Atom;
use util::prepend;
use eval::eval;

struct Parsed<T> {
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

type ParseResult<T> = Result<Parsed<T>, &'static str>;

pub fn parse(tokens: Vec<Token>) {
    let result = parse_atoms(tokens, false);
    match result {
        Ok(parsed) => {
            if parsed.remaining.len() > 0 {
                // Left over tokens not part of any production.
                println!("Syntax Error");
            }
        },
        Err(_) => println!("Syntax Error")
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
            Ok(Parsed{remaining: tokens,
                      value: vec![],
                      parsed_tree: Atom::List(vec![])})
        },
        _ => {
            // Atoms -> Atom Atoms
            let atom = try!(parse_atom(tokens, is_quoted));
            let mut atom_value = atom.value;
            if !is_quoted {
                atom_value = try!(eval(atom_value));
                println!("eval( {} ) -> {}", atom.parsed_tree, atom_value);
            } 

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

            // Occurs due to implementation detail of example solution,
            // duplicate just in case.
            if !is_quoted {
                println!("eval( ' ) -> '")
            }

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

fn parse_let() -> ParseResult<Atom> {
    Err("unimplemented")
}

fn parse_let_star() -> ParseResult<Atom> {
    Err("unimplemented")
}

fn parse_define() -> ParseResult<Atom> {
    Err("unimplemented")
}