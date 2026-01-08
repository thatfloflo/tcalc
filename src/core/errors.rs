use std::error::Error;
use std::fmt::Display;

use crate::core::parser::Position;

#[derive(Debug, Clone)]
pub struct ConversionError {
    pub msg: String,
    pub position: Option<Position>,
}

impl ConversionError {
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        Self {
            msg: msg.as_ref().to_string(),
            position: None,
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position_hint = if self.position.is_some() {
            format!(" at {}", self.position.unwrap())
        } else {
            "".to_string()
        };
        write!(f, "{}{}", self.msg, position_hint)
    }
}

impl Error for ConversionError {}

#[derive(Debug, Clone)]
pub struct InvalidOperationError {
    pub msg: String,
    pub position: Option<Position>,
}

impl InvalidOperationError {
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        Self {
            msg: msg.as_ref().to_string(),
            position: None,
        }
    }
}

impl Display for InvalidOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position_hint = if self.position.is_some() {
            format!(" at {}", self.position.unwrap())
        } else {
            "".to_string()
        };
        write!(f, "{}{}", self.msg, position_hint)
    }
}

impl Error for InvalidOperationError {}

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

impl Error for SyntaxError {}
