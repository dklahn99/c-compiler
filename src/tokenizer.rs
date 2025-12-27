use std::string;
use std::vec;

#[derive(Debug, PartialEq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Operator(String),      // e.g. =, ==, +
    Keyword(String),       // e.g. int, if, return
    Identifier(String),    // e.g. myvar or main
    IntegerLiteral(u64),   // e.g. 0, 1, 500
    StringLiteral(String), // e.g. "text"
}

pub fn tokenize(s: &str) -> Vec<Token> {
    return vec![Token::Semicolon];
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
        let result = tokenize(input);
        assert_eq!(result, expected);
        return Ok(());
    }
}
