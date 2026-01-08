use std::fmt::Display;

use crate::core::errors::ConversionError;
use crate::core::values::{Decimal, Integer};

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

impl TryInto<Integer> for Bitseq {
    type Error = ConversionError;

    fn try_into(self) -> Result<Integer, Self::Error> {
        match Integer::try_from(self.value) {
            Ok(v) => Ok(v),
            Err(_) => Err(ConversionError::new(
                "Bitseq too wide to convert to Integer",
            )),
        }
    }
}

impl TryInto<Decimal> for Bitseq {
    type Error = ConversionError;

    fn try_into(self) -> Result<Decimal, Self::Error> {
        let converted = self.value as Decimal;
        if converted as BitseqT == self.value {
            Ok(converted)
        } else {
            Err(ConversionError::new(
                "Bitseq too wide to convert to Decimal",
            ))
        }
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
        if value < 0 {
            Err(ConversionError::new(
                "Cannot convert negative Integer to Bitseq",
            ))
        } else {
            match BitseqT::try_from(value) {
                Ok(v) => Ok(Self::from(v)),
                Err(_) => Err(ConversionError::new(
                    "Non-natural Integer cannot be converted to Bitseq",
                )),
            }
            // let unsigned = match u64::try_from(value) {
            //     Ok(v) => v,
            //     Err(_) => {
            //         return Err(ConversionError::new(
            //             "Cannot convert non-natural Integer to Bitseq",
            //         ));
            //     }
            // };
            // // There's no direct (Try)From/Into Trait for Naturals and primitive unsigned types,
            // // so this isn't ideal but it works
            // let bitstr = format!("{:b}", unsigned);
            // match Self::from_str(&bitstr) {
            //     Some(v) => Ok(v),
            //     None => Err(ConversionError::new(
            //         "Cannot convert Integer that's too wide for u128 to Bitseq",
            //     )),
            // }
        }
    }
}

impl TryFrom<Decimal> for Bitseq {
    type Error = ConversionError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        let converted = value as BitseqT;
        if converted as Decimal == value {
            Ok(Self::from(converted))
        } else {
            Err(ConversionError::new(
                "Cannot convert Decimal to Bitseq that doesn't losslessly convert to Integer",
            ))
        }
        // match UInt::try_from(value) {
        //     Ok(v) => match Self::try_from(v) {
        //         Ok(b) => Ok(b),
        //         Err(e) => Err(ConversionError::new(e.msg.replace("Integer", "Rational"))),
        //     },
        //     Err(_) => Err(ConversionError::new(
        //         "Cannot convert Rational to Bitseq that doesn't losslessly convert to Integer",
        //     )),
        // }
    }
}
