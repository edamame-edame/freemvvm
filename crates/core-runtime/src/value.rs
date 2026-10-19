//! Owned Core values used at the ABI boundary.

/// An owned, validated UTF-8 byte string.
///
/// The value preserves its exact UTF-8 bytes, including embedded NUL bytes. Construction copies
/// the input before returning and performs no normalization or replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnedUtf8(String);

/// The currently implemented subset of the Core's tagged Property values.
///
/// Decimal and Property commit/notification behavior are added in separate units.
#[derive(Clone, Debug)]
pub(crate) enum TypedValue {
    Bool(bool),
    I64(i64),
    Utf8(OwnedUtf8),
    F64(f64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnedUtf8Error {
    InvalidUtf8,
    OutOfMemory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Utf8CopyOutcome {
    LengthOnly { required: usize },
    Written { length: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Utf8BufferTooSmall {
    pub(crate) required: usize,
}

pub(crate) fn f64_equivalent(left: f64, right: f64) -> bool {
    (left.is_nan() && right.is_nan()) || left == right
}

impl TypedValue {
    /// Compares values only when they have the same kind.
    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::I64(left), Self::I64(right)) => left == right,
            (Self::Utf8(left), Self::Utf8(right)) => left == right,
            (Self::F64(left), Self::F64(right)) => f64_equivalent(*left, *right),
            _ => false,
        }
    }
}

impl OwnedUtf8 {
    pub(crate) fn copy_from_bytes(bytes: &[u8]) -> Result<Self, OwnedUtf8Error> {
        let text = std::str::from_utf8(bytes).map_err(|_| OwnedUtf8Error::InvalidUtf8)?;
        let mut owned = String::new();
        owned
            .try_reserve_exact(text.len())
            .map_err(|_| OwnedUtf8Error::OutOfMemory)?;
        owned.push_str(text);
        Ok(Self(owned))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub(crate) fn copy_to_buffer(
        &self,
        buffer: Option<&mut [u8]>,
    ) -> Result<Utf8CopyOutcome, Utf8BufferTooSmall> {
        let bytes = self.as_bytes();
        let Some(buffer) = buffer else {
            return Ok(Utf8CopyOutcome::LengthOnly {
                required: bytes.len(),
            });
        };
        if buffer.len() < bytes.len() {
            return Err(Utf8BufferTooSmall {
                required: bytes.len(),
            });
        }
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(Utf8CopyOutcome::Written {
            length: bytes.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{OwnedUtf8, OwnedUtf8Error, TypedValue, Utf8BufferTooSmall, Utf8CopyOutcome};

    #[test]
    fn owned_utf8_copies_exact_validated_bytes() {
        let mut source = "日本語\0text".as_bytes().to_vec();
        let expected = source.clone();

        let value = OwnedUtf8::copy_from_bytes(&source).expect("valid UTF-8 is accepted");
        source.fill(b'x');

        assert_eq!(value.as_bytes(), expected);
        assert_eq!(value.as_str().as_bytes(), expected);
        assert!(value.as_str().contains('\0'));
    }

    #[test]
    fn owned_utf8_accepts_empty_input() {
        let value = OwnedUtf8::copy_from_bytes(&[]).expect("empty UTF-8 is valid");

        assert!(value.as_str().is_empty());
        assert!(value.as_bytes().is_empty());
    }

    #[test]
    fn owned_utf8_rejects_invalid_bytes_without_a_value() {
        assert_eq!(
            OwnedUtf8::copy_from_bytes(&[0xf0, 0x28, 0x8c, 0x28]),
            Err(OwnedUtf8Error::InvalidUtf8)
        );
    }

    #[test]
    fn owned_utf8_buffer_query_reports_byte_length_without_a_buffer() {
        let value = OwnedUtf8::copy_from_bytes("日本\0x".as_bytes()).expect("valid UTF-8");

        assert_eq!(
            value.copy_to_buffer(None),
            Ok(Utf8CopyOutcome::LengthOnly { required: 8 })
        );
    }

    #[test]
    fn owned_utf8_buffer_short_capacity_preserves_every_byte() {
        let value = OwnedUtf8::copy_from_bytes("日本\0x".as_bytes()).expect("valid UTF-8");
        let mut buffer = [0xa5; 7];

        assert_eq!(
            value.copy_to_buffer(Some(&mut buffer)),
            Err(Utf8BufferTooSmall { required: 8 })
        );
        assert_eq!(buffer, [0xa5; 7]);
    }

    #[test]
    fn owned_utf8_buffer_exact_capacity_copies_bytes_without_terminator() {
        let expected = "日本\0x".as_bytes();
        let value = OwnedUtf8::copy_from_bytes(expected).expect("valid UTF-8");
        let mut buffer = [0xa5; 8];

        assert_eq!(
            value.copy_to_buffer(Some(&mut buffer)),
            Ok(Utf8CopyOutcome::Written { length: 8 })
        );
        assert_eq!(buffer, expected);
    }

    #[test]
    fn f64_equivalent_handles_nan_zero_infinity_and_finite_values() {
        let first_nan = f64::from_bits(0x7ff0_0000_0000_0001);
        let second_nan = f64::from_bits(0xfff8_1234_5678_9abc);

        assert!(super::f64_equivalent(first_nan, second_nan));
        assert!(super::f64_equivalent(first_nan, first_nan));
        assert!(!super::f64_equivalent(first_nan, f64::INFINITY));
        assert!(super::f64_equivalent(0.0, -0.0));
        assert!(super::f64_equivalent(f64::INFINITY, f64::INFINITY));
        assert!(super::f64_equivalent(f64::NEG_INFINITY, f64::NEG_INFINITY));
        assert!(!super::f64_equivalent(f64::INFINITY, f64::NEG_INFINITY));
        assert!(super::f64_equivalent(1.25, 1.25));
        assert!(!super::f64_equivalent(
            1.25,
            f64::from_bits(1.25_f64.to_bits() + 1)
        ));
    }

    #[test]
    fn typed_value_compares_bool_i64_and_utf8_within_kind() {
        assert!(TypedValue::Bool(true).equivalent(&TypedValue::Bool(true)));
        assert!(!TypedValue::Bool(true).equivalent(&TypedValue::Bool(false)));
        assert!(TypedValue::I64(-42).equivalent(&TypedValue::I64(-42)));
        assert!(!TypedValue::I64(-42).equivalent(&TypedValue::I64(42)));

        let text = OwnedUtf8::copy_from_bytes("日本\0語".as_bytes()).expect("valid UTF-8");
        let same_text = OwnedUtf8::copy_from_bytes("日本\0語".as_bytes()).expect("valid UTF-8");
        let other_text = OwnedUtf8::copy_from_bytes("日本語".as_bytes()).expect("valid UTF-8");
        assert!(TypedValue::Utf8(text.clone()).equivalent(&TypedValue::Utf8(same_text)));
        assert!(!TypedValue::Utf8(text).equivalent(&TypedValue::Utf8(other_text)));
    }

    #[test]
    fn typed_value_uses_f64_equivalence_rules() {
        let first_nan = f64::from_bits(0x7ff0_0000_0000_0001);
        let second_nan = f64::from_bits(0xfff8_0000_0000_0001);

        assert!(TypedValue::F64(first_nan).equivalent(&TypedValue::F64(second_nan)));
        assert!(TypedValue::F64(0.0).equivalent(&TypedValue::F64(-0.0)));
        assert!(!TypedValue::F64(f64::INFINITY).equivalent(&TypedValue::F64(f64::NEG_INFINITY)));
        assert!(!TypedValue::F64(1.0).equivalent(&TypedValue::F64(1.0 + f64::EPSILON)));
    }

    #[test]
    fn typed_value_rejects_cross_kind_equivalence() {
        assert!(!TypedValue::Bool(true).equivalent(&TypedValue::I64(1)));
        assert!(!TypedValue::I64(1).equivalent(&TypedValue::F64(1.0)));
    }
}
