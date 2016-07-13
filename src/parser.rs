use std::fmt;
use std::fmt::{Display, Formatter, Write};
use tokenizer::Token;
use atom::Atom;
use util::prepend;

const ATOM_COUNT_ESTIMATE: usize = 10;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    is_quoted: bool,
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
            &Atom::Lambda(_) => fmt.write_str("<lambda>"),
            &Atom::Identifier(ref name) => name.fmt(fmt),
            &Atom::Integer(num) => num.fmt(fmt),
            &Atom::Quoted(ref atom) => {
                try!(fmt.write_str("' "));
                atom.fmt(fmt)
            }
        }
    }
}

type ParseResult = Result<Atom, &'static str>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            is_quoted: false
        }
    }

    pub fn parse(&mut self) -> Result<Atom, &'static str> {
        match self.parse_atoms() {
            Ok(value) => {
                if self.tokens.len() > 0 {
                    Err("Syntax Error: left over tokens")
                } else {
                    Ok(value)
                }
            },
            other => other
        }
    }

    fn parse_atoms(&mut self) -> ParseResult {
        if self.tokens.len() == 0 {
            // Atoms ->
            return Ok(Atom::List(Vec::with_capacity(ATOM_COUNT_ESTIMATE)))
        }
        
        let first_token = self.tokens[self.tokens.len() - 1].clone();

        match first_token {
            Token::CloseParen => {
                // Atoms ->
                Ok(Atom::List(Vec::with_capacity(ATOM_COUNT_ESTIMATE)))
            },
            _ => {
                // Atoms -> Atom Atoms
                let atom = try!(self.parse_atom());

                let atoms = try!(self.parse_atoms());
                let mut atoms_vec = match atoms {
                    Atom::List(val) => val,
                    _ => unreachable!()
                };
                Ok(Atom::List(prepend(atom, &mut atoms_vec)))
            }
        }
    }

    fn parse_atom(&mut self) -> ParseResult {
        match self.tokens.pop() {
            Some(Token::Quote) | None => {
                let atom = try!(self.parse_atom());
                Ok(Atom::Quoted(Box::new(atom)))
            },
            Some(Token::OpenParen) => self.parse_list(),
            Some(Token::Identifier(name)) => Ok(Atom::Identifier(name)),
            Some(Token::Integer(number)) => Ok(Atom::Integer(number)),
            _ => Err("unexpected token in parse_atom")
        }
    }

    fn parse_list(&mut self) -> ParseResult {
        let body = try!(self.parse_list_body());
        if self.tokens.pop() != Some(Token::CloseParen) {
            return Err("list did not end with )")
        }
        Ok(body)
    }

    fn parse_list_body(&mut self) -> ParseResult {
        match self.head_token() {
            Some(_) => self.parse_atoms(),
            None => Err("empty list body")
        }
    }

    fn head_token(&self) -> Option<Token> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens[self.tokens.len() - 1].clone())
        }
    }
}