use std::fmt::Display;

use malachite::base::num::basic::traits::Zero;
use malachite::base::num::conversion::string::options::FromSciStringOptions;
use malachite::base::num::conversion::traits::{FromSciString, FromStringBase};
use malachite::{Integer, Rational};

use crate::core::bitseq::Bitseq;
use crate::core::errors::{ConversionError, SyntaxError};
use crate::core::parser::Position;
use crate::core::patterns;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    Bitseq,
    Integer,
    Rational,
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
