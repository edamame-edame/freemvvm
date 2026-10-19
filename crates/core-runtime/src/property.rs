use crate::handle_registry::registry::ObjectIdentity;
use crate::value::{TypedValue, ValueCommit, ValueKind, ValueKindMismatch, ValueStorage};

/// Kind stored in the internal Runtime handle registry for an initial Property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PropertyHandleKind {
    Property,
    #[cfg(test)]
    Other,
}

/// Runtime-owned state for one Property with a fixed initial value kind.
pub(crate) struct PropertyState {
    value: ValueStorage,
}

/// Runtime-local first-change-ordered set for the next pending Property generation.
#[derive(Default)]
pub(crate) struct PropertyChangeQueue {
    pending: Vec<ObjectIdentity>,
}

impl PropertyChangeQueue {
    /// Reserves one queue slot before the corresponding Property value is committed.
    pub(crate) fn reserve_change(
        &mut self,
        identity: ObjectIdentity,
    ) -> Result<bool, std::collections::TryReserveError> {
        if self.pending.contains(&identity) {
            return Ok(false);
        }
        self.pending.try_reserve(1)?;
        Ok(true)
    }

    /// Adds a previously reserved identity without allocating.
    pub(crate) fn commit_change(&mut self, identity: ObjectIdentity) {
        if !self.pending.contains(&identity) {
            self.pending.push(identity);
        }
    }

    /// Starts the next generation; subsequent changes are collected independently.
    pub(crate) fn take_generation(&mut self) -> Vec<ObjectIdentity> {
        std::mem::take(&mut self.pending)
    }
}

impl PropertyState {
    pub(crate) fn new(initial: TypedValue) -> Self {
        Self {
            value: ValueStorage::new(initial),
        }
    }

    pub(crate) fn read(&self) -> &TypedValue {
        self.value.read()
    }

    pub(crate) fn kind(&self) -> ValueKind {
        self.value.read().kind()
    }

    pub(crate) fn would_change(&self, candidate: &TypedValue) -> Result<bool, ValueKindMismatch> {
        self.value.would_change(candidate)
    }

    pub(crate) fn apply_validated(
        &mut self,
        candidate: TypedValue,
    ) -> Result<ValueCommit, ValueKindMismatch> {
        self.value.apply_validated(candidate)
    }
}
