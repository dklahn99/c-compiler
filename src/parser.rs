use crate::tokenizer::{Token, tokenize};

#[derive(PartialEq, Debug)]
enum BinOp {
    Plus,
    Minus,
    Assign,
    Equals,
}

#[derive(PartialEq, Debug)]
enum Expr {
    IntLiteral(u64),
    StringLiteral(String),
    // TODO: CharLiteral,
    Variable(String),
    BinaryOperation {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(PartialEq, Debug)]
enum Statement {
    Return(Expr),
    Expression(Expr),
    VarDeclare {
        name: String,
        var_type: Type,
        value: Option<Expr>,
    },
    If {
        condition: Expr,
        true_block: Vec<Statement>,
        false_block: Option<Vec<Statement>>,
    },
}

#[derive(PartialEq, Debug)]
enum Type {
    Void,
    Int,
    Char,
    UserDefined(String),
    // TODO: float, ptr, etc.
}

#[derive(PartialEq, Debug)]
enum Declaration {
    Function {
        name: String,
        args: Vec<(Type, String)>,
        return_type: Type,
        statements: Vec<Statement>,
    },
}

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn len(&self) -> usize {
        self.tokens.len() - self.pos
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token<'a>> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token)
    }

    fn expect(&mut self, expected: &Token) -> Result<&Token<'a>, String> {
        match self.advance() {
            Some(t) if t == expected => Ok(t),
            Some(t) => Err(format!("Expected {:?}, but got {:?}", expected, t)),
            None => Err(format!("Expected {:?}, but got nothing.", expected)),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        println!("Parse Expression: Parsing: {:?}", self.peek());
        match self.advance() {
            Some(Token::IntegerLiteral(i)) => Ok(Expr::IntLiteral(*i)),
            Some(Token::StringLiteral(s)) => Ok(Expr::StringLiteral(s.to_string())),
            Some(Token::Identifier(name)) => Ok(Expr::Variable(name.to_string())),
            _ => Err(format!("Error parsing expression: {:?}", self.tokens)),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        println!(
            "Parsing variable declaration: {:?}",
            self.tokens[self.pos..].to_vec()
        );

        let var_type = match self.advance() {
            Some(Token::Keyword("void")) => Type::Void,
            Some(Token::Keyword("int")) => Type::Int,
            Some(Token::Keyword("char")) => Type::Char,
            Some(Token::Identifier(type_name)) => Type::UserDefined(type_name.to_string()),
            _ => {
                return Err(format!(
                    "Error parsing type from token {:?} at position {:?}",
                    self.tokens[self.pos - 1],
                    self.pos - 1
                ));
            }
        };
        let name: String = match self.advance() {
            Some(Token::Identifier(var_name)) => var_name.to_string(),
            _ => {
                return Err(format!(
                    "Error parsing variable name from token {:?} at position {:?}",
                    self.tokens[self.pos - 1],
                    self.pos - 1
                ));
            }
        };

        let value = match self.peek() {
            Some(Token::Semicolon) => {
                self.advance();
                None
            }
            _ => {
                self.expect(&Token::Operator("="))?;
                let expression = Some(self.parse_expression()?);
                self.expect(&Token::Semicolon)?;
                expression
            }
        };

        Ok(Statement::VarDeclare {
            name,
            var_type,
            value,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        println!("Parse Statement: Parsing: {:?}", self.peek());
        match self.peek() {
            Some(Token::Keyword("return")) => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(&Token::Semicolon)?;
                Ok(Statement::Return(expression))
            }
            // Token::Keyword("if") => null,
            Some(Token::Keyword("int"))
            | Some(Token::Keyword("char"))
            | Some(Token::Identifier(_)) => self.parse_variable_declaration(),
            None => Err("End of input.".to_string()),
            _ => {
                let expression = self.parse_expression()?;
                Ok(Statement::Expression(expression))
            }
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Declaration>, String> {
    // For now assume we're only parsing main functions
    let expected_prefix = tokenize("int main() {")?;
    assert_eq!(tokens[..expected_prefix.len()], expected_prefix);
    assert_eq!(*tokens.last().unwrap(), Token::CloseBrace);

    let function_body_tokens = tokens[expected_prefix.len()..].split_last().unwrap().1;
    println!("function body tokens: {:?}", function_body_tokens);
    let mut parser = Parser::new(function_body_tokens);

    let mut function_body: Vec<Statement> = vec![];
    while parser.len() != 0 {
        let statement = parser.parse_statement()?;
        println!("Parsed statement: {:?}", statement);
        function_body.push(statement);
    }

    Ok(vec![Declaration::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: Type::Int,
        statements: function_body,
    }])
}

mod tests {
    use crate::tokenizer::{self, tokenize};

    use super::*;

    #[test]
    fn test_main() -> Result<(), String> {
        let input: Vec<_> = tokenize("int main() { return 0; }")?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            statements: vec![Statement::Return(Expr::IntLiteral(0))],
        }];
        let result = parse(input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_variable_declaration() -> Result<(), String> {
        let z_value = "value of z".to_string();
        let tokenize_input = format!(
            "int main() {{ int x; int y = x; MyType z = \"{:}\"; }}",
            z_value
        );
        let input: Vec<_> = tokenize(&tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            statements: vec![
                Statement::VarDeclare {
                    name: "x".to_string(),
                    var_type: Type::Int,
                    value: None,
                },
                Statement::VarDeclare {
                    name: "y".to_string(),
                    var_type: Type::Int,
                    value: Some(Expr::Variable("x".to_string())),
                },
                Statement::VarDeclare {
                    name: "z".to_string(),
                    var_type: Type::UserDefined("MyType".to_string()),
                    value: Some(Expr::StringLiteral(z_value)),
                },
            ],
        }];
        let result = parse(input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
