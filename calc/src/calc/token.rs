
use std::fmt::Display;
use rust_decimal::Decimal;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Caret,
    LeftParen,
    RightParen,
    Number(Decimal),
    EOF
}

impl Token {
    pub fn get_precedence(&self) -> OperatorPrecedence {
        use Token::*;
        use OperatorPrecedence::*;

        match self {
            Add | Sub=> AddOrSub,
            Mul | Div => MulOrDiv,
            Caret => Power,
            _ => Default,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Caret => write!(f, "^"),
            LeftParen => write!(f, "("),
            RightParen => write!(f, ")"),
            Number(decimal) => write!(f, "{}",decimal),
            EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPrecedence {
    Default,
    AddOrSub,
    MulOrDiv,
    Power,
    Negative
}