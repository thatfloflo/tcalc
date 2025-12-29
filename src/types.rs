use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use malachite::{Integer, Natural, Rational};
use malachite::base::strings::ToBinaryString;
use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::string::options::FromSciStringOptions;
use malachite::base::num::conversion::traits::{FromSciString, FromStringBase};
use subenum::subenum;

use crate::patterns;

#[subenum(ValueType)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    Expression,
    #[subenum(ValueType)]
    Bitseq,
    #[subenum(ValueType)]
    Integer,
    #[subenum(ValueType)]
    Rational,
    AmbiguousOperator,
    UnaryOperator,
    BinaryOperator,
    VariableIdentifier,
    UnaryFunctionIdentifier,
    BinaryFunctionIdentifier,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Expression => "Expression",
                Self::Bitseq => "Bitseq",
                Self::Integer => "Integer",
                Self::Rational => "Rational",
                Self::AmbiguousOperator => "AmbiguousOperator",
                Self::UnaryOperator => "UnaryOperator",
                Self::BinaryOperator => "BinaryOperator",
                Self::VariableIdentifier => "VariableIdentifier",
                Self::UnaryFunctionIdentifier => "UnaryFunctionIdentifier",
                Self::BinaryFunctionIdentifier => "BinaryFunctionIdentifier",
            }
        )
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bitseq => "Bitseq",
                Self::Integer => "Integer",
                Self::Rational => "Rational",
            }
        )
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Bitseq {
    value: u128,
    len: usize,
}

impl Display for Bitseq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#0len$b}", self.value, len = self.len + 2)
    }
}

impl Bitseq {
    pub const ZERO: Bitseq = Bitseq { value: 0, len: 1 };

    pub fn new(value: u128, len: usize) -> Bitseq {
        if len >= 128 {
            panic!("Length of Bitseq can be 128 bits at most");
        }
        Bitseq { value, len }
    }

    pub fn from_str(s: &str) -> Option<Bitseq> {
        if s.len() < 1 || s.len() > 128 || s.chars().any(|c| !(&['0', '1'].contains(&c))) {
            None
        } else {
            let value = if let Some(v) = u128::from_string_base(2, s) {
                v
            } else {
                return None;
            };
            Some(Bitseq {
                value,
                len: s.len(),
            })
        }
    }
}

impl Into<Integer> for Bitseq {
    fn into(self) -> Integer {
        Integer::from(self.value)
    }
}

impl Into<Rational> for Bitseq {
    fn into(self) -> Rational {
        Rational::from(self.value)
    }
}

impl TryFrom<Integer> for Bitseq {
    type Error = ConversionError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(ConversionError {
                msg: "Cannot convert negative Integer to Bitseq".to_string(),
            })
        } else {
            let natural = match Natural::try_from(value) {
                Ok(v) => v,
                Err(_) => {
                    return Err(ConversionError {
                        msg: "Cannot convert non-natural Integer to Bitseq".to_string(),
                    });
                }
            };
            // There's no direct (Try)From/Into Trait for Naturals and primitive unsigned types,
            // so this isn't ideal but it works
            let bitstr = natural.to_binary_string();
            match Self::from_str(&bitstr) {
                Some(v) => Ok(v),
                None => Err(ConversionError {
                    msg: "Cannot convert Integer that's too wide for u128 to Bitseq".to_string(),
                }),
            }
        }
    }
}

impl TryFrom<Rational> for Bitseq {
    type Error = ConversionError;

    fn try_from(value: Rational) -> Result<Self, Self::Error> {
        match Integer::try_from(value) {
            Ok(v) => match Self::try_from(v) {
                Ok(b) => Ok(b),
                Err(e) => Err(ConversionError {
                    msg: e.msg.replace("Integer", "Rational"),
                }),
            },
            Err(_) => Err(ConversionError {
                msg: "Cannot convert Rational to Bitseq that doesn't losslessly convert to Integer"
                    .to_string(),
            }),
        }
    }
}


#[derive(Clone)]
pub struct Value {
    type_: ValueType,
    val_integer: Integer,
    val_rational: Rational,
    val_bitseq: Bitseq,
}

impl Value {
    fn _check_str_and_get_base(s: &str) -> Option<u8> {
        if patterns::BINARY_INTEGER.is_match(s) || patterns::BINARY_RATIONAL.is_match(s) {
            Some(2)
        } else if patterns::OCTAL_INTEGER.is_match(s) || patterns::OCTAL_RATIONAL.is_match(s) {
            Some(8)
        } else if patterns::DECIMAL_INTEGER.is_match(s) || patterns::DECIMAL_RATIONAL.is_match(s) {
            Some(10)
        } else if patterns::HEXADECIMAL_INTEGER.is_match(s)
            || patterns::HEXADECIMAL_RATIONAL.is_match(s)
        {
            Some(16)
        } else {
            None
        }
    }

    fn _has_fractional_separator(s: &str) -> bool {
        s.contains('.') || s.contains(',')
    }

    fn _has_base_prefix(s: &str) -> bool {
        patterns::BASE_PREFIX.is_match(s)
    }

    fn _strip_base_prefix(s: String) -> String {
        match s.get(2..) {
            Some(subs) => subs.to_string(),
            None => s,
        }
    }

    fn _strip_str(s: &str) -> String {
        let result = s.replace('_', "").replace(',', ".");
        if Self::_has_base_prefix(s) {
            Self::_strip_base_prefix(result)
        } else {
            result
        }
    }

    fn _from_bitseq_str(s: &str, position: Position) -> Result<Value, SyntaxError> {
        let norm_s = Self::_strip_str(s);
        match Bitseq::from_str(&norm_s) {
            Some(b) => Ok(Self::from_bitseq(b)),
            None => Err(SyntaxError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into bit-sequence value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn _from_int_str(s: &str, base: u8, position: Position) -> Result<Self, SyntaxError> {
        let norm_s = Self::_strip_str(s);
        match Integer::from_string_base(base, &norm_s) {
            Some(i) => Ok(Self::from_integer(i)),
            None => Err(SyntaxError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into integer value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn _from_frac_str(s: &str, base: u8, position: Position) -> Result<Self, SyntaxError> {
        let norm_s = Self::_strip_str(s);
        let mut options = FromSciStringOptions::default();
        options.set_base(base);
        match Rational::from_sci_string_with_options(&norm_s, options) {
            Some(r) => Ok(Self::from_rational(r)),
            None => Err(SyntaxError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into rational value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    pub fn from_str(s: &str, position: Position) -> Result<Self, SyntaxError> {
        let base: u8 = if let Some(b) = Self::_check_str_and_get_base(s) {
            b
        } else {
            return Err(SyntaxError {
                msg: format!("The pattern of the numeral string \"{}\" is invalid", s),
                position: position,
            });
        };
        if Self::_has_fractional_separator(s) {
            Self::_from_frac_str(s, base, position)
        } else if base == 2 {
            Value::_from_bitseq_str(s, position)
        } else {
            Self::_from_int_str(s, base, position)
        }
    }

    pub fn from_integer(i: Integer) -> Self {
        Self {
            type_: ValueType::Integer,
            val_integer: i,
            val_rational: Rational::ZERO,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_rational(r: Rational) -> Self {
        Self {
            type_: ValueType::Rational,
            val_integer: Integer::ZERO,
            val_rational: r,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_bitseq(b: Bitseq) -> Self {
        Self {
            type_: ValueType::Bitseq,
            val_integer: Integer::ZERO,
            val_rational: Rational::ZERO,
            val_bitseq: b,
        }
    }

    pub fn try_mutate_into(&mut self, into_type: ValueType) -> Result<(), ConversionError> {
        if into_type == self.type_ {
            return Ok(());
        }
        if self.type_ == ValueType::Bitseq {
            if into_type == ValueType::Integer {
                self.val_integer = self.val_bitseq.into();
                self.val_bitseq = Bitseq::ZERO;
                return Ok(());
            }
            if into_type == ValueType::Rational {
                self.val_rational = self.val_bitseq.into();
                self.val_bitseq = Bitseq::ZERO;
                return Ok(());
            }
        }
        if self.type_ == ValueType::Integer {
            if into_type == ValueType::Bitseq {
                match Bitseq::try_from(self.val_integer.clone()) {
                    Ok(v) => {
                        self.val_bitseq = v;
                        self.val_integer = Integer::ZERO;
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            if into_type == ValueType::Rational {
                self.val_rational = Rational::from(self.val_integer.clone());
                self.val_integer = Integer::ZERO;
                return Ok(());
            }
        }
        if self.type_ == ValueType::Rational {
            if into_type == ValueType::Bitseq {
                match Bitseq::try_from(self.val_rational.clone()) {
                    Ok(v) => {
                        self.val_bitseq = v;
                        self.val_rational = Rational::ZERO;
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            if into_type == ValueType::Integer {
                match Integer::try_from(self.val_rational.clone()) {
                    Ok(v) => {
                        self.val_integer = v;
                        self.val_rational = Rational::ZERO;
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(ConversionError {
                            msg: "Rational could not be losslessly converted to Integer"
                                .to_string(),
                        });
                    }
                }
            }
        }
        Err(ConversionError {
            msg: format!(
                "No known conversion path to mutate {} to {}",
                self.type_, into_type
            ),
        })
    }
}

impl From<Rational> for Value {
    fn from(item: Rational) -> Self {
        Self::from_rational(item)
    }
}

impl From<Integer> for Value {
    fn from(item: Integer) -> Self {
        Self::from_integer(item)
    }
}

impl From<Bitseq> for Value {
    fn from(item: Bitseq) -> Self {
        Self::from_bitseq(item)
    }
}

impl TryFrom<&str> for Value {
    type Error = SyntaxError;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        Self::from_str(item, Position::default())
    }
}

impl Into<Rational> for Value {
    fn into(self) -> Rational {
        match self.type_ {
            ValueType::Bitseq => self.val_bitseq.into(),
            ValueType::Integer => self.val_integer.into(),
            ValueType::Rational => self.val_rational,
        }
    }
}

impl TryInto<Integer> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Integer, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => Ok(self.val_bitseq.into()),
            ValueType::Integer => Ok(self.val_integer),
            ValueType::Rational => match self.val_rational.try_into() {
                Ok(v) => Ok(v),
                Err(_) => Err(ConversionError {
                    msg: "Cannot convert Rational with denominator > 1 to Integer losslessly"
                        .to_string(),
                }),
            },
        }
    }
}

impl TryInto<Bitseq> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Bitseq, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => Ok(self.val_bitseq),
            ValueType::Integer => Bitseq::try_from(self.val_integer),
            ValueType::Rational => Bitseq::try_from(self.val_rational),
        }
    }
}



impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vtype = match self.type_ {
            ValueType::Bitseq => "Bitseq",
            ValueType::Integer => "Integer",
            ValueType::Rational => "Rational",
        };
        let val = match self.type_ {
            ValueType::Bitseq => self.val_bitseq.to_string(),
            ValueType::Integer => self.val_integer.to_string(),
            ValueType::Rational => self.val_rational.to_string(),
        };
        write!(f, "Value({}: {})", vtype, val)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub chr: usize,
}

impl Position {
    pub fn new(line: usize, chr: usize) -> Self {
        Self {
            line,
            chr,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 0,
            chr: 0,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
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

pub struct AstNode {
    pub token: Token,
    pub subtree: Option<Ast>,
    pub value: Option<Value>,
}

impl AstNode {
    pub fn new_from_token(token: Token) -> Self {
        Self {
            token: token,
            subtree: None,
            value: None,
        }
    }

    pub fn new_with_subtree(token: Token, subtree: Ast) -> Self {
        Self {
            token: token,
            subtree: Some(subtree),
            value: None,
        }
    }

    pub fn has_subtree(&self) -> bool {
        self.subtree.is_some()
    }

    pub fn set_subtree(&mut self, subtree: Ast) {
        self.subtree = Some(subtree);
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Err(e) = write!(f, "- {}", self.token) {
            return Err(e);
        }
        match &self.value {
            None => {}
            Some(value) => {
                if let Err(e) = write!(f, " -> {}", value) {
                    return Err(e);
                }
            }
        }
        match &self.subtree {
            None => {}
            Some(tree) => {
                if let Err(e) = write!(f, "\n{}", tree) {
                    return Err(e);
                }
            }
        }
        write!(f, "")
    }
}

pub struct Ast {
    _vec: Vec<AstNode>,
    _level: usize,
}

impl Ast {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(&self) -> usize {
        self._level
    }

    pub fn push(&mut self, mut item: AstNode) {
        if item.has_subtree() {
            item.subtree.as_mut().unwrap().relevel_from(self._level);
        }
        self._vec.push(item)
    }

    pub fn push_token(&mut self, token: Token) {
        self._vec.push(AstNode::new_from_token(token))
    }

    pub fn push_subtree(&mut self, token: Token, mut subtree: Ast) {
        subtree.relevel_from(self._level + 1);
        self._vec
            .push(AstNode::new_with_subtree(token, subtree))
    }

    pub fn last(&self) -> Option<&AstNode> {
        self._vec.last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AstNode> {
        self._vec.iter()
    }

    pub fn len(&self) -> usize {
        self._vec.len()
    }

    pub fn relevel_from(&mut self, base_level: usize) {
        self._level = base_level;
        for node in self._vec.iter_mut() {
            if node.has_subtree() {
                node.subtree.as_mut().unwrap().relevel_from(base_level + 1);
            }
        }
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self {
            _vec: Vec::new(),
            _level: 0,
        }
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();
        let indent = "    ".repeat(self._level);
        for item in &self._vec {
            formatted.push_str(format!("{:2} {}{}\n", self._level, indent, item).as_str());
        }
        formatted.pop(); // drop last newline
        write!(f, "{}", formatted)
    }
}

impl IntoIterator for Ast {
    type Item = AstNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._vec.into_iter()
    }
}

impl Deref for Ast {
    type Target = Vec<AstNode>;

    fn deref(&self) -> &Self::Target {
        &self._vec
    }
}

impl DerefMut for Ast {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vec
    }
}

impl From<&mut Vec<AstNode>> for Ast {
    fn from(value: &mut Vec<AstNode>) -> Self {
        let mut tree = Self::new();
        tree.append(value);
        tree
    }
}

impl From<Vec<AstNode>> for Ast {
    fn from(value: Vec<AstNode>) -> Self {
        let mut tree = Self::new();
        for node in value {
            tree.push(node);
        }
        tree
    }
}

impl From<AstNode> for Ast {
    fn from(value: AstNode) -> Self {
        let mut tree = Self::new();
        tree.push(value);
        tree
    }
}

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
