use std::fmt::Display;

use malachite::base::num::conversion::traits::FromStringBase;
use malachite::base::strings::ToBinaryString;
use malachite::{Integer, Natural, Rational};

use crate::core::errors::ConversionError;

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
