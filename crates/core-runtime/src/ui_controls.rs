//! Core-owned state for the initial UI Control set.

use crate::handle_registry::registry::InternalOwner;
use crate::runtime::RuntimeIdentity;

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
    #[allow(dead_code)] // Read and mutated by the StackPanel/Layout unit.
    pub(crate) orientation: u32,
}

impl UiControlState {
    pub(crate) fn new(kind: UiControlKind, properties: Vec<UiControlPropertyOwner>) -> Self {
        Self {
            kind,
            properties,
            orientation: 1,
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
}

#[cfg(test)]
mod tests {
    use crate::abi_support::{
        UiControlApiError, handle_release, property_get_bool, property_get_utf8,
        runtime_create_handle, runtime_set_ui_loop_bridge, runtime_shutdown,
        ui_control_accessible_name_property, ui_control_button_label_property,
        ui_control_create_button, ui_control_create_stack_panel, ui_control_create_text,
        ui_control_create_text_box, ui_control_create_window, ui_control_is_enabled_property,
        ui_control_text_box_value_property, ui_control_text_content_property,
        ui_control_window_title_property,
    };
    use std::ffi::c_void;

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
}
