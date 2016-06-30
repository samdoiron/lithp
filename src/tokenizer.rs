#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Identifier(String),
    Quote,
    Integer(i64)
}

pub type TokenResult<T> = Result<T, &'static str>;

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
