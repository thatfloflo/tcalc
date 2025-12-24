#![allow(dead_code)]
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use lazy_static::lazy_static;
use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::string::options::FromSciStringOptions;
use malachite::base::num::conversion::traits::{FromSciString, FromStringBase};
use malachite::base::strings::ToBinaryString;
use malachite::{Integer, Natural, Rational};
use regex::Regex;
use subenum::subenum;

const NUMERAL_INITIALS: &str = "0123456789.,";
const NUMERAL_INTERNALS: &str = "0123456789.,abcdefoxABCDEFOX_";
const IGNORABLE_WHITESPACE: &str = " \t";
const OPERATOR_INITIALS: &str = "+-!^*/%¬<>=:&|?~";
const OPERATOR_INTERNALS: &str = OPERATOR_INITIALS;
const IDENTIFIER_INITIALS: &str = "abcdefghojklmnopqrstuvwxyzABCDEFGHOJKLMNOPQRSTUVWXYZ\\";
const IDENTIFIER_INTERNALS: &str = IDENTIFIER_INITIALS;
const AMBIGUOUS_OPERATORS: &[&str] = &["+", "-"];
const UNARY_OPERATORS: &[&str] = &["+", "-", "!", "¬", "~"];
const BINARY_OPERATORS: &[&str] = &[
    "^", "*", "/", "%", "+", "-", "<=>", "<=", ">=", ":=", "<<<", ">>>", "<<", ">>", "<>", "<",
    ">", "!=", "==", "&&", "||", "??", "!?", "&", "|",
];
const BUILTIN_UNARY_FUNCTIONS: &[&str] = &[
    "abs", "not", "sin", "cos", "tan", "cot", "sec", "csc", "exp", "ln", "lg", "log", "sqrt",
    "cbrt", "mem",
];
const BUILTIN_BINARY_FUNCTIONS: &[&str] = &["rt", "logb", "choose"];

lazy_static! {
    static ref BASE_PREFIX_PATTERN: Regex = Regex::new(r"^0[bBdDoOxX]").unwrap();
    static ref BINARY_INTEGER_PATTERN: Regex = Regex::new(r"^0[bB][01_]*[01]$").unwrap();
    static ref BINARY_RATIONAL_PATTERN: Regex =
        Regex::new(r"^0[bB][01_]*[.,](?:[01_]*[01])?$").unwrap();
    static ref DECIMAL_INTEGER_PATTERN: Regex =
        Regex::new(r"^(?:0[dD]_?[0-9]|[0-9])(?:[0-9_]*[0-9])?$").unwrap();
    static ref DECIMAL_RATIONAL_PATTERN: Regex =
        Regex::new(r"^(?:0[dD]_?)?(?:[0-9]*|[0-9][0-9_]*)[.,](?:[0-9]*|[0-9_]*[0-9])$").unwrap();
    static ref HEXADECIMAL_INTEGER_PATTERN: Regex =
        Regex::new(r"^0[xX][0-9a-fA-F_]*[0-9a-fA-F]$").unwrap();
    static ref HEXADECIMAL_RATIONAL_PATTERN: Regex =
        Regex::new(r"^0[xX][0-9a-fA-F_]*[.,](?:[0-9a-fA-F_]*[0-9a-fA-F])?$").unwrap();
    static ref OCTAL_INTEGER_PATTERN: Regex = Regex::new(r"^0[oO][0-7_]*[0-7]$").unwrap();
    static ref OCTAL_RATIONAL_PATTERN: Regex =
        Regex::new(r"^0[oO][0-7_]*[.,](?:[0-7_]*[0-7])?$").unwrap();
}

#[subenum(ValueType)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TokenType {
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
    ImplicitMultiplication,
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
                Self::ImplicitMultiplication => "ImplicitMultiplication",
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
struct Bitseq {
    value: u128,
    len: usize,
}

impl Display for Bitseq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#0len$b}", self.value, len = self.len + 2)
    }
}

impl Bitseq {
    const ZERO: Bitseq = Bitseq { value: 0, len: 1 };

    fn new(value: u128, len: usize) -> Bitseq {
        if len >= 128 {
            panic!("Length of Bitseq can be 128 bits at most");
        }
        Bitseq { value, len }
    }

    fn from_str(s: &str) -> Option<Bitseq> {
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
struct Value {
    type_: ValueType,
    val_integer: Integer,
    val_rational: Rational,
    val_bitseq: Bitseq,
}

impl Value {
    fn _check_str_and_get_base(s: &str) -> Option<u8> {
        if BINARY_INTEGER_PATTERN.is_match(s) || BINARY_RATIONAL_PATTERN.is_match(s) {
            Some(2)
        } else if OCTAL_INTEGER_PATTERN.is_match(s) || OCTAL_RATIONAL_PATTERN.is_match(s) {
            Some(8)
        } else if DECIMAL_INTEGER_PATTERN.is_match(s) || DECIMAL_RATIONAL_PATTERN.is_match(s) {
            Some(10)
        } else if HEXADECIMAL_INTEGER_PATTERN.is_match(s)
            || HEXADECIMAL_RATIONAL_PATTERN.is_match(s)
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
        BASE_PREFIX_PATTERN.is_match(s)
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

    fn _from_bitseq_str(s: &str, position: Position) -> Result<Value, ParseError> {
        let norm_s = Self::_strip_str(s);
        match Bitseq::from_str(&norm_s) {
            Some(b) => Ok(Self::from_bitseq(b)),
            None => Err(ParseError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into bit-sequence value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn _from_int_str(s: &str, base: u8, position: Position) -> Result<Self, ParseError> {
        let norm_s = Self::_strip_str(s);
        match Integer::from_string_base(base, &norm_s) {
            Some(i) => Ok(Self::from_integer(i)),
            None => Err(ParseError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into integer value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn _from_frac_str(s: &str, base: u8, position: Position) -> Result<Self, ParseError> {
        let norm_s = Self::_strip_str(s);
        let mut options = FromSciStringOptions::default();
        options.set_base(base);
        match Rational::from_sci_string_with_options(&norm_s, options) {
            Some(r) => Ok(Self::from_rational(r)),
            None => Err(ParseError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into rational value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn from_str(s: &str, position: Position) -> Result<Self, ParseError> {
        let base: u8 = if let Some(b) = Self::_check_str_and_get_base(s) {
            b
        } else {
            return Err(ParseError {
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

    fn from_integer(i: Integer) -> Self {
        Self {
            type_: ValueType::Integer,
            val_integer: i,
            val_rational: Rational::ZERO,
            val_bitseq: Bitseq::ZERO,
        }
    }

    fn from_rational(r: Rational) -> Self {
        Self {
            type_: ValueType::Rational,
            val_integer: Integer::ZERO,
            val_rational: r,
            val_bitseq: Bitseq::ZERO,
        }
    }

    fn from_bitseq(b: Bitseq) -> Self {
        Self {
            type_: ValueType::Bitseq,
            val_integer: Integer::ZERO,
            val_rational: Rational::ZERO,
            val_bitseq: b,
        }
    }

    fn try_mutate_into(&mut self, into_type: ValueType) -> Result<(), ConversionError> {
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
    type Error = ParseError;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        Self::from_str(item, Position { line: 0, chr: 0 })
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

struct ConversionError {
    msg: String,
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
struct Position {
    line: usize,
    chr: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
    }
}

#[derive(Debug)]
struct Token {
    type_: TokenType,
    content: Vec<char>,
    position: Position,
}

impl Token {
    fn content_to_string(&self) -> String {
        self.content.iter().collect()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token({:?}: \"{}\" at {})",
            self.type_,
            self.content_to_string(),
            self.position
        )
    }
}

struct ParseNode {
    token: Token,
    subtree: Option<ParseTree>,
    value: Option<Value>,
}

impl ParseNode {
    fn new_from_token(token: Token, subtree: Option<ParseTree>) -> ParseNode {
        ParseNode {
            token: token,
            subtree: subtree,
            value: None,
        }
    }
}

impl Display for ParseNode {
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

struct ParseTree {
    _vec: Vec<ParseNode>,
    _level: usize,
}

impl ParseTree {
    fn new() -> ParseTree {
        ParseTree {
            _vec: Vec::new(),
            _level: 0,
        }
    }

    fn new_subtree(level: usize) -> ParseTree {
        ParseTree {
            _vec: Vec::new(),
            _level: level,
        }
    }

    fn level(&self) -> usize {
        self._level
    }

    fn push(&mut self, item: ParseNode) {
        self._vec.push(item)
    }

    fn push_token(&mut self, token: Token) {
        self._vec.push(ParseNode::new_from_token(token, None))
    }

    fn push_subtree(&mut self, token: Token, mut subtree: ParseTree) {
        subtree._level = self._level + 1;
        self._vec
            .push(ParseNode::new_from_token(token, Some(subtree)))
    }

    fn last(&self) -> Option<&ParseNode> {
        self._vec.last()
    }

    fn iter(&self) -> impl Iterator<Item = &ParseNode> {
        self._vec.iter()
    }

    fn len(&self) -> usize {
        self._vec.len()
    }
}

impl Display for ParseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatted = String::new();
        let indent = "    ".repeat(self._level);
        for item in &self._vec {
            formatted.push_str(format!("{}{}\n", indent, item).as_str());
        }
        formatted.pop(); // drop last newline
        write!(f, "{}", formatted)
    }
}

impl IntoIterator for ParseTree {
    type Item = ParseNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._vec.into_iter()
    }
}

impl Deref for ParseTree {
    type Target = Vec<ParseNode>;

    fn deref(&self) -> &Self::Target {
        &self._vec
    }
}

impl DerefMut for ParseTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._vec
    }
}

#[derive(Debug, Clone)]
struct ParseError {
    msg: String,
    position: Position,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.msg, self.position)
    }
}

fn copy_while(input: &Vec<char>, charset: &str, start: usize, buf: &mut Vec<char>) {
    for character in &input[start..] {
        if charset.contains(*character) {
            buf.push(*character);
        } else {
            break;
        }
    }
}

fn copy_matchedspan(
    input: &Vec<char>,
    opening_char: char,
    closing_char: char,
    start: usize,
    buf: &mut Vec<char>,
) -> Result<(), ParseError> {
    let mut parens: usize = 1;
    for character in &input[start..] {
        if *character == opening_char {
            parens += 1;
        } else if *character == closing_char {
            parens -= 1;
        }
        if parens < 1 {
            break;
        }
        if parens > 0 {
            buf.push(*character);
        }
    }
    if parens > 0 {
        return Err(ParseError {
            msg: "Could not match open parenthesis with closing parenthesis".to_string(),
            position: Position {
                line: 0,
                chr: start,
            },
        });
    }
    return Ok(());
}

fn tokenize(
    input: String,
    line: Option<usize>,
    chr: Option<usize>,
    tree: &mut ParseTree,
) -> Result<(), ParseError> {
    let input: Vec<char> = input.chars().collect();
    let line: usize = line.unwrap_or(0);
    let chr: usize = chr.unwrap_or(0);
    let mut buf: Vec<char> = Vec::new();
    let mut i: usize = 0;
    while i < input.len() {
        if IGNORABLE_WHITESPACE.contains(input[i]) {
            // do naught
        } else if input[i] == '(' {
            // Match TokenType.Expression
            // Find matching closing parenthesis and consume input along the way
            if let Err(e) = copy_matchedspan(&input, '(', ')', i + 1, &mut buf) {
                return Err(ParseError {
                    msg: e.msg,
                    position: Position {
                        line: line,
                        chr: chr + i,
                    },
                });
            }
            let token = Token {
                type_: TokenType::Expression,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            };
            let mut subtree = ParseTree::new_subtree(tree.level() + 1);
            match tokenize(
                token.content_to_string(),
                Some(line),
                Some(chr + i),
                &mut subtree,
            ) {
                Err(e) => return Err(e),
                Ok(_) => tree.push_subtree(token, subtree),
            }
            i += buf.len() + 1; // Skip the closing paren
            buf.clear();
        } else if NUMERAL_INITIALS.contains(input[i]) {
            // Match TokenType.Numeral
            buf.push(input[i]);
            copy_while(&input, NUMERAL_INTERNALS, i + 1, &mut buf);
            let token_type: TokenType;
            if buf.contains(&'.') || buf.contains(&',') {
                token_type = TokenType::Rational;
            } else if buf.starts_with(&['0', 'b']) {
                token_type = TokenType::Bitseq;
            } else {
                token_type = TokenType::Integer;
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if IDENTIFIER_INITIALS.contains(input[i]) {
            // Match TokenType.Identifier
            buf.push(input[i]);
            copy_while(&input, IDENTIFIER_INTERNALS, i + 1, &mut buf);
            let token_type: TokenType;
            let buf_string = buf.iter().collect::<String>();
            if BUILTIN_UNARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                token_type = TokenType::UnaryFunctionIdentifier;
            } else if BUILTIN_BINARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                token_type = TokenType::BinaryFunctionIdentifier;
            } else {
                token_type = TokenType::VariableIdentifier;
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if OPERATOR_INITIALS.contains(input[i]) {
            // Match TokenType.Operator
            buf.push(input[i]);
            copy_while(&input, OPERATOR_INTERNALS, i + 1, &mut buf);
            let token_type: TokenType;
            let buf_string = buf.iter().collect::<String>();
            if AMBIGUOUS_OPERATORS.contains(&buf_string.as_str()) {
                token_type = TokenType::AmbiguousOperator;
            } else if UNARY_OPERATORS.contains(&&buf_string.as_str()) {
                token_type = TokenType::UnaryOperator;
            } else if BINARY_OPERATORS.contains(&&buf_string.as_str()) {
                token_type = TokenType::BinaryOperator;
            } else {
                return Err(ParseError {
                    msg: format!("Unknown operator '{}'", buf_string),
                    position: Position {
                        line: line,
                        chr: chr + i,
                    },
                });
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if input[i] == ')' {
            return Err(ParseError {
                msg: "Unexpected closing parenthesis".to_string(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
        } else {
            return Err(ParseError {
                msg: format!("Unknown character '{}'", input[i]),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
        }
        i += 1;
    }

    if let Err(e) = disambiguate_operators(tree) {
        return Err(e);
    }

    return expose_implicit_multiplications(tree);
}

fn expose_implicit_multiplications(tree: &mut ParseTree) -> Result<(), ParseError> {
    let mut i: usize = 0;
    while i + 1 < tree.len() {
        let is_value = match tree[i].token.type_ {
            TokenType::AmbiguousOperator => false,
            TokenType::BinaryFunctionIdentifier => false,
            TokenType::BinaryOperator => false,
            TokenType::Bitseq => true,
            TokenType::Expression => true,
            TokenType::ImplicitMultiplication => false,
            TokenType::Integer => true,
            TokenType::Rational => true,
            TokenType::UnaryFunctionIdentifier => false,
            TokenType::UnaryOperator => tree[i].token.content == vec!['!'],
            TokenType::VariableIdentifier => true,
        };
        let next_is_value = match tree[i + 1].token.type_ {
            TokenType::AmbiguousOperator => false, // We just don't know, so they ought to be disambiguated first
            TokenType::BinaryFunctionIdentifier => false,
            TokenType::BinaryOperator => false,
            TokenType::Bitseq => true,
            TokenType::Expression => true,
            TokenType::ImplicitMultiplication => false,
            TokenType::Integer => true,
            TokenType::Rational => true,
            TokenType::UnaryFunctionIdentifier => true,
            TokenType::UnaryOperator => tree[i + 1].token.content != vec!['!'],
            TokenType::VariableIdentifier => true,
        };
        if is_value && next_is_value {
            let token = Token {
                type_: TokenType::ImplicitMultiplication,
                content: vec![],
                position: tree[i + 1].token.position,
            };
            tree.insert(i + 1, ParseNode::new_from_token(token, None));
            i += 1;
        }
        i += 1;
    }
    return Ok(());
}

fn disambiguate_operators(tree: &mut ParseTree) -> Result<(), ParseError> {
    let mut i: usize = 0;
    while i < tree.len() {
        if tree[i].token.type_ == TokenType::AmbiguousOperator {
            let has_left_value: bool;
            if i < 1 {
                has_left_value = tree.level() == 0;
            } else {
                has_left_value = match tree[i - 1].token.type_ {
                    TokenType::AmbiguousOperator => false,
                    TokenType::BinaryFunctionIdentifier => false,
                    TokenType::BinaryOperator => false,
                    TokenType::Bitseq => true,
                    TokenType::Expression => true,
                    TokenType::ImplicitMultiplication => false,
                    TokenType::Integer => true,
                    TokenType::Rational => true,
                    TokenType::UnaryFunctionIdentifier => false,
                    TokenType::UnaryOperator => tree[i - 1].token.content == vec!['!'],
                    TokenType::VariableIdentifier => true,
                };
            }
            let has_right_value: bool;
            if i + 1 >= tree.len() {
                has_right_value = false; // +/- cannot be at end of expressions
            // Really just return ParseError here?
            } else {
                has_right_value = match tree[i + 1].token.type_ {
                    TokenType::AmbiguousOperator => true, // Will necessarily disambiguate to UnaryOp later
                    TokenType::BinaryFunctionIdentifier => true,
                    TokenType::BinaryOperator => false,
                    TokenType::Bitseq => true,
                    TokenType::Expression => true,
                    TokenType::ImplicitMultiplication => false,
                    TokenType::Integer => true,
                    TokenType::Rational => true,
                    TokenType::UnaryFunctionIdentifier => true,
                    TokenType::UnaryOperator => {
                        if tree[i + 1].token.content == vec!['!'] {
                            return Err(ParseError {
                                msg: format!(
                                    "Ambiguous operator '{}' cannot precede unary operator '!'",
                                    tree[i].token.content_to_string()
                                ),
                                position: tree[i].token.position,
                            });
                        }
                        true
                    }
                    TokenType::VariableIdentifier => true,
                };
            }
            if has_left_value == true && has_right_value == true {
                tree[i].token.type_ = TokenType::BinaryOperator;
            } else if has_left_value == false && has_right_value == true {
                tree[i].token.type_ = TokenType::UnaryOperator;
            } else {
                return Err(ParseError {
                    msg: format!(
                        "Could not disambiguate ambiguous operator '{}', consider using parentheses",
                        tree[i].token.content_to_string()
                    ),
                    position: tree[i].token.position,
                });
            }
        }
        i += 1;
    }
    return Ok(());
}

fn resolve_numerals(tree: &mut ParseTree) -> Result<(), ParseError> {
    let mut i: usize = 0;
    while i < tree.len() {
        if tree[i].token.type_ == TokenType::Bitseq
            || tree[i].token.type_ == TokenType::Integer
            || tree[i].token.type_ == TokenType::Rational
        {
            match Value::from_str(&tree[i].token.content_to_string(), tree[i].token.position) {
                Ok(v) => {
                    tree[i].value = Some(v);
                }
                Err(e) => return Err(e),
            };
        }
        i += 1;
    }
    Ok(())
}

fn resolve(tree: &mut ParseTree) -> Result<(), ParseError> {
    // - Resolve subexpressions to values (if any)
    // - Resolve numerals to values
    if let Err(e) = resolve_numerals(tree) {
        return Err(e);
    }
    // - Resolve variable identifiers to values
    // - Resolve unary operators to values (precedence, then RTL)
    // - Resolve unary functions to values (RTL)
    // - Resolve binary operators to values (precedence, then RTL)
    // - Resolve binary functios to values (RTL)
    Ok(())
}

fn main() {
    //let input = "0b1001101.100101 + 83_382_292x22 / 0b000101 * (0xDEADBEEF0 - D17,343 (28.1 + 3)) + sqrt(1+7)";
    let input = "0b0010010 - 2.55 0D587 0b010.01 ( - 7)";
    let mut parse_tree = ParseTree::new();
    println!("INPUT: {}", input);
    tokenize(input.to_string(), None, None, &mut parse_tree).unwrap();
    resolve(&mut parse_tree).unwrap();
    println!("===== PARSED TOKENS =====");
    println!("{}", parse_tree);
}
