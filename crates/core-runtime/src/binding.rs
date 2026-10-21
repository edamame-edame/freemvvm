use crate::handle_registry::registry::ObjectIdentity;
use std::collections::HashSet;

/// Core graph topology for connections between singleton Property synchronization groups.
///
/// TwoWay group merging is intentionally left to a later unit. Until then, each Property identity
/// is its own graph vertex.
#[derive(Default)]
pub(crate) struct BindingGraph {
    one_way_edges: Vec<OneWayEdge>,
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
        if self
            .one_way_edges
            .iter()
            .any(|edge| edge.source == source && edge.target == target)
        {
            return Err(BindingGraphError::DuplicateEdge);
        }
        if self.one_way_edges.iter().any(|edge| edge.target == target) {
            return Err(BindingGraphError::TargetAlreadyHasInput);
        }
        if self.has_path(target, source)? {
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

    fn has_path(
        &self,
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
                for edge in &self.one_way_edges {
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
}
