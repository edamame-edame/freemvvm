//! Core-owned state for the initial UI Control set.

use crate::handle_registry::registry::{InternalOwner, ObjectIdentity};
use crate::runtime::RuntimeIdentity;
use crate::ui_window::WindowLifecycle;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UiControlKind {
    Window,
    StackPanel,
    Text,
    TextBox,
    Button,
}

impl UiControlKind {
    pub(crate) fn is_container(self) -> bool {
        matches!(self, Self::Window | Self::StackPanel)
    }

    pub(crate) fn supports_property(self, property: UiControlProperty) -> bool {
        match property {
            UiControlProperty::IsEnabled | UiControlProperty::AccessibleName => true,
            UiControlProperty::WindowTitle => self == Self::Window,
            UiControlProperty::TextContent => self == Self::Text,
            UiControlProperty::TextBoxValue => self == Self::TextBox,
            UiControlProperty::ButtonLabel => self == Self::Button,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UiControlProperty {
    IsEnabled,
    AccessibleName,
    WindowTitle,
    TextContent,
    TextBoxValue,
    ButtonLabel,
}

pub(crate) struct UiControlPropertyOwner {
    pub(crate) property: UiControlProperty,
    pub(crate) owner: InternalOwner<RuntimeIdentity>,
}

/// Control state is stored separately from its shared Property objects and Logical Tree node.
pub(crate) struct UiControlState {
    pub(crate) kind: UiControlKind,
    pub(crate) properties: Vec<UiControlPropertyOwner>,
    pub(crate) orientation: u32,
    pub(crate) child_align: HashMap<ObjectIdentity, u32>,
    pub(crate) layout_dirty: bool,
    pub(crate) window: Option<WindowLifecycle>,
}

impl UiControlState {
    pub(crate) fn new(kind: UiControlKind, properties: Vec<UiControlPropertyOwner>) -> Self {
        Self {
            kind,
            properties,
            orientation: 1,
            child_align: HashMap::new(),
            layout_dirty: true,
            window: (kind == UiControlKind::Window).then(WindowLifecycle::default),
        }
    }

    pub(crate) fn property_owner(
        &self,
        property: UiControlProperty,
    ) -> Option<&InternalOwner<RuntimeIdentity>> {
        self.properties
            .iter()
            .find(|entry| entry.property == property)
            .map(|entry| &entry.owner)
    }

    pub(crate) fn is_focusable_by_default(&self) -> bool {
        matches!(self.kind, UiControlKind::TextBox | UiControlKind::Button)
    }

    pub(crate) fn child_align(&self, child: ObjectIdentity) -> u32 {
        self.child_align.get(&child).copied().unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use crate::abi_support::{
        UiControlApiError, handle_release, property_get_bool, property_get_utf8, property_set_bool,
        runtime_create_handle, runtime_dispatch_turn, runtime_set_ui_loop_bridge, runtime_shutdown,
        stack_panel_child_align, stack_panel_child_at, stack_panel_child_count, stack_panel_insert,
        stack_panel_move, stack_panel_orientation, stack_panel_remove, stack_panel_set_child_align,
        stack_panel_set_orientation, ui_control_accessible_name_property,
        ui_control_button_label_property, ui_control_create_button, ui_control_create_stack_panel,
        ui_control_create_text, ui_control_create_text_box, ui_control_create_window,
        ui_control_is_enabled_property, ui_control_layout_clean, ui_control_layout_dirty,
        ui_control_text_box_value_property, ui_control_text_content_property,
        ui_control_window_title_property, widget_has_focus, widget_request_focus,
        window_clear_content, window_close_intent, window_content, window_install_test_peer,
        window_open, window_platform_activation, window_set_content, window_set_content_with_peer,
    };
    use crate::ui_window::CloseDecision;
    use std::ffi::c_void;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestWindowPeer(AtomicUsize, AtomicUsize);

    impl crate::ui_window::WindowPeerAdapter for TestWindowPeer {
        fn open(&self, _: crate::handle_registry::registry::ObjectIdentity) -> Result<(), ()> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn close(&self, _: crate::handle_registry::registry::ObjectIdentity) -> Result<(), ()> {
            self.1.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    unsafe extern "C" fn noop_wake(_: *mut c_void, _: u32) {}

    #[test]
    fn text_control_exposes_shared_default_and_initial_properties() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let text = ui_control_create_text(runtime, "猫".as_bytes()).expect("create Text");
        let enabled = ui_control_is_enabled_property(text).expect("get enabled Property");
        let content = ui_control_text_content_property(text).expect("get content Property");

        assert!(property_get_bool(enabled).expect("read enabled Property"));
        let mut bytes = [0; 3];
        assert_eq!(
            property_get_utf8(content, Some(&mut bytes)).expect("read Text content"),
            3
        );
        assert_eq!(&bytes, "猫".as_bytes());

        handle_release(text).expect("release Text");
        assert_eq!(
            property_get_utf8(content, Some(&mut bytes)).expect("Property owner outlives Text"),
            3
        );
        handle_release(content).expect("release content Property");
        handle_release(enabled).expect("release enabled Property");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn all_initial_controls_use_shared_enabled_and_accessible_name_properties() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let controls = [
            ui_control_create_window(runtime).expect("create Window"),
            ui_control_create_stack_panel(runtime).expect("create StackPanel"),
            ui_control_create_text(runtime, b"static").expect("create Text"),
            ui_control_create_text_box(runtime, b"editable").expect("create TextBox"),
            ui_control_create_button(runtime, b"run").expect("create Button"),
        ];
        for control in controls {
            let enabled = ui_control_is_enabled_property(control).expect("get enabled Property");
            let name = ui_control_accessible_name_property(control).expect("get name Property");
            assert!(property_get_bool(enabled).expect("enabled default"));
            let mut empty = [0; 1];
            assert_eq!(
                property_get_utf8(name, Some(&mut empty)).expect("empty name default"),
                0
            );
            handle_release(enabled).expect("release enabled Property");
            handle_release(name).expect("release name Property");
            handle_release(control).expect("release Control");
        }
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn each_control_exposes_its_kind_specific_property() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let text = ui_control_create_text(runtime, b"read-only").expect("create Text");
        let text_box = ui_control_create_text_box(runtime, b"initial").expect("create TextBox");
        let button = ui_control_create_button(runtime, b"label").expect("create Button");
        let properties = [
            ui_control_window_title_property(window).expect("Window title"),
            ui_control_text_content_property(text).expect("Text content"),
            ui_control_text_box_value_property(text_box).expect("TextBox value"),
            ui_control_button_label_property(button).expect("Button label"),
        ];
        for (property, expected) in
            properties
                .into_iter()
                .zip([b"".as_slice(), b"read-only", b"initial", b"label"])
        {
            let mut bytes = vec![0; expected.len()];
            let copied = if bytes.is_empty() {
                property_get_utf8(property, None).expect("query empty value")
            } else {
                property_get_utf8(property, Some(&mut bytes)).expect("read initial value")
            };
            assert_eq!(copied, expected.len());
            assert_eq!(bytes, expected);
            handle_release(property).expect("release Property");
        }
        for control in [window, text, text_box, button] {
            handle_release(control).expect("release Control");
        }
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn invalid_utf8_wrong_property_kind_and_wrong_thread_are_rejected() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let button = ui_control_create_button(runtime, b"go").expect("create Button");

        assert_eq!(
            ui_control_create_text(runtime, &[0xff]),
            Err(UiControlApiError::InvalidArgument)
        );
        assert_eq!(
            ui_control_text_content_property(button),
            Err(UiControlApiError::WrongType)
        );
        assert_eq!(
            std::thread::spawn(move || ui_control_create_window(runtime))
                .join()
                .expect("worker completes"),
            Err(UiControlApiError::WrongThread)
        );

        handle_release(button).expect("release Button");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn stack_panel_orders_children_and_tree_owners_outlive_public_tokens() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let panel = ui_control_create_stack_panel(runtime).expect("create StackPanel");
        let first = ui_control_create_text(runtime, b"first").expect("create first Text");
        let second = ui_control_create_text(runtime, b"second").expect("create second Text");

        stack_panel_insert(panel, 0, first).expect("insert first child");
        stack_panel_insert(panel, 1, second).expect("insert second child");
        handle_release(first).expect("release first public token");
        handle_release(second).expect("release second public token");
        assert_eq!(stack_panel_child_count(panel).expect("count children"), 2);

        stack_panel_move(panel, 0, 1).expect("move first child to end");
        for (index, expected) in [(0, b"second".as_slice()), (1, b"first".as_slice())] {
            let child = stack_panel_child_at(panel, index).expect("issue child token");
            let property = ui_control_text_content_property(child).expect("get child Property");
            let mut actual = vec![0; expected.len()];
            assert_eq!(
                property_get_utf8(property, Some(&mut actual)).expect("read child content"),
                expected.len()
            );
            assert_eq!(actual, expected);
            handle_release(property).expect("release Property token");
            handle_release(child).expect("release issued child token");
        }

        stack_panel_remove(panel, 1).expect("detach child");
        assert_eq!(
            stack_panel_child_count(panel).expect("count after remove"),
            1
        );
        assert_eq!(
            stack_panel_remove(panel, 7),
            Err(UiControlApiError::InvalidArgument)
        );
        handle_release(panel).expect("release StackPanel");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn window_replacement_and_clear_detach_content() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let old = ui_control_create_text(runtime, b"old").expect("create old content");
        let new = ui_control_create_stack_panel(runtime).expect("create new content");

        window_set_content(window, old).expect("set initial content");
        assert_eq!(
            window_set_content_with_peer(window, new, || Err(())),
            Err(UiControlApiError::Internal)
        );
        assert!(
            window_content(window)
                .expect("old content remains")
                .is_some()
        );
        let observed = window_content(window)
            .expect("get content")
            .expect("content exists");
        handle_release(observed).expect("release getter token");
        window_set_content(window, new).expect("replace content");
        let detached_property =
            ui_control_text_content_property(old).expect("old content remains independently owned");
        handle_release(detached_property).expect("release detached Property");
        assert_eq!(
            window_set_content(window, window),
            Err(UiControlApiError::WrongType)
        );

        window_clear_content(window).expect("clear content");
        assert_eq!(window_content(window).expect("get cleared content"), None);
        window_set_content(window, old).expect("reattach detached content");
        handle_release(old).expect("release old public token");
        handle_release(new).expect("release new public token");
        handle_release(window).expect("release Window");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn tree_failures_leave_existing_parent_child_state_unchanged() {
        let runtime = runtime_create_handle().expect("create Runtime");
        let other_runtime = runtime_create_handle().expect("create other Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        runtime_set_ui_loop_bridge(other_runtime, noop_wake, 0, None)
            .expect("install other loop bridge");
        let panel = ui_control_create_stack_panel(runtime).expect("create StackPanel");
        let other_panel = ui_control_create_stack_panel(runtime).expect("create other StackPanel");
        let child = ui_control_create_stack_panel(runtime).expect("create child StackPanel");
        let foreign =
            ui_control_create_text(other_runtime, b"foreign").expect("create foreign Text");

        assert_eq!(
            stack_panel_insert(panel, 1, child),
            Err(UiControlApiError::InvalidArgument)
        );
        assert_eq!(stack_panel_child_count(panel).expect("empty panel"), 0);
        stack_panel_insert(panel, 0, child).expect("attach child");
        assert_eq!(
            stack_panel_insert(other_panel, 0, child),
            Err(UiControlApiError::InvalidArgument)
        );
        assert_eq!(
            stack_panel_insert(child, 0, panel),
            Err(UiControlApiError::InvalidArgument)
        );
        assert_eq!(
            stack_panel_insert(panel, 1, foreign),
            Err(UiControlApiError::WrongRuntime)
        );
        assert_eq!(
            stack_panel_child_count(panel).expect("original child remains"),
            1
        );
        assert_eq!(
            stack_panel_child_count(other_panel).expect("other remains empty"),
            0
        );
        assert_eq!(
            stack_panel_child_count(child).expect("cycle target remains empty"),
            0
        );

        handle_release(child).expect("release child public token");
        handle_release(foreign).expect("release foreign child");
        handle_release(panel).expect("release panel");
        handle_release(other_panel).expect("release other panel");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        runtime_shutdown(other_runtime).expect("shutdown other Runtime");
        handle_release(runtime).expect("release Runtime");
        handle_release(other_runtime).expect("release other Runtime");
    }

    #[test]
    fn stack_panel_settings_validate_before_marking_layout_dirty() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let panel = ui_control_create_stack_panel(runtime).expect("create StackPanel");
        let button = ui_control_create_button(runtime, b"run").expect("create Button");
        let detached = ui_control_create_text(runtime, b"detached").expect("create Text");
        window_set_content(window, panel).expect("set content");
        stack_panel_insert(panel, 0, button).expect("insert Button");

        ui_control_layout_clean(window).expect("consume Window layout dirty");
        ui_control_layout_clean(panel).expect("consume StackPanel layout dirty");

        assert_eq!(stack_panel_orientation(panel), Ok(1));
        ui_control_layout_clean(panel).expect("consume initial layout dirty");
        stack_panel_set_orientation(panel, 2).expect("set horizontal orientation");
        assert_eq!(stack_panel_orientation(panel), Ok(2));
        assert!(ui_control_layout_dirty(panel).expect("query layout dirty"));
        assert!(ui_control_layout_dirty(window).expect("layout dirtiness propagates to root"));
        ui_control_layout_clean(panel).expect("consume orientation dirty");
        assert_eq!(
            stack_panel_set_orientation(panel, 3),
            Err(UiControlApiError::InvalidArgument)
        );
        assert!(!ui_control_layout_dirty(panel).expect("invalid enum stays clean"));

        stack_panel_set_child_align(panel, button, 2).expect("set child align");
        assert_eq!(stack_panel_child_align(panel, button), Ok(2));
        assert!(ui_control_layout_dirty(panel).expect("align marks dirty"));
        ui_control_layout_clean(panel).expect("consume alignment dirty");
        assert_eq!(
            stack_panel_set_child_align(panel, detached, 1),
            Err(UiControlApiError::InvalidArgument)
        );
        assert!(!ui_control_layout_dirty(panel).expect("foreign child leaves layout clean"));

        for handle in [window, panel, button, detached] {
            handle_release(handle).expect("release Control");
        }
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn focus_requires_active_window_and_restores_only_valid_attached_targets() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let peer = Arc::new(TestWindowPeer(AtomicUsize::new(0), AtomicUsize::new(0)));
        window_install_test_peer(window, peer.clone()).expect("install fake peer");
        assert_eq!(window_open(window), Ok(1));
        assert_eq!(
            window_close_intent(
                window,
                Some(Box::new(move |_, _| {
                    let title = ui_control_window_title_property(window)
                        .expect("close callback can reenter Runtime APIs");
                    handle_release(title).expect("release title Property");
                    CloseDecision::Decline
                })),
            ),
            Ok(None)
        );
        let panel = ui_control_create_stack_panel(runtime).expect("create StackPanel");
        let text_box = ui_control_create_text_box(runtime, b"value").expect("create TextBox");
        let button = ui_control_create_button(runtime, b"run").expect("create Button");
        window_set_content(window, panel).expect("set content");
        stack_panel_insert(panel, 0, text_box).expect("insert TextBox");
        stack_panel_insert(panel, 1, button).expect("insert Button");

        assert_eq!(
            widget_request_focus(text_box),
            Err(UiControlApiError::InvalidArgument)
        );
        window_platform_activation(window, true).expect("activate Window");
        widget_request_focus(text_box).expect("focus TextBox");
        assert!(widget_has_focus(text_box).expect("TextBox focused"));
        let enabled = ui_control_is_enabled_property(button).expect("get Button enabled Property");
        property_set_bool(enabled, false).expect("disable Button");
        assert_eq!(
            widget_request_focus(button),
            Err(UiControlApiError::InvalidArgument)
        );
        assert!(widget_has_focus(text_box).expect("invalid request preserves old Focus"));
        property_set_bool(enabled, true).expect("enable Button");
        widget_request_focus(button).expect("move Focus to Button");
        window_platform_activation(window, false).expect("deactivate Window");
        assert!(!widget_has_focus(button).expect("deactivation clears active Focus"));
        window_platform_activation(window, true).expect("reactivate Window");
        assert!(widget_has_focus(button).expect("valid candidate restores"));

        window_clear_content(window).expect("detach entire focused subtree");
        assert!(!widget_has_focus(button).expect("detached target is not focused"));
        handle_release(enabled).expect("release enabled Property");
        handle_release(panel).expect("release panel");
        handle_release(text_box).expect("release TextBox");
        handle_release(button).expect("release Button");
        let shutdown = runtime_shutdown(runtime).expect("shutdown Runtime");
        if matches!(shutdown, crate::abi_support::ShutdownApiState::Pending) {
            runtime_dispatch_turn(runtime).expect("dispatch pending shutdown");
        }
        handle_release(window).expect("release Window after shutdown");
        handle_release(runtime).expect("release Runtime");
        assert_eq!(peer.0.load(Ordering::Relaxed), 1);
        assert!(peer.1.load(Ordering::Relaxed) >= 1);
    }
}
