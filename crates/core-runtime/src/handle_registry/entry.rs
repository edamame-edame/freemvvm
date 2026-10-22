/// One public owner reference plus the metadata used to validate its handle.
///
/// The generic parameters deliberately leave concrete kind, Runtime identity, and
/// object-reference representations to their owning subsystems.
#[allow(dead_code)] // Consumed by the synchronized registry integration slice.
pub(crate) struct HandleEntry<K, R, O> {
    kind: K,
    runtime_id: Option<R>,
    object_ref: Option<O>,
}

#[allow(dead_code)]
impl<K, R, O> HandleEntry<K, R, O> {
    pub(crate) fn new(kind: K, runtime_id: Option<R>, object_ref: O) -> Self {
        Self {
            kind,
            runtime_id,
            object_ref: Some(object_ref),
        }
    }

    pub(crate) fn is_live(&self) -> bool {
        self.object_ref.is_some()
    }

    pub(crate) fn kind(&self) -> &K {
        &self.kind
    }

    pub(crate) fn runtime_id(&self) -> Option<&R> {
        self.runtime_id.as_ref()
    }

    pub(crate) fn object_ref(&self) -> Option<&O> {
        self.object_ref.as_ref()
    }

    /// Consumes this entry's public owner reference once and returns it to the caller.
    pub(crate) fn consume_reference(&mut self) -> Option<O> {
        self.object_ref.take()
    }
}

#[allow(dead_code)]
impl<K: PartialEq, R, O> HandleEntry<K, R, O> {
    pub(crate) fn matches_kind(&self, expected: &K) -> bool {
        &self.kind == expected
    }
}

#[allow(dead_code)]
impl<K, R: PartialEq, O> HandleEntry<K, R, O> {
    pub(crate) fn matches_runtime(&self, expected: &R) -> bool {
        self.runtime_id
            .as_ref()
            .is_some_and(|runtime_id| runtime_id == expected)
    }
}

#[cfg(test)]
mod tests {
    use super::HandleEntry;
    use std::sync::Arc;

    #[test]
    fn handle_entry_matches_kind_and_runtime_without_consuming_owner() {
        let object = Arc::new(());
        let entry = HandleEntry::new(3_u8, Some(11_u64), Arc::clone(&object));

        assert!(entry.matches_kind(&3));
        assert!(!entry.matches_kind(&4));
        assert!(entry.matches_runtime(&11));
        assert!(!entry.matches_runtime(&12));
        assert!(entry.is_live());
        assert!(Arc::ptr_eq(
            entry.object_ref().expect("live owner"),
            &object
        ));

        let runtime_less_entry = HandleEntry::new(2_u8, None::<u64>, Arc::clone(&object));
        assert!(!runtime_less_entry.matches_runtime(&11));
        assert!(runtime_less_entry.is_live());
    }

    #[test]
    fn handle_entry_consumes_one_owner_reference_once() {
        let object = Arc::new(());
        let mut first = HandleEntry::new(3_u8, Some(11_u64), Arc::clone(&object));
        let mut second = HandleEntry::new(3_u8, Some(11_u64), Arc::clone(&object));

        let first_owner = first.consume_reference().expect("first owner reference");
        assert!(!first.is_live());
        assert!(first.consume_reference().is_none());
        assert!(second.is_live());
        assert!(Arc::ptr_eq(
            &first_owner,
            second.object_ref().expect("independent second owner")
        ));

        drop(first_owner);
        assert_eq!(Arc::strong_count(&object), 2);
        assert!(second.consume_reference().is_some());
        assert!(!second.is_live());
    }
}
