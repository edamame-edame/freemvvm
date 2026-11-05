use crate::handle_registry::registry::ObjectIdentity;
use crate::runtime::RuntimeIdentity;
use crate::ui_tree::{RuntimeTreeStore, TreeError};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum InputEventKind {
    KeyDown = 1,
    KeyUp = 2,
    TextCommit = 3,
    PrimaryPointerDown = 4,
    PrimaryPointerUp = 5,
}

impl InputEventKind {
    pub(crate) const fn mask_bit(self) -> u64 {
        1_u64 << (self as u32 - 1)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InputEventPayload {
    Key { identity: u32, modifiers: u32 },
    TextCommit(Arc<str>),
    PrimaryPointer { x_dip: f64, y_dip: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InputEvent {
    pub(crate) kind: InputEventKind,
    pub(crate) payload: InputEventPayload,
}

impl InputEvent {
    pub(crate) fn validate(&self) -> Result<(), InputEventError> {
        match (self.kind, &self.payload) {
            (InputEventKind::KeyDown | InputEventKind::KeyUp, InputEventPayload::Key { .. })
            | (InputEventKind::TextCommit, InputEventPayload::TextCommit(_)) => Ok(()),
            (
                InputEventKind::PrimaryPointerDown | InputEventKind::PrimaryPointerUp,
                InputEventPayload::PrimaryPointer { x_dip, y_dip },
            ) if x_dip.is_finite() && y_dip.is_finite() => Ok(()),
            (
                InputEventKind::PrimaryPointerDown | InputEventKind::PrimaryPointerUp,
                InputEventPayload::PrimaryPointer { .. },
            ) => Err(InputEventError::InvalidGeometry),
            _ => Err(InputEventError::PayloadKindMismatch),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputEventError {
    PayloadKindMismatch,
    InvalidGeometry,
}

/// Mutable only during one synchronous handler callback; the underlying payload is immutable.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InputEventFrame {
    pub(crate) event: InputEvent,
    pub(crate) original_target: ObjectIdentity,
    handled: bool,
}

impl InputEventFrame {
    pub(crate) fn new(event: InputEvent, original_target: ObjectIdentity) -> Self {
        Self {
            event,
            original_target,
            handled: false,
        }
    }

    pub(crate) fn is_handled(&self) -> bool {
        self.handled
    }

    /// Handled is monotonic for one event. Passing false never clears an earlier handler's value.
    pub(crate) fn set_handled(&mut self, handled: bool) {
        self.handled |= handled;
    }
}

/// A target-to-Window route captured before invoking user handlers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InputRouteSnapshot {
    window: ObjectIdentity,
    original_target: ObjectIdentity,
    route: Vec<ObjectIdentity>,
}

impl InputRouteSnapshot {
    pub(crate) fn from_tree(
        tree: &RuntimeTreeStore<RuntimeIdentity>,
        window: ObjectIdentity,
        target: ObjectIdentity,
    ) -> Result<Self, InputRouteError> {
        Self::from_parent_lookup(window, target, |identity| {
            tree.parent_identity(identity)
                .map_err(InputRouteError::Tree)
        })
    }

    fn from_parent_lookup(
        window: ObjectIdentity,
        target: ObjectIdentity,
        mut parent_of: impl FnMut(ObjectIdentity) -> Result<Option<ObjectIdentity>, InputRouteError>,
    ) -> Result<Self, InputRouteError> {
        let mut route = Vec::new();
        let mut current = target;
        loop {
            route
                .try_reserve(1)
                .map_err(|_| InputRouteError::AllocationFailed)?;
            route.push(current);
            if current == window {
                break;
            }
            current = parent_of(current)?.ok_or(InputRouteError::TargetOutsideWindow)?;
        }
        Ok(Self {
            window,
            original_target: target,
            route,
        })
    }

    pub(crate) fn window(&self) -> ObjectIdentity {
        self.window
    }

    pub(crate) fn original_target(&self) -> ObjectIdentity {
        self.original_target
    }

    pub(crate) fn nodes(&self) -> &[ObjectIdentity] {
        &self.route
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputRouteError {
    TargetOutsideWindow,
    AllocationFailed,
    WrongThread,
    Closed,
    SynchronizationPoisoned,
    Tree(TreeError),
}

#[cfg(test)]
mod tests {
    use super::{
        InputEvent, InputEventFrame, InputEventKind, InputEventPayload, InputRouteError,
        InputRouteSnapshot,
    };
    use crate::handle_registry::registry::ObjectIdentity;
    use std::collections::HashMap;
    use std::num::NonZeroU64;

    fn identity(slot: u64) -> ObjectIdentity {
        ObjectIdentity::new(slot, NonZeroU64::new(1).expect("nonzero generation"))
    }

    #[test]
    fn input_frame_validates_payload_and_handled_only_moves_forward() {
        let target = identity(3);
        let key = InputEvent {
            kind: InputEventKind::KeyDown,
            payload: InputEventPayload::Key {
                identity: 0x04,
                modifiers: 0,
            },
        };
        assert_eq!(key.validate(), Ok(()));
        let mut frame = InputEventFrame::new(key, target);
        assert!(!frame.is_handled());
        frame.set_handled(true);
        frame.set_handled(false);
        assert!(frame.is_handled());

        assert_eq!(
            InputEvent {
                kind: InputEventKind::TextCommit,
                payload: InputEventPayload::Key {
                    identity: 0,
                    modifiers: 0,
                },
            }
            .validate(),
            Err(super::InputEventError::PayloadKindMismatch)
        );
        assert_eq!(
            InputEvent {
                kind: InputEventKind::PrimaryPointerDown,
                payload: InputEventPayload::PrimaryPointer {
                    x_dip: f64::NAN,
                    y_dip: 0.0,
                },
            }
            .validate(),
            Err(super::InputEventError::InvalidGeometry)
        );
    }

    #[test]
    fn input_route_snapshot_runs_from_target_through_logical_ancestors_to_window() {
        let parents = HashMap::from([(identity(2), identity(1)), (identity(3), identity(2))]);
        let route = InputRouteSnapshot::from_parent_lookup(identity(1), identity(3), |node| {
            Ok(parents.get(&node).copied())
        })
        .expect("capture attached route");
        assert_eq!(route.window(), identity(1));
        assert_eq!(route.original_target(), identity(3));
        assert_eq!(route.nodes(), &[identity(3), identity(2), identity(1)]);
        assert_eq!(
            InputRouteSnapshot::from_parent_lookup(identity(1), identity(4), |node| {
                Ok(parents.get(&node).copied())
            }),
            Err(InputRouteError::TargetOutsideWindow)
        );
    }
}
