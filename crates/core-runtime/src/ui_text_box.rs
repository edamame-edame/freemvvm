//! Core-owned editing state for TextBox controls.

use crate::property::ValidationError;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // INCOMPLETE is consumed by the later View Input Validator unit.
pub(crate) enum TextInputStatus {
    Accept,
    Incomplete,
    Reject,
}

/// Immutable view of the current editing state. Positions are UTF-8 byte offsets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextSnapshot {
    pub(crate) text: Arc<str>,
    pub(crate) anchor: usize,
    pub(crate) caret: usize,
    pub(crate) status: TextInputStatus,
    pub(crate) reason: Option<Arc<ValidationError>>,
    pub(crate) revision: u64,
}

/// A fully prepared replacement. Committing it only swaps the immutable snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedTextEdit {
    snapshot: Arc<TextSnapshot>,
}

impl PreparedTextEdit {
    pub(crate) fn snapshot(&self) -> &TextSnapshot {
        &self.snapshot
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TextEditError {
    InvalidBoundary,
    RevisionExhausted,
    AllocationFailed,
}

pub(crate) struct TextBoxEditingState {
    snapshot: Arc<TextSnapshot>,
    committed_value: Arc<str>,
}

impl TextBoxEditingState {
    pub(crate) fn new(initial: &str) -> Result<Self, TextEditError> {
        let text = copy_str(initial)?;
        let end = text.len();
        Ok(Self {
            snapshot: Arc::new(TextSnapshot {
                text: Arc::clone(&text),
                anchor: end,
                caret: end,
                status: TextInputStatus::Accept,
                reason: None,
                revision: 0,
            }),
            committed_value: text,
        })
    }

    pub(crate) fn snapshot(&self) -> &Arc<TextSnapshot> {
        &self.snapshot
    }

    pub(crate) fn prepare_replace(
        &self,
        start: usize,
        end: usize,
        inserted: &str,
    ) -> Result<PreparedTextEdit, TextEditError> {
        let current = &self.snapshot.text;
        if start > end
            || end > current.len()
            || !current.is_char_boundary(start)
            || !current.is_char_boundary(end)
        {
            return Err(TextEditError::InvalidBoundary);
        }
        let next_len = current
            .len()
            .checked_sub(end - start)
            .and_then(|len| len.checked_add(inserted.len()))
            .ok_or(TextEditError::AllocationFailed)?;
        let mut text = String::new();
        text.try_reserve_exact(next_len)
            .map_err(|_| TextEditError::AllocationFailed)?;
        text.push_str(&current[..start]);
        text.push_str(inserted);
        text.push_str(&current[end..]);
        let caret = start + inserted.len();
        let revision = self
            .snapshot
            .revision
            .checked_add(1)
            .ok_or(TextEditError::RevisionExhausted)?;
        Ok(PreparedTextEdit {
            snapshot: Arc::new(TextSnapshot {
                text: Arc::from(text),
                anchor: caret,
                caret,
                status: TextInputStatus::Accept,
                reason: None,
                revision,
            }),
        })
    }

    pub(crate) fn commit_accepted(&mut self, prepared: PreparedTextEdit) {
        self.committed_value = Arc::clone(&prepared.snapshot.text);
        self.snapshot = prepared.snapshot;
    }

    /// Keeps a candidate visible when the TextBox Value Property Validator rejects it.
    pub(crate) fn retain_property_rejected_candidate(
        &mut self,
        prepared: PreparedTextEdit,
        reason: Option<Arc<ValidationError>>,
    ) {
        let mut snapshot = (*prepared.snapshot).clone();
        snapshot.status = TextInputStatus::Reject;
        snapshot.reason = reason;
        self.snapshot = Arc::new(snapshot);
    }

    /// Replaces editing state after an externally committed, unequal Value update.
    pub(crate) fn sync_external(&mut self, value: &str) -> Result<bool, TextEditError> {
        if self.committed_value.as_ref() == value {
            return Ok(false);
        }
        let text = copy_str(value)?;
        let end = text.len();
        let revision = self
            .snapshot
            .revision
            .checked_add(1)
            .ok_or(TextEditError::RevisionExhausted)?;
        self.snapshot = Arc::new(TextSnapshot {
            text: Arc::clone(&text),
            anchor: end,
            caret: end,
            status: TextInputStatus::Accept,
            reason: None,
            revision,
        });
        self.committed_value = text;
        Ok(true)
    }
}

fn copy_str(value: &str) -> Result<Arc<str>, TextEditError> {
    let mut copy = String::new();
    copy.try_reserve_exact(value.len())
        .map_err(|_| TextEditError::AllocationFailed)?;
    copy.push_str(value);
    Ok(Arc::from(copy))
}

#[cfg(test)]
mod tests {
    use super::{TextBoxEditingState, TextEditError, TextInputStatus, ValidationError};

    #[test]
    fn text_box_editing_initial_state() {
        for initial in ["", "hello", "日本語🙂"] {
            let editing = TextBoxEditingState::new(initial).expect("create editing state");
            let snapshot = editing.snapshot();
            assert_eq!(snapshot.text.as_ref(), initial);
            assert_eq!(snapshot.anchor, initial.len());
            assert_eq!(snapshot.caret, initial.len());
            assert_eq!(snapshot.revision, 0);
        }
    }

    #[test]
    fn text_box_editing_candidate_transaction() {
        let mut editing = TextBoxEditingState::new("ab日本語cd").expect("create state");
        let initial = Arc::clone(&editing.snapshot().text);
        let start = "ab日".len();
        let end = "ab日本語".len();
        let prepared = editing
            .prepare_replace(start, end, "🙂")
            .expect("prepare replacement");
        assert_eq!(editing.snapshot().text.as_ref(), "ab日本語cd");
        assert_eq!(initial.as_ref(), "ab日本語cd");
        editing.commit_accepted(prepared);
        assert_eq!(editing.snapshot().text.as_ref(), "ab日🙂cd");
        assert_eq!(editing.snapshot().caret, start + "🙂".len());
        assert_eq!(editing.snapshot().anchor, editing.snapshot().caret);
        assert_eq!(editing.snapshot().revision, 1);
        assert_eq!(
            editing.prepare_replace(start + 1, end, "x"),
            Err(TextEditError::InvalidBoundary)
        );
    }

    #[test]
    fn text_box_editing_property_validation() {
        let mut editing = TextBoxEditingState::new("old").expect("create state");
        let rejected = editing
            .prepare_replace(0, 3, "candidate")
            .expect("prepare candidate");
        let rejection = Arc::new(ValidationError {
            status: 11,
            domain: 2,
            code: 1,
            message: b"rejected".to_vec(),
        });
        editing.retain_property_rejected_candidate(rejected, Some(Arc::clone(&rejection)));
        assert_eq!(editing.snapshot().text.as_ref(), "candidate");
        assert_eq!(editing.snapshot().status, TextInputStatus::Reject);
        assert_eq!(
            editing.snapshot().reason.as_deref(),
            Some(rejection.as_ref())
        );
        assert_eq!(editing.committed_value.as_ref(), "old");
        assert!(
            !editing
                .sync_external("old")
                .expect("sync equal committed value")
        );
        assert_eq!(editing.snapshot().text.as_ref(), "candidate");
        // The next edit is independently validated; accepted candidates advance both values.
        let accepted = editing
            .prepare_replace(0, editing.snapshot().text.len(), "accepted")
            .expect("prepare accepted candidate");
        editing.commit_accepted(accepted);
        assert_eq!(editing.snapshot().text.as_ref(), "accepted");
        assert_eq!(editing.committed_value.as_ref(), "accepted");
    }

    #[test]
    fn text_box_editing_external_value_sync() {
        let mut editing = TextBoxEditingState::new("initial").expect("create state");
        let prepared = editing
            .prepare_replace(0, 7, "editing")
            .expect("prepare editing text");
        editing.commit_accepted(prepared);
        assert!(!editing.sync_external("editing").expect("sync equal value"));
        assert_eq!(editing.snapshot().text.as_ref(), "editing");
        assert!(
            editing
                .sync_external("external")
                .expect("sync external value")
        );
        assert_eq!(editing.snapshot().text.as_ref(), "external");
        assert_eq!(editing.snapshot().caret, "external".len());
        assert!(editing.snapshot().reason.is_none());
    }

    use std::sync::Arc;
}
