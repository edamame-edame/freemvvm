use crate::handle::{PendingToken, RuntimeAnchor, TicketReservation};
use crate::native_api::{self, SharedApi};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiError {
    raw_status: u32,
    domain: Option<u32>,
    code: Option<u32>,
    message: Option<String>,
}

impl UiError {
    pub(crate) fn from_status(raw_status: u32) -> Self {
        Self {
            raw_status,
            domain: None,
            code: None,
            message: None,
        }
    }

    pub(crate) fn capture(
        api: &SharedApi,
        operation_status: u32,
        error_token: u64,
        ticket: TicketReservation,
        runtime_token: u64,
        anchor: Option<Arc<RuntimeAnchor>>,
    ) -> Self {
        if error_token == 0 {
            ticket.release();
            return Self::from_status(operation_status);
        }

        let (status_query, error_status) = api.error_status(error_token);
        let raw_status = if status_query == native_api::OK {
            error_status
        } else {
            operation_status
        };
        let (code_query, domain, code) = api.error_code(error_token);
        let (domain, code) = if code_query == native_api::OK {
            (Some(domain), Some(code))
        } else {
            (None, None)
        };
        let message = read_message(api, error_token);

        if api.handle_release(error_token) == native_api::OK {
            ticket.release();
            drop(anchor);
        } else {
            ticket.defer(PendingToken {
                token: error_token,
                runtime_token,
                api: Arc::clone(api),
                anchor,
            });
        }

        Self {
            raw_status,
            domain,
            code,
            message,
        }
    }

    pub fn raw_status(&self) -> u32 {
        self.raw_status
    }

    pub fn domain(&self) -> Option<u32> {
        self.domain
    }

    pub fn code(&self) -> Option<u32> {
        self.code
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Display for UiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(message) = &self.message {
            return formatter.write_str(message);
        }
        write!(
            formatter,
            "FreeMvvm operation failed with status {}",
            self.raw_status
        )
    }
}

impl Error for UiError {}

fn read_message(api: &SharedApi, error: u64) -> Option<String> {
    let (status, length) = api.error_message_utf8(error, None);
    if status != native_api::OK || length > isize::MAX as u64 {
        return None;
    }
    if length == 0 {
        return Some(String::new());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(usize::try_from(length).ok()?)
        .ok()?;
    bytes.resize(usize::try_from(length).ok()?, 0);
    let (status, copied) = api.error_message_utf8(error, Some(&mut bytes));
    if status != native_api::OK || copied != length {
        return None;
    }
    String::from_utf8(bytes).ok()
}
