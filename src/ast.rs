use crate::tokenizer::Token;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Assign,
    Equals,
}

impl BinOp {
    pub fn from_token(token: &Token) -> Result<BinOp, String> {
        match token {
            Token::Operator("+") => Ok(BinOp::Add),
            Token::Operator("-") => Ok(BinOp::Sub),
            Token::Operator("*") => Ok(BinOp::Mul),
            Token::Operator("/") => Ok(BinOp::Div),
            Token::Operator("=") => Ok(BinOp::Assign),
            Token::Operator("==") => Ok(BinOp::Equals),
            _ => Err(format!("Cannot construct BinOp from {:?}", token)),
        }
    }

    pub fn precedence(&self) -> u32 {
        match self {
            BinOp::Add => 30,
            BinOp::Sub => 30,
            BinOp::Mul => 40,
            BinOp::Div => 40,
            BinOp::Assign => 10,
            BinOp::Equals => 20,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Expr {
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
pub enum Statement {
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
pub enum Type {
    Void,
    Int,
    Char,
    UserDefined(String),
    // TODO: float, ptr, etc.
}

#[derive(PartialEq, Debug)]
pub enum Declaration {
    Function {
        name: String,
        args: Vec<(Type, String)>,
        return_type: Type,
        statements: Vec<Statement>,
    },
}
