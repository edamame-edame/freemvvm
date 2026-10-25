use crate::handle_registry::HandleToken;
use crate::handle_registry::registry::{
    HandleRegistry, ObjectIdentity, RegistryError, TreeOwner, ValidationPin,
};
use crate::runtime::UiThreadContext;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum TreeError {
    InvalidIdentity,
    RuntimeMismatch,
    ParentNotContainer,
    AlreadyParented,
    Cycle,
    PeerPreparationFailed,
    PreparedStateChanged,
    ChildNotFound,
    AllocationFailed,
    RevisionExhausted,
    Registry(RegistryError),
    WrongThread,
}

impl From<RegistryError> for TreeError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}

struct ChildLink<R> {
    identity: ObjectIdentity,
    _owner: TreeOwner<R>,
}

/// Core-side logical tree state. The parent identity is non-owning; each child link owns once.
#[allow(dead_code)]
pub(crate) struct LogicalNode<R> {
    identity: ObjectIdentity,
    runtime_id: R,
    is_container: bool,
    ui_thread: UiThreadContext,
    parent: Option<ObjectIdentity>,
    children: Vec<ChildLink<R>>,
    revision: u64,
}

#[allow(dead_code)]
impl<R: Clone> LogicalNode<R> {
    pub(crate) fn new(
        pin: &ValidationPin<R>,
        is_container: bool,
        ui_thread: UiThreadContext,
    ) -> Self {
        Self {
            identity: pin.identity(),
            runtime_id: pin.runtime_id().clone(),
            is_container,
            ui_thread,
            parent: None,
            children: Vec::new(),
            revision: 0,
        }
    }

    pub(crate) fn identity(&self) -> ObjectIdentity {
        self.identity
    }

    pub(crate) fn parent(&self) -> Option<ObjectIdentity> {
        self.parent
    }

    pub(crate) fn children(&self) -> impl Iterator<Item = ObjectIdentity> + '_ {
        self.children.iter().map(|child| child.identity)
    }
}

#[allow(dead_code)]
pub(crate) struct PreparedAttach<R> {
    parent: ObjectIdentity,
    parent_revision: u64,
    child: ObjectIdentity,
    child_revision: u64,
    index: usize,
    owner: TreeOwner<R>,
}

/// Prepares and commits Logical Tree membership without exposing a partial parent/child link.
#[allow(dead_code)]
pub(crate) struct TreeCoordinator;

/// Runtime-owned index of the Logical Nodes that participate in a UI tree.
/// The index is UI-thread-affine; callers may wrap it in a mutex only to share cleanup hooks.
pub(crate) struct RuntimeTreeStore<R> {
    ui_thread: UiThreadContext,
    nodes: HashMap<ObjectIdentity, LogicalNode<R>>,
}

impl<R> RuntimeTreeStore<R> {
    pub(crate) fn new(ui_thread: UiThreadContext) -> Self {
        Self {
            ui_thread,
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, node: LogicalNode<R>) -> Result<(), TreeError> {
        self.ensure_ui_thread()?;
        if self.nodes.contains_key(&node.identity) {
            return Err(TreeError::InvalidIdentity);
        }
        self.nodes
            .try_reserve(1)
            .map_err(|_| TreeError::AllocationFailed)?;
        self.nodes.insert(node.identity, node);
        Ok(())
    }

    pub(crate) fn attach<F>(
        &mut self,
        parent_pin: &ValidationPin<R>,
        child_pin: &ValidationPin<R>,
        prepare_peer: F,
    ) -> Result<(), TreeError>
    where
        R: Clone + Eq + Send + Sync + 'static,
        F: FnOnce() -> Result<(), ()>,
    {
        let parent = self
            .nodes
            .get(&parent_pin.identity())
            .ok_or(TreeError::InvalidIdentity)?;
        self.attach_at(parent_pin, child_pin, parent.children.len(), prepare_peer)
    }

    pub(crate) fn attach_at<F>(
        &mut self,
        parent_pin: &ValidationPin<R>,
        child_pin: &ValidationPin<R>,
        index: usize,
        prepare_peer: F,
    ) -> Result<(), TreeError>
    where
        R: Clone + Eq + Send + Sync + 'static,
        F: FnOnce() -> Result<(), ()>,
    {
        self.ensure_ui_thread()?;
        let parent_id = parent_pin.identity();
        let child_id = child_pin.identity();
        if parent_id == child_id {
            return Err(TreeError::Cycle);
        }
        let ancestry = self.parent_ancestry(parent_id)?;
        let mut parent = self
            .nodes
            .remove(&parent_id)
            .ok_or(TreeError::InvalidIdentity)?;
        let Some(mut child) = self.nodes.remove(&child_id) else {
            self.nodes.insert(parent_id, parent);
            return Err(TreeError::InvalidIdentity);
        };

        let result = TreeCoordinator::prepare_attach(
            &parent,
            parent_pin,
            &child,
            child_pin,
            &ancestry,
            index,
            prepare_peer,
        )
        .and_then(|prepared| TreeCoordinator::commit_attach(&mut parent, &mut child, prepared));

        self.nodes.insert(parent_id, parent);
        self.nodes.insert(child_id, child);
        result
    }

    pub(crate) fn replace_single_child<F>(
        &mut self,
        parent_pin: &ValidationPin<R>,
        child_pin: &ValidationPin<R>,
        prepare_peer: F,
    ) -> Result<Option<TreeOwner<R>>, TreeError>
    where
        R: Clone + Eq + Send + Sync + 'static,
        F: FnOnce() -> Result<(), ()>,
    {
        self.ensure_ui_thread()?;
        let parent_id = parent_pin.identity();
        let child_id = child_pin.identity();
        if parent_id == child_id {
            return Err(TreeError::Cycle);
        }
        let ancestry = self.parent_ancestry(parent_id)?;
        let mut parent = self
            .nodes
            .remove(&parent_id)
            .ok_or(TreeError::InvalidIdentity)?;
        if parent.children.len() > 1 {
            self.nodes.insert(parent_id, parent);
            return Err(TreeError::InvalidIdentity);
        }

        let Some(mut child) = self.nodes.remove(&child_id) else {
            self.nodes.insert(parent_id, parent);
            return Err(TreeError::InvalidIdentity);
        };
        let old_identity = parent.children.first().map(|edge| edge.identity);
        if old_identity == Some(child_id) {
            self.nodes.insert(parent_id, parent);
            self.nodes.insert(child_id, child);
            return Ok(None);
        }

        let mut old_child = if let Some(identity) = old_identity {
            match self.nodes.remove(&identity) {
                Some(node) => Some(node),
                None => {
                    self.nodes.insert(parent_id, parent);
                    self.nodes.insert(child_id, child);
                    return Err(TreeError::InvalidIdentity);
                }
            }
        } else {
            None
        };
        let old_child_revision = match old_child.as_ref() {
            Some(node) if node.parent == Some(parent_id) => match node.revision.checked_add(1) {
                Some(revision) => Some(revision),
                None => {
                    self.nodes.insert(parent_id, parent);
                    self.nodes.insert(child_id, child);
                    self.nodes.insert(
                        old_identity.expect("old child identity"),
                        old_child.take().expect("old child node"),
                    );
                    return Err(TreeError::RevisionExhausted);
                }
            },
            Some(_) => {
                self.nodes.insert(parent_id, parent);
                self.nodes.insert(child_id, child);
                self.nodes.insert(
                    old_identity.expect("old child identity"),
                    old_child.take().expect("old child node"),
                );
                return Err(TreeError::InvalidIdentity);
            }
            None => None,
        };
        let final_parent_revision = match parent.revision.checked_add(1) {
            Some(revision) => revision,
            None => {
                self.nodes.insert(parent_id, parent);
                self.nodes.insert(child_id, child);
                if let Some(old) = old_child.take() {
                    self.nodes.insert(old.identity, old);
                }
                return Err(TreeError::RevisionExhausted);
            }
        };

        let prepared = match TreeCoordinator::prepare_attach(
            &parent,
            parent_pin,
            &child,
            child_pin,
            &ancestry,
            0,
            prepare_peer,
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.nodes.insert(parent_id, parent);
                self.nodes.insert(child_id, child);
                if let Some(old) = old_child.take() {
                    self.nodes.insert(old.identity, old);
                }
                return Err(error);
            }
        };
        if let Err(error) = TreeCoordinator::commit_attach(&mut parent, &mut child, prepared) {
            self.nodes.insert(parent_id, parent);
            self.nodes.insert(child_id, child);
            if let Some(old) = old_child.take() {
                self.nodes.insert(old.identity, old);
            }
            return Err(error);
        }

        let removed_owner = if let Some(old) = old_child.as_mut() {
            let old_link = parent.children.remove(1);
            old.parent = None;
            old.revision = old_child_revision.expect("old child revision was checked");
            debug_assert_eq!(parent.revision, final_parent_revision);
            Some(old_link._owner)
        } else {
            None
        };
        self.nodes.insert(parent_id, parent);
        self.nodes.insert(child_id, child);
        if let (Some(identity), Some(old)) = (old_identity, old_child) {
            self.nodes.insert(identity, old);
        }
        Ok(removed_owner)
    }

    pub(crate) fn detach(
        &mut self,
        parent: ObjectIdentity,
        child: ObjectIdentity,
    ) -> Result<TreeOwner<R>, TreeError> {
        self.ensure_ui_thread()?;
        let mut parent_node = self
            .nodes
            .remove(&parent)
            .ok_or(TreeError::InvalidIdentity)?;
        let Some(mut child_node) = self.nodes.remove(&child) else {
            self.nodes.insert(parent, parent_node);
            return Err(TreeError::InvalidIdentity);
        };
        let result = TreeCoordinator::detach(&mut parent_node, &mut child_node);
        self.nodes.insert(parent, parent_node);
        self.nodes.insert(child, child_node);
        result
    }

    pub(crate) fn child_handle<K>(
        &self,
        registry: &HandleRegistry<K, R>,
        parent: ObjectIdentity,
        child: ObjectIdentity,
        kind: K,
    ) -> Result<HandleToken, TreeError>
    where
        K: Clone + Eq + Send + Sync,
        R: Clone + Eq + Send + Sync + 'static,
    {
        self.ensure_ui_thread()?;
        let parent_node = self.nodes.get(&parent).ok_or(TreeError::InvalidIdentity)?;
        TreeCoordinator::child_handle(registry, parent_node, child, kind)
    }

    pub(crate) fn child_count(&self, parent: ObjectIdentity) -> Result<usize, TreeError> {
        self.ensure_ui_thread()?;
        self.nodes
            .get(&parent)
            .map(|node| node.children.len())
            .ok_or(TreeError::InvalidIdentity)
    }

    pub(crate) fn child_identity_at(
        &self,
        parent: ObjectIdentity,
        index: usize,
    ) -> Result<ObjectIdentity, TreeError> {
        self.ensure_ui_thread()?;
        self.nodes
            .get(&parent)
            .ok_or(TreeError::InvalidIdentity)?
            .children
            .get(index)
            .map(|child| child.identity)
            .ok_or(TreeError::ChildNotFound)
    }

    pub(crate) fn move_child(
        &mut self,
        parent: ObjectIdentity,
        from_index: usize,
        to_index: usize,
    ) -> Result<(), TreeError> {
        self.ensure_ui_thread()?;
        let node = self
            .nodes
            .get_mut(&parent)
            .ok_or(TreeError::InvalidIdentity)?;
        if from_index >= node.children.len() || to_index >= node.children.len() {
            return Err(TreeError::ChildNotFound);
        }
        if from_index == to_index {
            return Ok(());
        }
        let revision = node
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;
        let child = node.children.remove(from_index);
        node.children.insert(to_index, child);
        node.revision = revision;
        Ok(())
    }

    /// Removes a node during its UI-state finalizer and detaches surviving child subtrees.
    pub(crate) fn remove_for_cleanup(
        &mut self,
        identity: ObjectIdentity,
    ) -> Result<Option<LogicalNode<R>>, TreeError> {
        self.ensure_ui_thread()?;
        let Some(parent) = self.nodes.remove(&identity) else {
            return Ok(None);
        };
        for index in 0..parent.children.len() {
            let child_identity = parent.children[index].identity;
            if let Some(mut child) = self.nodes.remove(&child_identity) {
                if child.parent == Some(identity) {
                    child.parent = None;
                    child.revision = child.revision.saturating_add(1);
                } else {
                    debug_assert!(false, "tree child has an inconsistent parent identity");
                }
                self.nodes.insert(child_identity, child);
            } else {
                debug_assert!(false, "tree child node is missing during parent cleanup");
            }
        }
        Ok(Some(parent))
    }

    /// Drops all logical tree owners before Runtime clears the UI object store at shutdown.
    pub(crate) fn take_all(
        &mut self,
    ) -> Result<HashMap<ObjectIdentity, LogicalNode<R>>, TreeError> {
        self.ensure_ui_thread()?;
        Ok(std::mem::take(&mut self.nodes))
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn parent_ancestry(&self, parent: ObjectIdentity) -> Result<Vec<ObjectIdentity>, TreeError> {
        let mut ancestry = Vec::new();
        let mut current = Some(parent);
        while let Some(identity) = current {
            if ancestry.contains(&identity) {
                return Err(TreeError::Cycle);
            }
            ancestry
                .try_reserve(1)
                .map_err(|_| TreeError::AllocationFailed)?;
            ancestry.push(identity);
            current = self
                .nodes
                .get(&identity)
                .ok_or(TreeError::InvalidIdentity)?
                .parent;
        }
        Ok(ancestry)
    }

    fn ensure_ui_thread(&self) -> Result<(), TreeError> {
        if self.ui_thread.is_current() {
            Ok(())
        } else {
            Err(TreeError::WrongThread)
        }
    }
}

#[allow(dead_code)]
impl TreeCoordinator {
    pub(crate) fn child_handle<K, R>(
        registry: &HandleRegistry<K, R>,
        parent: &LogicalNode<R>,
        child: ObjectIdentity,
        kind: K,
    ) -> Result<HandleToken, TreeError>
    where
        K: Clone + Eq + Send + Sync,
        R: Clone + Eq + Send + Sync + 'static,
    {
        if !parent.ui_thread.is_current() {
            return Err(TreeError::WrongThread);
        }
        let edge = parent
            .children
            .iter()
            .find(|edge| edge.identity == child)
            .ok_or(TreeError::ChildNotFound)?;
        registry
            .register_from_tree_owner(kind, parent.runtime_id.clone(), &edge._owner)
            .map_err(TreeError::Registry)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_attach<R>(
        parent: &LogicalNode<R>,
        parent_pin: &ValidationPin<R>,
        child: &LogicalNode<R>,
        child_pin: &ValidationPin<R>,
        parent_ancestry: &[ObjectIdentity],
        index: usize,
        prepare_peer: impl FnOnce() -> Result<(), ()>,
    ) -> Result<PreparedAttach<R>, TreeError>
    where
        R: Clone + Eq + Send + Sync + 'static,
    {
        if parent.ui_thread != child.ui_thread {
            return Err(TreeError::RuntimeMismatch);
        }
        if !parent.ui_thread.is_current() {
            return Err(TreeError::WrongThread);
        }
        if parent.identity != parent_pin.identity()
            || child.identity != child_pin.identity()
            || !parent_ancestry.contains(&parent.identity)
        {
            return Err(TreeError::InvalidIdentity);
        }
        if parent.runtime_id != child.runtime_id
            || parent_pin.runtime_id() != &parent.runtime_id
            || child_pin.runtime_id() != &child.runtime_id
        {
            return Err(TreeError::RuntimeMismatch);
        }
        if !parent.is_container {
            return Err(TreeError::ParentNotContainer);
        }
        if parent.identity == child.identity || parent_ancestry.contains(&child.identity) {
            return Err(TreeError::Cycle);
        }
        if child.parent.is_some()
            || parent
                .children
                .iter()
                .any(|edge| edge.identity == child.identity)
        {
            return Err(TreeError::AlreadyParented);
        }
        if index > parent.children.len() {
            return Err(TreeError::ChildNotFound);
        }

        let parent_revision = parent
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;
        let child_revision = child
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;
        let owner = child_pin.acquire_tree_owner()?;
        prepare_peer().map_err(|()| TreeError::PeerPreparationFailed)?;

        Ok(PreparedAttach {
            parent: parent.identity,
            parent_revision,
            child: child.identity,
            child_revision,
            index,
            owner,
        })
    }

    pub(crate) fn commit_attach<R>(
        parent: &mut LogicalNode<R>,
        child: &mut LogicalNode<R>,
        prepared: PreparedAttach<R>,
    ) -> Result<(), TreeError> {
        if parent.ui_thread != child.ui_thread {
            return Err(TreeError::RuntimeMismatch);
        }
        if !parent.ui_thread.is_current() {
            return Err(TreeError::WrongThread);
        }
        if parent.identity != prepared.parent
            || child.identity != prepared.child
            || parent.revision.checked_add(1) != Some(prepared.parent_revision)
            || child.revision.checked_add(1) != Some(prepared.child_revision)
            || child.parent.is_some()
            || prepared.owner.identity() != child.identity
        {
            return Err(TreeError::PreparedStateChanged);
        }
        parent
            .children
            .try_reserve(1)
            .map_err(|_| TreeError::AllocationFailed)?;

        parent.children.insert(
            prepared.index,
            ChildLink {
                identity: child.identity,
                _owner: prepared.owner,
            },
        );
        child.parent = Some(parent.identity);
        parent.revision = prepared.parent_revision;
        child.revision = prepared.child_revision;
        Ok(())
    }

    pub(crate) fn detach<R>(
        parent: &mut LogicalNode<R>,
        child: &mut LogicalNode<R>,
    ) -> Result<TreeOwner<R>, TreeError> {
        if parent.ui_thread != child.ui_thread {
            return Err(TreeError::RuntimeMismatch);
        }
        if !parent.ui_thread.is_current() {
            return Err(TreeError::WrongThread);
        }
        if child.parent != Some(parent.identity) {
            return Err(TreeError::ChildNotFound);
        }
        let index = parent
            .children
            .iter()
            .position(|edge| edge.identity == child.identity)
            .ok_or(TreeError::ChildNotFound)?;
        let parent_revision = parent
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;
        let child_revision = child
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;

        let detached = parent.children.remove(index);
        child.parent = None;
        parent.revision = parent_revision;
        child.revision = child_revision;
        Ok(detached._owner)
    }

    pub(crate) fn detach_all<R>(
        parent: &mut LogicalNode<R>,
        children: &mut [LogicalNode<R>],
    ) -> Result<(), TreeError> {
        if children
            .iter()
            .any(|child| child.ui_thread != parent.ui_thread)
        {
            return Err(TreeError::RuntimeMismatch);
        }
        if !parent.ui_thread.is_current() {
            return Err(TreeError::WrongThread);
        }
        let parent_revision = parent
            .revision
            .checked_add(1)
            .ok_or(TreeError::RevisionExhausted)?;

        for edge in &parent.children {
            let child = children
                .iter()
                .find(|child| child.identity == edge.identity)
                .ok_or(TreeError::ChildNotFound)?;
            if child.parent != Some(parent.identity) {
                return Err(TreeError::PreparedStateChanged);
            }
            child
                .revision
                .checked_add(1)
                .ok_or(TreeError::RevisionExhausted)?;
        }

        let detached = std::mem::take(&mut parent.children);
        parent.revision = parent_revision;
        for edge in &detached {
            let child = children
                .iter_mut()
                .find(|child| child.identity == edge.identity)
                .expect("all children were validated before commit");
            child.parent = None;
            child.revision += 1;
        }
        drop(detached);
        Ok(())
    }
}
