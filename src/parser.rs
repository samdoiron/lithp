#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Identifier(String),
    Quote,
    Integer(String)
}

pub type TokenResult<T> = Result<T, &'static str>;
type ParseResult = Result<Vec<Token>, &'static str>;

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
    println!("S -> Atoms");
    let result = parse_atoms(tokens);
    match result {
        Ok(vec) => {
            if vec.len() > 0 {
                println!("Syntax Error");
            }
        },
        Err(_) => println!("Syntax Error")
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
        Ok(Token::Integer(token.to_string()))
    } else {
        Ok(Token::Identifier(token.to_string()))
    }
}

fn parse_atoms(tokens: Vec<Token>) -> ParseResult {
    if tokens.len() == 0 {
        println!("Atoms ->");
        return Ok(tokens);
    }
    let token = tokens[tokens.len() - 1].clone();

    match token {
        Token::CloseParen => {
            println!("Atoms ->");
            Ok(tokens)
        }
        _ => {
                println!("Atoms -> Atom Atoms");
                let after_atom = try!(parse_atom(tokens));
                let after_atoms = try!(parse_atoms(after_atom));
                Ok(after_atoms)
        }
    }
}

fn parse_atom(mut tokens: Vec<Token>) -> ParseResult {
    if tokens.len() == 0 {
        return Err("no tokens given to parse_atom");
    }
    let token = tokens[tokens.len() - 1].clone();

    match token {
        Token::Quote => {
            println!("Atom -> ' Atom");
            tokens.pop();
            let remaining = try!(parse_atom(tokens));
            Ok(remaining)
        },
        Token::OpenParen => {
            println!("Atom -> List");
            let remaining = try!(parse_list(tokens));
            Ok(remaining)
        },
        Token::Identifier(name) => {
            println!("Atom -> id[{}]", name);
            tokens.pop();
            Ok(tokens)
        },
        Token::Integer(number) => {
            println!("Atom -> int[{}]", number);
            tokens.pop();
            Ok(tokens)
        },
        _ => Err("unexpected token in parse_atom")
    }
}

fn parse_list(mut tokens: Vec<Token>) -> ParseResult {
    println!("List -> ( ListBody )");
    let open = tokens.pop();
    if open.is_none() || open.unwrap() != Token::OpenParen {
        return Err("list did not start with (");
    }

    let mut after_body = try!(parse_list_body(tokens));

    let close = after_body.pop();
    if close.is_none() || close.unwrap() != Token::CloseParen {
        return Err("list did not ent with )");
    }

    Ok(after_body)
}

fn parse_list_body(tokens: Vec<Token>) -> ParseResult {
    println!("ListBody -> Atoms");
    parse_atoms(tokens)
}