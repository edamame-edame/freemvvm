//! Core-owned state for the initial UI Control set.

use crate::handle_registry::registry::{InternalOwner, ObjectIdentity};
use crate::runtime::RuntimeIdentity;
use crate::ui_window::WindowLifecycle;
use std::collections::HashMap;
use std::sync::Arc;

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
    pub(crate) button_command: Option<Arc<crate::abi_support::CommandHandleEntry>>,
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
            button_command: None,
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
        Damage, DeviceScale, FramePhase, FrameTicket, PlatformWindowClientAreaSink, Rect,
        SceneSnapshot, WindowClientArea, WindowOpenInfo,
    };
    use crate::abi_support::{
        UiControlApiError, handle_release, property_get_bool, property_get_utf8, property_set_bool,
        property_set_utf8, runtime_create_handle, runtime_create_handle_with_platform_factory,
        runtime_dispatch_turn, runtime_set_ui_loop_bridge, runtime_shutdown,
        stack_panel_child_align, stack_panel_child_at, stack_panel_child_count, stack_panel_insert,
        stack_panel_move, stack_panel_orientation, stack_panel_remove, stack_panel_set_child_align,
        stack_panel_set_orientation, ui_control_accessible_name_property,
        ui_control_button_label_property, ui_control_create_button, ui_control_create_stack_panel,
        ui_control_create_text, ui_control_create_text_box, ui_control_create_window,
        ui_control_is_enabled_property, ui_control_layout_clean, ui_control_layout_dirty,
        ui_control_text_box_value_property, ui_control_text_content_property,
        ui_control_window_title_property, widget_has_focus, widget_request_focus,
        window_background_rgb, window_calculate_layout_for_test,
        window_calculate_layout_with_orientation_change_for_test,
        window_calculate_layout_with_platform_text_for_test, window_clear_content,
        window_client_area_for_test, window_close, window_close_intent,
        window_close_intent_subscribed, window_close_request_current, window_close_request_defer,
        window_close_request_resolve, window_close_request_subscribe, window_content,
        window_control_tree_snapshot_for_test, window_device_scale_for_test, window_frame_phase,
        window_install_test_peer, window_layout_and_control_tree_with_text_for_test,
        window_layout_snapshot_for_test, window_open, window_platform_activation,
        window_reset_background_rgb, window_set_background_rgb, window_set_content,
        window_set_content_with_peer, window_submit_frame,
    };
    use crate::ui_layout::Size;
    use crate::ui_window::CloseDecision;
    use crate::visual::{ControlViewConnector, VisualNodeDescriptor};
    use std::collections::HashMap;
    use std::ffi::c_void;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestWindowPeer(AtomicUsize, AtomicUsize);

    impl crate::ui_window::WindowPeerAdapter for TestWindowPeer {
        fn open(
            &self,
            _: crate::handle_registry::registry::ObjectIdentity,
            _: u64,
        ) -> Result<WindowOpenInfo, ()> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(WindowOpenInfo {
                client_area: WindowClientArea {
                    width_dip: 640.0,
                    height_dip: 480.0,
                },
                device_scale: DeviceScale::new(1.25, 1.5).expect("valid test scale"),
            })
        }

        fn close(&self, _: crate::handle_registry::registry::ObjectIdentity) -> Result<(), ()> {
            self.1.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn submit_frame(&self, _: FrameTicket, _: Arc<SceneSnapshot>) -> Result<(), ()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct PlatformProbe {
        created: AtomicUsize,
        opened: AtomicUsize,
        closed: AtomicUsize,
        fail_next_create: std::sync::atomic::AtomicBool,
        completion_sink:
            std::sync::Mutex<Option<Arc<dyn crate::abi_support::PlatformFrameCompletionSink>>>,
        client_area_sink: std::sync::Mutex<Option<Arc<dyn PlatformWindowClientAreaSink>>>,
        submitted: std::sync::Mutex<Vec<(FrameTicket, Arc<SceneSnapshot>)>>,
        fail_next_submit: std::sync::atomic::AtomicBool,
        text_service: std::sync::Mutex<Option<Arc<dyn crate::abi_support::TextService>>>,
    }

    struct TestPlatformFactory(Arc<PlatformProbe>);

    impl crate::abi_support::PlatformRuntimeFactory for TestPlatformFactory {
        fn create_window_peer(
            &self,
            completion_sink: Arc<dyn crate::abi_support::PlatformFrameCompletionSink>,
        ) -> Result<
            Arc<dyn crate::abi_support::PlatformWindowPeer>,
            crate::abi_support::PlatformOperationError,
        > {
            self.0.created.fetch_add(1, Ordering::SeqCst);
            if self.0.fail_next_create.swap(false, Ordering::SeqCst) {
                return Err(crate::abi_support::PlatformOperationError);
            }
            *self.0.completion_sink.lock().expect("completion sink lock") =
                Some(Arc::clone(&completion_sink));
            Ok(Arc::new(TestPlatformWindowPeer {
                probe: Arc::clone(&self.0),
                _completion_sink: completion_sink,
            }))
        }

        fn text_service(&self) -> Option<Arc<dyn crate::abi_support::TextService>> {
            self.0
                .text_service
                .lock()
                .expect("text service lock")
                .clone()
        }
    }

    #[derive(Default)]
    struct TextServiceProbe {
        requests: std::sync::Mutex<Vec<(u64, String)>>,
        fail: std::sync::atomic::AtomicBool,
    }

    impl crate::abi_support::TextService for TextServiceProbe {
        fn resolve_and_shape(
            &self,
            request: &crate::abi_support::TextRequest,
        ) -> Result<crate::abi_support::ResolvedTextSnapshot, crate::abi_support::TextServiceError>
        {
            self.requests
                .lock()
                .expect("text request lock")
                .push((request.revision, request.text.to_string()));
            if self.fail.load(Ordering::SeqCst) {
                return Err(crate::abi_support::TextServiceError::Unavailable);
            }
            let digest = [0x5a; 32];
            let glyph = crate::abi_support::GlyphPlacement::new(42, 0.0, 0.0, 18.0, 0.0)?;
            let glyphs = crate::abi_support::GlyphRunSnapshot::new(
                digest,
                14.0,
                crate::abi_support::Rect::new(0.0, 0.0, 18.0, 20.0)?,
                Arc::<[crate::abi_support::GlyphPlacement]>::from([glyph]),
            )?;
            let font = crate::abi_support::ResolvedFontIdentity {
                resolver_profile: Arc::from("test-profile"),
                family: Arc::from("Test Sans"),
                style: Arc::from("Regular"),
                asset_digest: digest,
                face_index: 0,
            };
            let run = crate::abi_support::ResolvedGlyphRun::new(font, glyphs)?;
            Ok(crate::abi_support::ResolvedTextSnapshot::new(
                request.revision,
                crate::abi_support::TextMetrics::new(18.0, 20.0)?,
                Arc::<[crate::abi_support::ResolvedGlyphRun]>::from([run]),
            ))
        }
    }

    struct TestPlatformWindowPeer {
        probe: Arc<PlatformProbe>,
        _completion_sink: Arc<dyn crate::abi_support::PlatformFrameCompletionSink>,
    }

    impl crate::abi_support::PlatformWindowPeer for TestPlatformWindowPeer {
        fn open(
            &self,
            _: u64,
        ) -> Result<WindowOpenInfo, crate::abi_support::PlatformOperationError> {
            self.probe.opened.fetch_add(1, Ordering::SeqCst);
            Ok(WindowOpenInfo {
                client_area: WindowClientArea {
                    width_dip: 640.0,
                    height_dip: 480.0,
                },
                device_scale: DeviceScale::new(1.25, 1.5).expect("valid test scale"),
            })
        }

        fn set_client_area_sink(&self, sink: Arc<dyn PlatformWindowClientAreaSink>) {
            *self
                .probe
                .client_area_sink
                .lock()
                .expect("client-area sink lock") = Some(sink);
        }

        fn close(&self) -> Result<(), crate::abi_support::PlatformOperationError> {
            self.probe.closed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn submit_frame(
            &self,
            ticket: FrameTicket,
            scene: Arc<SceneSnapshot>,
        ) -> Result<(), crate::abi_support::PlatformOperationError> {
            self.probe
                .submitted
                .lock()
                .expect("submitted frames lock")
                .push((ticket, scene));
            if self.probe.fail_next_submit.swap(false, Ordering::SeqCst) {
                return Err(crate::abi_support::PlatformOperationError);
            }
            Ok(())
        }
    }

    unsafe extern "C" fn noop_wake(_: *mut c_void, _: u32) {}

    fn partial_scene(revision: u64) -> Arc<SceneSnapshot> {
        let bounds = Rect::new(1.0, 2.0, 3.0, 4.0).expect("damage bounds");
        Arc::new(
            SceneSnapshot::new(
                revision,
                crate::abi_support::DeviceScale::new(1.0, 1.0).expect("valid test scale"),
                [0.0, 0.0, 0.0],
                Arc::from([]),
                Damage::rect(bounds),
            )
            .expect("valid scene"),
        )
    }

    #[test]
    fn window_client_area_updates_are_dispatched_validated_and_generation_checked() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with fake peer");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let content = ui_control_create_stack_panel(runtime).expect("create Window content");
        window_set_content(window, content).expect("attach Window content");
        assert_eq!(window_open(window), Ok(1));
        assert_eq!(
            window_client_area_for_test(window),
            Ok(Some((
                1,
                WindowClientArea {
                    width_dip: 640.0,
                    height_dip: 480.0,
                }
            )))
        );
        ui_control_layout_clean(window).expect("clear initial layout dirty");
        ui_control_layout_clean(content).expect("clear content layout dirty");
        let sink = probe
            .client_area_sink
            .lock()
            .expect("client-area sink lock")
            .as_ref()
            .expect("Core client-area sink")
            .clone();

        let resized = WindowClientArea {
            width_dip: 800.0,
            height_dip: 600.0,
        };
        sink.client_area_changed(1, resized)
            .expect("enqueue current-generation resize");
        assert!(!ui_control_layout_dirty(window).expect("dirty before dispatch"));
        runtime_dispatch_turn(runtime).expect("apply resize on UI thread");
        assert!(ui_control_layout_dirty(window).expect("resize marks Layout dirty"));
        assert!(ui_control_layout_dirty(content).expect("resize marks root subtree dirty"));
        assert_eq!(window_client_area_for_test(window), Ok(Some((1, resized))));

        ui_control_layout_clean(window).expect("clear resize dirty");
        ui_control_layout_clean(content).expect("clear resize subtree dirty");
        assert!(
            sink.client_area_changed(
                1,
                WindowClientArea {
                    width_dip: f64::NAN,
                    height_dip: 300.0,
                },
            )
            .is_err()
        );
        sink.client_area_changed(0, resized)
            .expect("enqueue stale-generation resize for discard");
        runtime_dispatch_turn(runtime).expect("discard stale resize on UI thread");
        assert!(!ui_control_layout_dirty(window).expect("stale/invalid area leaves clean"));
        assert!(!ui_control_layout_dirty(content).expect("stale area leaves subtree clean"));
        assert_eq!(window_client_area_for_test(window), Ok(Some((1, resized))));

        window_close(window).expect("close Window");
        assert_eq!(window_client_area_for_test(window), Ok(None));
        assert_eq!(window_open(window), Ok(2));
        assert_eq!(
            window_client_area_for_test(window),
            Ok(Some((
                2,
                WindowClientArea {
                    width_dip: 640.0,
                    height_dip: 480.0,
                }
            )))
        );
        ui_control_layout_clean(window).expect("clear reopened Layout dirty");
        ui_control_layout_clean(content).expect("clear reopened subtree dirty");
        sink.client_area_changed(1, resized)
            .expect("enqueue old peer resize");
        runtime_dispatch_turn(runtime).expect("discard old peer resize");
        assert!(!ui_control_layout_dirty(window).expect("old generation leaves clean"));
        assert!(!ui_control_layout_dirty(content).expect("old generation leaves subtree clean"));

        sink.client_area_changed(
            2,
            WindowClientArea {
                width_dip: 0.0,
                height_dip: 0.0,
            },
        )
        .expect("enqueue valid empty client area");
        runtime_dispatch_turn(runtime).expect("apply empty client area");
        assert!(ui_control_layout_dirty(window).expect("empty area is a valid update"));
        assert!(ui_control_layout_dirty(content).expect("empty area dirties root subtree"));
        assert_eq!(
            window_client_area_for_test(window),
            Ok(Some((
                2,
                WindowClientArea {
                    width_dip: 0.0,
                    height_dip: 0.0,
                }
            )))
        );
        runtime_shutdown(runtime).expect("shutdown test Runtime");
    }

    #[test]
    fn runtime_layout_snapshot_failure_preserves_control_tree_snapshot_and_dirty_state() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with fake peer");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let root = ui_control_create_stack_panel(runtime).expect("create root panel");
        let nested = ui_control_create_stack_panel(runtime).expect("create nested panel");
        let leaf = ui_control_create_button(runtime, b"leaf").expect("create leaf Button");
        window_set_content(window, root).expect("attach root content");
        stack_panel_insert(root, 0, nested).expect("attach nested panel");
        stack_panel_insert(nested, 0, leaf).expect("attach Button leaf");
        assert_eq!(window_open(window), Ok(1));

        let first = window_calculate_layout_for_test(window, 24.0, 12.0, false)
            .expect("calculate from the live Control tree");
        assert_eq!(first.window_generation, 1);
        assert_eq!(first.tree_revision, 7);
        assert_eq!(first.layout_revision, 1);
        assert_eq!(first.client_area.width, 640.0);
        assert_eq!(first.client_area.height, 480.0);
        assert_eq!(first.node_count(), 3);
        assert_eq!(
            window_layout_snapshot_for_test(window),
            Ok(Some(first.clone()))
        );
        for control in [window, root, nested, leaf] {
            assert!(!ui_control_layout_dirty(control).expect("successful pass clears dirty"));
        }

        assert_eq!(
            window_calculate_layout_with_orientation_change_for_test(window, nested, 2),
            Err(UiControlApiError::Internal)
        );
        assert_eq!(
            window_layout_snapshot_for_test(window),
            Ok(Some(first.clone()))
        );
        assert!(ui_control_layout_dirty(nested).expect("configuration dirties Layout"));
        assert_eq!(
            window_calculate_layout_for_test(window, 24.0, 12.0, true),
            Err(UiControlApiError::Internal)
        );
        assert_eq!(window_layout_snapshot_for_test(window), Ok(Some(first)));
        assert!(ui_control_layout_dirty(nested).expect("failed pass retains dirty state"));

        let second =
            window_calculate_layout_for_test(window, 24.0, 12.0, false).expect("retry succeeds");
        assert_eq!(second.layout_revision, 2);
        assert_eq!(second.tree_revision, 7);
        assert!(!ui_control_layout_dirty(nested).expect("retry clears dirty state"));

        window_close(window).expect("close Window");
        assert_eq!(window_layout_snapshot_for_test(window), Ok(None));
        assert_eq!(window_open(window), Ok(2));
        let reopened =
            window_calculate_layout_for_test(window, 24.0, 12.0, false).expect("reopened layout");
        assert_eq!(reopened.window_generation, 2);
        assert_eq!(reopened.layout_revision, 3);

        let empty_window = ui_control_create_window(runtime).expect("create empty Window");
        assert_eq!(window_open(empty_window), Ok(1));
        let empty = window_calculate_layout_for_test(empty_window, 24.0, 12.0, false)
            .expect("empty content produces complete empty snapshot");
        assert_eq!(empty.node_count(), 0);
        runtime_shutdown(runtime).expect("shutdown test Runtime");
    }

    #[test]
    fn platform_default_text_snapshot_is_shared_by_layout_and_visual_scene() {
        let probe = Arc::new(PlatformProbe::default());
        let text_service = Arc::new(TextServiceProbe::default());
        *probe.text_service.lock().expect("text service lock") = Some(text_service.clone());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with text service");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let root = ui_control_create_stack_panel(runtime).expect("create root panel");
        let text = ui_control_create_text(runtime, b"plain text").expect("create Text");
        let text_box =
            ui_control_create_text_box(runtime, b"editing value").expect("create TextBox");
        let button = ui_control_create_button(runtime, b"label").expect("create Button");
        window_set_content(window, root).expect("attach root panel");
        stack_panel_insert(root, 0, text).expect("attach Text");
        stack_panel_insert(root, 1, text_box).expect("attach TextBox");
        stack_panel_insert(root, 2, button).expect("attach Button");
        window_open(window).expect("open fake Window");

        let calculated_layout = window_calculate_layout_with_platform_text_for_test(window)
            .expect("TextService measures the attached Text");
        let (layout, tree) = window_layout_and_control_tree_with_text_for_test(window)
            .expect("capture the matching text and layout snapshots");
        assert!(Arc::ptr_eq(&calculated_layout, &layout));
        let tree = tree.expect("attached Control tree");
        let text_nodes = &tree.root.children;
        assert_eq!(text_nodes.len(), 3);
        let text_snapshots = text_nodes
            .iter()
            .map(|node| node.text.as_ref().expect("resolved text snapshot"))
            .collect::<Vec<_>>();
        assert_eq!(
            text_snapshots
                .iter()
                .map(|snapshot| snapshot.revision)
                .collect::<Vec<_>>(),
            vec![layout.layout_revision; 3]
        );
        for node in text_nodes {
            assert_eq!(
                layout
                    .node(node.identity)
                    .expect("text layout node")
                    .desired_size,
                Size::new(18.0, 20.0).expect("valid test metrics")
            );
        }
        assert_eq!(
            text_service
                .requests
                .lock()
                .expect("request list")
                .as_slice(),
            &[
                (layout.layout_revision, String::from("plain text")),
                (layout.layout_revision, String::from("editing value")),
                (layout.layout_revision, String::from("label")),
            ]
        );

        let mut descriptors = HashMap::new();
        for identity in [tree.root.identity]
            .into_iter()
            .chain(text_nodes.iter().map(|node| node.identity))
        {
            descriptors.insert(
                identity,
                VisualNodeDescriptor {
                    clips_children: false,
                    operations: Vec::new(),
                },
            );
        }
        let scene = ControlViewConnector::build_from_control_tree(
            &tree,
            &layout,
            &descriptors,
            window_device_scale_for_test(window).expect("open Window scale"),
            [1.0, 1.0, 1.0],
            crate::abi_support::Damage::Full,
        )
        .expect("build Scene from retained resolved text");
        let scene_glyphs = scene
            .operations()
            .iter()
            .filter_map(|operation| match operation {
                crate::abi_support::DrawOp::GlyphRun { run, .. } => Some(run.font_digest()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(scene_glyphs.len(), 3);
        assert!(
            scene_glyphs
                .iter()
                .zip(text_snapshots)
                .all(|(digest, snapshot)| { *digest == &snapshot.glyph_runs[0].font.asset_digest })
        );

        let property = ui_control_text_content_property(text).expect("Text content Property");
        property_set_utf8(property, b"updated text").expect("update Text content");
        assert!(ui_control_layout_dirty(window).expect("display change dirties Layout"));
        let updated_layout = window_calculate_layout_with_platform_text_for_test(window)
            .expect("resolve updated Text content");
        assert!(updated_layout.layout_revision > layout.layout_revision);
        assert!(
            text_service
                .requests
                .lock()
                .expect("request list")
                .iter()
                .any(|(_, text)| text == "updated text")
        );

        text_service.fail.store(true, Ordering::SeqCst);
        let previous = window_layout_snapshot_for_test(window)
            .expect("query Layout")
            .expect("previous Layout exists");
        assert_eq!(
            window_calculate_layout_with_platform_text_for_test(window),
            Err(UiControlApiError::Internal)
        );
        let retained = window_layout_snapshot_for_test(window)
            .expect("query Layout after failure")
            .expect("last complete Layout remains available");
        assert!(Arc::ptr_eq(&previous, &retained));
        runtime_shutdown(runtime).expect("shutdown test Runtime");
    }

    #[test]
    fn runtime_control_tree_snapshot_tracks_attach_detach_order_and_rejects_stale_layout() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(probe));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with fake peer");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let root = ui_control_create_stack_panel(runtime).expect("create content root");
        let first = ui_control_create_button(runtime, b"first").expect("create first Button");
        let second = ui_control_create_button(runtime, b"second").expect("create second Button");
        window_set_content(window, root).expect("attach content root");
        stack_panel_insert(root, 0, first).expect("attach first Button");
        assert_eq!(window_open(window), Ok(1));

        let first_layout = window_calculate_layout_for_test(window, 24.0, 12.0, false)
            .expect("calculate first LayoutSnapshot");
        let first_tree = window_control_tree_snapshot_for_test(window)
            .expect("capture first live tree")
            .expect("content root exists");
        assert_eq!(first_tree.tree_revision, first_layout.tree_revision);
        assert_eq!(first_tree.root.children.len(), 1);
        let first_identity = first_tree.root.children[0].identity;

        stack_panel_insert(root, 1, second).expect("attach second Button");
        assert_eq!(
            window_control_tree_snapshot_for_test(window),
            Err(UiControlApiError::Internal)
        );
        let attached_layout = window_calculate_layout_for_test(window, 24.0, 12.0, false)
            .expect("calculate LayoutSnapshot after attach");
        let attached_tree = window_control_tree_snapshot_for_test(window)
            .expect("capture attached tree")
            .expect("content root exists");
        assert_eq!(attached_tree.tree_revision, attached_layout.tree_revision);
        assert_eq!(attached_tree.root.children.len(), 2);
        assert_eq!(attached_tree.root.children[0].identity, first_identity);
        let second_identity = attached_tree.root.children[1].identity;

        stack_panel_move(root, 1, 0).expect("reorder attached children");
        assert_eq!(
            window_control_tree_snapshot_for_test(window),
            Err(UiControlApiError::Internal)
        );
        let reordered_layout = window_calculate_layout_for_test(window, 24.0, 12.0, false)
            .expect("calculate LayoutSnapshot after reorder");
        let reordered_tree = window_control_tree_snapshot_for_test(window)
            .expect("capture reordered tree")
            .expect("content root exists");
        assert_eq!(reordered_tree.tree_revision, reordered_layout.tree_revision);
        assert_eq!(reordered_tree.root.children[0].identity, second_identity);
        assert_eq!(reordered_tree.root.children[1].identity, first_identity);

        stack_panel_remove(root, 0).expect("detach first child");
        assert_eq!(
            window_control_tree_snapshot_for_test(window),
            Err(UiControlApiError::Internal)
        );
        let detached_layout = window_calculate_layout_for_test(window, 24.0, 12.0, false)
            .expect("calculate LayoutSnapshot after detach");
        let detached_tree = window_control_tree_snapshot_for_test(window)
            .expect("capture detached tree")
            .expect("content root exists");
        assert_eq!(detached_tree.tree_revision, detached_layout.tree_revision);
        assert_eq!(detached_tree.root.children.len(), 1);
        assert_eq!(detached_tree.root.children[0].identity, first_identity);

        window_close(window).expect("close Window");
        assert_eq!(
            window_control_tree_snapshot_for_test(window),
            Err(UiControlApiError::Internal)
        );
        runtime_shutdown(runtime).expect("shutdown test Runtime");
    }

    #[test]
    fn runtime_window_device_scale_flows_into_scene_without_scaling_dip_client_area() {
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::new(PlatformProbe::default())));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with fake peer");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let root = ui_control_create_stack_panel(runtime).expect("create content root");
        let button = ui_control_create_button(runtime, b"button").expect("create Button");
        window_set_content(window, root).expect("attach content root");
        stack_panel_insert(root, 0, button).expect("attach Button");
        assert_eq!(window_open(window), Ok(1));

        let layout = window_calculate_layout_for_test(window, 640.0, 480.0, false)
            .expect("calculate LayoutSnapshot");
        let control_tree = window_control_tree_snapshot_for_test(window)
            .expect("capture live control tree")
            .expect("content tree exists");
        let scale = window_device_scale_for_test(window).expect("Core owns open scale");
        assert_eq!(scale, DeviceScale::new(1.25, 1.5).expect("expected scale"));
        assert_eq!(layout.client_area().width, 640.0);
        assert_eq!(layout.client_area().height, 480.0);

        let descriptors = [
            (
                control_tree.root.identity,
                VisualNodeDescriptor {
                    clips_children: true,
                    operations: Vec::new(),
                },
            ),
            (
                control_tree.root.children[0].identity,
                VisualNodeDescriptor {
                    clips_children: true,
                    operations: Vec::new(),
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        let scene = ControlViewConnector::build_from_control_tree(
            &control_tree,
            &layout,
            &descriptors,
            scale,
            [1.0, 1.0, 1.0],
            Damage::Full,
        )
        .expect("build scene with Core-owned scale");
        assert_eq!(scene.device_scale(), scale);
        assert_eq!(scene.layout_revision(), layout.layout_revision);

        window_close(window).expect("close Window");
        assert_eq!(
            window_device_scale_for_test(window),
            Err(UiControlApiError::InvalidArgument)
        );
        runtime_shutdown(runtime).expect("shutdown test Runtime");
    }

    #[test]
    fn platform_render_core_completion_is_dispatched_ticketed_and_retries_full_damage() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with platform factory");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        assert_eq!(window_open(window), Ok(1));
        let sink = probe
            .completion_sink
            .lock()
            .expect("completion sink lock")
            .as_ref()
            .expect("Core completion sink")
            .clone();

        probe.fail_next_submit.store(true, Ordering::SeqCst);
        assert_eq!(
            window_submit_frame(window, partial_scene(1)),
            Err(UiControlApiError::Internal)
        );
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Idle));
        let retry = window_submit_frame(window, partial_scene(1)).expect("submit full retry");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Rendering(retry)));
        {
            let submitted = probe.submitted.lock().expect("submitted frames lock");
            assert_eq!(submitted[0].1.damage(), Damage::Full);
            assert_eq!(submitted[1].1.damage(), Damage::Full);
        }

        sink.complete_frame(retry)
            .expect("queue backend completion");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Rendering(retry)));
        runtime_dispatch_turn(runtime).expect("dispatch completion on UI thread");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Presented(retry)));

        sink.fail_frame(retry)
            .expect("queue duplicate stale failure");
        runtime_dispatch_turn(runtime).expect("dispatch stale completion");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Presented(retry)));

        let async_failure = window_submit_frame(window, partial_scene(2))
            .expect("submit frame before asynchronous failure");
        sink.fail_frame(async_failure)
            .expect("queue asynchronous backend failure");
        runtime_dispatch_turn(runtime).expect("dispatch asynchronous failure on UI thread");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Idle));

        let old_generation = window_submit_frame(window, partial_scene(2))
            .expect("submit before surface replacement");
        window_close(window).expect("close old surface");
        assert_eq!(window_frame_phase(window), Ok(FramePhase::Idle));
        assert_eq!(window_open(window), Ok(2));
        let current =
            window_submit_frame(window, partial_scene(3)).expect("submit new-generation frame");
        assert_ne!(
            old_generation.surface_generation(),
            current.surface_generation()
        );
        sink.complete_frame(old_generation)
            .expect("queue old-generation completion");
        runtime_dispatch_turn(runtime).expect("discard old-generation completion");
        assert_eq!(
            window_frame_phase(window),
            Ok(FramePhase::Rendering(current))
        );

        sink.complete_frame(current)
            .expect("queue current-generation completion");
        runtime_dispatch_turn(runtime).expect("dispatch current-generation completion");
        assert_eq!(
            window_frame_phase(window),
            Ok(FramePhase::Presented(current))
        );

        runtime_shutdown(runtime).expect("shutdown Runtime");
        assert!(sink.complete_frame(current).is_err());
        handle_release(window).expect("release Window");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn window_background_color_is_core_owned_validated_and_forwarded_to_scene() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with platform factory");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let other_window = ui_control_create_window(runtime).expect("create second Window");
        assert_eq!(window_background_rgb(window), Ok([1.0, 1.0, 1.0]));
        assert_eq!(window_background_rgb(other_window), Ok([1.0, 1.0, 1.0]));
        assert_eq!(window_open(window), Ok(1));
        let sink = probe
            .completion_sink
            .lock()
            .expect("completion sink lock")
            .as_ref()
            .expect("Core completion sink")
            .clone();

        let first = window_submit_frame(window, partial_scene(1)).expect("submit default frame");
        {
            let submitted = probe.submitted.lock().expect("submitted frames lock");
            assert_eq!(submitted[0].1.background_rgb(), [1.0, 1.0, 1.0]);
            assert_eq!(submitted[0].1.damage(), Damage::Full);
        }
        sink.complete_frame(first)
            .expect("queue default frame completion");
        runtime_dispatch_turn(runtime).expect("dispatch default frame completion");

        assert_eq!(
            window_set_background_rgb(window, f32::NAN, 0.2, 0.3),
            Err(UiControlApiError::InvalidArgument)
        );
        assert_eq!(window_background_rgb(window), Ok([1.0, 1.0, 1.0]));
        let after_invalid =
            window_submit_frame(window, partial_scene(2)).expect("submit after invalid update");
        {
            let submitted = probe.submitted.lock().expect("submitted frames lock");
            assert_eq!(submitted[1].1.background_rgb(), [1.0, 1.0, 1.0]);
            assert!(matches!(submitted[1].1.damage(), Damage::Rect(_)));
        }
        sink.complete_frame(after_invalid)
            .expect("queue post-invalid frame completion");
        runtime_dispatch_turn(runtime).expect("dispatch post-invalid frame completion");

        ui_control_layout_clean(window).expect("clear Layout before paint-only change");
        window_set_background_rgb(window, 0.2, 0.4, 0.8).expect("set Window color");
        assert!(!ui_control_layout_dirty(window).expect("background does not dirty Layout"));
        assert_eq!(window_background_rgb(window), Ok([0.2, 0.4, 0.8]));
        assert_eq!(window_background_rgb(other_window), Ok([1.0, 1.0, 1.0]));
        let explicit = window_submit_frame(window, partial_scene(3)).expect("submit colored frame");
        {
            let submitted = probe.submitted.lock().expect("submitted frames lock");
            assert_eq!(submitted[2].1.background_rgb(), [0.2, 0.4, 0.8]);
            assert_eq!(submitted[2].1.damage(), Damage::Full);
        }
        sink.complete_frame(explicit)
            .expect("queue colored frame completion");
        runtime_dispatch_turn(runtime).expect("dispatch colored frame completion");

        window_reset_background_rgb(window).expect("reset Window color");
        assert_eq!(window_background_rgb(window), Ok([1.0, 1.0, 1.0]));
        let reset = window_submit_frame(window, partial_scene(4)).expect("submit reset frame");
        assert_eq!(
            probe.submitted.lock().expect("submitted frames lock")[3]
                .1
                .background_rgb(),
            [1.0, 1.0, 1.0]
        );
        sink.complete_frame(reset)
            .expect("queue reset frame completion");
        runtime_dispatch_turn(runtime).expect("dispatch reset frame completion");

        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(window).expect("release Window");
        handle_release(other_window).expect("release second Window");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn platform_factory_is_lazy_and_window_close_preserves_core_control() {
        let probe = Arc::new(PlatformProbe::default());
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with platform factory");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");

        assert_eq!(probe.created.load(Ordering::SeqCst), 0);
        assert_eq!(probe.opened.load(Ordering::SeqCst), 0);
        assert_eq!(window_open(window), Ok(1));
        assert_eq!(probe.created.load(Ordering::SeqCst), 1);
        assert_eq!(probe.opened.load(Ordering::SeqCst), 1);
        assert_eq!(window_open(window), Ok(1));
        assert_eq!(probe.opened.load(Ordering::SeqCst), 1);

        window_close(window).expect("close only the platform peer");
        assert_eq!(probe.closed.load(Ordering::SeqCst), 1);
        let title = ui_control_window_title_property(window).expect("Core Window remains alive");
        handle_release(title).expect("release title Property");
        assert_eq!(window_open(window), Ok(2));
        assert_eq!(probe.created.load(Ordering::SeqCst), 1);
        assert_eq!(probe.opened.load(Ordering::SeqCst), 2);

        window_close(window).expect("close reopened peer");
        handle_release(window).expect("release Window");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[test]
    fn platform_factory_creation_failure_leaves_window_retryable() {
        let probe = Arc::new(PlatformProbe::default());
        probe.fail_next_create.store(true, Ordering::SeqCst);
        let factory: Arc<dyn crate::abi_support::PlatformRuntimeFactory> =
            Arc::new(TestPlatformFactory(Arc::clone(&probe)));
        let runtime = runtime_create_handle_with_platform_factory(factory)
            .expect("create Runtime with platform factory");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");

        assert_eq!(window_open(window), Err(UiControlApiError::Internal));
        assert_eq!(probe.opened.load(Ordering::SeqCst), 0);
        assert_eq!(window_open(window), Ok(1));
        assert_eq!(probe.created.load(Ordering::SeqCst), 2);
        assert_eq!(probe.opened.load(Ordering::SeqCst), 1);

        window_close(window).expect("close platform peer");
        handle_release(window).expect("release Window");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }

    #[derive(Default)]
    struct CloseRequestProbe {
        requests: std::sync::Mutex<Vec<u64>>,
        destroys: AtomicUsize,
        action: std::sync::atomic::AtomicUsize,
    }

    unsafe extern "C" fn defer_close_request(userdata: *mut c_void) {
        // SAFETY: the owning test keeps this probe alive through subscription cancellation.
        let probe = unsafe { &*userdata.cast::<CloseRequestProbe>() };
        let borrowed = window_close_request_current();
        match probe.action.load(Ordering::SeqCst) {
            0 => {
                let owned = window_close_request_defer(borrowed).expect("defer close request");
                probe
                    .requests
                    .lock()
                    .expect("request list lock")
                    .push(owned);
            }
            1 => window_close_request_resolve(borrowed, 2).expect("decline borrowed request"),
            _ => {}
        }
    }

    unsafe extern "C" fn destroy_close_probe(userdata: *mut c_void) {
        // SAFETY: the owning test keeps this probe alive through subscription cancellation.
        unsafe { &*userdata.cast::<CloseRequestProbe>() }
            .destroys
            .fetch_add(1, Ordering::SeqCst);
    }

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

    #[test]
    fn close_subscription_defers_resolves_and_declines_when_last_owner_is_released() {
        let runtime = runtime_create_handle().expect("create Runtime");
        runtime_set_ui_loop_bridge(runtime, noop_wake, 0, None).expect("install loop bridge");
        let window = ui_control_create_window(runtime).expect("create Window");
        let peer = Arc::new(TestWindowPeer(AtomicUsize::new(0), AtomicUsize::new(0)));
        window_install_test_peer(window, peer.clone()).expect("install fake peer");
        window_open(window).expect("open Window");

        let probe = CloseRequestProbe::default();
        let subscription = window_close_request_subscribe(
            window,
            defer_close_request,
            (&probe as *const CloseRequestProbe) as usize,
            destroy_close_probe,
        )
        .expect("register close callback");
        assert_eq!(
            window_close_request_subscribe(
                window,
                defer_close_request,
                (&probe as *const CloseRequestProbe) as usize,
                destroy_close_probe,
            ),
            Err(UiControlApiError::InvalidArgument)
        );

        assert_eq!(window_close_intent_subscribed(window), Ok(Some((1, 1))));
        let first = probe.requests.lock().expect("request list lock")[0];
        let first_clone =
            crate::abi_support::runtime_retain_handle(first).expect("retain deferred request");
        assert_eq!(peer.1.load(Ordering::Relaxed), 0);
        handle_release(first).expect("release one request owner");
        window_close_request_resolve(first_clone, 1).expect("resolve deferred request");
        assert_eq!(peer.1.load(Ordering::Relaxed), 1);
        handle_release(first_clone).expect("release resolved request");

        window_open(window).expect("reopen Window");
        assert_eq!(window_close_intent_subscribed(window), Ok(Some((2, 2))));
        let second = probe.requests.lock().expect("request list lock")[1];
        let second_clone = crate::abi_support::runtime_retain_handle(second)
            .expect("retain second deferred request");
        handle_release(second).expect("release unresolved request as Decline");
        assert_eq!(peer.1.load(Ordering::SeqCst), 1);
        handle_release(second_clone).expect("last owner declines unresolved request");
        assert_eq!(peer.1.load(Ordering::SeqCst), 1);
        window_close(window).expect("close Window after declined intent");
        assert_eq!(peer.1.load(Ordering::SeqCst), 2);

        window_open(window).expect("reopen Window for immediate Decline");
        probe.action.store(1, Ordering::SeqCst);
        assert_eq!(window_close_intent_subscribed(window), Ok(None));
        assert_eq!(peer.1.load(Ordering::SeqCst), 2);
        assert_eq!(window_open(window), Ok(3));

        probe.action.store(2, Ordering::SeqCst);
        assert_eq!(window_close_intent_subscribed(window), Ok(None));
        assert_eq!(peer.1.load(Ordering::SeqCst), 3);

        crate::abi_support::subscription_cancel(subscription).expect("cancel subscription");
        assert_eq!(probe.destroys.load(Ordering::SeqCst), 1);
        handle_release(subscription).expect("release subscription token");
        handle_release(window).expect("release Window");
        runtime_shutdown(runtime).expect("shutdown Runtime");
        handle_release(runtime).expect("release Runtime");
    }
}
