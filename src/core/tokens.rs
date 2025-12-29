use std::fmt::Display;

use crate::core::parser::Position;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    AmbiguousOperator,
    BinaryFunctionIdentifier,
    BinaryOperator,
    Bitseq,
    Expression,
    Integer,
    Rational,
    UnaryFunctionIdentifier,
    UnaryOperator,
    VariableIdentifier,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AmbiguousOperator => "AmbiguousOperator",
                Self::BinaryFunctionIdentifier => "BinaryFunctionIdentifier",
                Self::BinaryOperator => "BinaryOperator",
                Self::Bitseq => "Bitseq",
                Self::Expression => "Expression",
                Self::Integer => "Integer",
                Self::Rational => "Rational",
                Self::UnaryFunctionIdentifier => "UnaryFunctionIdentifier",
                Self::UnaryOperator => "UnaryOperator",
                Self::VariableIdentifier => "VariableIdentifier",
            }
        )
    }
}

#[derive(Debug)]
pub struct Token {
    pub type_: TokenType,
    pub content: Vec<char>,
    pub position: Position,
    pub implicit: bool,
}

impl Token {
    pub fn new(type_: TokenType, content: Vec<char>, position: Position) -> Self {
        Self {
            type_,
            content,
            position,
            implicit: false,
        }
    }

    pub fn new_implicit(type_: TokenType, content: Vec<char>, position: Position) -> Self {
        Self {
            type_,
            content,
            position,
            implicit: true,
        }
    }

    pub fn content_to_string(&self) -> String {
        self.content.iter().collect()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let implicit_note = if self.implicit { " (implicit)" } else { "" };
        write!(
            f,
            "Token({:?}{}: \"{}\" at {})",
            self.type_,
            implicit_note,
            self.content_to_string(),
            self.position
        )
    }
}
