use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt::Display;
use std::ops::{Add, Neg};

use fastnum::I512;

use crate::core::bitseqs::{Bitseq, BitseqT};
use crate::core::decimals::Decimal;
use crate::core::errors::{ConversionError, InvalidOperationError, SyntaxError};
use crate::core::parser::Position;

pub type IntegerT = I512;

#[derive(Clone, Copy, Debug)]
pub struct Integer {
    value: IntegerT,
}

impl Integer {
    pub const ZERO: Self = Self {
        value: IntegerT::ZERO,
    };

    pub const ONE: Self = Self {
        value: IntegerT::ONE,
    };

    const MAX_FACTORIAL: Self = Self {
        value: IntegerT::from_u8(97u8),
    };

    pub const BITSEQ_MAX_VALUE: Self = Self {
        // Evaluates to 340_282_366_920_938_463_463_374_607_431_768_211_455 == u128::MAX
        value: IntegerT::from_digits([
            18446744073709551615,
            18446744073709551615,
            0,
            0,
            0,
            0,
            0,
            0,
        ]),
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

    pub fn factorial(self) -> Result<Self, InvalidOperationError> {
        if self < Self::ZERO {
            return Err(InvalidOperationError::new(
                "Factorial undefined for values < 0",
            ));
        }
        if self > Self::MAX_FACTORIAL {
            return Err(InvalidOperationError::new(format!(
                "Factorial of value > {} exceeds size of Integer type, consider approximating the factorial via `gamma (x + 1)`",
                Self::MAX_FACTORIAL
            )));
        }
        let mut result = IntegerT::ONE;
        let mut i = IntegerT::ZERO;
        while i < self.value {
            i = i + IntegerT::ONE;
            result = result * i;
        }
        Ok(Self { value: result })
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

impl From<BitseqT> for Integer {
    fn from(value: BitseqT) -> Self {
        Self {
            value: IntegerT::from_u128(value).unwrap(),
        }
    }
}

impl From<bool> for Integer {
    fn from(value: bool) -> Self {
        Self {
            value: if value { IntegerT::ONE } else { IntegerT::ZERO },
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

impl Ord for Integer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Integer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Integer {}

impl PartialEq for Integer {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Neg for Integer {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

impl Add for Integer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}
