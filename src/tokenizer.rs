/*
* TODOs:
*   - floating point literals
*   - Comments
*/

const KEYWORDS: [&'static str; 4] = ["int", "return", "if", "else"];
const OPERATORS: [&'static str; 4] = ["+", "-", "=", "=="];

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Operator(&'a str),      // e.g. =, ==, +
    Keyword(&'a str),       // e.g. int, if, return
    Identifier(&'a str),    // e.g. myvar or main
    IntegerLiteral(u64),    // e.g. 0, 1, 500
    StringLiteral(&'a str), // e.g. "text"
}

fn tokenize_operator(s: &str) -> Result<(Token, usize), ()> {
    assert!(s.len() != 0);

    let mut ptr = 0;
    while ptr < s.len() {
        // Increment the pointer until the next increment causes the buffer to no
        // longer match any operators.
        let buf = &s[..ptr + 1];
        if !OPERATORS
            .iter()
            .map(|op| buf.len() <= op.len() && buf == &op[..ptr + 1])
            .any(|x| x)
        {
            break;
        }
        ptr += 1;
    }

    let matched = &s[..ptr];
    if OPERATORS.contains(&matched) {
        return Ok((Token::Operator(matched), matched.len()));
    }

    Err(())
}

fn tokenize_string_literal(s: &str) -> Result<(Token, usize), ()> {
    assert!(s.len() != 0);

    let quote = '"';
    if s.chars().nth(0).unwrap() != quote {
        return Err(());
    }

    let next_quote_index = s[1..]
        .find(quote)
        .expect("Tokenization Error: String Literal: missing matching quote.");

    Ok((
        Token::StringLiteral(&s[1..next_quote_index + 1]),
        next_quote_index + 2, // Add two extra consumed characters for the quotes
    ))
}

fn tokenize_keywords_integers_ids(s: &str) -> Result<(Token, usize), ()> {
    assert!(s.len() != 0);

    let mut substr = s;
    for (i, c) in s.chars().enumerate() {
        if !(c.is_alphanumeric() || c == '_') {
            substr = &s[..i];
            break;
        }
    }

    if substr.len() == 0 {
        return Err(());
    }

    if KEYWORDS.contains(&substr) {
        return Ok((Token::Keyword(substr), substr.len()));
    }

    let as_int = substr.parse::<u64>();
    if as_int.is_ok() {
        return Ok((Token::IntegerLiteral(as_int.unwrap()), substr.len()));
    }

    Ok((Token::Identifier(substr), substr.len()))
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, String> {
    let mut ptr = 0;
    let mut tokens: Vec<Token> = Vec::new();
    while ptr < s.len() {
        // TODO: nth() is O(n). If we assume the input file is ASCII
        // we can use byte indexing which is faster
        let c = s.chars().nth(ptr).ok_or("Out of Bounds Error")?;
        if c.is_whitespace() {
            ptr += 1;
            continue;
        }

        let (next_token, num_chars) = match c {
            '(' => (Token::OpenParen, 1),
            ')' => (Token::CloseParen, 1),
            '{' => (Token::OpenBrace, 1),
            '}' => (Token::CloseBrace, 1),
            ';' => (Token::Semicolon, 1),
            _ => tokenize_operator(&s[ptr..])
                .or_else(|()| tokenize_string_literal(&s[ptr..]))
                .or_else(|()| tokenize_keywords_integers_ids(&s[ptr..]))
                .or(Err(format!(
                    "Tokenization error at position {} character {}",
                    ptr, c
                )))?,
        };

        tokens.push(next_token);
        ptr += num_chars;
    }

    Ok(tokens)
}

mod tests {
    use super::*;

    #[test]
    fn test_symbols() -> Result<(), String> {
        let input = "(){};";
        let expected: Vec<Token> = vec![
            Token::OpenParen,
            Token::CloseParen,
            Token::OpenBrace,
            Token::CloseBrace,
            Token::Semicolon,
        ];
        let result = tokenize(input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_operators() -> Result<(), String> {
        let input = "+-===";
        let expected: Vec<Token> = vec![
            Token::Operator("+"),
            Token::Operator("-"),
            Token::Operator("=="),
            Token::Operator("="),
        ];
        let result = tokenize(input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_keywords_and_identifiers() -> Result<(), String> {
        let identifier = "my_identifier123";
        let input = KEYWORDS.join(" ") + " " + identifier;

        let mut expected: Vec<Token> = KEYWORDS
            .iter()
            .map(|k| Token::Keyword(k))
            .collect::<Vec<_>>();
        expected.append(&mut vec![Token::Identifier(identifier)]);

        let result = tokenize(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_literals() -> Result<(), String> {
        let input = "100 \"My_String\"";
        let expected: Vec<Token> = vec![
            Token::IntegerLiteral(100),
            Token::StringLiteral("My_String"),
        ];
        let result = tokenize(input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
