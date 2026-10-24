use crate::error::UiError;
use crate::handle::{Owned, RuntimeAnchor, TicketReservation};
use crate::native_api::{self, SharedApi};
use crate::property::{Property, PropertyTag, PropertyValue};
use crate::runtime::UiContext;
use crate::subscription::Subscription;
use std::marker::PhantomData;
use std::sync::Arc;

struct BindingTag;

const BINDING_ONCE: u32 = 1;
const BINDING_ONE_WAY: u32 = 2;
const BINDING_TWO_WAY: u32 = 3;
const DIRECTION_SOURCE_TO_TARGET: u32 = 1;
const DIRECTION_TARGET_TO_SOURCE: u32 = 2;

/// Controls how values move between two Properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingMode {
    /// Copies the source value once and leaves no persistent Binding.
    OneTime,
    /// Propagates source changes to the target.
    OneWay,
    /// Propagates direct changes in either direction.
    TwoWay,
}

impl BindingMode {
    fn raw(self) -> u32 {
        match self {
            Self::OneTime => BINDING_ONCE,
            Self::OneWay => BINDING_ONE_WAY,
            Self::TwoWay => BINDING_TWO_WAY,
        }
    }
}

/// Direction in which a Binding propagation was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingDirection {
    SourceToTarget,
    TargetToSource,
}

impl BindingDirection {
    fn from_raw(value: u32) -> Option<Self> {
        match value {
            DIRECTION_SOURCE_TO_TARGET => Some(Self::SourceToTarget),
            DIRECTION_TARGET_TO_SOURCE => Some(Self::TargetToSource),
            _ => None,
        }
    }
}

/// An immutable snapshot of a rejected Binding propagation.
pub struct BindingError<T: PropertyValue> {
    error: UiError,
    rejected_property: Property<T>,
    direction: BindingDirection,
}

impl<T: PropertyValue> BindingError<T> {
    /// Returns the immutable machine-readable error captured at this snapshot.
    pub fn error(&self) -> &UiError {
        &self.error
    }

    /// Returns an independently owned view of the Property that rejected propagation.
    pub fn rejected_property(&self) -> &Property<T> {
        &self.rejected_property
    }

    /// Returns the propagation direction that was rejected.
    pub fn direction(&self) -> BindingDirection {
        self.direction
    }
}

/// A move-only owner of a persistent Property Binding.
///
/// Releasing this handle does not cancel the Runtime-owned connection. Call
/// [`Binding::cancel`] to stop future propagation.
///
/// ```compile_fail
/// use freemvvm::Binding;
/// fn require_send<T: Send>() {}
/// require_send::<Binding<i64>>();
/// ```
pub struct Binding<T: PropertyValue> {
    owner: Owned<BindingTag>,
    identity: Arc<()>,
    _value: PhantomData<T>,
}

impl<T: PropertyValue> Binding<T> {
    pub(crate) fn create(
        source: &Property<T>,
        target: &Property<T>,
        ui: &UiContext<'_>,
        mode: BindingMode,
    ) -> Result<Option<Self>, UiError> {
        source.check_context(ui)?;
        target.check_context(ui)?;
        ui.check_ui_thread()?;
        let source_token = source.token_for_call()?;
        let target_token = target.token_for_call()?;
        let runtime = source.runtime_token();
        let api = Arc::clone(source.api());
        let anchor = source
            .anchor()
            .ok_or_else(|| UiError::from_status(native_api::CLOSED))?;
        let binding_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;

        let mut binding = 0;
        let mut error = 0;
        // SAFETY: both Property tokens are borrowed for this call and outputs are writable.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_binding_create(
                source_token,
                target_token,
                mode.raw(),
                &mut binding,
                &mut error,
            )
        };

        if status == native_api::OK && error == 0 {
            match (mode, binding) {
                (BindingMode::OneTime, 0) => {
                    binding_ticket.release();
                    error_ticket.release();
                    return Ok(None);
                }
                (BindingMode::OneWay | BindingMode::TwoWay, token) if token != 0 => {
                    error_ticket.release();
                    return Ok(Some(Self {
                        owner: Owned::from_token(token, api, binding_ticket, runtime, Some(anchor)),
                        identity: source.identity(),
                        _value: PhantomData,
                    }));
                }
                _ => {}
            }
        }

        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        dispose_unexpected_binding(binding, binding_ticket, &api, runtime, Some(anchor));
        Err(UiError::capture(
            &api,
            result_status,
            error,
            error_ticket,
            runtime,
            source.anchor(),
        ))
    }

    /// Explicitly cancels this connection. A failed cancellation leaves the owner usable for retry.
    pub fn cancel(&mut self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut error = 0;
        // SAFETY: `error` is writable storage and the token is borrowed for this call.
        let status = unsafe { freemvvm_sys::ui_abi_1_binding_cancel(token, &mut error) };
        if status == native_api::OK && error == 0 {
            error_ticket.release();
            return Ok(());
        }
        Err(UiError::capture(
            self.owner.api(),
            if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            },
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        ))
    }

    /// Returns the current delayed propagation error, if any, as an immutable owned snapshot.
    pub fn error_snapshot(&self, ui: &UiContext<'_>) -> Result<Option<BindingError<T>>, UiError> {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let api = Arc::clone(self.owner.api());
        let runtime = self.owner.runtime_token();
        let anchor = self.owner.anchor();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let property_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let mut error = 0;
        let mut rejected_property = 0;
        let mut direction = 0;
        // SAFETY: all outputs are writable and `token` is borrowed for this call.
        let status = unsafe {
            freemvvm_sys::ui_abi_1_binding_error_snapshot(
                token,
                &mut error,
                &mut rejected_property,
                &mut direction,
            )
        };

        if status != native_api::OK {
            dispose_snapshot_tokens(
                error,
                rejected_property,
                error_ticket,
                property_ticket,
                &api,
                runtime,
                anchor,
            );
            return Err(UiError::from_status(status));
        }
        if error == 0 && rejected_property == 0 && direction == 0 {
            error_ticket.release();
            property_ticket.release();
            return Ok(None);
        }
        let Some(direction) = BindingDirection::from_raw(direction) else {
            dispose_snapshot_tokens(
                error,
                rejected_property,
                error_ticket,
                property_ticket,
                &api,
                runtime,
                anchor,
            );
            return Err(UiError::from_status(native_api::INTERNAL_ERROR));
        };
        if error == 0 || rejected_property == 0 {
            dispose_snapshot_tokens(
                error,
                rejected_property,
                error_ticket,
                property_ticket,
                &api,
                runtime,
                anchor,
            );
            return Err(UiError::from_status(native_api::INTERNAL_ERROR));
        }

        let snapshot_error = UiError::capture(
            &api,
            native_api::OK,
            error,
            error_ticket,
            runtime,
            anchor.clone(),
        );
        let property_owner =
            Owned::from_token(rejected_property, api, property_ticket, runtime, anchor);
        Ok(Some(BindingError {
            error: snapshot_error,
            rejected_property: Property::from_owned(property_owner, Arc::clone(&self.identity)),
            direction,
        }))
    }

    /// Subscribes to changes in this Binding's delayed error state.
    pub fn subscribe_error<F>(
        &self,
        ui: &UiContext<'_>,
        callback: F,
    ) -> Result<Subscription, UiError>
    where
        F: FnMut() + 'static,
    {
        self.check_context(ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let anchor = self
            .owner
            .anchor()
            .ok_or_else(|| UiError::from_status(native_api::CLOSED))?;
        Subscription::register_binding_error(
            token,
            Arc::clone(self.owner.api()),
            self.owner.runtime_token(),
            anchor,
            Arc::clone(&self.identity),
            callback,
        )
    }

    /// Releases only the public Binding handle; it does not cancel the connection.
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

fn dispose_unexpected_binding(
    token: u64,
    ticket: TicketReservation,
    api: &SharedApi,
    runtime: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) {
    if token == 0 {
        ticket.release();
        return;
    }
    // SAFETY: cancellation accepts an optional error output and the token is live from the call.
    let _ = unsafe { freemvvm_sys::ui_abi_1_binding_cancel(token, std::ptr::null_mut()) };
    drop(Owned::<BindingTag>::from_token(
        token,
        Arc::clone(api),
        ticket,
        runtime,
        anchor,
    ));
}

fn dispose_snapshot_tokens(
    error: u64,
    rejected_property: u64,
    error_ticket: TicketReservation,
    property_ticket: TicketReservation,
    api: &SharedApi,
    runtime: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) {
    if error == 0 {
        error_ticket.release();
    } else {
        let _ = UiError::capture(
            api,
            native_api::INTERNAL_ERROR,
            error,
            error_ticket,
            runtime,
            anchor.clone(),
        );
    }
    if rejected_property == 0 {
        property_ticket.release();
    } else {
        drop(Owned::<PropertyTag>::from_token(
            rejected_property,
            Arc::clone(api),
            property_ticket,
            runtime,
            anchor,
        ));
    }
}
