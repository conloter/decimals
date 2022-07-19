#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

use std::{convert::TryFrom, fmt};
use uint::construct_uint;

use crate::common::*;
use crate::error::*;
use crate::rate::*;

construct_uint! {
    pub struct U192(3);
}

/// Large decimal values, precise to 18 digits
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Decimal(pub U192);

impl Decimal {
    /// One
    pub fn one() -> Self {
        Self(Self::wad())
    }

    /// Zero
    pub fn zero() -> Self {
        Self(U192::zero())
    }

    // OPTIMIZE: use const slice when fixed in BPF toolchain
    fn wad() -> U192 {
        U192::from(WAD)
    }

    // OPTIMIZE: use const slice when fixed in BPF toolchain
    fn half_wad() -> U192 {
        U192::from(HALF_WAD)
    }

    /// Create scaled decimal from percent value
    pub fn from_percent(percent: u8) -> Self {
        Self(U192::from(percent as u64 * PERCENT_SCALER))
    }

    pub fn from_percent_u64(percent: u64) -> Self {
        Self(U192::from(percent * PERCENT_SCALER))
    }

    /// Create scaled decimal from percent value
    pub fn to_percent(&self) -> Result<u128, DecimalError> {
        u128::try_from(self.0 / PERCENT_SCALER).map_err(|_| DecimalError::MathOverflow)
    }

    /// Create scaled decimal from percent value
    pub fn to_bps(&self) -> Result<u128, DecimalError> {
        u128::try_from(self.0 / BPS_SCALER).map_err(|_| DecimalError::MathOverflow)
    }

    /// Create scaled decimal from bps value
    pub fn from_bps(bps: u16) -> Self {
        Self(U192::from(bps as u64 * BPS_SCALER))
    }

    /// Return raw scaled value if it fits within u128
    #[allow(clippy::wrong_self_convention)]
    pub fn to_scaled_val(&self) -> Result<u128, DecimalError> {
        u128::try_from(self.0).map_err(|_| DecimalError::MathOverflow)
    }

    /// Create decimal from scaled value
    pub fn from_scaled_val(scaled_val: u128) -> Self {
        Self(U192::from(scaled_val))
    }

    /// Round scaled decimal to u64
    pub fn try_round_u64(&self) -> Result<u64, DecimalError> {
        let rounded_val = Self::half_wad()
            .checked_add(self.0)
            .ok_or(DecimalError::MathOverflow)?
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u64::try_from(rounded_val).map_err(|_| DecimalError::MathOverflow)
    }

    /// Round scaled decimal to u64
    pub fn try_round_u128(&self) -> Result<u128, DecimalError> {
        let rounded_val = Self::half_wad()
            .checked_add(self.0)
            .ok_or(DecimalError::MathOverflow)?
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u128::try_from(rounded_val).map_err(|_| DecimalError::MathOverflow)
    }

    /// Ceiling scaled decimal to u64
    pub fn try_ceil_u64(&self) -> Result<u64, DecimalError> {
        let ceil_val = Self::wad()
            .checked_sub(U192::from(1u64))
            .ok_or(DecimalError::MathOverflow)?
            .checked_add(self.0)
            .ok_or(DecimalError::MathOverflow)?
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u64::try_from(ceil_val).map_err(|_| DecimalError::MathOverflow)
    }

    /// Ceiling scaled decimal to u128
    pub fn try_ceil_u128(&self) -> Result<u128, DecimalError> {
        let ceil_val = Self::wad()
            .checked_sub(U192::from(1u64))
            .ok_or(DecimalError::MathOverflow)?
            .checked_add(self.0)
            .ok_or(DecimalError::MathOverflow)?
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u128::try_from(ceil_val).map_err(|_| DecimalError::MathOverflow)
    }

    /// Floor scaled decimal to u64
    pub fn try_floor_u64(&self) -> Result<u64, DecimalError> {
        let ceil_val = self
            .0
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u64::try_from(ceil_val).map_err(|_| DecimalError::MathOverflow)
    }

    pub fn try_floor_u128(&self) -> Result<u128, DecimalError> {
        let ceil_val = self
            .0
            .checked_div(Self::wad())
            .ok_or(DecimalError::MathOverflow)?;
        u128::try_from(ceil_val).map_err(|_| DecimalError::MathOverflow)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut scaled_val = self.0.to_string();
        if scaled_val.len() <= SCALE {
            scaled_val.insert_str(0, &vec!["0"; SCALE - scaled_val.len()].join(""));
            scaled_val.insert_str(0, "0.");
        } else {
            scaled_val.insert(scaled_val.len() - SCALE, '.');
        }
        f.write_str(&scaled_val)
    }
}

impl From<u64> for Decimal {
    fn from(val: u64) -> Self {
        Self(Self::wad() * U192::from(val))
    }
}

impl From<u128> for Decimal {
    fn from(val: u128) -> Self {
        Self(Self::wad() * U192::from(val))
    }
}

impl From<Rate> for Decimal {
    fn from(val: Rate) -> Self {
        Self(U192::from(val.to_scaled_val()))
    }
}

impl TryAdd for Decimal {
    fn try_add(self, rhs: Self) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_add(rhs.0)
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TrySub for Decimal {
    fn try_sub(self, rhs: Self) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_sub(rhs.0)
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TryDiv<u64> for Decimal {
    fn try_div(self, rhs: u64) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_div(U192::from(rhs))
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}
impl TryDiv<u128> for Decimal {
    fn try_div(self, rhs: u128) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_div(U192::from(rhs))
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TryDiv<Rate> for Decimal {
    fn try_div(self, rhs: Rate) -> Result<Self, DecimalError> {
        self.try_div(Self::from(rhs))
    }
}

impl TryDiv<Decimal> for Decimal {
    fn try_div(self, rhs: Self) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_mul(Self::wad())
                .ok_or(DecimalError::MathOverflow)?
                .checked_div(rhs.0)
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TryMul<u64> for Decimal {
    fn try_mul(self, rhs: u64) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_mul(U192::from(rhs))
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TryMul<u128> for Decimal {
    fn try_mul(self, rhs: u128) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_mul(U192::from(rhs))
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

impl TryMul<Rate> for Decimal {
    fn try_mul(self, rhs: Rate) -> Result<Self, DecimalError> {
        self.try_mul(Self::from(rhs))
    }
}

impl TryMul<Decimal> for Decimal {
    fn try_mul(self, rhs: Self) -> Result<Self, DecimalError> {
        Ok(Self(
            self.0
                .checked_mul(rhs.0)
                .ok_or(DecimalError::MathOverflow)?
                .checked_div(Self::wad())
                .ok_or(DecimalError::MathOverflow)?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scaler() {
        assert_eq!(U192::exp10(SCALE), Decimal::wad());
    }

    #[test]
    fn test_decimal_from_to_percent() {
        let pct = 10; // 10%
        let x = Decimal::from_percent(pct);
        let pct_actual = x.to_percent().unwrap();

        assert_eq!(pct as u128, pct_actual);
    }
}
