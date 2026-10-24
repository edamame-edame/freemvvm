use crate::error::UiError;
use crate::handle::{Owned, RuntimeAnchor, TicketReservation};
use crate::native_api;
use crate::runtime::UiContext;
use std::marker::PhantomData;
use std::sync::Arc;

struct PropertyTag;

mod sealed {
    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for i64 {}
    impl Sealed for String {}
}

/// A typed property backed by the shared C ABI.
pub struct Property<T: PropertyValue> {
    owner: Owned<PropertyTag>,
    identity: Arc<()>,
    _value: PhantomData<T>,
}

/// Value types supported by the safe Property API.
///
/// This trait is sealed so safe user code cannot add unchecked FFI conversions.
pub trait PropertyValue: sealed::Sealed + 'static {
    #[doc(hidden)]
    fn create(runtime: u64, initial: Self) -> (u32, u64, u64);
    #[doc(hidden)]
    fn get(property: u64) -> (u32, Self);
    #[doc(hidden)]
    fn set(property: u64, value: Self) -> (u32, u64);
}

impl PropertyValue for bool {
    fn create(runtime: u64, initial: Self) -> (u32, u64, u64) {
        let mut property = 0;
        let mut error = 0;
        // SAFETY: both outputs point to initialized local storage and the bool is encoded as 0/1.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_create_bool(
                runtime,
                u8::from(initial),
                &mut property,
                &mut error,
            )
        };
        (status, property, error)
    }

    fn get(property: u64) -> (u32, Self) {
        let mut value = 0;
        // SAFETY: `value` is writable output storage for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_property_get_bool(property, &mut value) };
        if status != native_api::OK {
            (status, false)
        } else if value > 1 {
            (native_api::INTERNAL_ERROR, false)
        } else {
            (status, value == 1)
        }
    }

    fn set(property: u64, value: Self) -> (u32, u64) {
        let mut error = 0;
        // SAFETY: error is writable output storage and bool is encoded as 0/1.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_set_bool(property, u8::from(value), &mut error)
        };
        (status, error)
    }
}

impl PropertyValue for i64 {
    fn create(runtime: u64, initial: Self) -> (u32, u64, u64) {
        let mut property = 0;
        let mut error = 0;
        // SAFETY: both outputs point to initialized local storage.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_create_i64(runtime, initial, &mut property, &mut error)
        };
        (status, property, error)
    }

    fn get(property: u64) -> (u32, Self) {
        let mut value = 0;
        // SAFETY: `value` is writable output storage for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_property_get_i64(property, &mut value) };
        (status, value)
    }

    fn set(property: u64, value: Self) -> (u32, u64) {
        let mut error = 0;
        // SAFETY: error is writable output storage for this call.
        let status =
            unsafe { freemvvm_sys::ui_abi_1_property_set_i64(property, value, &mut error) };
        (status, error)
    }
}

impl PropertyValue for String {
    fn create(runtime: u64, initial: Self) -> (u32, u64, u64) {
        let mut property = 0;
        let mut error = 0;
        let bytes = initial.as_bytes();
        let Ok(byte_length) = u64::try_from(bytes.len()) else {
            return (native_api::OUT_OF_MEMORY, 0, 0);
        };
        // SAFETY: the byte slice is readable for the declared length and both outputs are writable.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_create_utf8(
                runtime,
                bytes.as_ptr(),
                byte_length,
                &mut property,
                &mut error,
            )
        };
        (status, property, error)
    }

    fn get(property: u64) -> (u32, Self) {
        read_utf8(property)
    }

    fn set(property: u64, value: Self) -> (u32, u64) {
        let mut error = 0;
        let bytes = value.as_bytes();
        let Ok(byte_length) = u64::try_from(bytes.len()) else {
            return (native_api::OUT_OF_MEMORY, 0);
        };
        // SAFETY: the byte slice is readable for the declared length and error is writable.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_set_utf8(
                property,
                bytes.as_ptr(),
                byte_length,
                &mut error,
            )
        };
        (status, error)
    }
}

fn read_utf8(property: u64) -> (u32, String) {
    read_utf8_with(|buffer| {
        let mut length = 0;
        let (pointer, capacity) = match buffer {
            Some(bytes) => {
                let Ok(capacity) = u64::try_from(bytes.len()) else {
                    return (native_api::OUT_OF_MEMORY, 0);
                };
                (bytes.as_mut_ptr(), capacity)
            }
            None => (std::ptr::null_mut(), 0),
        };
        // SAFETY: provided slices supply writable storage; None uses the documented null/zero query.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_property_get_utf8(property, pointer, capacity, &mut length)
        };
        (status, length)
    })
    .map_or_else(
        |status| (status, String::new()),
        |value| (native_api::OK, value),
    )
}

fn read_utf8_with(mut get: impl FnMut(Option<&mut [u8]>) -> (u32, u64)) -> Result<String, u32> {
    loop {
        let (status, required) = get(None);
        if status != native_api::OK {
            return Err(status);
        }
        let length = usize::try_from(required).map_err(|_| native_api::OUT_OF_MEMORY)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| native_api::OUT_OF_MEMORY)?;
        bytes.resize(length, 0);

        let (status, actual) = get(Some(&mut bytes));
        if status == freemvvm_sys::BUFFER_TOO_SMALL {
            let next_length = usize::try_from(actual).map_err(|_| native_api::OUT_OF_MEMORY)?;
            if next_length <= bytes.len() {
                return Err(native_api::INTERNAL_ERROR);
            }
            continue;
        }
        if status != native_api::OK {
            return Err(status);
        }
        let actual = usize::try_from(actual).map_err(|_| native_api::INTERNAL_ERROR)?;
        if actual > bytes.len() {
            return Err(native_api::INTERNAL_ERROR);
        }
        bytes.truncate(actual);
        return String::from_utf8(bytes).map_err(|_| native_api::INTERNAL_ERROR);
    }
}

impl<T: PropertyValue> Property<T> {
    /// Creates a Property on the Runtime represented by `ui`.
    pub fn try_new(ui: &UiContext<'_>, initial: T) -> Result<Self, UiError> {
        ui.check_ui_thread()?;
        let runtime = ui.runtime_token();
        let api = Arc::clone(ui.api());
        let anchor = RuntimeAnchor::try_create(Arc::clone(&api), runtime)?;
        let property_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;

        let (status, token, error) = T::create(runtime, initial);
        if status != native_api::OK || token == 0 || error != 0 {
            let result_status = if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            };
            if token == 0 {
                property_ticket.release();
            } else {
                drop(Owned::<PropertyTag>::from_token(
                    token,
                    Arc::clone(&api),
                    property_ticket,
                    runtime,
                    Some(Arc::clone(&anchor)),
                ));
            }
            return Err(UiError::capture(
                &api,
                result_status,
                error,
                error_ticket,
                runtime,
                Some(anchor),
            ));
        }

        Ok(Self {
            owner: Owned::from_token(token, api, property_ticket, runtime, Some(anchor)),
            identity: ui.identity(),
            _value: PhantomData,
        })
    }

    pub fn get(&self, ui: &UiContext<'_>) -> Result<T, UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let (status, value) = T::get(token);
        if status != native_api::OK {
            return Err(UiError::from_status(status));
        }
        Ok(value)
    }

    pub fn set(&self, ui: &UiContext<'_>, value: T) -> Result<(), UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = T::set(token, value);
        if status != native_api::OK || error != 0 {
            let result_status = if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            };
            return Err(UiError::capture(
                self.owner.api(),
                result_status,
                error,
                error_ticket,
                self.owner.runtime_token(),
                self.owner.anchor(),
            ));
        }
        error_ticket.release();
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }

    fn check_context(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        if Arc::ptr_eq(&self.identity, &ui.identity()) {
            Ok(())
        } else {
            Err(UiError::from_status(native_api::WRONG_RUNTIME))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_get_retries_when_value_grows_between_query_and_copy() {
        let mut call = 0;
        let value = read_utf8_with(|buffer| {
            call += 1;
            match (call, buffer) {
                (1, None) => (native_api::OK, 2),
                (2, Some(bytes)) => {
                    assert_eq!(bytes, [0, 0]);
                    (freemvvm_sys::BUFFER_TOO_SMALL, 5)
                }
                (3, None) => (native_api::OK, 5),
                (4, Some(bytes)) => {
                    bytes.copy_from_slice(&[b'a', 0, 0xE3, 0x81, 0x82]);
                    (native_api::OK, 5)
                }
                _ => panic!("unexpected UTF-8 getter call"),
            }
        })
        .expect("UTF-8 value should be read after retry");

        assert_eq!(value, "a\0あ");
        assert_eq!(call, 4);
    }

    #[test]
    fn utf8_get_rejects_invalid_native_bytes() {
        let error = read_utf8_with(|buffer| match buffer {
            None => (native_api::OK, 1),
            Some(bytes) => {
                bytes[0] = 0xFF;
                (native_api::OK, 1)
            }
        })
        .expect_err("invalid native UTF-8 must not become a String");

        assert_eq!(error, native_api::INTERNAL_ERROR);
    }
}
