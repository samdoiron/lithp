#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Identifier(String),
    Quote,
    Integer(i64)
}

pub type TokenResult<T> = Result<T, &'static str>;

pub fn tokenize(program: &str) -> TokenResult<Vec<Token>> {
    let mut tokens = vec![];
    let mut long_token = String::new();
    for c in program.chars() {
        if !long_token.is_empty() && (c == ')' || c == '(' || c == '\'' || c.is_whitespace()) {
            tokens.push(try!(match_long_token(&long_token)));
            long_token = String::new();
        }
        match c {
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '\'' => tokens.push(Token::Quote),
            x if x.is_whitespace() => (),
            _ => long_token.push(c)
        }
    }
    if !long_token.is_empty() {
        tokens.push(try!(match_long_token(&long_token)));
    }
    tokens.reverse();
    Ok(tokens)
}

fn match_long_token(token: &str) -> TokenResult<Token> {
    assert!(!token.is_empty());
    if token.chars().all(|c| c.is_digit(10)) {
        Ok(Token::Integer(token.parse().unwrap()))
    } else {
        Ok(Token::Identifier(token.to_string()))
    }
}
