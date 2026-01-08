use std::fmt::Display;
use std::str::FromStr;

use fastnum::{D512, I512};
use fastnum::decimal::{Context, ParseError};

use crate::core::integers::Integer;
use crate::core::bitseqs::Bitseq;

pub const DECIMAL_CONTEXT: Context = Context::default();

pub type DecimalT = D512;

#[derive(Clone, Copy, Debug)]
pub struct Decimal {
    value: DecimalT
}

impl Decimal {
    pub const ZERO: Self = Self { value: DecimalT::ZERO.with_ctx(DECIMAL_CONTEXT) };
    pub const PI: Self = Self { value: DecimalT::PI.with_ctx(DECIMAL_CONTEXT) };
    pub const TAU: Self = Self { value: DecimalT::TAU.with_ctx(DECIMAL_CONTEXT) };
    pub const E: Self = Self { value: DecimalT::E.with_ctx(DECIMAL_CONTEXT) };

    pub fn inner_value(self) -> DecimalT {
        self.value
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.value.to_string();
        if s.contains(".") {
            // Trim trailing zeroes on the fractional part
            let (p1, p2) = s.split_once(".").unwrap();
            let fractional = p2.trim_end_matches("0");
            if fractional.len() == 0 {
                write!(f, "{}.0", p1)
            } else {
                write!(f, "{}.{}", p1, fractional)
            }
        } else {
            write!(f, "{}.0", s)
        }
    }
}

impl Into<DecimalT> for Decimal {
    fn into(self) -> DecimalT {
        self.value
    }
}

impl From<u128> for Decimal {
    fn from(value: u128) -> Self {
        Self { value: DecimalT::from_u128(value).unwrap() }
    }
}

impl From<Bitseq> for Decimal {
    fn from(value: Bitseq) -> Self {
        Self::from(value.inner_value())
    }
}

impl From<Integer> for Decimal {
    fn from(value: Integer) -> Self {
        use fastnum::decimal::Sign;
        let value: I512 = value.into();
        let sign = if value.is_negative() { Sign::Minus } else { Sign::Plus };
        let digits = value.abs().to_bits();
        let value = DecimalT::from_parts(digits, 0, sign, DECIMAL_CONTEXT);
        Self { value: value }
    }
}

impl FromStr for Decimal {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match DecimalT::from_str(s, DECIMAL_CONTEXT) {
            Ok(value) => Ok(Self { value }),
            Err(e) => Err(e),
        }
    }
}