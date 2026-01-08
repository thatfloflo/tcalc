use std::fmt::Display;

use fastnum::I512;

use crate::core::bitseqs::Bitseq;
use crate::core::decimals::Decimal;
use crate::core::errors::{ConversionError, SyntaxError};
use crate::core::parser::Position;

pub type IntegerT = I512;

#[derive(Clone, Copy, Debug)]
pub struct Integer {
    value: IntegerT,
}

impl Integer {
    const ZERO: Self = Self {
        value: IntegerT::ZERO,
    };

    pub fn from_str_radix<S: AsRef<str>>(src: S, radix: u32) -> Result<Self, SyntaxError> {
        match IntegerT::from_str_radix(src.as_ref(), radix) {
            Ok(value) => Ok(Self { value }),
            Err(_) => Err(SyntaxError::new(
                "Failed to parse string \"{}\" of base {} into Integer",
                Position::default(),
            )),
        }
    }

    pub fn inner_value(self) -> IntegerT {
        self.value
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<IntegerT> for Integer {
    fn into(self) -> IntegerT {
        self.value
    }
}

impl From<Bitseq> for Integer {
    fn from(value: Bitseq) -> Self {
        Self {
            value: IntegerT::from_u128(value.into()).unwrap(),
        }
    }
}

impl TryFrom<Decimal> for Integer {
    type Error = ConversionError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        use crate::core::decimals::DecimalT;
        let raw: DecimalT = value.into();
        if raw.fractional_digits_count() > 0 {
            return Err(ConversionError::new(
                "Cannot convert Decimal with a fractional part to Integer",
            ));
        }
        match IntegerT::from_str(&raw.to_string()) {
            Ok(value) => Ok(Self { value }),
            Err(_) => Err(ConversionError::new(
                "Decimal too large to convert to Integer",
            )),
        }
    }
}
