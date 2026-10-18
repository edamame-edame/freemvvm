mod entry;
mod handle;
pub(super) mod registry;
mod token;

pub(super) use handle::HandleToken;
use std::num::NonZeroU64;

/// Issues a value from the same process-wide namespace used by public handles.
pub(super) fn issue_runtime_identity() -> Option<NonZeroU64> {
    token::PROCESS_TOKEN_ISSUER.issue()
}

pub(super) fn issue_handle_token() -> Option<HandleToken> {
    token::PROCESS_TOKEN_ISSUER
        .issue()
        .map(HandleToken::from_issued)
}
