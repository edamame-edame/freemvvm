use crate::error::UiError;
use crate::handle::{Owned, RuntimeAnchor, TicketReservation};
use crate::native_api::{self, SharedApi};
use crate::runtime::UiContext;
use crate::subscription::{Subscription, SubscriptionTag};
use crate::ui::{Button, StackPanel, Text, TextBox, Widget, WidgetBorrow, WidgetRef, Window};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::BitOr;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Arc;

struct InputEventTag;

/// Platform-neutral physical key identity. `0` means unknown or unmapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyIdentity(pub u32);

/// Modifier bits reported with a keyboard event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(pub u32);

impl Modifiers {
    pub const SHIFT: u32 = native_api::INPUT_MODIFIER_SHIFT;
    pub const CONTROL: u32 = native_api::INPUT_MODIFIER_CONTROL;
    pub const ALT: u32 = native_api::INPUT_MODIFIER_ALT;
    pub const META: u32 = native_api::INPUT_MODIFIER_META;
}

/// Pointer position in device-independent pixels relative to the Window client area.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointDip {
    pub x: f64,
    pub y: f64,
}

/// Bit mask selecting Input event kinds for a Widget subscription.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InputEventMask(pub u64);

impl InputEventMask {
    pub const KEY_DOWN: Self = Self(1 << (native_api::INPUT_KEY_DOWN - 1));
    pub const KEY_UP: Self = Self(1 << (native_api::INPUT_KEY_UP - 1));
    pub const TEXT_COMMIT: Self = Self(1 << (native_api::INPUT_TEXT_COMMIT - 1));
    pub const PRIMARY_POINTER_DOWN: Self = Self(1 << (native_api::INPUT_POINTER_DOWN - 1));
    pub const PRIMARY_POINTER_UP: Self = Self(1 << (native_api::INPUT_POINTER_UP - 1));
    pub const ALL: Self = Self(native_api::INPUT_MASK_ALL);
}

impl BitOr for InputEventMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Handle storage hidden from documentation but carried by each callback-scoped event variant.
#[doc(hidden)]
pub struct InputEventHandle<'event> {
    event: u64,
    _lifetime: PhantomData<&'event InputEventTag>,
    _not_send: PhantomData<Rc<()>>,
    api: SharedApi,
    runtime: u64,
    anchor: Arc<RuntimeAnchor>,
}

/// An Input event value that is valid only while its callback is running.
///
/// ```compile_fail
/// fn escape(event: freemvvm::InputEvent<'_>) -> freemvvm::InputEvent<'static> { event }
/// ```
pub enum InputEvent<'event> {
    KeyDown {
        key: KeyIdentity,
        modifiers: Modifiers,
        #[doc(hidden)]
        _handle: InputEventHandle<'event>,
    },
    KeyUp {
        key: KeyIdentity,
        modifiers: Modifiers,
        #[doc(hidden)]
        _handle: InputEventHandle<'event>,
    },
    TextCommit {
        text: &'event str,
        #[doc(hidden)]
        _handle: InputEventHandle<'event>,
    },
    PrimaryPointerDown {
        position: PointDip,
        #[doc(hidden)]
        _handle: InputEventHandle<'event>,
    },
    PrimaryPointerUp {
        position: PointDip,
        #[doc(hidden)]
        _handle: InputEventHandle<'event>,
    },
}

struct InputCallbackState<F> {
    callback: F,
    runtime: u64,
    anchor: Arc<RuntimeAnchor>,
    api: SharedApi,
}

impl InputEvent<'_> {
    /// Marks this event handled so later handlers and its default action do not run.
    pub fn set_handled(&mut self) -> Result<(), UiError> {
        let handle = match self {
            Self::KeyDown { _handle, .. }
            | Self::KeyUp { _handle, .. }
            | Self::TextCommit { _handle, .. }
            | Self::PrimaryPointerDown { _handle, .. }
            | Self::PrimaryPointerUp { _handle, .. } => _handle,
        };
        handle.set_handled()
    }
}

impl InputEventHandle<'_> {
    fn set_handled(&self) -> Result<(), UiError> {
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = self.api.input_event_set_handled(self.event, true);
        finish_status(
            &self.api,
            status,
            error,
            error_ticket,
            self.runtime,
            Some(Arc::clone(&self.anchor)),
        )
    }
}

macro_rules! impl_input_subscriber {
    ($type:ty) => {
        impl $type {
            pub fn subscribe_input<F>(
                &self,
                ui: &UiContext<'_>,
                kinds: InputEventMask,
                callback: F,
            ) -> Result<Subscription, UiError>
            where
                F: for<'event> FnMut(&mut InputEvent<'event>) + 'static,
            {
                subscribe(self, ui, kinds, callback)
            }
        }
    };
}

impl_input_subscriber!(Window);
impl_input_subscriber!(StackPanel);
impl_input_subscriber!(TextBox);
impl_input_subscriber!(Text);
impl_input_subscriber!(Button);
impl_input_subscriber!(Widget);

fn subscribe<F>(
    widget: &impl WidgetRef,
    ui: &UiContext<'_>,
    kinds: InputEventMask,
    callback: F,
) -> Result<Subscription, UiError>
where
    F: for<'event> FnMut(&mut InputEvent<'event>) + 'static,
{
    let widget = widget.__borrow_widget()?;
    check_context(widget.identity(), ui)?;
    ui.check_ui_thread()?;
    let anchor = widget.anchor();
    let subscription_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let error_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let state = Box::new(InputCallbackState {
        callback,
        runtime: ui.runtime_token(),
        anchor: Arc::clone(&anchor),
        api: Arc::clone(ui.api()),
    });
    let userdata = Box::into_raw(state).cast::<c_void>();
    let (status, token, error) = ui.api().input_subscribe(
        widget.token(),
        kinds.0,
        Some(input_trampoline::<F>),
        userdata,
        Some(input_destroy::<F>),
    );
    if status == native_api::OK && token != 0 && error == 0 {
        return Ok(Subscription::from_owner(
            Owned::from_token(
                token,
                Arc::clone(ui.api()),
                subscription_ticket,
                ui.runtime_token(),
                Some(anchor),
            ),
            ui.identity(),
        ));
    }

    if status != native_api::OK && token == 0 {
        // SAFETY: failed registration leaves userdata owned by the wrapper.
        drop(unsafe { Box::from_raw(userdata.cast::<InputCallbackState<F>>()) });
    } else if token != 0 {
        // SAFETY: a returned token may represent a live registration; cancel before release.
        let cancel_status =
            unsafe { freemvvm_sys::ui_abi_1_subscription_cancel(token, std::ptr::null_mut()) };
        if cancel_status == native_api::OK && status != native_api::OK {
            // The failed registration retained userdata; cancellation has no root to destroy.
            // SAFETY: failed registration left the Box wrapper-owned.
            drop(unsafe { Box::from_raw(userdata.cast::<InputCallbackState<F>>()) });
        }
        drop(Owned::<SubscriptionTag>::from_token(
            token,
            Arc::clone(ui.api()),
            subscription_ticket,
            ui.runtime_token(),
            Some(Arc::clone(&anchor)),
        ));
    } else {
        // Success without a token is inconsistent; keep userdata alive in case Core owns it.
        subscription_ticket.release();
    }
    let result_status = if status == native_api::OK {
        native_api::INTERNAL_ERROR
    } else {
        status
    };
    Err(UiError::capture(
        ui.api(),
        result_status,
        error,
        error_ticket,
        ui.runtime_token(),
        Some(anchor),
    ))
}

fn check_context(identity: &Arc<()>, ui: &UiContext<'_>) -> Result<(), UiError> {
    if Arc::ptr_eq(identity, &ui.identity()) {
        Ok(())
    } else {
        Err(UiError::from_status(native_api::WRONG_RUNTIME))
    }
}

unsafe extern "C" fn input_trampoline<F>(
    userdata: *mut c_void,
    _current_widget: u64,
    _original_target: u64,
    event: u64,
) where
    F: for<'event> FnMut(&mut InputEvent<'event>) + 'static,
{
    if userdata.is_null() || event == 0 {
        return;
    }
    // SAFETY: Core invokes UI callbacks serially and retains userdata until its destroy callback.
    let state = unsafe { &mut *userdata.cast::<InputCallbackState<F>>() };
    let handle = InputEventHandle {
        event,
        _lifetime: PhantomData,
        _not_send: PhantomData,
        api: Arc::clone(&state.api),
        runtime: state.runtime,
        anchor: Arc::clone(&state.anchor),
    };
    let (status, kind) = state.api.input_event_kind(event);
    if status != native_api::OK {
        let _ = handle.set_handled();
        return;
    }

    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), ()> {
        match kind {
            native_api::INPUT_KEY_DOWN => {
                let (status, key, modifiers) = state.api.input_event_key(event);
                if status != native_api::OK {
                    return Err(());
                }
                let mut view = InputEvent::KeyDown {
                    key: KeyIdentity(key),
                    modifiers: Modifiers(modifiers),
                    _handle: clone_handle(&handle),
                };
                (state.callback)(&mut view);
                Ok(())
            }
            native_api::INPUT_KEY_UP => {
                let (status, key, modifiers) = state.api.input_event_key(event);
                if status != native_api::OK {
                    return Err(());
                }
                let mut view = InputEvent::KeyUp {
                    key: KeyIdentity(key),
                    modifiers: Modifiers(modifiers),
                    _handle: clone_handle(&handle),
                };
                (state.callback)(&mut view);
                Ok(())
            }
            native_api::INPUT_TEXT_COMMIT => {
                let text = read_event_text(&*state.api, event).ok_or(())?;
                let mut view = InputEvent::TextCommit {
                    text: &text,
                    _handle: clone_handle(&handle),
                };
                (state.callback)(&mut view);
                Ok(())
            }
            native_api::INPUT_POINTER_DOWN => {
                let (status, x, y, button) = state.api.input_event_pointer(event);
                if status != native_api::OK || button != native_api::INPUT_POINTER_PRIMARY {
                    return Err(());
                }
                let mut view = InputEvent::PrimaryPointerDown {
                    position: PointDip { x, y },
                    _handle: clone_handle(&handle),
                };
                (state.callback)(&mut view);
                Ok(())
            }
            native_api::INPUT_POINTER_UP => {
                let (status, x, y, button) = state.api.input_event_pointer(event);
                if status != native_api::OK || button != native_api::INPUT_POINTER_PRIMARY {
                    return Err(());
                }
                let mut view = InputEvent::PrimaryPointerUp {
                    position: PointDip { x, y },
                    _handle: clone_handle(&handle),
                };
                (state.callback)(&mut view);
                Ok(())
            }
            _ => Err(()),
        }
    }));
    // Only user-code panics are specified to suppress the default action. A
    // malformed/unavailable payload is an ABI failure, not a handled event.
    if outcome.is_err() {
        let _ = handle.set_handled();
    }
}

fn clone_handle<'event>(handle: &InputEventHandle<'event>) -> InputEventHandle<'event> {
    InputEventHandle {
        event: handle.event,
        _lifetime: PhantomData,
        _not_send: PhantomData,
        api: Arc::clone(&handle.api),
        runtime: handle.runtime,
        anchor: Arc::clone(&handle.anchor),
    }
}

fn read_event_text(api: &dyn crate::native_api::NativeApi, event: u64) -> Option<String> {
    let (status, length) = api.input_event_text(event, None);
    if status != native_api::BUFFER_TOO_SMALL && !(status == native_api::OK && length == 0) {
        return None;
    }
    let length = usize::try_from(length).ok()?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).ok()?;
    bytes.resize(length, 0);
    let (status, actual) = api.input_event_text(event, Some(&mut bytes));
    if status != native_api::OK || actual != length as u64 {
        return None;
    }
    String::from_utf8(bytes).ok()
}

unsafe extern "C" fn input_destroy<F>(userdata: *mut c_void)
where
    F: for<'event> FnMut(&mut InputEvent<'event>) + 'static,
{
    if userdata.is_null() {
        return;
    }
    // SAFETY: the C ABI transfers the registered Box exactly once to this destroy callback.
    let state = unsafe { Box::from_raw(userdata.cast::<InputCallbackState<F>>()) };
    let _ = catch_unwind(AssertUnwindSafe(|| drop(state)));
}

fn finish_status(
    api: &SharedApi,
    status: u32,
    error: u64,
    error_ticket: TicketReservation,
    runtime: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) -> Result<(), UiError> {
    if status == native_api::OK && error == 0 {
        error_ticket.release();
        Ok(())
    } else {
        Err(UiError::capture(
            api,
            if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            },
            error,
            error_ticket,
            runtime,
            anchor,
        ))
    }
}

#[allow(dead_code)]
fn _widget_borrow_is_used(_: &WidgetBorrow<'_>) {}
