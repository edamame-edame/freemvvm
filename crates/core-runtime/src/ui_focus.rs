use crate::handle_registry::registry::ObjectIdentity;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FocusManager {
    active_window: Option<ObjectIdentity>,
    focused: Option<ObjectIdentity>,
    restore_candidate: HashMap<ObjectIdentity, ObjectIdentity>,
}

impl FocusManager {
    pub(crate) fn active_window(&self) -> Option<ObjectIdentity> {
        self.active_window
    }

    pub(crate) fn focused(&self) -> Option<ObjectIdentity> {
        self.focused
    }

    pub(crate) fn request(&mut self, window: ObjectIdentity, target: ObjectIdentity) {
        if self.active_window == Some(window) && self.focused != Some(target) {
            self.focused = Some(target);
            self.restore_candidate.remove(&window);
        }
    }

    pub(crate) fn activate(&mut self, window: ObjectIdentity, restored: Option<ObjectIdentity>) {
        if let Some(previous) = self.active_window.filter(|previous| *previous != window) {
            if let Some(focused) = self.focused.take() {
                self.restore_candidate.insert(previous, focused);
            }
        }
        self.active_window = Some(window);
        self.focused = restored;
    }

    pub(crate) fn deactivate(&mut self, window: ObjectIdentity) {
        if self.active_window != Some(window) {
            return;
        }
        if let Some(focused) = self.focused.take() {
            self.restore_candidate.insert(window, focused);
        }
        self.active_window = None;
    }

    pub(crate) fn candidate(&self, window: ObjectIdentity) -> Option<ObjectIdentity> {
        self.restore_candidate.get(&window).copied()
    }

    pub(crate) fn clear_detached_subtree(
        &mut self,
        root: ObjectIdentity,
        mut is_in_subtree: impl FnMut(ObjectIdentity) -> bool,
    ) {
        if self
            .focused
            .is_some_and(|identity| identity == root || is_in_subtree(identity))
        {
            self.focused = None;
        }
        self.restore_candidate
            .retain(|_, candidate| !(*candidate == root || is_in_subtree(*candidate)));
    }

    pub(crate) fn clear_for_window(&mut self, window: ObjectIdentity) {
        if self.active_window == Some(window) {
            self.active_window = None;
            self.focused = None;
        }
        self.restore_candidate.remove(&window);
    }
}
