use crate::command::{AsyncCommand, CommandTag};
use crate::error::UiError;
use crate::handle::{Borrowed, Owned, RuntimeAnchor, TicketReservation};
use crate::native_api::{
    self, SharedApi, UiAction, UiButtonAction, UiControlKind, UiPropertyKind, UiStackPanelAction,
    UiTreeAction,
};
use crate::property::{Property, PropertyTag};
use crate::runtime::UiContext;
use std::sync::Arc;

struct WindowTag;
struct StackPanelTag;
struct TextBoxTag;
struct TextTag;
struct ButtonTag;
struct WidgetTag;

mod sealed {
    pub trait Sealed {}
}

/// A safe source for a short-lived borrowed Widget handle.
///
/// Implementations are limited to FreeMvvm UI owners. The borrowed ABI token is not exposed.
pub trait WidgetRef: sealed::Sealed {
    #[doc(hidden)]
    fn __borrow_widget(&self) -> Result<WidgetBorrow<'_>, UiError>;

    fn is_enabled_property(&self, ui: &UiContext<'_>) -> Result<Property<bool>, UiError> {
        widget_property(self, ui, UiPropertyKind::IsEnabled)
    }

    fn accessible_name_property(&self, ui: &UiContext<'_>) -> Result<Property<String>, UiError> {
        widget_property(self, ui, UiPropertyKind::AccessibleName)
    }

    fn request_focus(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        let widget = self.__borrow_widget()?;
        check_widget_context(&widget, ui)?;
        ui.check_ui_thread()?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = ui.api().ui_action(UiAction::RequestFocus, widget.token());
        finish_status_call(
            ui.api(),
            status,
            error,
            error_ticket,
            ui.runtime_token(),
            None,
        )
    }
}

/// Opaque borrowed Widget handle used by the sealed [`WidgetRef`] implementation.
#[doc(hidden)]
pub struct WidgetBorrow<'a> {
    token: Borrowed<'a, WidgetTag>,
    identity: &'a Arc<()>,
}

impl WidgetBorrow<'_> {
    fn token(&self) -> u64 {
        self.token.token()
    }
}

pub struct Window {
    owner: Owned<WindowTag>,
    identity: Arc<()>,
}

pub struct StackPanel {
    owner: Owned<StackPanelTag>,
    identity: Arc<()>,
}

pub struct TextBox {
    owner: Owned<TextBoxTag>,
    identity: Arc<()>,
}

pub struct Text {
    owner: Owned<TextTag>,
    identity: Arc<()>,
}

pub struct Button {
    owner: Owned<ButtonTag>,
    identity: Arc<()>,
}

/// An owned generic child handle returned by a Tree query.
pub struct Widget {
    owner: Owned<WidgetTag>,
    identity: Arc<()>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Vertical,
    Horizontal,
    Unknown(u32),
}

impl Orientation {
    fn from_raw(value: u32) -> Self {
        match value {
            freemvvm_sys::UI_ORIENTATION_VERTICAL => Self::Vertical,
            freemvvm_sys::UI_ORIENTATION_HORIZONTAL => Self::Horizontal,
            value => Self::Unknown(value),
        }
    }

    fn to_raw(self) -> Result<u32, UiError> {
        match self {
            Self::Vertical => Ok(freemvvm_sys::UI_ORIENTATION_VERTICAL),
            Self::Horizontal => Ok(freemvvm_sys::UI_ORIENTATION_HORIZONTAL),
            Self::Unknown(_) => Err(UiError::from_status(native_api::INVALID_ARGUMENT)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildAlign {
    Start,
    Center,
    End,
    Stretch,
    Unknown(u32),
}

impl ChildAlign {
    fn to_raw(self) -> Result<u32, UiError> {
        match self {
            Self::Start => Ok(freemvvm_sys::UI_ALIGN_START),
            Self::Center => Ok(freemvvm_sys::UI_ALIGN_CENTER),
            Self::End => Ok(freemvvm_sys::UI_ALIGN_END),
            Self::Stretch => Ok(freemvvm_sys::UI_ALIGN_STRETCH),
            Self::Unknown(_) => Err(UiError::from_status(native_api::INVALID_ARGUMENT)),
        }
    }
}

fn create_owner<T>(
    ui: &UiContext<'_>,
    kind: UiControlKind,
    initial: &[u8],
) -> Result<Owned<T>, UiError> {
    ui.check_ui_thread()?;
    let runtime = ui.runtime_token();
    let api = Arc::clone(ui.api());
    let anchor = RuntimeAnchor::try_create(Arc::clone(&api), runtime)?;
    let owner_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let error_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let (status, token, error) = api.ui_create(kind, runtime, initial);
    if status != native_api::OK || token == 0 || error != 0 {
        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        if token == 0 {
            owner_ticket.release();
        } else {
            drop(Owned::<T>::from_token(
                token,
                Arc::clone(&api),
                owner_ticket,
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
    error_ticket.release();
    Ok(Owned::from_token(
        token,
        api,
        owner_ticket,
        runtime,
        Some(anchor),
    ))
}

fn finish_status_call(
    api: &SharedApi,
    status: u32,
    error: u64,
    error_ticket: TicketReservation,
    runtime: u64,
    anchor: Option<Arc<RuntimeAnchor>>,
) -> Result<(), UiError> {
    if status == native_api::OK && error == 0 {
        error_ticket.release();
        return Ok(());
    }
    let result_status = if status == native_api::OK {
        native_api::INTERNAL_ERROR
    } else {
        status
    };
    Err(UiError::capture(
        api,
        result_status,
        error,
        error_ticket,
        runtime,
        anchor,
    ))
}

fn check_widget_context(widget: &WidgetBorrow<'_>, ui: &UiContext<'_>) -> Result<(), UiError> {
    if Arc::ptr_eq(widget.identity, &ui.identity()) {
        Ok(())
    } else {
        Err(UiError::from_status(native_api::WRONG_RUNTIME))
    }
}

fn widget_property<T: crate::property::PropertyValue, W: WidgetRef + ?Sized>(
    widget: &W,
    ui: &UiContext<'_>,
    kind: UiPropertyKind,
) -> Result<Property<T>, UiError> {
    let widget = widget.__borrow_widget()?;
    check_widget_context(&widget, ui)?;
    ui.check_ui_thread()?;
    let api = Arc::clone(ui.api());
    let runtime = ui.runtime_token();
    let anchor = RuntimeAnchor::try_create(Arc::clone(&api), runtime)?;
    let owner_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let (status, token) = api.ui_property(kind, widget.token());
    if status != native_api::OK || token == 0 {
        if token == 0 {
            owner_ticket.release();
        } else {
            drop(Owned::<PropertyTag>::from_token(
                token,
                Arc::clone(&api),
                owner_ticket,
                runtime,
                Some(Arc::clone(&anchor)),
            ));
        }
        return Err(UiError::from_status(if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        }));
    }
    Ok(Property::from_owner(
        Owned::from_token(token, api, owner_ticket, runtime, Some(anchor)),
        ui.identity(),
    ))
}

fn tree_call(
    ui: &UiContext<'_>,
    parent: &Owned<impl Sized>,
    action: UiTreeAction,
    child: u64,
    index: u64,
    to_index: u64,
) -> Result<u64, UiError> {
    ui.check_ui_thread()?;
    let token = parent.borrow()?.token();
    let error_ticket = TicketReservation::reserve()
        .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
    let (status, value, error) = ui.api().ui_tree(action, token, child, index, to_index);
    if status != native_api::OK || error != 0 {
        let result_status = if status == native_api::OK {
            native_api::INTERNAL_ERROR
        } else {
            status
        };
        return Err(UiError::capture(
            ui.api(),
            result_status,
            error,
            error_ticket,
            parent.runtime_token(),
            parent.anchor(),
        ));
    }
    error_ticket.release();
    Ok(value)
}

fn widget_from_owned_output(
    ui: &UiContext<'_>,
    token: u64,
    error: u64,
    status: u32,
    ticket: TicketReservation,
    error_ticket: TicketReservation,
    anchor: Arc<RuntimeAnchor>,
) -> Result<Option<Widget>, UiError> {
    let api = Arc::clone(ui.api());
    let runtime = ui.runtime_token();
    if status != native_api::OK || error != 0 {
        if token == 0 {
            ticket.release();
        } else {
            drop(Owned::<WidgetTag>::from_token(
                token,
                Arc::clone(&api),
                ticket,
                runtime,
                Some(Arc::clone(&anchor)),
            ));
        }
        return Err(UiError::capture(
            &api,
            if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            },
            error,
            error_ticket,
            runtime,
            Some(anchor),
        ));
    }
    if token == 0 {
        ticket.release();
        error_ticket.release();
        return Ok(None);
    }
    error_ticket.release();
    Ok(Some(Widget {
        owner: Owned::from_token(token, api, ticket, runtime, Some(anchor)),
        identity: ui.identity(),
    }))
}

macro_rules! impl_widget_ref {
    ($type:ty, $tag:ty) => {
        impl sealed::Sealed for $type {}
        impl WidgetRef for $type {
            fn __borrow_widget(&self) -> Result<WidgetBorrow<'_>, UiError> {
                Ok(WidgetBorrow {
                    token: Borrowed::from_token(self.owner.borrow()?.token(), &self.owner),
                    identity: &self.identity,
                })
            }
        }
    };
}

impl_widget_ref!(Window, WindowTag);
impl_widget_ref!(StackPanel, StackPanelTag);
impl_widget_ref!(TextBox, TextBoxTag);
impl_widget_ref!(Text, TextTag);
impl_widget_ref!(Button, ButtonTag);
impl_widget_ref!(Widget, WidgetTag);

impl Window {
    pub fn create(ui: &UiContext<'_>) -> Result<Self, UiError> {
        Ok(Self {
            owner: create_owner(ui, UiControlKind::Window, &[])?,
            identity: ui.identity(),
        })
    }

    pub fn title_property(&self, ui: &UiContext<'_>) -> Result<Property<String>, UiError> {
        widget_property(self, ui, UiPropertyKind::WindowTitle)
    }

    pub fn content(&self, ui: &UiContext<'_>) -> Result<Option<Widget>, UiError> {
        let view = self.owner.borrow()?;
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let api = Arc::clone(ui.api());
        let anchor = self
            .owner
            .anchor()
            .ok_or_else(|| UiError::from_status(native_api::CLOSED))?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, token, error) =
            api.ui_tree(UiTreeAction::WindowContent, view.token(), 0, 0, 0);
        widget_from_owned_output(ui, token, error, status, ticket, error_ticket, anchor)
    }

    pub fn set_content(&self, ui: &UiContext<'_>, child: &impl WidgetRef) -> Result<(), UiError> {
        let view = child.__borrow_widget()?;
        check_context(&self.identity, ui)?;
        check_widget_context(&view, ui)?;
        ui.check_ui_thread()?;
        let token = view.token();
        let _parent = self.owner.borrow()?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, _, error) = ui.api().ui_tree(
            UiTreeAction::WindowSetContent,
            self.owner.borrow()?.token(),
            token,
            0,
            0,
        );
        finish_status_call(
            ui.api(),
            status,
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn clear_content(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        tree_call(ui, &self.owner, UiTreeAction::WindowClearContent, 0, 0, 0).map(|_| ())
    }

    pub fn open(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.window_action(ui, UiAction::WindowOpen)
    }

    pub fn close_peer(&self, ui: &UiContext<'_>) -> Result<(), UiError> {
        self.window_action(ui, UiAction::WindowClose)
    }

    fn window_action(&self, ui: &UiContext<'_>, action: UiAction) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, error) = ui.api().ui_action(action, token);
        finish_status_call(
            ui.api(),
            status,
            error,
            error_ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

impl StackPanel {
    pub fn create(ui: &UiContext<'_>) -> Result<Self, UiError> {
        Ok(Self {
            owner: create_owner(ui, UiControlKind::StackPanel, &[])?,
            identity: ui.identity(),
        })
    }

    pub fn orientation(&self, ui: &UiContext<'_>) -> Result<Orientation, UiError> {
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let token = self.owner.borrow()?.token();
        let (status, value, error) =
            ui.api()
                .ui_stack_panel(UiStackPanelAction::Orientation, token, 0, 0);
        if status != native_api::OK || error != 0 {
            return Err(UiError::from_status(if status == native_api::OK {
                native_api::INTERNAL_ERROR
            } else {
                status
            }));
        }
        Ok(Orientation::from_raw(value as u32))
    }

    pub fn set_orientation(&self, ui: &UiContext<'_>, value: Orientation) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let raw = value.to_raw()?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, _, error) = ui.api().ui_stack_panel(
            UiStackPanelAction::SetOrientation,
            self.owner.borrow()?.token(),
            0,
            raw,
        );
        finish_status_call(
            ui.api(),
            status,
            error,
            ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn insert(
        &self,
        ui: &UiContext<'_>,
        index: u64,
        child: &impl WidgetRef,
    ) -> Result<(), UiError> {
        self.tree_child(ui, UiTreeAction::Insert, child, index)
    }

    pub fn remove(&self, ui: &UiContext<'_>, child: &impl WidgetRef) -> Result<(), UiError> {
        self.tree_child(ui, UiTreeAction::Remove, child, 0)
    }

    fn tree_child(
        &self,
        ui: &UiContext<'_>,
        action: UiTreeAction,
        child: &impl WidgetRef,
        index: u64,
    ) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        let child = child.__borrow_widget()?;
        check_widget_context(&child, ui)?;
        ui.check_ui_thread()?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, _, error) = ui.api().ui_tree(
            action,
            self.owner.borrow()?.token(),
            child.token(),
            index,
            0,
        );
        finish_status_call(
            ui.api(),
            status,
            error,
            ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn move_child(&self, ui: &UiContext<'_>, from: u64, to: u64) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        let _ = tree_call(ui, &self.owner, UiTreeAction::Move, 0, from, to)?;
        Ok(())
    }

    pub fn child_count(&self, ui: &UiContext<'_>) -> Result<u64, UiError> {
        check_context(&self.identity, ui)?;
        tree_call(ui, &self.owner, UiTreeAction::ChildCount, 0, 0, 0)
    }

    pub fn child_at(&self, ui: &UiContext<'_>, index: u64) -> Result<Option<Widget>, UiError> {
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let api = Arc::clone(ui.api());
        let anchor = self
            .owner
            .anchor()
            .ok_or_else(|| UiError::from_status(native_api::CLOSED))?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, token, error) = api.ui_tree(
            UiTreeAction::ChildAt,
            self.owner.borrow()?.token(),
            0,
            index,
            0,
        );
        widget_from_owned_output(ui, token, error, status, ticket, error_ticket, anchor)
    }

    pub fn set_child_align(
        &self,
        ui: &UiContext<'_>,
        child: &impl WidgetRef,
        align: ChildAlign,
    ) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        let child = child.__borrow_widget()?;
        check_widget_context(&child, ui)?;
        ui.check_ui_thread()?;
        let raw = align.to_raw()?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, _, error) = ui.api().ui_stack_panel(
            UiStackPanelAction::SetChildAlign,
            self.owner.borrow()?.token(),
            child.token(),
            raw,
        );
        finish_status_call(
            ui.api(),
            status,
            error,
            ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

impl Text {
    pub fn create(ui: &UiContext<'_>, initial: &str) -> Result<Self, UiError> {
        Ok(Self {
            owner: create_owner(ui, UiControlKind::Text, initial.as_bytes())?,
            identity: ui.identity(),
        })
    }

    pub fn content_property(&self, ui: &UiContext<'_>) -> Result<Property<String>, UiError> {
        widget_property(self, ui, UiPropertyKind::TextContent)
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

impl TextBox {
    pub fn create(ui: &UiContext<'_>, initial: &str) -> Result<Self, UiError> {
        Ok(Self {
            owner: create_owner(ui, UiControlKind::TextBox, initial.as_bytes())?,
            identity: ui.identity(),
        })
    }

    pub fn value_property(&self, ui: &UiContext<'_>) -> Result<Property<String>, UiError> {
        widget_property(self, ui, UiPropertyKind::TextBoxValue)
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

impl Button {
    pub fn create(ui: &UiContext<'_>, label: &str) -> Result<Self, UiError> {
        Ok(Self {
            owner: create_owner(ui, UiControlKind::Button, label.as_bytes())?,
            identity: ui.identity(),
        })
    }

    pub fn label_property(&self, ui: &UiContext<'_>) -> Result<Property<String>, UiError> {
        widget_property(self, ui, UiPropertyKind::ButtonLabel)
    }

    pub fn set_command(
        &self,
        ui: &UiContext<'_>,
        command: Option<&AsyncCommand>,
    ) -> Result<(), UiError> {
        check_context(&self.identity, ui)?;
        if let Some(command) = command {
            command.check_ui_context(ui)?;
        }
        ui.check_ui_thread()?;
        let token = command.map_or(Ok(0), AsyncCommand::token_for_call)?;
        let ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, _, error) = ui.api().ui_button(
            UiButtonAction::SetCommand,
            self.owner.borrow()?.token(),
            token,
        );
        finish_status_call(
            ui.api(),
            status,
            error,
            ticket,
            self.owner.runtime_token(),
            self.owner.anchor(),
        )
    }

    pub fn command(&self, ui: &UiContext<'_>) -> Result<Option<AsyncCommand>, UiError> {
        check_context(&self.identity, ui)?;
        ui.check_ui_thread()?;
        let api = Arc::clone(ui.api());
        let runtime = ui.runtime_token();
        let anchor = self
            .owner
            .anchor()
            .ok_or_else(|| UiError::from_status(native_api::CLOSED))?;
        let owner_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let error_ticket = TicketReservation::reserve()
            .map_err(|()| UiError::from_status(native_api::OUT_OF_MEMORY))?;
        let (status, token, error) =
            api.ui_button(UiButtonAction::Command, self.owner.borrow()?.token(), 0);
        if status != native_api::OK || error != 0 {
            if token == 0 {
                owner_ticket.release();
            } else {
                drop(Owned::<CommandTag>::from_token(
                    token,
                    Arc::clone(&api),
                    owner_ticket,
                    runtime,
                    Some(Arc::clone(&anchor)),
                ));
            }
            return Err(UiError::capture(
                &api,
                if status == native_api::OK {
                    native_api::INTERNAL_ERROR
                } else {
                    status
                },
                error,
                error_ticket,
                runtime,
                Some(anchor),
            ));
        }
        error_ticket.release();
        if token == 0 {
            owner_ticket.release();
            return Ok(None);
        }
        Ok(Some(AsyncCommand::from_owner(
            Owned::from_token(token, api, owner_ticket, runtime, Some(anchor)),
            ui.identity(),
        )))
    }

    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

impl Widget {
    pub fn close(&mut self) -> Result<(), UiError> {
        self.owner.close()
    }
}

fn check_context(identity: &Arc<()>, ui: &UiContext<'_>) -> Result<(), UiError> {
    if Arc::ptr_eq(identity, &ui.identity()) {
        Ok(())
    } else {
        Err(UiError::from_status(native_api::WRONG_RUNTIME))
    }
}
