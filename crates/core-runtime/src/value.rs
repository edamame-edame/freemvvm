//! Owned Core values used at the ABI boundary.

/// An owned, validated UTF-8 byte string.
///
/// The value preserves its exact UTF-8 bytes, including embedded NUL bytes. Construction copies
/// the input before returning and performs no normalization or replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnedUtf8(String);

/// The currently implemented Core tagged Property values.
///
/// Property commit/notification behavior is added in a separate unit.
#[derive(Clone, Debug)]
pub(crate) enum TypedValue {
    Bool(bool),
    I64(i64),
    Utf8(OwnedUtf8),
    F64(f64),
    Decimal(DecimalValue),
}

/// An exact base-10 value with its original sign, scale, and coefficient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DecimalValue {
    negative: bool,
    scale: u8,
    coefficient: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValueKind {
    Bool,
    I64,
    Utf8,
    F64,
    Decimal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValueCommit {
    Unchanged,
    Changed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValueKindMismatch {
    pub(crate) expected: ValueKind,
    pub(crate) actual: ValueKind,
}

/// A value slot whose kind is fixed by its initial value.
pub(crate) struct ValueStorage {
    value: TypedValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalValueError {
    ScaleOutOfRange,
    CoefficientOutOfRange,
}

const DECIMAL_MAX_SCALE: u8 = 31;
const DECIMAL_MAX_COEFFICIENT: u64 = (1_u64 << 58) - 1;

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
    fn kind(&self) -> ValueKind {
        match self {
            Self::Bool(_) => ValueKind::Bool,
            Self::I64(_) => ValueKind::I64,
            Self::Utf8(_) => ValueKind::Utf8,
            Self::F64(_) => ValueKind::F64,
            Self::Decimal(_) => ValueKind::Decimal,
        }
    }

    /// Compares values only when they have the same kind.
    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::I64(left), Self::I64(right)) => left == right,
            (Self::Utf8(left), Self::Utf8(right)) => left == right,
            (Self::F64(left), Self::F64(right)) => f64_equivalent(*left, *right),
            (Self::Decimal(left), Self::Decimal(right)) => left.equivalent(*right),
            _ => false,
        }
    }
}

impl ValueStorage {
    pub(crate) fn new(initial: TypedValue) -> Self {
        Self { value: initial }
    }

    pub(crate) fn read(&self) -> &TypedValue {
        &self.value
    }

    /// Applies a candidate after the caller has completed all registered value validation.
    ///
    /// An equivalent candidate preserves the existing representation and reports no change.
    pub(crate) fn apply_validated(
        &mut self,
        candidate: TypedValue,
    ) -> Result<ValueCommit, ValueKindMismatch> {
        let expected = self.value.kind();
        let actual = candidate.kind();
        if expected != actual {
            return Err(ValueKindMismatch { expected, actual });
        }
        if self.value.equivalent(&candidate) {
            return Ok(ValueCommit::Unchanged);
        }
        self.value = candidate;
        Ok(ValueCommit::Changed)
    }
}

impl DecimalValue {
    /// Decodes the canonical numeric `uint64_t` layout without host byte-order conversion.
    pub(crate) fn from_canonical_bits(bits: u64) -> Self {
        Self {
            negative: bits >> 63 != 0,
            scale: ((bits >> 58) & 0x1f) as u8,
            coefficient: bits & DECIMAL_MAX_COEFFICIENT,
        }
    }

    /// Encodes sign, scale, and coefficient into the canonical numeric `uint64_t` layout.
    pub(crate) fn to_canonical_bits(self) -> u64 {
        (u64::from(self.negative) << 63) | (u64::from(self.scale) << 58) | self.coefficient
    }

    pub(crate) fn new(
        negative: bool,
        scale: u8,
        coefficient: u64,
    ) -> Result<Self, DecimalValueError> {
        if scale > DECIMAL_MAX_SCALE {
            return Err(DecimalValueError::ScaleOutOfRange);
        }
        if coefficient > DECIMAL_MAX_COEFFICIENT {
            return Err(DecimalValueError::CoefficientOutOfRange);
        }
        Ok(Self {
            negative,
            scale,
            coefficient,
        })
    }

    /// Tests numeric equality without changing the retained representation.
    pub(crate) fn equivalent(self, other: Self) -> bool {
        self.comparison_parts() == other.comparison_parts()
    }

    fn comparison_parts(self) -> (bool, u8, u64) {
        if self.coefficient == 0 {
            return (false, 0, 0);
        }

        let mut scale = self.scale;
        let mut coefficient = self.coefficient;
        while scale > 0 && coefficient % 10 == 0 {
            coefficient /= 10;
            scale -= 1;
        }
        (self.negative, scale, coefficient)
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
    use super::{
        DECIMAL_MAX_COEFFICIENT, DECIMAL_MAX_SCALE, DecimalValue, DecimalValueError, OwnedUtf8,
        OwnedUtf8Error, TypedValue, Utf8BufferTooSmall, Utf8CopyOutcome, ValueCommit, ValueKind,
        ValueKindMismatch, ValueStorage,
    };

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

    #[test]
    fn decimal_value_accepts_inclusive_scale_and_coefficient_boundaries() {
        assert!(DecimalValue::new(false, 0, 0).is_ok());
        assert!(DecimalValue::new(true, 31, (1_u64 << 58) - 1).is_ok());
    }

    #[test]
    fn decimal_value_rejects_scale_and_coefficient_outside_approved_range() {
        assert_eq!(
            DecimalValue::new(false, 32, 0),
            Err(DecimalValueError::ScaleOutOfRange)
        );
        assert_eq!(
            DecimalValue::new(false, 0, 1_u64 << 58),
            Err(DecimalValueError::CoefficientOutOfRange)
        );
    }

    #[test]
    fn decimal_value_compares_numeric_equivalence_without_rewriting_storage() {
        let one_point_zero = DecimalValue::new(false, 1, 10).expect("valid decimal");
        let one_point_zero_zero = DecimalValue::new(false, 2, 100).expect("valid decimal");
        let one_point_one = DecimalValue::new(false, 1, 11).expect("valid decimal");
        let negative_one = DecimalValue::new(true, 0, 1).expect("valid decimal");
        let positive_zero = DecimalValue::new(false, 31, 0).expect("valid decimal");
        let negative_zero = DecimalValue::new(true, 0, 0).expect("valid decimal");

        assert!(one_point_zero.equivalent(one_point_zero_zero));
        assert!(!one_point_zero.equivalent(one_point_one));
        assert!(!one_point_zero.equivalent(negative_one));
        assert!(positive_zero.equivalent(negative_zero));

        let stored = one_point_zero;
        assert!(stored.equivalent(one_point_zero_zero));
        assert_eq!(stored, one_point_zero);
        assert_eq!(stored.scale, 1);
        assert_eq!(stored.coefficient, 10);
    }

    #[test]
    fn decimal_value_typed_comparison_uses_kind_and_numeric_equivalence() {
        let first = DecimalValue::new(false, 1, 10).expect("valid decimal");
        let equivalent = DecimalValue::new(false, 2, 100).expect("valid decimal");
        assert!(TypedValue::Decimal(first).equivalent(&TypedValue::Decimal(equivalent)));
        assert!(!TypedValue::Decimal(first).equivalent(&TypedValue::I64(1)));
    }

    #[test]
    fn decimal_canonical_encoding_round_trips_sign_scale_and_coefficient_boundaries() {
        let coefficients = [0, 1, 123, DECIMAL_MAX_COEFFICIENT];

        for negative in [false, true] {
            for scale in 0..=DECIMAL_MAX_SCALE {
                for coefficient in coefficients {
                    let value = DecimalValue::new(negative, scale, coefficient)
                        .expect("boundary decimal is valid");
                    let bits = value.to_canonical_bits();
                    let decoded = DecimalValue::from_canonical_bits(bits);

                    assert_eq!(decoded, value);
                    assert_eq!(decoded.to_canonical_bits(), bits);
                }
            }
        }
    }

    #[test]
    fn decimal_canonical_encoding_matches_approved_one_point_two_three_example() {
        let encoded = (2_u64 << 58) | 123;
        let value = DecimalValue::from_canonical_bits(encoded);

        assert!(!value.negative);
        assert_eq!(value.scale, 2);
        assert_eq!(value.coefficient, 123);
        assert_eq!(value.to_canonical_bits(), encoded);
    }

    #[test]
    fn value_storage_preserves_equivalent_representations() {
        let original_nan = f64::from_bits(0x7ff0_0000_0000_0001);
        let other_nan = f64::from_bits(0xfff8_0000_0000_0001);
        let original_decimal = DecimalValue::new(false, 1, 10).expect("valid decimal");
        let equivalent_decimal = DecimalValue::new(false, 2, 100).expect("valid decimal");
        let mut float_storage = ValueStorage::new(TypedValue::F64(original_nan));
        let mut decimal_storage = ValueStorage::new(TypedValue::Decimal(original_decimal));

        assert_eq!(
            float_storage.apply_validated(TypedValue::F64(other_nan)),
            Ok(ValueCommit::Unchanged)
        );
        assert_eq!(
            decimal_storage.apply_validated(TypedValue::Decimal(equivalent_decimal)),
            Ok(ValueCommit::Unchanged)
        );
        assert!(
            matches!(float_storage.read(), TypedValue::F64(value) if value.to_bits() == original_nan.to_bits())
        );
        assert!(
            matches!(decimal_storage.read(), TypedValue::Decimal(value) if *value == original_decimal)
        );
    }

    #[test]
    fn value_storage_commits_changed_candidates_of_every_kind() {
        let cases = [
            (TypedValue::Bool(false), TypedValue::Bool(true)),
            (TypedValue::I64(1), TypedValue::I64(2)),
            (
                TypedValue::Utf8(OwnedUtf8::copy_from_bytes(b"old").expect("valid UTF-8")),
                TypedValue::Utf8(OwnedUtf8::copy_from_bytes(b"new").expect("valid UTF-8")),
            ),
            (TypedValue::F64(1.0), TypedValue::F64(2.0)),
            (
                TypedValue::Decimal(DecimalValue::new(false, 0, 1).expect("valid decimal")),
                TypedValue::Decimal(DecimalValue::new(false, 0, 2).expect("valid decimal")),
            ),
        ];

        for (initial, candidate) in cases {
            let mut storage = ValueStorage::new(initial.clone());
            assert_eq!(storage.apply_validated(initial), Ok(ValueCommit::Unchanged));
            assert_eq!(
                storage.apply_validated(candidate.clone()),
                Ok(ValueCommit::Changed)
            );
            assert!(storage.read().equivalent(&candidate));
        }
    }

    #[test]
    fn value_storage_rejects_other_kinds_without_mutating_current_value() {
        let mut storage = ValueStorage::new(TypedValue::I64(7));

        assert_eq!(
            storage.apply_validated(TypedValue::Bool(true)),
            Err(ValueKindMismatch {
                expected: ValueKind::I64,
                actual: ValueKind::Bool,
            })
        );
        assert!(matches!(storage.read(), TypedValue::I64(value) if *value == 7));
    }
}
