use std::collections::HashMap;
use std::convert::From;
use std::fmt::Display;

use crate::core::bitseqs::Bitseq;
use crate::core::errors::{ConversionError, InvalidOperationError, SyntaxError};
use crate::core::parser::Position;
use crate::core::patterns;

pub type Integer = i128;
pub type Decimal = f64;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    Bitseq,
    Decimal,
    Integer,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bitseq => "Bitseq",
                Self::Decimal => "Decimal",
                Self::Integer => "Integer",
            }
        )
    }
}

#[derive(Clone)]
pub struct Value {
    type_: ValueType,
    val_bitseq: Bitseq,
    val_decimal: Decimal,
    val_integer: Integer,
}

impl Value {
    fn _check_str_and_get_base<S: AsRef<str>>(s: S) -> Option<u8> {
        let s = s.as_ref();
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

    fn _has_fractional_separator<S: AsRef<str>>(s: S) -> bool {
        let s = s.as_ref().to_string();
        s.contains('.') || s.contains(',')
    }

    fn _has_base_prefix<S: AsRef<str>>(s: S) -> bool {
        let s = s.as_ref();
        patterns::BASE_PREFIX.is_match(s)
    }

    fn _strip_base_prefix<S: AsRef<str>>(s: S) -> String {
        let s = s.as_ref().to_string();
        match s.get(2..) {
            Some(subs) => subs.to_string(),
            None => s,
        }
    }

    fn _strip_str<S: AsRef<str>>(s: S) -> String {
        let s = s.as_ref().to_string();
        let result = s.replace('_', "").replace(',', ".");
        if Self::_has_base_prefix(s) {
            Self::_strip_base_prefix(result)
        } else {
            result
        }
    }

    fn _char_to_val(c: char) -> u8 {
        match c {
            '0'..='9' => (c as u8) - b'0',
            'a'..='z' => (c as u8) - b'a' + 10,
            'A'..='Z' => (c as u8) - b'A' + 10,
            _ => panic!("Invalid character for digit or exceeded maximal base for conversion")
        }
    }

    fn _to_base_10<S: AsRef<str>>(s: S, base: u8) -> String {
        let s = s.as_ref();
        let base: u32 = base.into();

        let mut parts = s.split('.');
        let int_part = parts.next().unwrap();
        let frac_part = parts.next();

        let mut int_value: u32 = 0;
        for c in int_part.chars() {
            int_value = int_value * base + Self::_char_to_val(c) as u32;
        }

        if let Some(frac) = frac_part {
            let base = base as f64;
            let mut frac_value =  0f64;
            let mut divisor = base;
            for c in frac.chars() {
                frac_value += (Self::_char_to_val(c) as f64) / divisor;
                divisor *= base;
            }
            return format!("{}.{}", int_value, frac_value.to_string().split('.').nth(1).unwrap_or("0"))
        }

        format!("{}", int_value)
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
        match Integer::from_str_radix(&norm_s, base.into()) {
            Ok(i) => Ok(Self::from_integer(i)),
            Err(_) => Err(SyntaxError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\" into integer value",
                    s, norm_s
                ),
                position: position,
            }),
        }
    }

    fn _from_dec_str(s: &str, base: u8, position: Position) -> Result<Self, SyntaxError> {
        let mut norm_s = Self::_strip_str(s);
        norm_s = Self::_to_base_10(norm_s, base);
        match norm_s.parse::<Decimal>() {
            Ok(d) => Ok(Self::from_decimal(d)),
            Err(_) => Err(SyntaxError {
                msg: format!(
                    "Failed to parse string \"{}\" (normalised to \"{}\") into decimal value",
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
            Self::_from_dec_str(s, base, position)
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
            val_decimal: 0.0,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_decimal(d: Decimal) -> Self {
        Self {
            type_: ValueType::Decimal,
            val_integer: 0,
            val_decimal: d,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_bitseq(b: Bitseq) -> Self {
        Self {
            type_: ValueType::Bitseq,
            val_integer: 0,
            val_decimal: 0.0,
            val_bitseq: b,
        }
    }

    pub fn try_mutate_into(&mut self, into_type: ValueType) -> Result<(), ConversionError> {
        if into_type == self.type_ {
            return Ok(());
        }
        if self.type_ == ValueType::Bitseq {
            if into_type == ValueType::Integer {
                self.val_integer = self.val_bitseq.try_into()?;
                self.val_bitseq = Bitseq::ZERO;
                self.type_ = into_type;
                return Ok(());
            }
            if into_type == ValueType::Decimal {
                self.val_decimal = self.val_bitseq.try_into()?;
                self.val_bitseq = Bitseq::ZERO;
                self.type_ = into_type;
                return Ok(());
            }
        }
        if self.type_ == ValueType::Integer {
            if into_type == ValueType::Bitseq {
                self.val_bitseq = Bitseq::try_from(self.val_integer.clone())?;
                self.val_integer = 0;
                self.type_ = into_type;
                return Ok(());
            }
            if into_type == ValueType::Decimal {
                let converted = self.val_integer as Decimal;
                if converted as Integer == self.val_integer {
                    self.val_decimal = converted;
                    self.val_integer = 0;
                    self.type_ = into_type;
                    return Ok(());
                } else {
                    return Err(ConversionError::new(
                        "Integer too wide to convert to Decimal",
                    ));
                }
            }
        }
        if self.type_ == ValueType::Decimal {
            if into_type == ValueType::Bitseq {
                self.val_bitseq = Bitseq::try_from(self.val_decimal.clone())?;
                self.val_decimal = 0.0;
                self.type_ = into_type;
                return Ok(());
            }
            if into_type == ValueType::Integer {
                let converted = self.val_decimal as Integer;
                if converted as Decimal == self.val_decimal {
                    self.val_integer = converted;
                    self.val_decimal = 0.0;
                    self.type_ = into_type;
                    return Ok(());
                } else {
                    return Err(ConversionError::new(
                        "Decimal could not be losslessly converted to Integer",
                    ));
                }
            }
        }
        Err(ConversionError::new(format!(
            "No known conversion path to mutate {} to {}",
            self.type_, into_type
        )))
    }

    pub fn unary_pos(&self) -> Self {
        self.clone()
    }

    pub fn unary_neg(&self) -> Self {
        let mut result = self.clone();
        if result.type_ == ValueType::Bitseq {
            result.try_mutate_into(ValueType::Integer).unwrap();
        }
        match result.type_ {
            ValueType::Bitseq => { /* unreachable */ },
            ValueType::Decimal => { result.val_decimal = -result.val_decimal; }
            ValueType::Integer => { result.val_integer = -result.val_integer; }
        }
        result
    }

    pub fn factorial(&self) -> Result<Self, InvalidOperationError> {
        let mut result = self.clone();
        if result.type_ == ValueType::Bitseq {
            result.try_mutate_into(ValueType::Integer).unwrap();
        }
        if result.type_ == ValueType::Integer {
            match result.val_integer.signum() {
                0 | 1 => {
                    result.val_integer = match (1..result.val_integer).try_fold(result.val_integer, Integer::checked_mul) {
                        Some(v) => v,
                        None => { return Err(InvalidOperationError::new("Factorial too large to fit Integer")); },
                    };
                }
                _ => { // == -1
                    return Err(InvalidOperationError::new(
                        "Factorial operation undefined for negative integers",
                    ));
                },
            }
        } else {
            // == ValueType::Rational
            todo!();
            // match result.val_decimal.signum() {
            //     0.0 | 1.0 => {
            //         //...
            //     },
            //     -1 => {
            //         return Err(InvalidOperationError::new("Factorial operation undefined for negative integers"));
            //     },
            // }
        }
        Ok(result)
    }

    pub fn logical_neg(&self) -> Self {
        let is_zero = match self.type_ {
            ValueType::Bitseq => { self.val_bitseq.is_zero() },
            ValueType::Decimal => { self.val_decimal == 0.0 },
            ValueType::Integer => { self.val_integer == 0 },
        };
        Self::from_integer(is_zero.into())
    }

    pub fn bitwise_neg(&self) -> Result<Self, ConversionError> {
        if self.type_ != ValueType::Bitseq {
            return Err(ConversionError::new("Cannot apply bitwise negation to a value other than a bit-sequence (Bitseq)"));
        }
        let mut result = self.clone();
        result.val_bitseq.neg_mut();
        Ok(result)
    }
}

impl From<Decimal> for Value {
    fn from(item: Decimal) -> Self {
        Self::from_decimal(item)
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

impl TryInto<Decimal> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => self.val_bitseq.try_into(),
            ValueType::Decimal => Ok(self.val_decimal),
            ValueType::Integer => {
                let converted = self.val_integer as Decimal;
                if converted as Integer == self.val_integer {
                    Ok(converted)
                } else {
                    Err(ConversionError::new("Integer too wide to convert to Decimal"))
                }
            },
        }
    }
}

impl TryInto<Integer> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Integer, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => self.val_bitseq.try_into(),
            ValueType::Integer => Ok(self.val_integer),
            ValueType::Decimal => {
                let converted = self.val_decimal as Integer;
                if converted as Decimal == self.val_decimal {
                    Ok(converted)
                } else {
                    Err(ConversionError::new(
                    "Cannot convert Decimal with with fractional part to Integer losslessly",
                ))
                }
            },
            // ValueType::Decimal => match self.val_decimal.try_into() {
            //     Ok(v) => Ok(v),
            //     Err(_) => Err(ConversionError::new(
            //         "Cannot convert Rational with denominator > 1 to Integer losslessly",
            //     )),
            // },
        }
    }
}

impl TryInto<Bitseq> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Bitseq, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => Ok(self.val_bitseq),
            ValueType::Integer => Bitseq::try_from(self.val_integer),
            ValueType::Decimal => Bitseq::try_from(self.val_decimal),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vtype = match self.type_ {
            ValueType::Bitseq => "Bitseq",
            ValueType::Integer => "Integer",
            ValueType::Decimal => "Rational",
        };
        let val = match self.type_ {
            ValueType::Bitseq => self.val_bitseq.to_string(),
            ValueType::Integer => self.val_integer.to_string(),
            ValueType::Decimal => self.val_decimal.to_string(),
        };
        write!(f, "Value({}: {})", vtype, val)
    }
}

pub struct ValueStore {
    pub map: HashMap<String, Value>,
    _protected_keys: Vec<String>,
}

impl ValueStore {
    pub fn new() -> Self {
        let mut vs = Self::default();
        vs.set(
            "test",
            Value::from_str("12345.06789", Position::default()).unwrap(),
        );
        vs.set(
            "\\blah",
            Value::from_str("0b000101011", Position::default()).unwrap(),
        );
        vs
    }

    pub fn with_protected_keys<S: AsRef<str>>(keys: Vec<S>) -> Self {
        let mut protected_keys = Vec::with_capacity(keys.len());
        for k in keys.into_iter() {
            protected_keys.push(k.as_ref().to_string())
        }
        Self {
            _protected_keys: protected_keys,
            ..Default::default()
        }
    }

    pub fn set<S: AsRef<str>>(&mut self, identifier: S, value: Value) {
        self.map.insert(identifier.as_ref().to_string(), value);
    }

    pub fn get<S: AsRef<str>>(&self, identifier: S) -> Option<&Value> {
        self.map.get(&identifier.as_ref().to_string())
    }

    pub fn contains_identifier<S: AsRef<str>>(&self, identifier: S) -> bool {
        self.map.contains_key(&identifier.as_ref().to_string())
    }

    pub fn clear(&mut self) {
        self.map.retain(|k, _| self._protected_keys.contains(&k));
    }

    pub fn clear_all(&mut self) {
        self.map.clear();
        self._protected_keys.clear();
    }
}

impl Default for ValueStore {
    fn default() -> Self {
        Self {
            map: HashMap::with_capacity(20),
            _protected_keys: vec![],
        }
    }
}

impl From<HashMap<String, Value>> for ValueStore {
    fn from(value: HashMap<String, Value>) -> Self {
        Self {
            map: value,
            ..Default::default()
        }
    }
}
