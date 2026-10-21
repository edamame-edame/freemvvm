use crate::handle_registry::registry::ObjectIdentity;
use std::collections::{HashMap, HashSet};

/// Core graph topology for connections between singleton Property synchronization groups.
///
/// TwoWay group membership aliases Property identities to a stable representative; OneWay edges
/// are interpreted through those representatives.
#[derive(Default)]
pub(crate) struct BindingGraph {
    one_way_edges: Vec<OneWayEdge>,
    group_parent: HashMap<ObjectIdentity, ObjectIdentity>,
    generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OneWayEdge {
    source: ObjectIdentity,
    target: ObjectIdentity,
}

/// A topology change that was validated against one graph generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreparedOneWayEdge {
    source: ObjectIdentity,
    target: ObjectIdentity,
    generation: u64,
}

/// A proposed union of two distinct synchronization groups.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreparedTwoWayMerge {
    source_group: ObjectIdentity,
    target_group: ObjectIdentity,
    generation: u64,
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

impl BindingGraph {
    /// Validates a OneWay edge and reserves storage without changing the accepted topology.
    pub(crate) fn prepare_one_way(
        &mut self,
        source: ObjectIdentity,
        target: ObjectIdentity,
    ) -> Result<PreparedOneWayEdge, BindingGraphError> {
        if source == target {
            return Err(BindingGraphError::SelfConnection);
        }
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
        if prepared.generation != self.generation {
            return Err(BindingGraphError::StaleProposal);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BindingGraphError::GenerationExhausted)?;
        self.one_way_edges.push(OneWayEdge {
            source: prepared.source,
            target: prepared.target,
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
        Ok(PreparedTwoWayMerge {
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
        if prepared.generation != self.generation {
            return Err(BindingGraphError::StaleProposal);
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(BindingGraphError::GenerationExhausted)?;
        self.group_parent
            .insert(prepared.target_group, prepared.source_group);
        self.generation = next_generation;
        Ok(())
    }

    pub(crate) fn same_group(&self, first: ObjectIdentity, second: ObjectIdentity) -> bool {
        self.resolve_group(first) == self.resolve_group(second)
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
            normalized.push(OneWayEdge { source, target });
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
}
