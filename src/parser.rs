use crate::ast::*;
use crate::tokenizer::{Token, tokenize};

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
    scope_id_counter: ScopeIdCounter,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            pos: 0,
            scope_id_counter: ScopeIdCounter { counter: 0 },
        }
    }

    fn peek(&self) -> Option<&Token<'a>> {
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

    fn parse_brace_block(&mut self) -> Result<Vec<Statement>, String> {
        self.expect(&Token::OpenBrace)?;

        let mut brace_block: Vec<Statement> = vec![];
        while self.peek() != Some(&Token::CloseBrace) {
            brace_block.push(self.parse_statement()?);
        }
        self.expect(&Token::CloseBrace)?;

        Ok(brace_block)
    }

    fn parse_parenthesis(&mut self) -> Result<Expr, String> {
        self.expect(&Token::OpenParen)?;
        let inner = self.parse_expression()?;
        self.expect(&Token::CloseParen)?;
        Ok(inner)
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::IntegerLiteral(i)) => {
                let int_literal = *i;
                self.advance();
                Ok(Expr::IntLiteral(int_literal))
            }
            Some(Token::StringLiteral(s)) => {
                let str_literal = s.to_string();
                self.advance();
                Ok(Expr::StringLiteral(str_literal))
            }
            Some(Token::Identifier(name)) => {
                let var_name = name.to_string();
                self.advance();
                Ok(Expr::Variable(var_name))
            }
            Some(Token::OpenParen) => self.parse_parenthesis(),
            _ => Err(format!(
                "Error parsing token {:?} at position {:?}",
                self.tokens.get(self.pos),
                self.pos
            )),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_primary_expression()?;
        self.parse_expression_precedence(lhs, 0)
    }

    fn parse_expression_precedence(
        &mut self,
        mut lhs: Expr,
        min_precedence: u32,
    ) -> Result<Expr, String> {
        while let Some(token) = self.peek() {
            // Try to get the operator and its precedence
            let op = match BinOp::from_token(token) {
                Ok(op) if op.precedence() >= min_precedence => op,
                _ => break, // Not an operator or precedence too low
            };

            self.advance(); // Consume the operator

            let mut rhs = self.parse_primary_expression()?;

            // Look ahead to see if we should bind rhs to the next operator first
            while let Some(next_token) = self.peek() {
                let next_op = match BinOp::from_token(next_token) {
                    Ok(next_op) if next_op.precedence() > op.precedence() => next_op,
                    _ => break,
                };

                // Next operator has higher precedence, recurse
                rhs = self.parse_expression_precedence(rhs, next_op.precedence())?;
            }

            // Build the binary expression
            lhs = Expr::BinaryOperation {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
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

    fn parse_if_else(&mut self) -> Result<Statement, String> {
        self.expect(&Token::Keyword("if"))?;
        self.expect(&Token::OpenParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::CloseParen)?;

        let true_statements = self.parse_brace_block()?;

        let false_statements = match self.peek() {
            Some(&Token::Keyword("else")) => {
                self.expect(&Token::Keyword("else"))?;
                Some(Scope::from_statements(
                    self.parse_brace_block()?,
                    &mut self.scope_id_counter,
                ))
            }
            _ => None,
        };

        Ok(Statement::If {
            condition,
            true_block: Scope::from_statements(true_statements, &mut self.scope_id_counter),
            false_block: false_statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek();
        let next_token = self.tokens.get(self.pos + 1);
        match (token, next_token) {
            (Some(Token::Keyword("return")), _) => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(&Token::Semicolon)?;
                Ok(Statement::Return(expression))
            }
            (Some(Token::Keyword("if")), _) => self.parse_if_else(),
            (Some(Token::Keyword("int")), _)
            | (Some(Token::Keyword("char")), _)
            | (Some(Token::Identifier(_)), Some(Token::Identifier(_))) => {
                self.parse_variable_declaration()
            }
            (None, _) => Err("End of input.".to_string()),
            _ => {
                let expression = self.parse_expression()?;
                self.expect(&Token::Semicolon)?;
                Ok(Statement::Expression(expression))
            }
        }
    }
}

pub fn parse(tokens: &Vec<Token>) -> Result<Vec<Declaration>, String> {
    // For now assume we're only parsing main functions
    let expected_prefix = tokenize("int main()")?;
    assert_eq!(tokens[..expected_prefix.len()], expected_prefix);
    assert_eq!(*tokens.last().unwrap(), Token::CloseBrace);

    let function_body_tokens = tokens[expected_prefix.len()..].to_vec();
    let mut parser = Parser::new(&function_body_tokens);

    let function_body = parser.parse_brace_block()?;

    Ok(vec![Declaration::Function {
        name: "main".to_string(),
        args: vec![],
        return_type: Type::Int,
        scope: Scope::from_statements(function_body, &mut parser.scope_id_counter),
    }])
}

mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_main() -> Result<(), String> {
        let input: Vec<_> = tokenize("int main() { return 0; }")?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 1,
                statements: vec![Statement::Return(Expr::IntLiteral(0))],
            },
        }];
        let result = parse(&input)?;
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
            scope: Scope {
                id: 1,
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
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_if() -> Result<(), String> {
        let tokenize_input = "int main() { if(x) { return 0; } return 1;}";
        let input: Vec<_> = tokenize(tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 2,
                statements: vec![
                    Statement::If {
                        condition: Expr::Variable("x".to_string()),
                        true_block: Scope {
                            id: 1,
                            statements: vec![Statement::Return(Expr::IntLiteral(0))],
                        },
                        false_block: None,
                    },
                    Statement::Return(Expr::IntLiteral(1)),
                ],
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_if_else() -> Result<(), String> {
        let tokenize_input = "int main() { if(x){ return 1; }else{ return 0; }}";
        let input: Vec<_> = tokenize(tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 3,
                statements: vec![Statement::If {
                    condition: Expr::Variable("x".to_string()),
                    true_block: Scope {
                        id: 2,
                        statements: vec![Statement::Return(Expr::IntLiteral(1))],
                    },
                    false_block: Some(Scope {
                        id: 1,
                        statements: vec![Statement::Return(Expr::IntLiteral(0))],
                    }),
                }],
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_assign() -> Result<(), String> {
        let tokenize_input = "int main() { x = 1; }";
        let input: Vec<_> = tokenize(tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 1,
                statements: vec![Statement::Expression(Expr::BinaryOperation {
                    op: BinOp::Assign,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                })],
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_precedence() -> Result<(), String> {
        let tokenize_input = "int main() { x = 1 + 2 * 3; x = 1 * 2 + 3; }";
        let input: Vec<_> = tokenize(tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 1,
                statements: vec![
                    Statement::Expression(Expr::BinaryOperation {
                        op: BinOp::Assign,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::BinaryOperation {
                            op: BinOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::BinaryOperation {
                                op: BinOp::Mul,
                                left: Box::new(Expr::IntLiteral(2)),
                                right: Box::new(Expr::IntLiteral(3)),
                            }),
                        }),
                    }),
                    Statement::Expression(Expr::BinaryOperation {
                        op: BinOp::Assign,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::BinaryOperation {
                            op: BinOp::Add,
                            left: Box::new(Expr::BinaryOperation {
                                op: BinOp::Mul,
                                left: Box::new(Expr::IntLiteral(1)),
                                right: Box::new(Expr::IntLiteral(2)),
                            }),
                            right: Box::new(Expr::IntLiteral(3)),
                        }),
                    }),
                ],
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_parens() -> Result<(), String> {
        let tokenize_input = "int main() { x = ((1 + 2) * 3); }";
        let input: Vec<_> = tokenize(tokenize_input)?;
        let expected: Vec<Declaration> = vec![Declaration::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: Type::Int,
            scope: Scope {
                id: 1,
                statements: vec![Statement::Expression(Expr::BinaryOperation {
                    op: BinOp::Assign,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::BinaryOperation {
                        op: BinOp::Mul,
                        left: Box::new(Expr::BinaryOperation {
                            op: BinOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    }),
                })],
            },
        }];
        let result = parse(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
