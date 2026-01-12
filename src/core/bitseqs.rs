use std::fmt::Display;
use std::ops::Neg;

use crate::core::decimals::Decimal;
use crate::core::errors::ConversionError;
use crate::core::integers::Integer;

pub type BitseqT = u128;

#[derive(Clone, Copy, Debug)]
pub struct Bitseq {
    value: BitseqT,
    len: usize,
}

impl Display for Bitseq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#0len$b}", self.value, len = self.len + 2)
    }
}

impl Bitseq {
    pub const ZERO: Bitseq = Bitseq { value: 0, len: 1 };
    pub const ONE: Bitseq = Bitseq { value: 1, len: 1 };

    pub fn new(value: BitseqT, len: usize) -> Self {
        if len >= BitseqT::BITS as usize {
            panic!("Length of Bitseq can be 128 bits at most");
        }
        Self { value, len }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() < 1
            || s.len() > BitseqT::BITS as usize
            || s.chars().any(|c| !(&['0', '1'].contains(&c)))
        {
            None
        } else {
            let value = if let Ok(v) = BitseqT::from_str_radix(s, 2) {
                v
            } else {
                return None;
            };
            Some(Self {
                value,
                len: s.len(),
            })
        }
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn neg_mut(&mut self) {
        let mut mask: BitseqT = 0;
        for i in 0..self.len {
            mask |= 1 << i;
        }
        self.value ^= mask;
    }

    pub fn inner_value(self) -> BitseqT {
        self.value
    }
}

impl Into<BitseqT> for Bitseq {
    fn into(self) -> BitseqT {
        self.value
    }
}

impl From<BitseqT> for Bitseq {
    fn from(value: BitseqT) -> Self {
        Self {
            value,
            len: (BitseqT::BITS - value.leading_zeros()) as usize,
        }
    }
}

impl TryFrom<Integer> for Bitseq {
    type Error = ConversionError;

    fn try_from(value: Integer) -> Result<Self, Self::Error> {
        if value < Integer::ZERO {
            return Err(ConversionError::new(
                "Cannot convert negative Integer to Bitseq",
            ));
        }
        if value > Integer::BITSEQ_MAX_VALUE {
            return Err(ConversionError::new(
                "Integer too large to convert to Bitseq",
            ));
        }
        match value.inner_value().to_u128() {
            Ok(v) => Ok(Self::from(v)),
            Err(e) => Err(ConversionError::new(format!("{}", e))),
        }
    }
}

impl TryFrom<Decimal> for Bitseq {
    type Error = ConversionError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        match Integer::try_from(value) {
            Err(_) => Err(ConversionError::new(
                "Cannot convert Decimal with fractional part to Bitseq",
            )),
            Ok(int_value) => match Self::try_from(int_value) {
                Err(e) => Err(ConversionError::new(e.msg.replace("Integer", "Decimal"))),
                Ok(bitseq_value) => Ok(bitseq_value),
            },
        }
    }
}

impl Neg for Bitseq {
    type Output = Integer;

    fn neg(self) -> Self::Output {
        -Integer::from(self.value)
    }
}
