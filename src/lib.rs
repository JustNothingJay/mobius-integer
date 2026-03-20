//! # MöbiusInteger
//!
//! A dual-strand integer type that carries both a machine `i64` (fast, wraps on overflow)
//! and an exact `BigInt` (arbitrary precision, never overflows).
//!
//! Arithmetic propagates both strands. Comparison and collapse always use the exact strand.
//! The machine strand may silently overflow — but it's never consulted for truth.
//!
//! ```
//! use mobius_integer::Mi;
//!
//! let max = Mi::new(i64::MAX);
//! let result = max + Mi::new(1);
//!
//! // The machine strand wrapped to i64::MIN
//! assert_eq!(result.machine(), i64::MIN);
//!
//! // The exact strand knows the truth
//! assert!(result.is_corrupted());
//! assert_eq!(result.collapse().to_string(), "9223372036854775808");
//! ```

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// Error returned when a narrowing conversion would overflow.
#[derive(Debug, Clone)]
pub struct OverflowError {
    pub exact_value: BigInt,
    pub target_bits: u32,
}

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "overflow: {} does not fit in i{}",
            self.exact_value, self.target_bits
        )
    }
}

impl std::error::Error for OverflowError {}

/// A dual-strand integer: machine i64 (fast, wraps) + exact BigInt (never overflows).
///
/// The machine strand uses wrapping arithmetic — it behaves exactly like hardware.
/// The exact strand uses arbitrary-precision BigInt — it never overflows.
/// Comparison and collapse always use the exact strand.
#[derive(Clone, Debug)]
pub struct MobiusInteger {
    machine: i64,
    exact: BigInt,
}

/// Convenience alias — `Mi` is to `MobiusInteger` what `M` is to `MobiusNumber`.
pub type Mi = MobiusInteger;

impl MobiusInteger {
    /// Create from an i64. Both strands start in agreement.
    pub fn new(value: i64) -> Self {
        MobiusInteger {
            machine: value,
            exact: BigInt::from(value),
        }
    }

    /// Create from a BigInt. The machine strand saturates to i64::MAX/MIN if it doesn't fit.
    pub fn from_big(value: BigInt) -> Self {
        let machine = value
            .to_i64()
            .unwrap_or(if value > BigInt::from(0i64) {
                i64::MAX
            } else {
                i64::MIN
            });
        MobiusInteger {
            machine,
            exact: value,
        }
    }

    /// The hardware-fast strand. May have wrapped.
    pub fn machine(&self) -> i64 {
        self.machine
    }

    /// The exact strand. Never overflows.
    pub fn exact(&self) -> &BigInt {
        &self.exact
    }

    /// Collapse to the exact value. The Möbius traversal — truth governs.
    pub fn collapse(&self) -> BigInt {
        self.exact.clone()
    }

    /// True if the machine strand has diverged from exact.
    /// When this returns true, the machine strand is lying.
    pub fn is_corrupted(&self) -> bool {
        BigInt::from(self.machine) != self.exact
    }

    /// Narrow to i16, using the exact strand. Returns Err if value doesn't fit.
    /// This is how the Ariane 5 should have worked.
    pub fn narrow_i16(&self) -> Result<i16, OverflowError> {
        self.exact.to_i16().ok_or_else(|| OverflowError {
            exact_value: self.exact.clone(),
            target_bits: 16,
        })
    }

    /// Narrow to i32, using the exact strand. Returns Err if value doesn't fit.
    /// This is how YouTube's Gangnam Style counter should have worked.
    pub fn narrow_i32(&self) -> Result<i32, OverflowError> {
        self.exact.to_i32().ok_or_else(|| OverflowError {
            exact_value: self.exact.clone(),
            target_bits: 32,
        })
    }

    /// Narrow to i64, using the exact strand. Returns Err if value doesn't fit.
    pub fn narrow_i64(&self) -> Result<i64, OverflowError> {
        self.exact.to_i64().ok_or_else(|| OverflowError {
            exact_value: self.exact.clone(),
            target_bits: 64,
        })
    }
}

// ---------------------------------------------------------------------------
// Arithmetic — propagate both strands
// ---------------------------------------------------------------------------

impl Add for MobiusInteger {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        MobiusInteger {
            machine: self.machine.wrapping_add(rhs.machine),
            exact: self.exact + rhs.exact,
        }
    }
}

impl Sub for MobiusInteger {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        MobiusInteger {
            machine: self.machine.wrapping_sub(rhs.machine),
            exact: self.exact - rhs.exact,
        }
    }
}

impl Mul for MobiusInteger {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        MobiusInteger {
            machine: self.machine.wrapping_mul(rhs.machine),
            exact: self.exact * rhs.exact,
        }
    }
}

impl Div for MobiusInteger {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        MobiusInteger {
            machine: if rhs.machine != 0 {
                self.machine.wrapping_div(rhs.machine)
            } else {
                0 // machine strand corrupts; exact strand panics below if zero
            },
            exact: self.exact / rhs.exact, // panics on div-by-zero (correct behavior)
        }
    }
}

impl Rem for MobiusInteger {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        MobiusInteger {
            machine: if rhs.machine != 0 {
                self.machine.wrapping_rem(rhs.machine)
            } else {
                0
            },
            exact: self.exact % rhs.exact,
        }
    }
}

impl Neg for MobiusInteger {
    type Output = Self;
    fn neg(self) -> Self {
        MobiusInteger {
            machine: self.machine.wrapping_neg(),
            exact: -self.exact,
        }
    }
}

// ---------------------------------------------------------------------------
// References — so you don't have to clone everywhere
// ---------------------------------------------------------------------------

impl Add for &MobiusInteger {
    type Output = MobiusInteger;
    fn add(self, rhs: Self) -> MobiusInteger {
        MobiusInteger {
            machine: self.machine.wrapping_add(rhs.machine),
            exact: &self.exact + &rhs.exact,
        }
    }
}

impl Sub for &MobiusInteger {
    type Output = MobiusInteger;
    fn sub(self, rhs: Self) -> MobiusInteger {
        MobiusInteger {
            machine: self.machine.wrapping_sub(rhs.machine),
            exact: &self.exact - &rhs.exact,
        }
    }
}

impl Mul for &MobiusInteger {
    type Output = MobiusInteger;
    fn mul(self, rhs: Self) -> MobiusInteger {
        MobiusInteger {
            machine: self.machine.wrapping_mul(rhs.machine),
            exact: &self.exact * &rhs.exact,
        }
    }
}

// ---------------------------------------------------------------------------
// Comparison — always uses exact strand
// ---------------------------------------------------------------------------

impl PartialEq for MobiusInteger {
    fn eq(&self, other: &Self) -> bool {
        self.exact == other.exact
    }
}

impl Eq for MobiusInteger {}

impl PartialOrd for MobiusInteger {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MobiusInteger {
    fn cmp(&self, other: &Self) -> Ordering {
        self.exact.cmp(&other.exact)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for MobiusInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_corrupted() {
            write!(f, "{} (machine: {}, CORRUPTED)", self.exact, self.machine)
        } else {
            write!(f, "{}", self.exact)
        }
    }
}

// ---------------------------------------------------------------------------
// From conversions
// ---------------------------------------------------------------------------

impl From<i64> for MobiusInteger {
    fn from(v: i64) -> Self {
        Self::new(v)
    }
}

impl From<i32> for MobiusInteger {
    fn from(v: i32) -> Self {
        Self::new(v as i64)
    }
}

impl From<i16> for MobiusInteger {
    fn from(v: i16) -> Self {
        Self::new(v as i64)
    }
}

impl From<u32> for MobiusInteger {
    fn from(v: u32) -> Self {
        Self::new(v as i64)
    }
}

impl From<BigInt> for MobiusInteger {
    fn from(v: BigInt) -> Self {
        Self::from_big(v)
    }
}

#[cfg(test)]
mod tests;
