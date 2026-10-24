use crate::handle_registry::HandleToken;
use crate::handle_registry::registry::{
    HandleRegistry, ObjectIdentity, RegistryError, TreeOwner, ValidationPin,
};
use crate::runtime::UiThreadContext;

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
    owner: TreeOwner<R>,
}

/// Prepares and commits Logical Tree membership without exposing a partial parent/child link.
#[allow(dead_code)]
pub(crate) struct TreeCoordinator;

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

        parent.children.push(ChildLink {
            identity: child.identity,
            _owner: prepared.owner,
        });
        child.parent = Some(parent.identity);
        parent.revision = prepared.parent_revision;
        child.revision = prepared.child_revision;
        Ok(())
    }

    pub(crate) fn detach<R>(
        parent: &mut LogicalNode<R>,
        child: &mut LogicalNode<R>,
    ) -> Result<(), TreeError> {
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
        drop(detached);
        Ok(())
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
