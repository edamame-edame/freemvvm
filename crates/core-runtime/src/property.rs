use crate::value::{TypedValue, ValueCommit, ValueKindMismatch, ValueStorage};

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

impl PropertyState {
    pub(crate) fn new(initial: TypedValue) -> Self {
        Self {
            value: ValueStorage::new(initial),
        }
    }

    pub(crate) fn read(&self) -> &TypedValue {
        self.value.read()
    }

    pub(crate) fn apply_validated(
        &mut self,
        candidate: TypedValue,
    ) -> Result<ValueCommit, ValueKindMismatch> {
        self.value.apply_validated(candidate)
    }
}
