#[derive(Debug, PartialEq, Eq)]
enum Token {
    OpenParen,
    CloseParen,
    Identifier(String),
    Quote,
    Integer(u64)
}

type TokenResult<T> = Result<T, &'static str>;

// Input must be formatted so that each token is separated by a space.
fn tokenize(program: &str) -> TokenResult<Vec<Token>> {
    let mut tokens = vec![];
    for token in program.split_whitespace() {
        let token = try!(match_token(token));
        tokens.push(token);
    }
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
        Ok(Token::Integer(token.parse::<u64>().unwrap()))   
    } else {
        Err("non-alphabetic long token")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseAction {
    Production(&'static str, &'static str),
    SyntaxError
}

fn parse(mut tokens: Vec<Token>) -> Vec<ParseAction> {
    parse_atoms(tokens);
}

fn parse_atoms(mut tokens: Vec<Token>) -> (Vec<Token>, Vec<ParseAction>) {
    let first_token = tokens.pop();
    if first_token.is_none() {
        return (tokens, vec![ParseAction::SyntaxError]);
    }

    match first_token.unwrap() {
        Token::CloseParen => vec![ParseAction::Production("Atoms", "")],
        Token::Quote
            | Token::Identifier(_)
            | Token::Integer(_)
            | Token::OpenParen
            => {
                let mut actions = vec![
                    ParseAction::Production("Atoms", "Atom Atoms"),
                ];
                let (mut after_atom, atom_actions) = parse_atom(tokens);
                let (mut after_atoms, atoms_actions) = parse_atoms(with_atom);

                actions.append(atom_actions).append(atoms_actions);
                return (after_atoms, actions);
            },
        _ => vec![ParseAction::SyntaxError]
    }
}

fn parse_atom(tokens: Vec<Token>) -> (Vec<Token>, Vec<ParseAction>) {
    return vec![];
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::{tokenize, parse, Token, ParseAction};

    #[test]
    fn tokenize__empty_string__no_tokens() {
        let tokens = unwrap_tokens("");
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn tokenize__single_token__returns_token() {
        assert_tokenizes_to(vec![Token::OpenParen], "(")
    }

    #[test]
    fn tokenize__both_parens__returns_both() {
        assert_tokenizes_to(vec![Token::OpenParen, Token::CloseParen], "( )")
    }

    #[test]
    fn tokenize__identifier__contains_given_value() {
        assert_tokenizes_to(vec![Token::Identifier("hello".to_string())],
                            "hello")
    }

    #[test]
    fn tokenize__int_literal__contains_given_value() {
        assert_tokenizes_to(vec![Token::Integer(123)], "123")
    }

    #[test]
    fn tokenize__identifier_starts_with_number__is_invalid() {
        let result = tokenize("123abc");
        assert!(result.is_err())
    }

    #[test]
    fn tokenize__containing_quote__returns_quote() {
        assert_tokenizes_to(vec![Token::Quote], "'");
    }

    #[test]
    fn tokenize__large_example__returns_correct_tokens() {
        assert_tokenizes_to(
            vec![
                Token::OpenParen,
                Token::Identifier("define".to_string()),
                Token::Quote,
                Token::Identifier("hello".to_string()),
                Token::Integer(123),
                Token::CloseParen
            ],
            "( define ' hello 123 )"
        );
    }

    #[test]
    fn parse__empty_string__atoms_to_epsilon() {
        assert_parses_to(
            vec![
                ParseAction::Production("S", "Atoms"),
                ParseAction::Production("Atoms", "")
            ],
            ""
        );
    }

    #[test]
    fn parse__assignment_example_2__syntax_error() {
        assert_parses_to(
            vec![
                ParseAction::Production("S", "Atoms"),
                ParseAction::Production("Atoms", "Atom Atoms"),
                ParseAction::Production("Atom", "List"),
                ParseAction::Production("List", "( ListBody )"),
                ParseAction::Production("ListBody", "Atoms"),
                ParseAction::Production("Atoms", "Atom Atoms"),
                ParseAction::Production("Atom", "'"),
                ParseAction::SyntaxError
            ],
            "( ' )"
        )
    }

    fn assert_parses_to(expected: Vec<ParseAction>, program: &'static str) {
        let mut tokens = unwrap_tokens(program);
        let actual = parse(tokens);
        assert_eq!(expected, actual);
    }

    fn assert_tokenizes_to(expected: Vec<Token>, program: &'static str) {
        let actual = unwrap_tokens(program);
        assert_eq!(expected, actual);
    }

    fn unwrap_tokens(program: &'static str) -> Vec<Token> {
        let result = tokenize(program);
        result.expect(&format!("token stream '{}' should be valid", program))
    }
}