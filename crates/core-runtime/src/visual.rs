//! Builds renderer-neutral scene operations from an ephemeral visual snapshot.

use crate::handle_registry::registry::ObjectIdentity;
use crate::platform::scene::{
    Damage, DeviceScale, DrawOp, GlyphPlacement, GlyphRunSnapshot, Rect, SceneError, SceneSnapshot,
};
use crate::text_service::ResolvedTextSnapshot;
use crate::ui_layout::{LayoutSnapshot, Rect as LayoutRect};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Local-coordinate drawing data supplied while the Core visual snapshot is assembled.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum VisualOperation {
    FillRect { bounds: Rect, rgba: [f32; 4] },
    GlyphRun(GlyphRunSnapshot),
}

/// Ephemeral backend-neutral node data; it is consumed while producing one SceneSnapshot.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VisualNodeSnapshot {
    pub(crate) identity: ObjectIdentity,
    pub(crate) clips_children: bool,
    pub(crate) operations: Vec<VisualOperation>,
    pub(crate) children: Vec<VisualNodeSnapshot>,
}

/// A revision-stamped, non-owning copy of the attached logical tree's ordered identities.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ControlTreeSnapshot {
    pub(crate) tree_revision: u64,
    pub(crate) root: ControlTreeNodeSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ControlTreeNodeSnapshot {
    pub(crate) identity: ObjectIdentity,
    pub(crate) visual: VisualNodeDescriptor,
    pub(crate) text: Option<Arc<ResolvedTextSnapshot>>,
    pub(crate) children: Vec<Self>,
}

/// Explicit visual data for one Control; it never owns a Control or renderer resource.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VisualNodeDescriptor {
    pub(crate) clips_children: bool,
    pub(crate) operations: Vec<VisualOperation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisualBuildError {
    MissingLayout,
    RevisionMismatch,
    MissingDescriptor,
    UnexpectedDescriptor,
    InvalidGeometry,
    AllocationFailed,
    InvalidScene,
}

pub(crate) struct ControlViewConnector;

impl ControlViewConnector {
    /// Builds a scene from descriptors captured from the live Core Control state.
    pub(crate) fn build_from_production_control_tree(
        tree: &ControlTreeSnapshot,
        layout: &LayoutSnapshot,
        device_scale: DeviceScale,
        background_rgb: [f32; 3],
        damage: Damage,
    ) -> Result<SceneSnapshot, VisualBuildError> {
        if tree.tree_revision != layout.tree_revision {
            return Err(VisualBuildError::RevisionMismatch);
        }
        let mut visited = HashSet::new();
        let root = Self::materialize_production_node(&tree.root, layout, &mut visited)?;
        if visited.len() != layout.node_count() {
            return Err(VisualBuildError::MissingLayout);
        }
        Self::build_snapshot(&root, layout, device_scale, background_rgb, damage)
    }

    fn materialize_production_node(
        node: &ControlTreeNodeSnapshot,
        layout: &LayoutSnapshot,
        visited: &mut HashSet<ObjectIdentity>,
    ) -> Result<VisualNodeSnapshot, VisualBuildError> {
        visited
            .try_reserve(1)
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        if !visited.insert(node.identity) {
            return Err(VisualBuildError::InvalidGeometry);
        }
        if layout.node(node.identity).is_none() {
            return Err(VisualBuildError::MissingLayout);
        }
        let text_operation_count = if let Some(snapshot) = &node.text {
            if snapshot.revision != layout.layout_revision {
                return Err(VisualBuildError::RevisionMismatch);
            }
            snapshot.glyph_runs.len()
        } else {
            0
        };
        let mut operations = Vec::new();
        operations
            .try_reserve(
                node.visual
                    .operations
                    .len()
                    .saturating_add(text_operation_count),
            )
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        operations.extend(node.visual.operations.iter().cloned());
        if let Some(text) = &node.text {
            operations.extend(
                text.glyph_runs
                    .iter()
                    .map(|run| VisualOperation::GlyphRun(run.glyphs.clone())),
            );
        }
        let mut children = Vec::new();
        children
            .try_reserve(node.children.len())
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        for child in &node.children {
            children.push(Self::materialize_production_node(child, layout, visited)?);
        }
        Ok(VisualNodeSnapshot {
            identity: node.identity,
            clips_children: node.visual.clips_children,
            operations,
            children,
        })
    }

    pub(crate) fn build_from_control_tree(
        tree: &ControlTreeSnapshot,
        layout: &LayoutSnapshot,
        descriptors: &HashMap<ObjectIdentity, VisualNodeDescriptor>,
        device_scale: DeviceScale,
        background_rgb: [f32; 3],
        damage: Damage,
    ) -> Result<SceneSnapshot, VisualBuildError> {
        if tree.tree_revision != layout.tree_revision {
            return Err(VisualBuildError::RevisionMismatch);
        }
        let mut visited = HashSet::new();
        let root = Self::materialize_visual_node(&tree.root, layout, descriptors, &mut visited)?;
        if visited.len() != layout.node_count() {
            return Err(VisualBuildError::MissingLayout);
        }
        if visited.len() != descriptors.len() {
            return Err(VisualBuildError::UnexpectedDescriptor);
        }
        Self::build_snapshot(&root, layout, device_scale, background_rgb, damage)
    }

    fn materialize_visual_node(
        node: &ControlTreeNodeSnapshot,
        layout: &LayoutSnapshot,
        descriptors: &HashMap<ObjectIdentity, VisualNodeDescriptor>,
        visited: &mut HashSet<ObjectIdentity>,
    ) -> Result<VisualNodeSnapshot, VisualBuildError> {
        visited
            .try_reserve(1)
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        if !visited.insert(node.identity) {
            return Err(VisualBuildError::InvalidGeometry);
        }
        let descriptor = descriptors
            .get(&node.identity)
            .ok_or(VisualBuildError::MissingDescriptor)?;
        let text_operation_count = node.text.as_ref().map_or(Ok(0), |snapshot| {
            if snapshot.revision != layout.layout_revision {
                Err(VisualBuildError::RevisionMismatch)
            } else {
                Ok(snapshot.glyph_runs.len())
            }
        })?;
        let mut operations = Vec::new();
        operations
            .try_reserve(
                descriptor
                    .operations
                    .len()
                    .saturating_add(text_operation_count),
            )
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        operations.extend(descriptor.operations.iter().cloned());
        if let Some(text) = &node.text {
            operations.extend(
                text.glyph_runs
                    .iter()
                    .map(|run| VisualOperation::GlyphRun(run.glyphs.clone())),
            );
        }
        let mut children = Vec::new();
        children
            .try_reserve(node.children.len())
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        for child in &node.children {
            children.push(Self::materialize_visual_node(
                child,
                layout,
                descriptors,
                visited,
            )?);
        }
        Ok(VisualNodeSnapshot {
            identity: node.identity,
            clips_children: descriptor.clips_children,
            operations,
            children,
        })
    }

    pub(crate) fn build_snapshot(
        root: &VisualNodeSnapshot,
        layout: &LayoutSnapshot,
        device_scale: DeviceScale,
        background_rgb: [f32; 3],
        damage: Damage,
    ) -> Result<SceneSnapshot, VisualBuildError> {
        let client = layout.client_area();
        let initial_clip = Rect::new(0.0, 0.0, client.width, client.height)
            .map_err(|_| VisualBuildError::InvalidGeometry)?;
        let mut operations = Vec::new();
        let mut visited = HashSet::new();
        Self::append_node(
            root,
            layout,
            (0.0, 0.0),
            initial_clip,
            &mut visited,
            &mut operations,
        )?;
        if visited.len() != layout.node_count() {
            return Err(VisualBuildError::MissingLayout);
        }
        SceneSnapshot::new(
            layout.layout_revision,
            device_scale,
            background_rgb,
            Arc::<[DrawOp]>::from(operations),
            damage,
        )
        .map_err(|error| match error {
            SceneError::InvalidGeometry => VisualBuildError::InvalidGeometry,
            _ => VisualBuildError::InvalidScene,
        })
    }

    fn append_node(
        node: &VisualNodeSnapshot,
        layout: &LayoutSnapshot,
        parent_origin: (f64, f64),
        parent_clip: Rect,
        visited: &mut HashSet<ObjectIdentity>,
        output: &mut Vec<DrawOp>,
    ) -> Result<(), VisualBuildError> {
        visited
            .try_reserve(1)
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        if !visited.insert(node.identity) {
            return Err(VisualBuildError::InvalidGeometry);
        }
        let bounds = layout
            .node(node.identity)
            .ok_or(VisualBuildError::MissingLayout)?
            .final_bounds;
        let origin = (parent_origin.0 + bounds.x, parent_origin.1 + bounds.y);
        if !origin.0.is_finite() || !origin.1.is_finite() {
            return Err(VisualBuildError::InvalidGeometry);
        }
        let absolute_bounds = translate_layout_rect(bounds, parent_origin)?;
        let own_clip = intersect(parent_clip, absolute_bounds)?;

        output
            .try_reserve(1 + node.operations.len())
            .map_err(|_| VisualBuildError::AllocationFailed)?;
        output.push(DrawOp::Clip(own_clip));
        for operation in &node.operations {
            output.push(match operation {
                VisualOperation::FillRect { bounds, rgba } => DrawOp::FillRect {
                    bounds: translate_scene_rect(*bounds, origin)?,
                    rgba: *rgba,
                },
                VisualOperation::GlyphRun(run) => {
                    let glyphs = run
                        .glyphs()
                        .iter()
                        .map(|glyph| translate_glyph(*glyph, origin))
                        .collect::<Result<Vec<_>, _>>()?;
                    let moved = GlyphRunSnapshot::new(
                        *run.font_digest(),
                        run.font_size(),
                        translate_scene_rect(run.bounds(), origin)?,
                        Arc::<[GlyphPlacement]>::from(glyphs),
                    )
                    .map_err(|_| VisualBuildError::InvalidGeometry)?;
                    DrawOp::GlyphRun {
                        run: moved,
                        rgba: [0.0, 0.0, 0.0, 1.0],
                    }
                }
            });
        }

        let child_clip = if node.clips_children {
            own_clip
        } else {
            parent_clip
        };
        for child in &node.children {
            Self::append_node(child, layout, origin, child_clip, visited, output)?;
        }
        Ok(())
    }
}

fn translate_layout_rect(rect: LayoutRect, origin: (f64, f64)) -> Result<Rect, VisualBuildError> {
    Rect::new(
        origin.0 + rect.x,
        origin.1 + rect.y,
        rect.width,
        rect.height,
    )
    .map_err(|_| VisualBuildError::InvalidGeometry)
}

fn translate_scene_rect(rect: Rect, origin: (f64, f64)) -> Result<Rect, VisualBuildError> {
    Rect::new(
        origin.0 + rect.x(),
        origin.1 + rect.y(),
        rect.width(),
        rect.height(),
    )
    .map_err(|_| VisualBuildError::InvalidGeometry)
}

fn translate_glyph(
    glyph: GlyphPlacement,
    origin: (f64, f64),
) -> Result<GlyphPlacement, VisualBuildError> {
    let (x, y) = glyph.position();
    let (advance_x, advance_y) = glyph.advance();
    GlyphPlacement::new(
        glyph.glyph_id(),
        x + origin.0,
        y + origin.1,
        advance_x,
        advance_y,
    )
    .map_err(|_| VisualBuildError::InvalidGeometry)
}

fn intersect(left: Rect, right: Rect) -> Result<Rect, VisualBuildError> {
    let x = left.x().max(right.x());
    let y = left.y().max(right.y());
    let edge_x = (left.x() + left.width()).min(right.x() + right.width());
    let edge_y = (left.y() + left.height()).min(right.y() + right.height());
    Rect::new(x, y, (edge_x - x).max(0.0), (edge_y - y).max(0.0))
        .map_err(|_| VisualBuildError::InvalidGeometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_layout::{
        IntrinsicMeasureProvider, LayoutConstraints, LayoutEngine, LayoutError, LayoutNodeSnapshot,
        Orientation, Size,
    };
    use std::num::NonZeroU64;

    fn identity(slot: u64) -> ObjectIdentity {
        ObjectIdentity::new(slot, NonZeroU64::new(1).unwrap())
    }

    fn tree_node(
        identity: ObjectIdentity,
        children: Vec<ControlTreeNodeSnapshot>,
    ) -> ControlTreeNodeSnapshot {
        ControlTreeNodeSnapshot {
            identity,
            visual: VisualNodeDescriptor {
                clips_children: !children.is_empty(),
                operations: Vec::new(),
            },
            text: None,
            children,
        }
    }

    fn descriptors(identities: &[ObjectIdentity]) -> HashMap<ObjectIdentity, VisualNodeDescriptor> {
        identities
            .iter()
            .copied()
            .map(|identity| {
                (
                    identity,
                    VisualNodeDescriptor {
                        clips_children: true,
                        operations: vec![VisualOperation::FillRect {
                            bounds: Rect::new(0.0, 0.0, 1.0, 1.0).unwrap(),
                            rgba: [0.2, 0.3, 0.4, 1.0],
                        }],
                    },
                )
            })
            .collect()
    }

    #[test]
    fn control_tree_visual_snapshot_preserves_attach_detach_order_and_draw_operations() {
        let root = identity(1);
        let first = identity(2);
        let second = identity(3);
        let layout = calculate_layout(LayoutNodeSnapshot::stack_panel(
            root,
            Orientation::Vertical,
            vec![
                LayoutNodeSnapshot::leaf(first),
                LayoutNodeSnapshot::leaf(second),
            ],
        ));
        let scale = DeviceScale::new(1.0, 1.0).unwrap();
        let damage = Damage::Full;

        let attached = ControlTreeSnapshot {
            tree_revision: layout.tree_revision,
            root: tree_node(
                root,
                vec![tree_node(first, vec![]), tree_node(second, vec![])],
            ),
        };
        let attached_scene = ControlViewConnector::build_from_control_tree(
            &attached,
            &layout,
            &descriptors(&[root, first, second]),
            scale,
            [0.0, 0.0, 0.0],
            damage,
        )
        .unwrap();
        let fill_bounds = |scene: &SceneSnapshot| {
            scene
                .operations()
                .iter()
                .filter_map(|op| match op {
                    DrawOp::FillRect { bounds, .. } => Some(*bounds),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let attached_bounds = fill_bounds(&attached_scene);
        assert_eq!(attached_scene.layout_revision(), layout.layout_revision);
        assert_eq!(attached_bounds.len(), 3);
        assert!(attached_bounds[1].y() < attached_bounds[2].y());

        let detached_layout = calculate_layout(LayoutNodeSnapshot::stack_panel(
            root,
            Orientation::Vertical,
            vec![LayoutNodeSnapshot::leaf(second)],
        ));
        let detached = ControlTreeSnapshot {
            tree_revision: detached_layout.tree_revision,
            root: tree_node(root, vec![tree_node(second, vec![])]),
        };
        let detached_scene = ControlViewConnector::build_from_control_tree(
            &detached,
            &detached_layout,
            &descriptors(&[root, second]),
            scale,
            [0.0, 0.0, 0.0],
            damage,
        )
        .unwrap();
        assert_eq!(fill_bounds(&detached_scene).len(), 2);

        let reordered_layout = calculate_layout(LayoutNodeSnapshot::stack_panel(
            root,
            Orientation::Vertical,
            vec![
                LayoutNodeSnapshot::leaf(second),
                LayoutNodeSnapshot::leaf(first),
            ],
        ));
        let reordered = ControlTreeSnapshot {
            tree_revision: reordered_layout.tree_revision,
            root: tree_node(
                root,
                vec![tree_node(second, vec![]), tree_node(first, vec![])],
            ),
        };
        let reordered_scene = ControlViewConnector::build_from_control_tree(
            &reordered,
            &reordered_layout,
            &descriptors(&[root, first, second]),
            scale,
            [0.0, 0.0, 0.0],
            damage,
        )
        .unwrap();
        let reordered_bounds = fill_bounds(&reordered_scene);
        assert!(reordered_bounds[1].y() < reordered_bounds[2].y());
    }

    #[test]
    fn control_tree_visual_snapshot_rejects_revision_and_descriptor_mismatch_atomically() {
        let root = identity(10);
        let child = identity(11);
        let layout = calculate_layout(LayoutNodeSnapshot::stack_panel(
            root,
            Orientation::Vertical,
            vec![LayoutNodeSnapshot::leaf(child)],
        ));
        let tree = ControlTreeSnapshot {
            tree_revision: layout.tree_revision + 1,
            root: tree_node(root, vec![tree_node(child, vec![])]),
        };
        let args = (
            &layout,
            &descriptors(&[root, child]),
            DeviceScale::new(1.0, 1.0).unwrap(),
            [0.0, 0.0, 0.0],
            Damage::Full,
        );
        assert_eq!(
            ControlViewConnector::build_from_control_tree(
                &tree, args.0, args.1, args.2, args.3, args.4
            ),
            Err(VisualBuildError::RevisionMismatch)
        );

        let tree = ControlTreeSnapshot {
            tree_revision: layout.tree_revision,
            root: tree_node(root, vec![tree_node(child, vec![])]),
        };
        assert_eq!(
            ControlViewConnector::build_from_control_tree(
                &tree,
                &layout,
                &descriptors(&[root]),
                DeviceScale::new(1.0, 1.0).unwrap(),
                [0.0, 0.0, 0.0],
                Damage::Full,
            ),
            Err(VisualBuildError::MissingDescriptor)
        );

        assert_eq!(
            ControlViewConnector::build_from_control_tree(
                &tree,
                &layout,
                &descriptors(&[root, child, identity(12)]),
                DeviceScale::new(1.0, 1.0).unwrap(),
                [0.0, 0.0, 0.0],
                Damage::Full,
            ),
            Err(VisualBuildError::UnexpectedDescriptor)
        );
    }

    struct FixedMeasure;

    impl IntrinsicMeasureProvider for FixedMeasure {
        fn measure(&self, _: ObjectIdentity, _: LayoutConstraints) -> Result<Size, LayoutError> {
            Size::new(70.0, 30.0)
        }
    }

    #[test]
    fn control_view_connector_flattens_nested_bounds_and_intersects_ancestor_clips() {
        let root_id = identity(1);
        let first_id = identity(2);
        let panel_id = identity(3);
        let leaf_id = identity(4);
        let input = LayoutNodeSnapshot::stack_panel(
            root_id,
            Orientation::Vertical,
            vec![
                LayoutNodeSnapshot::leaf(first_id),
                LayoutNodeSnapshot::stack_panel(
                    panel_id,
                    Orientation::Horizontal,
                    vec![LayoutNodeSnapshot::leaf(leaf_id)],
                ),
            ],
        );
        let layout =
            LayoutEngine::calculate(&input, 4, 8, Size::new(100.0, 80.0).unwrap(), &FixedMeasure)
                .unwrap();
        let node = VisualNodeSnapshot {
            identity: root_id,
            clips_children: true,
            operations: vec![],
            children: vec![
                VisualNodeSnapshot {
                    identity: first_id,
                    clips_children: false,
                    operations: vec![],
                    children: vec![],
                },
                VisualNodeSnapshot {
                    identity: panel_id,
                    clips_children: true,
                    operations: vec![],
                    children: vec![VisualNodeSnapshot {
                        identity: leaf_id,
                        clips_children: false,
                        operations: vec![VisualOperation::FillRect {
                            bounds: Rect::new(-4.0, -3.0, 40.0, 20.0).unwrap(),
                            rgba: [1.0, 0.0, 0.0, 1.0],
                        }],
                        children: vec![],
                    }],
                },
            ],
        };

        let scene = ControlViewConnector::build_snapshot(
            &node,
            &layout,
            DeviceScale::new(1.0, 1.0).unwrap(),
            [0.2, 0.3, 0.4],
            Damage::Full,
        )
        .unwrap();

        let clips = scene
            .operations()
            .iter()
            .filter_map(|op| match op {
                DrawOp::Clip(rect) => Some(*rect),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(clips.len(), 4);
        assert_eq!(clips[0], Rect::new(0.0, 0.0, 100.0, 80.0).unwrap());
        let panel_bounds = layout.node(panel_id).unwrap().final_bounds;
        assert_eq!(panel_bounds.y, 30.0);
        assert_eq!(
            clips[2],
            Rect::new(
                panel_bounds.x,
                panel_bounds.y,
                panel_bounds.width,
                panel_bounds.height
            )
            .unwrap()
        );
        let leaf_bounds = layout.node(leaf_id).unwrap().final_bounds;
        assert_eq!(
            clips[3],
            Rect::new(
                panel_bounds.x + leaf_bounds.x,
                panel_bounds.y + leaf_bounds.y,
                leaf_bounds.width,
                leaf_bounds.height
            )
            .unwrap()
        );
        let DrawOp::FillRect { bounds, .. } = scene.operations()[4] else {
            panic!("leaf fill should follow its clip");
        };
        assert_eq!(bounds.x(), panel_bounds.x + leaf_bounds.x - 4.0);
        assert_eq!(bounds.y(), panel_bounds.y + leaf_bounds.y - 3.0);
        assert_eq!(scene.layout_revision(), 8);
    }

    #[test]
    fn control_view_connector_rejects_missing_layout_without_partial_scene() {
        let layout = LayoutSnapshot::empty(1, 1, Size::new(20.0, 10.0).unwrap());
        let node = VisualNodeSnapshot {
            identity: identity(9),
            clips_children: false,
            operations: vec![],
            children: vec![],
        };
        assert_eq!(
            ControlViewConnector::build_snapshot(
                &node,
                &layout,
                DeviceScale::new(1.0, 1.0).unwrap(),
                [1.0, 1.0, 1.0],
                Damage::Full,
            ),
            Err(VisualBuildError::MissingLayout)
        );
    }

    fn calculate_layout(root: LayoutNodeSnapshot) -> LayoutSnapshot {
        LayoutEngine::calculate(&root, 7, 9, Size::new(120.0, 80.0).unwrap(), &FixedMeasure)
            .unwrap()
    }
}
