use crate::tokenizer::Token;
use std::cell::Cell;

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

pub struct ScopeIdCounter {
    pub counter: u32,
}

#[derive(Debug, PartialEq)]
pub struct Scope {
    pub id: u32,
    pub statements: Vec<Statement>,
}

impl Scope {
    pub fn from_statements(statements: Vec<Statement>, id_counter: &mut ScopeIdCounter) -> Self {
        id_counter.counter += 1;
        Scope {
            id: id_counter.counter,
            statements,
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
        true_block: Scope,
        false_block: Option<Scope>,
    },
}

#[derive(Clone, Debug, PartialEq)]
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
        args: Vec<VarInfo>,
        return_type: Type,
        scope: Scope,
    },
}

#[derive(Debug, PartialEq)]
pub struct VarInfo {
    pub name: String,
    pub var_type: Type,
}
