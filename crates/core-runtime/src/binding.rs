use crate::handle_registry::registry::ObjectIdentity;
use crate::value::{BindingCopyError, TypedValue, ValueStorage};
use std::collections::{HashMap, HashSet};

/// Core graph topology for connections between singleton Property synchronization groups.
///
/// TwoWay group membership aliases Property identities to a stable representative; OneWay edges
/// are interpreted through those representatives.
#[derive(Default)]
pub(crate) struct BindingGraph {
    one_way_edges: Vec<OneWayEdge>,
    two_way_edges: Vec<TwoWayEdge>,
    group_parent: HashMap<ObjectIdentity, ObjectIdentity>,
    generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OneWayEdge {
    id: BindingConnectionId,
    source: ObjectIdentity,
    target: ObjectIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TwoWayEdge {
    id: BindingConnectionId,
    source: ObjectIdentity,
    target: ObjectIdentity,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingConnectionId {
    first: ObjectIdentity,
    second: ObjectIdentity,
    two_way: bool,
}

impl BindingConnectionId {
    pub(crate) fn direction_from(self, origin: ObjectIdentity) -> u32 {
        if origin == self.second { 2 } else { 1 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OneWayPropagationEdge {
    pub(crate) id: BindingConnectionId,
    pub(crate) source: ObjectIdentity,
    pub(crate) target: ObjectIdentity,
}

/// A topology change that was validated against one graph generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreparedOneWayEdge {
    id: BindingConnectionId,
    source: ObjectIdentity,
    target: ObjectIdentity,
    generation: u64,
}

/// A validated OneWay edge and the uncommitted initial value for its target.
pub(crate) struct PreparedOneWayBinding {
    edge: PreparedOneWayEdge,
    initial_value: Option<TypedValue>,
}

impl PreparedOneWayBinding {
    /// Splits the proposal for a caller that coordinates the eventual cross-state commit.
    pub(crate) fn into_parts(self) -> (PreparedOneWayEdge, Option<TypedValue>) {
        (self.edge, self.initial_value)
    }
}

impl PreparedOneWayEdge {
    pub(crate) fn connection_id(self) -> BindingConnectionId {
        self.id
    }
}

/// A proposed union of two distinct synchronization groups.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreparedTwoWayMerge {
    id: BindingConnectionId,
    source_group: ObjectIdentity,
    target_group: ObjectIdentity,
    generation: u64,
}

impl PreparedTwoWayMerge {
    pub(crate) fn connection_id(self) -> BindingConnectionId {
        self.id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingGraphError {
    SelfConnection,
    DuplicateEdge,
    TargetAlreadyHasInput,
    Cycle,
    StaleProposal,
    GenerationExhausted,
    OutOfMemory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingInitialPrepareError {
    Graph(BindingGraphError),
    InitialCopy(BindingCopyError),
}

impl BindingGraph {
    /// Prepares a OneWay topology edge and the target's initial value as one proposal.
    ///
    /// Neither the accepted graph topology nor the target value is committed here. The edge
    /// reservation may increase vector capacity, but a rejected proposal leaves observable graph
    /// state and the target value unchanged.
    pub(crate) fn prepare_one_way_with_initial_copy(
        &mut self,
        source: ObjectIdentity,
        target: ObjectIdentity,
        source_value: &TypedValue,
        target_value: &ValueStorage,
    ) -> Result<PreparedOneWayBinding, BindingInitialPrepareError> {
        let edge = self
            .prepare_one_way(source, target)
            .map_err(BindingInitialPrepareError::Graph)?;
        let initial_value = target_value
            .prepare_binding_initial_copy(source_value)
            .map_err(BindingInitialPrepareError::InitialCopy)?;
        Ok(PreparedOneWayBinding {
            edge,
            initial_value,
        })
    }

    /// Validates a OneWay edge and reserves storage without changing the accepted topology.
    pub(crate) fn prepare_one_way(
        &mut self,
        source: ObjectIdentity,
        target: ObjectIdentity,
    ) -> Result<PreparedOneWayEdge, BindingGraphError> {
        if source == target {
            return Err(BindingGraphError::SelfConnection);
        }
        let id = BindingConnectionId {
            first: source,
            second: target,
            two_way: false,
        };
        let source = self.resolve_group(source);
        let target = self.resolve_group(target);
        if source == target {
            return Err(BindingGraphError::SelfConnection);
        }
        if self.one_way_edges.iter().any(|edge| {
            self.resolve_group(edge.source) == source && self.resolve_group(edge.target) == target
        }) {
            return Err(BindingGraphError::DuplicateEdge);
        }
        if self
            .one_way_edges
            .iter()
            .any(|edge| self.resolve_group(edge.target) == target)
        {
            return Err(BindingGraphError::TargetAlreadyHasInput);
        }
        if self.has_path(target, source, None)? {
            return Err(BindingGraphError::Cycle);
        }
        if self.generation.checked_add(1).is_none() {
            return Err(BindingGraphError::GenerationExhausted);
        }
        self.one_way_edges
            .try_reserve(1)
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        Ok(PreparedOneWayEdge {
            id,
            source,
            target,
            generation: self.generation,
        })
    }

    /// Commits a prepared edge only if no topology change has occurred since validation.
    pub(crate) fn commit_one_way(
        &mut self,
        prepared: PreparedOneWayEdge,
    ) -> Result<(), BindingGraphError> {
        self.commit_one_way_with(prepared, || {})
    }

    /// Commits related state while the graph proposal is known to be current, then publishes its edge.
    pub(crate) fn commit_one_way_with(
        &mut self,
        prepared: PreparedOneWayEdge,
        commit_related_state: impl FnOnce(),
    ) -> Result<(), BindingGraphError> {
        if prepared.generation != self.generation {
            return Err(BindingGraphError::StaleProposal);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BindingGraphError::GenerationExhausted)?;
        commit_related_state();
        self.one_way_edges.push(OneWayEdge {
            id: prepared.id,
            source: prepared.id.first,
            target: prepared.id.second,
        });
        self.generation = next_generation;
        Ok(())
    }

    /// Validates the topology that would result from merging two Property groups.
    pub(crate) fn prepare_two_way_merge(
        &mut self,
        first: ObjectIdentity,
        second: ObjectIdentity,
    ) -> Result<PreparedTwoWayMerge, BindingGraphError> {
        let source_group = self.resolve_group(first);
        let target_group = self.resolve_group(second);
        if source_group == target_group {
            return Err(BindingGraphError::DuplicateEdge);
        }
        let proposed_merge = (source_group, target_group);
        let edges = self.normalized_edges(Some(proposed_merge))?;
        for edge in &edges {
            if Self::path_exists(&edges, edge.target, edge.source)? {
                return Err(BindingGraphError::Cycle);
            }
        }
        if self.generation.checked_add(1).is_none() {
            return Err(BindingGraphError::GenerationExhausted);
        }
        self.group_parent
            .try_reserve(1)
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        self.two_way_edges
            .try_reserve(1)
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        Ok(PreparedTwoWayMerge {
            id: BindingConnectionId {
                first,
                second,
                two_way: true,
            },
            source_group,
            target_group,
            generation: self.generation,
        })
    }

    /// Commits a validated group union only if its source topology is still current.
    pub(crate) fn commit_two_way_merge(
        &mut self,
        prepared: PreparedTwoWayMerge,
    ) -> Result<(), BindingGraphError> {
        self.commit_two_way_merge_with(prepared, || {})
    }

    /// Commits related state with a validated group union before publishing the new topology.
    pub(crate) fn commit_two_way_merge_with(
        &mut self,
        prepared: PreparedTwoWayMerge,
        commit_related_state: impl FnOnce(),
    ) -> Result<(), BindingGraphError> {
        if prepared.generation != self.generation {
            return Err(BindingGraphError::StaleProposal);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BindingGraphError::GenerationExhausted)?;
        commit_related_state();
        self.two_way_edges.push(TwoWayEdge {
            id: prepared.id,
            source: prepared.id.first,
            target: prepared.id.second,
        });
        self.group_parent
            .insert(prepared.target_group, prepared.source_group);
        self.generation = next_generation;
        Ok(())
    }

    pub(crate) fn remove_connection(
        &mut self,
        id: BindingConnectionId,
    ) -> Result<bool, BindingGraphError> {
        let one_way_index = self.one_way_edges.iter().position(|edge| edge.id == id);
        let two_way_index = self.two_way_edges.iter().position(|edge| edge.id == id);
        if one_way_index.is_none() && two_way_index.is_none() {
            return Ok(false);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BindingGraphError::GenerationExhausted)?;
        if two_way_index.is_some() {
            let remaining_edges = self.two_way_edges.len().saturating_sub(1);
            if self.group_parent.capacity() < remaining_edges {
                self.group_parent
                    .try_reserve(remaining_edges.saturating_sub(self.group_parent.len()))
                    .map_err(|_| BindingGraphError::OutOfMemory)?;
            }
        }
        if let Some(index) = one_way_index {
            self.one_way_edges.remove(index);
        }
        if let Some(index) = two_way_index {
            self.two_way_edges.remove(index);
            self.group_parent.clear();
            for edge in &self.two_way_edges {
                let source = self.resolve_group(edge.source);
                let target = self.resolve_group(edge.target);
                if source != target {
                    self.group_parent.insert(target, source);
                }
            }
        }
        self.generation = next_generation;
        Ok(true)
    }

    pub(crate) fn clear(&mut self) {
        self.one_way_edges.clear();
        self.two_way_edges.clear();
        self.group_parent.clear();
    }

    pub(crate) fn connection_id(
        &self,
        first: ObjectIdentity,
        second: ObjectIdentity,
        two_way: bool,
    ) -> Option<BindingConnectionId> {
        if two_way {
            self.two_way_edges
                .iter()
                .find(|edge| edge.id.first == first && edge.id.second == second)
                .map(|edge| edge.id)
        } else {
            self.one_way_edges
                .iter()
                .find(|edge| edge.id.first == first && edge.id.second == second)
                .map(|edge| edge.id)
        }
    }

    pub(crate) fn two_way_connection_for_group(
        &self,
        property: ObjectIdentity,
    ) -> Option<BindingConnectionId> {
        let representative = self.resolve_group(property);
        self.two_way_edges
            .iter()
            .find(|edge| self.resolve_group(edge.source) == representative)
            .map(|edge| edge.id)
    }

    pub(crate) fn same_group(&self, first: ObjectIdentity, second: ObjectIdentity) -> bool {
        self.resolve_group(first) == self.resolve_group(second)
    }

    pub(crate) fn representative(&self, property: ObjectIdentity) -> ObjectIdentity {
        self.resolve_group(property)
    }

    pub(crate) fn group_members(
        &self,
        property: ObjectIdentity,
    ) -> Result<Vec<ObjectIdentity>, BindingGraphError> {
        let representative = self.resolve_group(property);
        let mut members = Vec::new();
        members
            .try_reserve(self.group_parent.len().saturating_add(1))
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        members.push(representative);
        members.extend(self.group_parent.keys().copied().filter(|member| {
            *member != representative && self.resolve_group(*member) == representative
        }));
        members[1..].sort_by_key(|member| (member.slot(), member.generation().get()));
        Ok(members)
    }

    /// Snapshots reachable OneWay edges in stable upstream-to-downstream order.
    ///
    /// This only prepares topology for a later propagation stage; it does not read or write
    /// Property values. Group aliases are resolved before reachability is evaluated.
    pub(crate) fn prepare_one_way_propagation(
        &self,
        dirty_properties: &[ObjectIdentity],
    ) -> Result<Vec<OneWayPropagationEdge>, BindingGraphError> {
        let edges = self.normalized_edges(None)?;
        let mut reachable = Vec::new();
        reachable
            .try_reserve(dirty_properties.len())
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        for property in dirty_properties {
            let group = self.resolve_group(*property);
            if !reachable.contains(&group) {
                reachable.push(group);
            }
        }

        let mut cursor = 0;
        while cursor < reachable.len() {
            let source = reachable[cursor];
            cursor += 1;
            for edge in &edges {
                if edge.source == source && !reachable.contains(&edge.target) {
                    reachable
                        .try_reserve(1)
                        .map_err(|_| BindingGraphError::OutOfMemory)?;
                    reachable.push(edge.target);
                }
            }
        }

        let mut pending = Vec::new();
        pending
            .try_reserve(edges.len())
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        for edge in edges {
            if reachable.contains(&edge.source) {
                pending.push(edge);
            }
        }

        let mut ordered = Vec::new();
        ordered
            .try_reserve(pending.len())
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        while !pending.is_empty() {
            let ready = pending.iter().position(|candidate| {
                !pending
                    .iter()
                    .any(|incoming| incoming.target == candidate.source)
            });
            let Some(index) = ready else {
                return Err(BindingGraphError::Cycle);
            };
            let edge = pending.remove(index);
            ordered.push(OneWayPropagationEdge {
                id: edge.id,
                source: edge.source,
                target: edge.target,
            });
        }
        Ok(ordered)
    }

    fn has_path(
        &self,
        start: ObjectIdentity,
        destination: ObjectIdentity,
        proposed_merge: Option<(ObjectIdentity, ObjectIdentity)>,
    ) -> Result<bool, BindingGraphError> {
        let edges = self.normalized_edges(proposed_merge)?;
        Self::path_exists(&edges, start, destination)
    }

    fn path_exists(
        edges: &[OneWayEdge],
        start: ObjectIdentity,
        destination: ObjectIdentity,
    ) -> Result<bool, BindingGraphError> {
        let mut pending = Vec::new();
        pending
            .try_reserve(1)
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        pending.push(start);

        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            if current == destination {
                return Ok(true);
            }
            if !visited.contains(&current) {
                visited
                    .try_reserve(1)
                    .map_err(|_| BindingGraphError::OutOfMemory)?;
                visited.insert(current);
                for edge in edges {
                    if edge.source == current && !visited.contains(&edge.target) {
                        pending
                            .try_reserve(1)
                            .map_err(|_| BindingGraphError::OutOfMemory)?;
                        pending.push(edge.target);
                    }
                }
            }
        }
        Ok(false)
    }

    fn normalized_edges(
        &self,
        proposed_merge: Option<(ObjectIdentity, ObjectIdentity)>,
    ) -> Result<Vec<OneWayEdge>, BindingGraphError> {
        let mut normalized = Vec::new();
        normalized
            .try_reserve(self.one_way_edges.len())
            .map_err(|_| BindingGraphError::OutOfMemory)?;
        for edge in &self.one_way_edges {
            let source = self.apply_proposed_merge(self.resolve_group(edge.source), proposed_merge);
            let target = self.apply_proposed_merge(self.resolve_group(edge.target), proposed_merge);
            if source == target {
                return Err(BindingGraphError::SelfConnection);
            }
            if normalized
                .iter()
                .any(|previous: &OneWayEdge| previous.source == source && previous.target == target)
            {
                return Err(BindingGraphError::DuplicateEdge);
            }
            if normalized.iter().any(|previous| previous.target == target) {
                return Err(BindingGraphError::TargetAlreadyHasInput);
            }
            normalized.push(OneWayEdge {
                id: edge.id,
                source,
                target,
            });
        }
        Ok(normalized)
    }

    fn resolve_group(&self, mut group: ObjectIdentity) -> ObjectIdentity {
        while let Some(parent) = self.group_parent.get(&group).copied() {
            group = parent;
        }
        group
    }

    fn apply_proposed_merge(
        &self,
        group: ObjectIdentity,
        proposed_merge: Option<(ObjectIdentity, ObjectIdentity)>,
    ) -> ObjectIdentity {
        match proposed_merge {
            Some((source, target)) if group == target => source,
            _ => group,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BindingGraph, BindingGraphError};
    use crate::handle_registry::registry::ObjectIdentity;
    use crate::value::{DecimalValue, OwnedUtf8, TypedValue, ValueStorage};
    use std::num::NonZeroU64;

    fn identity(slot: u64) -> ObjectIdentity {
        ObjectIdentity::new(slot, NonZeroU64::new(1).expect("nonzero generation"))
    }

    fn connect(graph: &mut BindingGraph, source: u64, target: u64) {
        let prepared = graph
            .prepare_one_way(identity(source), identity(target))
            .expect("OneWay edge should prepare");
        graph
            .commit_one_way(prepared)
            .expect("prepared edge should commit");
    }

    #[test]
    fn binding_graph_accepts_one_way_chain_and_rejects_invalid_topology_atomically() {
        let mut graph = BindingGraph::default();
        connect(&mut graph, 1, 2);
        connect(&mut graph, 2, 3);
        assert_eq!(graph.one_way_edges.len(), 2);

        assert_eq!(
            graph.prepare_one_way(identity(1), identity(1)),
            Err(BindingGraphError::SelfConnection)
        );
        assert_eq!(
            graph.prepare_one_way(identity(1), identity(2)),
            Err(BindingGraphError::DuplicateEdge)
        );
        assert_eq!(
            graph.prepare_one_way(identity(3), identity(2)),
            Err(BindingGraphError::TargetAlreadyHasInput)
        );
        assert_eq!(
            graph.prepare_one_way(identity(3), identity(1)),
            Err(BindingGraphError::Cycle)
        );
        assert_eq!(graph.one_way_edges.len(), 2);

        connect(&mut graph, 4, 5);
        assert_eq!(graph.one_way_edges.len(), 3);
    }

    #[test]
    fn binding_graph_rejects_a_proposal_prepared_for_an_older_generation() {
        let mut graph = BindingGraph::default();
        let first = graph
            .prepare_one_way(identity(1), identity(2))
            .expect("first edge prepares");
        let stale = graph
            .prepare_one_way(identity(3), identity(4))
            .expect("independent edge also prepares before commit");
        graph.commit_one_way(first).expect("first proposal commits");

        assert_eq!(
            graph.commit_one_way(stale),
            Err(BindingGraphError::StaleProposal)
        );
        assert_eq!(graph.one_way_edges.len(), 1);
        assert_eq!(graph.one_way_edges[0].source, identity(1));
        assert_eq!(graph.one_way_edges[0].target, identity(2));
    }

    #[test]
    fn binding_propagation_order_snapshots_reachable_edges_upstream_first() {
        let mut graph = BindingGraph::default();
        // Commit a downstream edge first to ensure evaluation order comes from topology, not
        // edge insertion order.
        connect(&mut graph, 2, 3);
        connect(&mut graph, 1, 2);
        connect(&mut graph, 1, 4);
        connect(&mut graph, 4, 5);
        connect(&mut graph, 7, 8);
        let generation = graph.generation;
        let edge_count = graph.one_way_edges.len();

        let snapshot = graph
            .prepare_one_way_propagation(&[identity(1)])
            .expect("reachable propagation edges should be ordered");
        let pairs: Vec<_> = snapshot
            .iter()
            .map(|edge| (edge.source, edge.target))
            .collect();

        assert_eq!(
            pairs,
            vec![
                (identity(1), identity(2)),
                (identity(2), identity(3)),
                (identity(1), identity(4)),
                (identity(4), identity(5)),
            ]
        );
        assert_eq!(graph.generation, generation);
        assert_eq!(graph.one_way_edges.len(), edge_count);
    }

    #[test]
    fn binding_propagation_order_resolves_two_way_group_sources_and_empty_input() {
        let mut graph = BindingGraph::default();
        connect(&mut graph, 1, 3);
        let merge = graph
            .prepare_two_way_merge(identity(1), identity(2))
            .expect("source group merge should preserve the DAG");
        graph
            .commit_two_way_merge(merge)
            .expect("valid source group merge commits");

        let from_group_member = graph
            .prepare_one_way_propagation(&[identity(2)])
            .expect("group member should reach the normalized edge");
        assert_eq!(
            from_group_member,
            vec![super::OneWayPropagationEdge {
                id: super::BindingConnectionId {
                    first: identity(1),
                    second: identity(3),
                    two_way: false,
                },
                source: identity(1),
                target: identity(3),
            }]
        );
        assert!(
            graph
                .prepare_one_way_propagation(&[])
                .expect("empty dirty set should produce no work")
                .is_empty()
        );
    }

    #[test]
    fn binding_graph_merges_groups_when_the_resulting_one_way_topology_is_valid() {
        let mut graph = BindingGraph::default();
        connect(&mut graph, 1, 2);
        let merge = graph
            .prepare_two_way_merge(identity(3), identity(4))
            .expect("unconnected groups can merge");
        graph
            .commit_two_way_merge(merge)
            .expect("valid group merge commits");

        assert!(graph.same_group(identity(3), identity(4)));
        assert!(!graph.same_group(identity(1), identity(3)));
        assert_eq!(graph.one_way_edges.len(), 1);
    }

    #[test]
    fn binding_graph_cancel_removes_one_way_edges_and_splits_two_way_groups() {
        let mut graph = BindingGraph::default();
        let one_way = graph
            .prepare_one_way(identity(1), identity(2))
            .expect("OneWay edge prepares");
        let one_way_id = one_way.connection_id();
        graph.commit_one_way(one_way).expect("OneWay edge commits");
        assert_eq!(graph.one_way_edges.len(), 1);
        assert!(graph.remove_connection(one_way_id).expect("cancel edge"));
        assert!(graph.one_way_edges.is_empty());
        assert!(!graph.remove_connection(one_way_id).expect("repeat cancel"));

        let mut groups = BindingGraph::default();
        let first = groups
            .prepare_two_way_merge(identity(1), identity(2))
            .expect("first TwoWay edge prepares");
        let first_id = first.connection_id();
        groups
            .commit_two_way_merge(first)
            .expect("first edge commits");
        let second = groups
            .prepare_two_way_merge(identity(2), identity(3))
            .expect("second TwoWay edge prepares");
        groups
            .commit_two_way_merge(second)
            .expect("second edge commits");
        assert!(groups.same_group(identity(1), identity(3)));

        assert!(
            groups
                .remove_connection(first_id)
                .expect("cancel first edge")
        );
        assert!(!groups.same_group(identity(1), identity(2)));
        assert!(groups.same_group(identity(2), identity(3)));
        assert!(!groups.remove_connection(first_id).expect("repeat cancel"));

        let mut alias_edge = BindingGraph::default();
        let merge = alias_edge
            .prepare_two_way_merge(identity(1), identity(2))
            .expect("alias merge prepares");
        let merge_id = merge.connection_id();
        alias_edge
            .commit_two_way_merge(merge)
            .expect("alias merge commits");
        connect(&mut alias_edge, 2, 3);
        assert!(
            alias_edge
                .remove_connection(merge_id)
                .expect("alias merge cancels")
        );
        assert_eq!(
            alias_edge
                .prepare_one_way_propagation(&[identity(2)])
                .expect("original OneWay endpoint remains reachable"),
            vec![super::OneWayPropagationEdge {
                id: super::BindingConnectionId {
                    first: identity(2),
                    second: identity(3),
                    two_way: false,
                },
                source: identity(2),
                target: identity(3),
            }]
        );
    }

    #[test]
    fn binding_graph_rejects_group_merges_that_break_one_way_topology_atomically() {
        let mut self_edge = BindingGraph::default();
        connect(&mut self_edge, 1, 2);
        assert_eq!(
            self_edge.prepare_two_way_merge(identity(1), identity(2)),
            Err(BindingGraphError::SelfConnection)
        );
        assert!(!self_edge.same_group(identity(1), identity(2)));
        connect(&mut self_edge, 3, 4);

        let mut cycle = BindingGraph::default();
        connect(&mut cycle, 1, 2);
        connect(&mut cycle, 2, 3);
        assert_eq!(
            cycle.prepare_two_way_merge(identity(1), identity(3)),
            Err(BindingGraphError::Cycle)
        );
        assert!(!cycle.same_group(identity(1), identity(3)));
        connect(&mut cycle, 4, 5);

        let mut duplicate = BindingGraph::default();
        connect(&mut duplicate, 1, 3);
        connect(&mut duplicate, 1, 4);
        assert_eq!(
            duplicate.prepare_two_way_merge(identity(3), identity(4)),
            Err(BindingGraphError::DuplicateEdge)
        );
        assert!(!duplicate.same_group(identity(3), identity(4)));

        let mut multiple_inputs = BindingGraph::default();
        connect(&mut multiple_inputs, 1, 3);
        connect(&mut multiple_inputs, 2, 4);
        assert_eq!(
            multiple_inputs.prepare_two_way_merge(identity(3), identity(4)),
            Err(BindingGraphError::TargetAlreadyHasInput)
        );
        assert!(!multiple_inputs.same_group(identity(3), identity(4)));
    }

    #[test]
    fn binding_graph_rejects_a_two_way_merge_proposal_after_another_commit() {
        let mut graph = BindingGraph::default();
        let first = graph
            .prepare_two_way_merge(identity(1), identity(2))
            .expect("first merge prepares");
        let stale = graph
            .prepare_two_way_merge(identity(3), identity(4))
            .expect("independent merge also prepares before commit");
        graph
            .commit_two_way_merge(first)
            .expect("first merge commits");

        assert_eq!(
            graph.commit_two_way_merge(stale),
            Err(BindingGraphError::StaleProposal)
        );
        assert!(graph.same_group(identity(1), identity(2)));
        assert!(!graph.same_group(identity(3), identity(4)));
    }

    #[test]
    fn binding_initial_prepare_returns_graph_and_value_candidates_without_committing() {
        let mut graph = BindingGraph::default();
        let original = DecimalValue::new(false, 1, 10).expect("valid decimal");
        let incoming = DecimalValue::new(false, 0, 2).expect("valid decimal");
        let target = ValueStorage::new(TypedValue::Decimal(original));
        let generation = graph.generation;

        let proposal = graph
            .prepare_one_way_with_initial_copy(
                identity(1),
                identity(2),
                &TypedValue::Decimal(incoming),
                &target,
            )
            .expect("valid topology and same-kind copy prepare");

        assert_eq!(graph.generation, generation);
        assert!(graph.one_way_edges.is_empty());
        assert!(matches!(target.read(), TypedValue::Decimal(value) if *value == original));
        let (_, candidate) = proposal.into_parts();
        assert!(matches!(candidate, Some(TypedValue::Decimal(value)) if value == incoming));
    }

    #[test]
    fn binding_initial_prepare_rejections_preserve_topology_and_target() {
        let mut graph = BindingGraph::default();
        connect(&mut graph, 1, 2);
        connect(&mut graph, 2, 3);
        let edge_count = graph.one_way_edges.len();
        let generation = graph.generation;
        let target = ValueStorage::new(TypedValue::I64(99));

        assert!(matches!(
            graph.prepare_one_way_with_initial_copy(
                identity(3),
                identity(1),
                &TypedValue::I64(0),
                &target,
            ),
            Err(super::BindingInitialPrepareError::Graph(
                BindingGraphError::Cycle
            ))
        ));
        assert!(matches!(
            graph.prepare_one_way_with_initial_copy(
                identity(4),
                identity(3),
                &TypedValue::I64(0),
                &target,
            ),
            Err(super::BindingInitialPrepareError::Graph(
                BindingGraphError::TargetAlreadyHasInput
            ))
        ));
        assert!(matches!(
            graph.prepare_one_way_with_initial_copy(
                identity(1),
                identity(2),
                &TypedValue::I64(0),
                &target,
            ),
            Err(super::BindingInitialPrepareError::Graph(
                BindingGraphError::DuplicateEdge
            ))
        ));
        assert!(matches!(
            graph.prepare_one_way_with_initial_copy(
                identity(4),
                identity(4),
                &TypedValue::I64(0),
                &target,
            ),
            Err(super::BindingInitialPrepareError::Graph(
                BindingGraphError::SelfConnection
            ))
        ));
        assert!(matches!(
            graph.prepare_one_way_with_initial_copy(
                identity(4),
                identity(5),
                &TypedValue::Bool(true),
                &target,
            ),
            Err(super::BindingInitialPrepareError::InitialCopy(
                crate::value::BindingCopyError::KindMismatch(_)
            ))
        ));

        assert_eq!(graph.one_way_edges.len(), edge_count);
        assert_eq!(graph.generation, generation);
        assert!(matches!(target.read(), TypedValue::I64(99)));
    }

    #[test]
    fn binding_initial_prepare_owns_source_utf8_in_candidate() {
        let mut graph = BindingGraph::default();
        let source = TypedValue::Utf8(
            OwnedUtf8::copy_from_bytes("初期\0source".as_bytes()).expect("valid UTF-8"),
        );
        let target = ValueStorage::new(TypedValue::Utf8(
            OwnedUtf8::copy_from_bytes(b"target").expect("valid UTF-8"),
        ));

        let proposal = graph
            .prepare_one_way_with_initial_copy(identity(8), identity(9), &source, &target)
            .expect("valid topology and same-kind copy prepare");
        drop(source);

        let (_, candidate) = proposal.into_parts();
        assert!(
            matches!(candidate, Some(TypedValue::Utf8(value)) if value.as_bytes() == "初期\0source".as_bytes())
        );
        assert!(matches!(target.read(), TypedValue::Utf8(value) if value.as_bytes() == b"target"));
    }
}
