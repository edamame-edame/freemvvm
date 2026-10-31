//! Builds renderer-neutral scene operations from an ephemeral visual snapshot.

use crate::handle_registry::registry::ObjectIdentity;
use crate::platform::scene::{
    Damage, DeviceScale, DrawOp, GlyphPlacement, GlyphRunSnapshot, Rect, SceneError, SceneSnapshot,
};
use crate::ui_layout::{LayoutSnapshot, Rect as LayoutRect};
use std::collections::HashSet;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisualBuildError {
    MissingLayout,
    InvalidGeometry,
    AllocationFailed,
    InvalidScene,
}

pub(crate) struct ControlViewConnector;

impl ControlViewConnector {
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
                    DrawOp::GlyphRun(moved)
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
}
