use crate::error::UiError;

const SIGN_MASK: u64 = 1 << 63;
const SCALE_SHIFT: u32 = 58;
const SCALE_MASK: u64 = 0x1f;
const COEFFICIENT_MASK: u64 = (1 << SCALE_SHIFT) - 1;
const MAX_COEFFICIENT: u64 = COEFFICIENT_MASK;

/// A canonical 64-bit decimal value used by DECIMAL Properties.
///
/// The type provides representation access only; decimal arithmetic and text conversion are not
/// part of the initial Rust wrapper API.
#[derive(Clone, Copy, Debug)]
pub struct Decimal64 {
    bits: u64,
}

impl Decimal64 {
    /// Creates a canonical value from its sign, decimal scale, and unsigned coefficient.
    pub fn try_from_parts(negative: bool, scale: u8, coefficient: u64) -> Result<Self, UiError> {
        if scale > 31 || coefficient > MAX_COEFFICIENT {
            return Err(UiError::from_status(freemvvm_sys::INVALID_ARGUMENT));
        }

        let bits = (if negative { SIGN_MASK } else { 0 })
            | (u64::from(scale) << SCALE_SHIFT)
            | coefficient;
        Ok(Self { bits })
    }

    pub const fn is_negative(self) -> bool {
        self.bits & SIGN_MASK != 0
    }

    pub const fn scale(self) -> u8 {
        ((self.bits >> SCALE_SHIFT) & SCALE_MASK) as u8
    }

    pub const fn coefficient(self) -> u64 {
        self.bits & COEFFICIENT_MASK
    }

    pub(crate) const fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    pub(crate) const fn bits(self) -> u64 {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal64_constructs_and_reads_boundary_parts() {
        let value = Decimal64::try_from_parts(true, 31, MAX_COEFFICIENT)
            .expect("maximum scale and coefficient are representable");

        assert!(value.is_negative());
        assert_eq!(value.scale(), 31);
        assert_eq!(value.coefficient(), MAX_COEFFICIENT);
    }

    #[test]
    fn decimal64_rejects_out_of_range_parts() {
        let invalid_scale =
            Decimal64::try_from_parts(false, 32, 0).expect_err("scale above 31 must be rejected");
        let invalid_coefficient = Decimal64::try_from_parts(false, 0, MAX_COEFFICIENT + 1)
            .expect_err("coefficient above 58 bits must be rejected");

        assert_eq!(invalid_scale.raw_status(), freemvvm_sys::INVALID_ARGUMENT);
        assert_eq!(
            invalid_coefficient.raw_status(),
            freemvvm_sys::INVALID_ARGUMENT
        );
    }
}
