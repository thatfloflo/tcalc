use std::fmt::Display;

use crate::core::errors::InputPosition;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    AmbiguousOperator,
    BinaryFunctionIdentifier,
    BinaryOperator,
    Bitseq,
    Expression,
    Integer,
    Decimal,
    UnaryFunctionIdentifier,
    UnaryOperator,
    VariableIdentifier,
}

impl TokenType {
    pub fn is_numeral(self) -> bool {
        matches!(self, Self::Bitseq | Self:: Integer | Self::Decimal)
    }
    pub fn is_operator(self) -> bool {
        matches!(self, Self::AmbiguousOperator | Self::BinaryOperator | Self::UnaryOperator)
    }
    pub fn is_resolved_operator(self) -> bool {
        matches!(self, Self::BinaryOperator | Self::UnaryOperator)
    }
    pub fn is_unary(self) -> bool {
        matches!(self, Self::UnaryFunctionIdentifier | Self::UnaryOperator)
    }
    pub fn is_binary(self) -> bool {
        matches!(self, Self::BinaryFunctionIdentifier | Self::BinaryOperator)
    }
    pub fn is_identifier(self) -> bool {
        matches!(self, Self::BinaryFunctionIdentifier | Self::UnaryFunctionIdentifier | Self::VariableIdentifier)
    }
    pub fn is_function_identifier(self) -> bool {
        matches!(self, Self::BinaryFunctionIdentifier | Self::UnaryFunctionIdentifier)
    }
    pub fn is_variable_identifier(self) -> bool {
        self == Self::VariableIdentifier
    }
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Bitseq | Self::Integer | Self::Decimal | Self::VariableIdentifier)
    }
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
                Self::Decimal => "Decimal",
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
    pub position: InputPosition,
    pub implicit: bool,
}

impl Token {
    pub fn new(type_: TokenType, content: Vec<char>, position: InputPosition) -> Self {
        Self {
            type_,
            content,
            position,
            implicit: false,
        }
    }

    pub fn new_implicit(type_: TokenType, content: Vec<char>, position: InputPosition) -> Self {
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
