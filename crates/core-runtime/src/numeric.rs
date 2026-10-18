//! Deterministic conversions between IEEE-754 binary32 and binary64 values.

const _: () = {
    assert!(core::mem::size_of::<f32>() == 4);
    assert!(f32::RADIX == 2 && f32::MANTISSA_DIGITS == 24);
    assert!(f32::MIN_EXP == -125 && f32::MAX_EXP == 128);
    assert!(core::mem::size_of::<f64>() == 8);
    assert!(f64::RADIX == 2 && f64::MANTISSA_DIGITS == 53);
    assert!(f64::MIN_EXP == -1021 && f64::MAX_EXP == 1024);
};

/// The precision policy used when narrowing a binary64 value to binary32.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NarrowMode {
    /// Reject finite values that do not round-trip exactly.
    Exact,
    /// Round to the nearest binary32 value, resolving ties toward an even significand.
    NearestEven,
}

/// A finite binary64 value is not representable under the selected narrowing policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Unrepresentable;

/// Expand a binary32 value to binary64 without losing finite precision.
///
/// NaN payloads are intentionally not preserved; NaNs become quiet NaNs with the same sign.
pub fn expand_f32_to_f64(value: f32) -> f64 {
    let bits = value.to_bits();
    let exponent = bits & 0x7f80_0000;
    let fraction = bits & 0x007f_ffff;

    if exponent == 0x7f80_0000 && fraction != 0 {
        let sign = u64::from(bits >> 31) << 63;
        return f64::from_bits(sign | 0x7ff8_0000_0000_0000);
    }

    value as f64
}

/// Narrow a binary64 value to binary32 using the selected precision policy.
///
/// Finite values outside the binary32 range and non-exact finite values in `Exact` mode are
/// rejected. NaNs are normalized to quiet NaNs, and signed zero and infinity are preserved.
pub fn narrow_f64_to_f32(value: f64, mode: NarrowMode) -> Result<f32, Unrepresentable> {
    let bits = value.to_bits();
    let sign = ((bits >> 63) as u32) << 31;
    let exponent = ((bits >> 52) & 0x7ff) as u32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;

    if exponent == 0x7ff {
        if fraction == 0 {
            return Ok(f32::from_bits(sign | 0x7f80_0000));
        }
        return Ok(f32::from_bits(sign | 0x7fc0_0000));
    }

    if exponent == 0 && fraction == 0 {
        return Ok(f32::from_bits(sign));
    }

    if bits & 0x7fff_ffff_ffff_ffff > (f32::MAX as f64).to_bits() {
        return Err(Unrepresentable);
    }

    let significand;
    let exponent2;
    if exponent == 0 {
        significand = fraction;
        exponent2 = -1074;
    } else {
        significand = (1_u64 << 52) | fraction;
        exponent2 = exponent as i32 - 1023 - 52;
    }

    let highest_bit = 63 - significand.leading_zeros() as i32;
    let unbiased_exponent = highest_bit + exponent2;
    let magnitude = if unbiased_exponent >= -126 {
        let rounded = round_shift_right(significand, (highest_bit - 23) as u32);
        let mut exponent32 = unbiased_exponent;
        let mut significand32 = rounded;
        if significand32 == (1 << 24) {
            significand32 >>= 1;
            exponent32 += 1;
        }
        (((exponent32 + 127) as u32) << 23) | (significand32 as u32 & 0x007f_ffff)
    } else {
        let shift = -(exponent2 + 149);
        let rounded = round_shift_right(significand, shift as u32);
        if rounded >= (1 << 23) {
            1 << 23
        } else {
            rounded as u32
        }
    };

    let result = f32::from_bits(sign | magnitude);
    if mode == NarrowMode::Exact && (result as f64).to_bits() != bits {
        return Err(Unrepresentable);
    }

    Ok(result)
}

fn round_shift_right(value: u64, shift: u32) -> u64 {
    if shift == 0 {
        return value;
    }
    if shift >= 64 {
        return 0;
    }

    let quotient = value >> shift;
    let remainder = value & ((1_u64 << shift) - 1);
    let halfway = 1_u64 << (shift - 1);
    if remainder > halfway || (remainder == halfway && quotient & 1 != 0) {
        quotient + 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::{NarrowMode, Unrepresentable, expand_f32_to_f64, narrow_f64_to_f32};

    #[test]
    fn expands_finite_values_exactly_and_quiets_nan() {
        for bits in [
            0x0000_0001,
            0x007f_ffff,
            0x0080_0000,
            0x3f80_0000,
            0x7f7f_ffff,
            0x8000_0000,
            0xff80_0000,
        ] {
            let value = f32::from_bits(bits);
            assert_eq!(expand_f32_to_f64(value), value as f64);
        }

        let quiet_nan = expand_f32_to_f64(f32::from_bits(0xff80_0001));
        assert_eq!(quiet_nan.to_bits(), 0xfff8_0000_0000_0000);
    }

    #[test]
    fn narrows_exact_values_and_preserves_zero_and_infinity_signs() {
        for bits in [
            0x0000_0000,
            0x8000_0000,
            0x0000_0001,
            0x3fc0_0000,
            0x7f7f_ffff,
            0x7f80_0000,
            0xff80_0000,
        ] {
            let value = f32::from_bits(bits) as f64;
            let actual = narrow_f64_to_f32(value, NarrowMode::Exact).expect("exact value");
            assert_eq!(actual.to_bits(), bits);
        }
    }

    #[test]
    fn rejects_inexact_exact_mode_and_finite_overflow() {
        assert_eq!(
            narrow_f64_to_f32(0.1, NarrowMode::Exact),
            Err(Unrepresentable)
        );
        assert_eq!(
            narrow_f64_to_f32(
                f32::MAX as f64 + f64::EPSILON * (f32::MAX as f64),
                NarrowMode::NearestEven
            ),
            Err(Unrepresentable)
        );
        assert_eq!(
            narrow_f64_to_f32(f64::MAX, NarrowMode::NearestEven),
            Err(Unrepresentable)
        );
    }

    #[test]
    fn nearest_even_rounds_midpoints_toward_even_significand() {
        let even_lower = 1.0_f64 + 2_f64.powi(-24);
        let odd_lower = f32::from_bits(0x3f80_0001) as f64 + 2_f64.powi(-24);

        assert_eq!(
            narrow_f64_to_f32(even_lower, NarrowMode::NearestEven)
                .expect("nearest representable")
                .to_bits(),
            0x3f80_0000
        );
        assert_eq!(
            narrow_f64_to_f32(-even_lower, NarrowMode::NearestEven)
                .expect("negative nearest representable")
                .to_bits(),
            0xbf80_0000
        );
        assert_eq!(
            narrow_f64_to_f32(odd_lower, NarrowMode::NearestEven)
                .expect("nearest representable")
                .to_bits(),
            0x3f80_0002
        );
    }

    #[test]
    fn nearest_even_handles_subnormal_midpoints_and_normal_boundary() {
        let min_subnormal = f32::from_bits(1) as f64;
        assert_eq!(
            narrow_f64_to_f32(min_subnormal / 2.0, NarrowMode::NearestEven)
                .expect("ties to even zero")
                .to_bits(),
            0
        );
        assert_eq!(
            narrow_f64_to_f32(min_subnormal * 1.5, NarrowMode::NearestEven)
                .expect("ties to even two")
                .to_bits(),
            2
        );

        let just_below_normal = f32::from_bits(0x007f_ffff) as f64;
        assert_eq!(
            narrow_f64_to_f32(just_below_normal, NarrowMode::NearestEven)
                .expect("exact subnormal")
                .to_bits(),
            0x007f_ffff
        );
    }

    #[test]
    fn nan_payload_is_not_preserved() {
        let value = f64::from_bits(0xfff0_0000_0000_0001);
        assert_eq!(
            narrow_f64_to_f32(value, NarrowMode::Exact)
                .expect("NaN is accepted")
                .to_bits(),
            0xffc0_0000
        );
    }

    #[test]
    fn nearest_even_matches_ieee_cast_for_deterministic_finite_samples_in_range() {
        let max_f32_as_f64 = (f32::MAX as f64).to_bits();
        let mut state = 0x8f3d_9a72_c615_04eb_u64;

        for _ in 0..50_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let value = f64::from_bits(state);
            let magnitude = state & 0x7fff_ffff_ffff_ffff;
            if value.is_finite() && magnitude <= max_f32_as_f64 {
                let actual = narrow_f64_to_f32(value, NarrowMode::NearestEven)
                    .expect("finite in-range value");
                assert_eq!(actual.to_bits(), (value as f32).to_bits(), "{value:?}");
            }
        }
    }
}
