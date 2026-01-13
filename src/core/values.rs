use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::fmt::Display;

use crate::core::bitseqs::Bitseq;
use crate::core::decimals::Decimal;
use crate::core::errors::{ConversionError, InvalidOperationError, SyntaxError};
use crate::core::integers::Integer;
use crate::core::parser::Position;
use crate::core::patterns;

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
        if patterns::BINARY_INTEGER.is_match(s) || patterns::BINARY_DECIMAL.is_match(s) {
            Some(2)
        } else if patterns::OCTAL_INTEGER.is_match(s) || patterns::OCTAL_DECIMAL.is_match(s) {
            Some(8)
        } else if patterns::DECIMAL_INTEGER.is_match(s) || patterns::DECIMAL_DECIMAL.is_match(s) {
            Some(10)
        } else if patterns::HEXADECIMAL_INTEGER.is_match(s)
            || patterns::HEXADECIMAL_DECIMAL.is_match(s)
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
            _ => panic!("Invalid character for digit or exceeded maximal base for conversion"),
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
            let mut frac_value = 0f64;
            let mut divisor = base;
            for c in frac.chars() {
                frac_value += (Self::_char_to_val(c) as f64) / divisor;
                divisor *= base;
            }
            return format!(
                "{}.{}",
                int_value,
                frac_value.to_string().split('.').nth(1).unwrap_or("0")
            );
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
            val_decimal: Decimal::ZERO,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_decimal(d: Decimal) -> Self {
        Self {
            type_: ValueType::Decimal,
            val_integer: Integer::ZERO,
            val_decimal: d,
            val_bitseq: Bitseq::ZERO,
        }
    }

    pub fn from_bitseq(b: Bitseq) -> Self {
        Self {
            type_: ValueType::Bitseq,
            val_integer: Integer::ZERO,
            val_decimal: Decimal::ZERO,
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
            }
            if into_type == ValueType::Decimal {
                self.val_decimal = self.val_bitseq.into();
            }
            self.val_bitseq = Bitseq::ZERO;
            self.type_ = into_type;
            return Ok(());
        }
        if self.type_ == ValueType::Integer {
            if into_type == ValueType::Bitseq {
                match Bitseq::try_from(self.val_integer.clone()) {
                    Err(e) => return Err(e),
                    Ok(converted) => {
                        self.val_bitseq = converted;
                    }
                }
            }
            if into_type == ValueType::Decimal {
                self.val_decimal = self.val_integer.into();
            }
            self.val_integer = Integer::ZERO;
            self.type_ = into_type;
            return Ok(());
        }
        if self.type_ == ValueType::Decimal {
            if into_type == ValueType::Bitseq {
                match Bitseq::try_from(self.val_decimal.clone()) {
                    Err(e) => return Err(e),
                    Ok(converted) => {
                        self.val_bitseq = converted;
                    }
                }
            }
            if into_type == ValueType::Integer {
                match Integer::try_from(self.val_decimal.clone()) {
                    Err(e) => return Err(e),
                    Ok(converted) => {
                        self.val_integer = converted;
                    }
                }
            }
            self.val_decimal = Decimal::ZERO;
            self.type_ = into_type;
            return Ok(());
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
        match self.type_ {
            ValueType::Bitseq => Self::from(-self.val_bitseq),
            ValueType::Decimal => Self::from(-self.val_decimal),
            ValueType::Integer => Self::from(-self.val_integer),
        }
    }

    pub fn logical_neg(&self) -> Self {
        let is_zero = match self.type_ {
            ValueType::Bitseq => self.val_bitseq.is_zero(),
            ValueType::Decimal => self.val_decimal == Decimal::ZERO,
            ValueType::Integer => self.val_integer == Integer::ZERO,
        };
        Self::from(Integer::from(is_zero))
    }

    pub fn bitwise_neg(&self) -> Result<Self, ConversionError> {
        let mut result = self.clone();
        if result.type_ != ValueType::Bitseq {
            result.try_mutate_into(ValueType::Bitseq)?;
        }
        result.val_bitseq.neg_mut();
        Ok(result)
    }

    pub fn factorial(&self) -> Result<Self, InvalidOperationError> {
        let mut result = if self.type_ == ValueType::Bitseq {
            Self::from(Integer::from(self.val_bitseq))
        } else {
            self.clone()
        };
        match result.type_ {
            ValueType::Bitseq => { /* Unreachable */ }
            ValueType::Decimal => {
                result.val_decimal = (result.val_decimal + Decimal::ONE).gamma()?
            }
            ValueType::Integer => result.val_integer = result.val_integer.factorial()?,
        }
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

impl Into<Decimal> for Value {
    fn into(self) -> Decimal {
        match self.type_ {
            ValueType::Bitseq => self.val_bitseq.into(),
            ValueType::Decimal => self.val_decimal,
            ValueType::Integer => self.val_integer.into(),
        }
    }
}

impl TryInto<Integer> for Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<Integer, Self::Error> {
        match self.type_ {
            ValueType::Bitseq => Ok(self.val_bitseq.into()),
            ValueType::Integer => Ok(self.val_integer),
            ValueType::Decimal => self.val_decimal.try_into(),
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
            ValueType::Decimal => "Decimal",
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
    _protected_keys: HashSet<String>,
    _readonly_keys: HashSet<String>,
}

impl ValueStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_protected_keys<S: AsRef<str>>(keys: Vec<S>) -> Self {
        let mut protected_keys: HashSet<String> = HashSet::with_capacity(keys.len());
        for k in keys.into_iter() {
            protected_keys.insert(k.as_ref().to_lowercase());
        }
        Self {
            _protected_keys: protected_keys,
            ..Default::default()
        }
    }

    pub fn add_protected_key<S: AsRef<str>>(&mut self, key: S) {
        self._protected_keys.insert(key.as_ref().to_lowercase());
    }

    pub fn remove_protected_key<S: AsRef<str>>(&mut self, key: S) {
        self._protected_keys.remove(&key.as_ref().to_lowercase());
    }

    pub fn set_readonly<S: AsRef<str>>(&mut self, identifier: S, value: Value) -> bool {
        let readonly_identifier = identifier.as_ref().to_lowercase();
        if !self.set(identifier, value) {
            return false;
        }
        self._readonly_keys.insert(readonly_identifier);
        true
    }

    pub fn set<S: AsRef<str>>(&mut self, identifier: S, value: Value) -> bool {
        let identifier = identifier.as_ref().to_lowercase();
        if self._readonly_keys.contains(&identifier) {
            return false;
        }
        self.map.insert(identifier, value);
        true
    }

    pub fn get<S: AsRef<str>>(&self, identifier: S) -> Option<&Value> {
        self.map.get(&identifier.as_ref().to_lowercase())
    }

    pub fn contains<S: AsRef<str>>(&self, identifier: S) -> bool {
        self.map.contains_key(&identifier.as_ref().to_lowercase())
    }

    pub fn clear(&mut self) {
        self.map.retain(|k, _| self._protected_keys.contains(k));
        self._readonly_keys
            .retain(|k| self._protected_keys.contains(k));
    }

    pub fn clear_all(&mut self) {
        self.map.clear();
        self._protected_keys.clear();
        self._readonly_keys.clear();
    }
}

impl Default for ValueStore {
    fn default() -> Self {
        Self {
            map: HashMap::with_capacity(20),
            _protected_keys: HashSet::new(),
            _readonly_keys: HashSet::new(),
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
