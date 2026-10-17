use std::num::NonZeroU64;

/// Core's opaque representation of a public `ui_handle` token.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(dead_code)] // Used by the ABI and registry integration slices.
pub(crate) struct HandleToken(NonZeroU64);

#[allow(dead_code)]
impl HandleToken {
    pub(crate) const fn from_issued(token: NonZeroU64) -> Self {
        Self(token)
    }

    pub(crate) const fn from_abi(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(token) => Some(Self(token)),
            None => None,
        }
    }

    pub(crate) const fn into_abi(self) -> u64 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::HandleToken;
    use std::mem::{align_of, size_of};
    use std::num::NonZeroU64;

    #[test]
    fn handle_token_rejects_zero() {
        assert_eq!(HandleToken::from_abi(0), None);
    }

    #[test]
    fn handle_token_round_trips_nonzero_boundary_values() {
        for value in [1, u64::MAX] {
            let token = HandleToken::from_abi(value).expect("nonzero handle token");
            assert_eq!(token.into_abi(), value);
            assert_eq!(
                token,
                HandleToken::from_issued(NonZeroU64::new(value).expect("nonzero value"))
            );
        }

        assert_eq!(size_of::<HandleToken>(), size_of::<u64>());
        assert_eq!(align_of::<HandleToken>(), align_of::<u64>());
    }
}
