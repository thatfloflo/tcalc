use std::fmt::Display;

use crate::core::parser::Position;

pub struct ConversionError {
    pub msg: String,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub msg: String,
    pub position: Position,
}

impl SyntaxError {
    pub fn new<S: AsRef<str>>(msg: S, position: Position) -> Self {
        Self {
            msg: msg.as_ref().to_string(),
            position: position,
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.msg, self.position)
    }
}
