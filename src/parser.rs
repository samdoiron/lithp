use std::fmt;
use std::fmt::{Display, Formatter, Write};
use tokenizer::Token;
use atom::Atom;
use util::prepend;
use eval::eval;

struct Parsed<T> {
    value: T,
    parsed_tree: Atom
}

pub struct Parser {
    tokens: Vec<Token>,
    is_quoted: bool
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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            is_quoted: false
        }
    }

    pub fn parse(&mut self) {
        match self.parse_atoms() {
            Ok(parsed) => {
                if self.tokens.len() > 0 {
                    // Left over tokens not part of any production.
                    println!("Syntax Error");
                }
            },
            Err(_) => println!("Syntax Error")
        }
    }

    fn parse_atoms(&mut self) -> ParseResult<Vec<Atom>> {
        if self.tokens.len() == 0 {
            // Atoms ->
            return Ok(Parsed{
                value: vec![],
                parsed_tree: Atom::List(vec![])
            })
        }
        
        let first_token = self.tokens[self.tokens.len() - 1].clone();

        match first_token {
            Token::CloseParen => {
                // Atoms ->
                Ok(Parsed{value: vec![],
                          parsed_tree: Atom::List(vec![])})
            },
            _ => {
                // Atoms -> Atom Atoms
                let atom = try!(self.parse_atom());
                let mut atom_value = atom.value;
                if !self.is_quoted {
                    atom_value = try!(eval(atom_value));
                    println!("eval( {} ) -> {}", atom.parsed_tree, atom_value);
                } 

                let mut atoms = try!(self.parse_atoms());
                let mut atoms_tree = match atoms.parsed_tree {
                    Atom::List(val) => val,
                    _ => unreachable!()
                };

                Ok(Parsed{
                    parsed_tree: Atom::List(prepend(atom.parsed_tree, &mut atoms_tree)),
                    value: prepend(atom_value, &mut atoms.value)
                })
            }
        }
    }

    fn parse_atom(&mut self) -> ParseResult<Atom> {
        if self.tokens.len() == 0 {
            return Err("no tokens given to parse_atom");
        }
        let first_token = self.tokens[self.tokens.len() - 1].clone();

        match first_token {
            Token::Quote => {
                // Atom -> ' Atom
                self.tokens.pop();

                // Occurs due to implementation detail of example solution,
                // duplicate just in case.
                if !self.is_quoted {
                    println!("eval( ' ) -> '")
                }

                let atom = try!(self.parse_atom());
                Ok(Parsed{value: Atom::Quoted(Box::new(atom.value)),
                          parsed_tree: Atom::Quoted(Box::new(atom.parsed_tree))})
            },
            Token::OpenParen => {
                // Atom -> List
                let list = try!(self.parse_list());
                Ok(Parsed{value: Atom::List(list.value),
                          parsed_tree: list.parsed_tree})
            },
            Token::Identifier(name) => {
                // Atom -> id
                self.tokens.pop();
                Ok(Parsed{value: Atom::Identifier(name.clone()),
                          parsed_tree: Atom::Identifier(name)})
            },
            Token::Integer(number) => {
                // Atom -> int
                self.tokens.pop();
                Ok(Parsed{value: Atom::Integer(number),
                          parsed_tree: Atom::Integer(number)})
            },
            _ => Err("unexpected token in parse_atom")
        }
    }

    fn parse_list(&mut self) -> ParseResult<Vec<Atom>> {
        // List -> ( ListBody )
        if self.tokens.pop() != Some(Token::OpenParen) {
            return Err("list did not start with (");
        }
        let mut body = try!(self.parse_list_body());
        if self.tokens.pop() != Some(Token::CloseParen) {
            return Err("list did not end with )")
        }
        Ok(Parsed{value: body.value,
                  parsed_tree: body.parsed_tree})
    }


    fn parse_list_body(&mut self) -> ParseResult<Vec<Atom>> {
        // ListBody -> Atoms
        self.parse_atoms()
    }
}
