//! <https://github.com/EOSIO/eosio.cdt/blob/4985359a30da1f883418b7133593f835927b8046/libraries/eosiolib/core/eosio/asset.hpp#L18-L369>
use crate::{
    symbol_from_chars, CheckedAdd, CheckedDiv, CheckedMul, CheckedRem,
    CheckedSub, NumBytes, ParseSymbolError, Read, Symbol, Write,
};
use serde::{Deserialize, Serialize, Serializer};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub,
    SubAssign,
};
use std::str::FromStr;

/// Stores information for owner of asset
#[derive(
    Debug, PartialEq, Clone, Copy, Default, Read, Write, NumBytes, Deserialize,
)]
#[eosio_core_root_path = "crate"]
pub struct Asset {
    /// The amount of the asset
    pub amount: i64,
    /// The symbol name of the asset
    pub symbol: Symbol,
}

impl Asset {
    /// Check if the asset is valid. A valid asset has its amount <= max_amount
    /// and its symbol name valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.symbol.is_valid()
    }
}

impl fmt::Display for Asset {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let precision = self.symbol.precision();
        let symbol_code = self.symbol.code();
        if precision == 0 {
            write!(f, "{} {}", self.amount, symbol_code)
        } else {
            let precision = usize::from(precision);
            let formatted = format!(
                "{:0precision$}",
                self.amount,
                precision = precision + if self.amount < 0 { 2 } else { 1 }
            );
            let index = formatted.len() - precision;
            let whole = formatted.get(..index).unwrap_or_else(|| "");
            let fraction = formatted.get(index..).unwrap_or_else(|| "");
            write!(f, "{}.{} {}", whole, fraction, symbol_code)
        }
    }
}

/// TODO docs
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseAssetError {
    /// TODO docs
    BadChar(char),
    /// TODO docs
    BadPrecision,
    /// TODO docs
    SymbolIsEmpty,
    /// TODO docs
    SymbolTooLong,
}

impl From<ParseSymbolError> for ParseAssetError {
    #[inline]
    fn from(value: ParseSymbolError) -> Self {
        match value {
            ParseSymbolError::IsEmpty => ParseAssetError::SymbolIsEmpty,
            ParseSymbolError::TooLong => ParseAssetError::SymbolTooLong,
            ParseSymbolError::BadChar(c) => ParseAssetError::BadChar(c),
            ParseSymbolError::BadPrecision => ParseAssetError::BadPrecision,
        }
    }
}

impl FromStr for Asset {
    type Err = ParseAssetError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: refactor ugly code below
        let s = s.trim();
        let mut chars = s.chars();
        let mut index = 0_usize;
        let mut precision: Option<u64> = None;
        // Find numbers
        loop {
            let c = match chars.next() {
                Some(c) => c,
                None => return Err(ParseAssetError::SymbolIsEmpty),
            };
            if index == 0 {
                if '0' <= c && c <= '9' || c == '-' || c == '+' {
                    index += 1;
                    continue;
                } else {
                    return Err(ParseAssetError::BadChar(c));
                }
            }

            index += 1;
            if '0' <= c && c <= '9' {
                if let Some(p) = precision {
                    precision = Some(p + 1);
                }
            } else if c == ' ' {
                match precision {
                    Some(0) => return Err(ParseAssetError::BadPrecision),
                    _ => break,
                }
            } else if c == '.' {
                precision = Some(0);
            } else {
                return Err(ParseAssetError::BadChar(c));
            }
        }

        let precision = u8::try_from(precision.unwrap_or_default())
            .map_err(|_| ParseAssetError::BadPrecision)?;
        let symbol = symbol_from_chars(precision, chars)
            .map_err(ParseAssetError::from)?;

        let end_index = if precision == 0 {
            index
        } else {
            index - (precision as usize) - 1
        } as usize;
        // TODO: clean up code/unwraps below
        let amount = s.get(0..end_index - 1).unwrap();
        if precision == 0 {
            let amount =
                amount.parse::<i64>().expect("error parsing asset amount");
            Ok(Self {
                amount,
                symbol: symbol.into(),
            })
        } else {
            let fraction = s.get(end_index..(index - 1) as usize).unwrap();
            let amount = format!("{}{}", amount, fraction)
                .parse::<i64>()
                .expect("error parsing asset amount");
            Ok(Self {
                amount,
                symbol: symbol.into(),
            })
        }
    }
}

impl TryFrom<&str> for Asset {
    type Error = ParseAssetError;
    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for Asset {
    type Error = ParseAssetError;
    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Serialize for Asset {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(s.as_str())
    }
}

/// TODO docs
#[derive(Debug, Clone, Copy)]
pub enum AssetOpError {
    /// TODO docs
    Overflow,
    /// TODO docs
    DifferentSymbols,
}

impl fmt::Display for AssetOpError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            AssetOpError::Overflow => "integer overflow",
            AssetOpError::DifferentSymbols => "assets have different symbols",
        };
        write!(f, "{}", msg)
    }
}

impl Error for AssetOpError {}

/// TODO docs
#[derive(Debug, Clone, Copy)]
pub enum AssetDivOpError {
    /// TODO docs
    Overflow,
    /// TODO docs
    DifferentSymbols,
    /// TODO docs
    DivideByZero,
}

impl fmt::Display for AssetDivOpError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            AssetDivOpError::Overflow => "integer overflow",
            AssetDivOpError::DifferentSymbols => {
                "assets have different symbols"
            }
            AssetDivOpError::DivideByZero => "divide by zero",
        };
        write!(f, "{}", msg)
    }
}

impl Error for AssetDivOpError {}

macro_rules! impl_op {
    ($($checked_trait:ident, $checked_error:ident, $checked_fn:ident, $op_trait:ident, $op_fn:ident, $assign_trait:ident, $assign_fn:ident)*) => ($(
        impl $checked_trait<i64> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: i64) -> Self::Output {
                self.amount.$checked_fn(other).map(|amount| Self {
                    amount,
                    symbol: self.symbol,
                })
            }
        }

        impl $checked_trait<u64> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: u64) -> Self::Output {
                u64::try_from(other).ok().and_then(|other| self.$checked_fn(other))
            }
        }

        impl $checked_trait<u128> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: u128) -> Self::Output {
                u64::try_from(other).ok().and_then(|other| self.$checked_fn(other))
            }
        }

        impl $checked_trait<i128> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: i128) -> Self::Output {
                u64::try_from(other).ok().and_then(|other| self.$checked_fn(other))
            }
        }

        impl $checked_trait<isize> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: isize) -> Self::Output {
                u64::try_from(other).ok().and_then(|other| self.$checked_fn(other))
            }
        }

        impl $checked_trait<usize> for Asset {
            type Output = Option<Self>;
            #[inline]
            fn $checked_fn(self, other: usize) -> Self::Output {
                u64::try_from(other).ok().and_then(|other| self.$checked_fn(other))
            }
        }

        impl $checked_trait for Asset {
            type Output = Result<Self, $checked_error>;
            #[inline]
            fn $checked_fn(self, other: Self) -> Self::Output {
                if self.symbol == other.symbol {
                    self.$checked_fn(other.amount)
                        .ok_or_else(|| $checked_error::Overflow)
                } else {
                    Err($checked_error::DifferentSymbols)
                }
            }
        }

        impl $op_trait for Asset {
            type Output = Self;
            #[inline]
            fn $op_fn(self, rhs: Self) -> Self::Output {
                match self.$checked_fn(rhs) {
                    Ok(output) => output,
                    Err(error) => panic!(
                        "can't perform operation on asset, {}", error
                    ),
                }
            }
        }

        impl $op_trait<i64> for Asset {
            type Output = Self;
            #[inline]
            fn $op_fn(self, rhs: i64) -> Self::Output {
                match self.$checked_fn(rhs) {
                    Some(output) => output,
                    None => panic!(
                        "can't perform operation on asset, result would overflow"
                    ),
                }
            }
        }

        impl $op_trait<Asset> for i64 {
            type Output = Asset;
            #[inline]
            fn $op_fn(self, rhs: Asset) -> Self::Output {
                rhs.$op_fn(self)
            }
        }

        impl $assign_trait for Asset {
            #[inline]
            fn $assign_fn(&mut self, rhs: Self) {
                *self = self.$op_fn(rhs);
            }
        }

        impl $assign_trait<i64> for Asset {
            #[inline]
            fn $assign_fn(&mut self, rhs: i64) {
                *self = self.$op_fn(rhs);
            }
        }
    )*)
}

impl_op! {
    CheckedAdd, AssetOpError, checked_add, Add, add, AddAssign, add_assign
    CheckedSub, AssetOpError, checked_sub, Sub, sub, SubAssign, sub_assign
    CheckedMul, AssetOpError, checked_mul, Mul, mul, MulAssign, mul_assign
    CheckedDiv, AssetDivOpError, checked_div, Div, div, DivAssign, div_assign
    CheckedRem, AssetOpError, checked_rem, Rem, rem, RemAssign, rem_assign
}

#[cfg(test)]
mod tests {
    use super::*;
    use eosio_core_macros::{n, s};

    macro_rules! test_to_string {
        ($($name:ident, $amount:expr, $symbol:expr, $expected:expr)*) => ($(
            #[test]
            fn $name() {
                let asset = Asset {
                    amount: $amount,
                    symbol: $symbol.into(),
                };
                assert_eq!(asset.to_string(), $expected);
            }
        )*)
    }

    test_to_string! {
        to_string, 1_0000, s!(4, EOS), "1.0000 EOS"
        to_string_signed, -1_0000, s!(4, EOS), "-1.0000 EOS"
        to_string_fraction, 1_0001, s!(4, EOS), "1.0001 EOS"
        to_string_zero_precision, 10_001, s!(0, EOS), "10001 EOS"
        to_string_zero_precision_unsigned, -10_001, s!(0, EOS), "-10001 EOS"
        to_string_max_number, std::i64::MAX, s!(4, EOS), "922337203685477.5807 EOS"
        to_string_min_number, std::i64::MIN, s!(4, EOS), "-922337203685477.5808 EOS"
        to_string_very_small_number, 1, s!(255, TST), "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001 TST"
        to_string_very_small_number_neg, -1, s!(255, TST), "-0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001 TST"
    }

    macro_rules! test_from_str_ok {
        ($($name:ident, $input:expr, $expected_amount:expr, $expected_symbol:expr)*) => ($(
            #[test]
            fn $name() {
                let ok = Ok(Asset {
                    amount: $expected_amount,
                    symbol: $expected_symbol.into(),
                });
                assert_eq!(Asset::from_str($input), ok);
                assert_eq!(Asset::try_from($input), ok);
            }
        )*)
    }

    test_from_str_ok! {
        from_str_ok_basic, "1.0000 EOS", 1_0000, s!(4, EOS)
        from_str_ok_zero_precision, "1 TST", 1, s!(0, TST)
        from_str_ok_long, "1234567890.12345 TMP", 1234567890_12345, s!(5, TMP)
        from_str_ok_signed_neg, "-1.0000 TLOS", -1_0000, s!(4, TLOS)
        from_str_ok_signed_zero_precision, "-1 SYS", -1, s!(0, SYS)
        from_str_ok_signed_long, "-1234567890.12345 TGFT", -1234567890_12345, s!(5, TGFT)
        from_str_ok_pos_sign, "+1 TST", 1, s!(0, TST)
        from_str_ok_fraction, "0.0001 EOS", 1, s!(4, EOS)
        from_str_ok_zero, "0.0000 EOS", 0, s!(4, EOS)
        from_str_whitespace_around, "            1.0000 EOS   ", 1_0000, s!(4, EOS)
        from_str_zero_padded, "0001.0000 EOS", 1_0000, s!(4, EOS)
        from_str_very_small_num, "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001 TST", 1, s!(255, TST)
        from_str_very_small_num_neg, "-0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001 TST", -1, s!(255, TST)
    }

    macro_rules! test_from_str_err {
        ($($name:ident, $input:expr, $expected:expr)*) => ($(
            #[test]
            fn $name() {
                let err = Err($expected);
                assert_eq!(Asset::from_str($input), err);
                assert_eq!(Asset::try_from($input), err);
            }
        )*)
    }

    test_from_str_err! {
        from_str_bad_char1, "tst", ParseAssetError::BadChar('t')
        from_str_multi_spaces, "1.0000  EOS", ParseAssetError::BadChar(' ')
        from_str_lowercase_symbol, "1.0000 eos", ParseAssetError::BadChar('e')
        from_str_no_space, "1EOS", ParseAssetError::BadChar('E')
        from_str_no_symbol1, "1.2345 ", ParseAssetError::SymbolIsEmpty
        from_str_no_symbol2, "1", ParseAssetError::SymbolIsEmpty
        from_str_bad_char2, "1.a", ParseAssetError::BadChar('a')
        from_str_bad_precision, "1. EOS", ParseAssetError::BadPrecision
    }

    #[test]
    fn test_ops() {
        let mut asset = Asset {
            amount: 10_0000,
            symbol: s!(4, EOS).into(),
        };
        asset += 1;
        assert_eq!(asset.amount, 10_0001);
        asset -= 1;
        assert_eq!(asset.amount, 10_0000);
        asset /= 10;
        assert_eq!(asset.amount, 1_0000);
    }
}
