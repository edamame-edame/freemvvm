use crate::handle_registry::registry::ObjectIdentity;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Size {
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl Size {
    pub(crate) fn new(width: f64, height: f64) -> Result<Self, LayoutError> {
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(LayoutError::InvalidGeometry);
        }
        Ok(Self { width, height })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl Rect {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, LayoutError> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width < 0.0
            || height < 0.0
            || !(x + width).is_finite()
            || !(y + height).is_finite()
        {
            return Err(LayoutError::InvalidGeometry);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Maximum {
    Finite(f64),
    Unbounded,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AxisConstraints {
    min: f64,
    max: Maximum,
}

impl AxisConstraints {
    pub(crate) fn new(min: f64, max: Option<f64>) -> Result<Self, LayoutError> {
        if !min.is_finite() || min < 0.0 {
            return Err(LayoutError::InvalidConstraints);
        }
        let max = match max {
            Some(value) if value.is_finite() && value >= min => Maximum::Finite(value),
            Some(_) => return Err(LayoutError::InvalidConstraints),
            None => Maximum::Unbounded,
        };
        Ok(Self { min, max })
    }

    fn unbounded() -> Self {
        Self {
            min: 0.0,
            max: Maximum::Unbounded,
        }
    }

    fn bounded_by(max: f64) -> Result<Self, LayoutError> {
        Self::new(0.0, Some(max))
    }

    fn intersect(self, available: Self) -> Result<Self, LayoutError> {
        let max = match (self.max, available.max) {
            (Maximum::Finite(a), Maximum::Finite(b)) => Maximum::Finite(a.min(b)),
            (Maximum::Finite(value), Maximum::Unbounded)
            | (Maximum::Unbounded, Maximum::Finite(value)) => Maximum::Finite(value),
            (Maximum::Unbounded, Maximum::Unbounded) => Maximum::Unbounded,
        };
        let upper = match max {
            Maximum::Finite(value) => value,
            Maximum::Unbounded => f64::INFINITY,
        };
        let min = self.min.max(available.min);
        if min > upper {
            return Err(LayoutError::UnsatisfiableConstraints);
        }
        Ok(Self { min, max })
    }

    fn clamp(self, value: f64) -> Result<f64, LayoutError> {
        if !value.is_finite() || value < 0.0 {
            return Err(LayoutError::InvalidGeometry);
        }
        let bounded = match self.max {
            Maximum::Finite(max) => value.min(max),
            Maximum::Unbounded => value,
        };
        Ok(bounded.max(self.min))
    }

    fn with_max(self, max: Maximum) -> Result<Self, LayoutError> {
        let max_value = match max {
            Maximum::Finite(value) => Some(value),
            Maximum::Unbounded => None,
        };
        Self::new(self.min, max_value)
    }

    fn maximum(self) -> Maximum {
        self.max
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayoutConstraints {
    pub(crate) horizontal: AxisConstraints,
    pub(crate) vertical: AxisConstraints,
}

impl LayoutConstraints {
    pub(crate) fn new(
        min_width: f64,
        max_width: Option<f64>,
        min_height: f64,
        max_height: Option<f64>,
    ) -> Result<Self, LayoutError> {
        Ok(Self {
            horizontal: AxisConstraints::new(min_width, max_width)?,
            vertical: AxisConstraints::new(min_height, max_height)?,
        })
    }

    fn unbounded() -> Self {
        Self {
            horizontal: AxisConstraints::unbounded(),
            vertical: AxisConstraints::unbounded(),
        }
    }

    fn bounded_by(size: Size) -> Result<Self, LayoutError> {
        Ok(Self {
            horizontal: AxisConstraints::bounded_by(size.width)?,
            vertical: AxisConstraints::bounded_by(size.height)?,
        })
    }

    fn intersect(self, available: Self) -> Result<Self, LayoutError> {
        Ok(Self {
            horizontal: self.horizontal.intersect(available.horizontal)?,
            vertical: self.vertical.intersect(available.vertical)?,
        })
    }

    fn clamp(self, size: Size) -> Result<Size, LayoutError> {
        Size::new(
            self.horizontal.clamp(size.width)?,
            self.vertical.clamp(size.height)?,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayoutNodeKind {
    Leaf,
    StackPanel(Orientation),
}

/// Immutable input copied from the Visual tree for a single layout calculation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutNodeSnapshot {
    pub(crate) identity: ObjectIdentity,
    pub(crate) kind: LayoutNodeKind,
    pub(crate) constraints: LayoutConstraints,
    /// Alignment of this node in its StackPanel parent's cross axis.
    pub(crate) align: Align,
    pub(crate) children: Vec<Self>,
}

impl LayoutNodeSnapshot {
    pub(crate) fn leaf(identity: ObjectIdentity) -> Self {
        Self {
            identity,
            kind: LayoutNodeKind::Leaf,
            constraints: LayoutConstraints::unbounded(),
            align: Align::Stretch,
            children: Vec::new(),
        }
    }

    pub(crate) fn stack_panel(
        identity: ObjectIdentity,
        orientation: Orientation,
        children: Vec<Self>,
    ) -> Self {
        Self {
            identity,
            kind: LayoutNodeKind::StackPanel(orientation),
            constraints: LayoutConstraints::unbounded(),
            align: Align::Stretch,
            children,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayoutError {
    InvalidGeometry,
    InvalidConstraints,
    UnsatisfiableConstraints,
    InvalidTree,
    MeasurementFailed,
    ArithmeticOverflow,
    AllocationFailed,
}

pub(crate) trait IntrinsicMeasureProvider {
    fn measure(
        &self,
        identity: ObjectIdentity,
        constraints: LayoutConstraints,
    ) -> Result<Size, LayoutError>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NodeLayout {
    pub(crate) desired_size: Size,
    pub(crate) final_bounds: Rect,
}

/// A complete immutable result. Callers replace the currently published snapshot only on `Ok`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutSnapshot {
    pub(crate) window_generation: u64,
    pub(crate) tree_revision: u64,
    pub(crate) layout_revision: u64,
    pub(crate) client_area: Size,
    nodes: HashMap<ObjectIdentity, NodeLayout>,
}

impl LayoutSnapshot {
    pub(crate) fn node(&self, identity: ObjectIdentity) -> Option<NodeLayout> {
        self.nodes.get(&identity).copied()
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn empty(tree_revision: u64, layout_revision: u64, client_area: Size) -> Self {
        Self {
            window_generation: 0,
            tree_revision,
            layout_revision,
            client_area,
            nodes: HashMap::new(),
        }
    }
}

pub(crate) struct LayoutEngine;

impl LayoutEngine {
    pub(crate) fn calculate(
        root: &LayoutNodeSnapshot,
        tree_revision: u64,
        layout_revision: u64,
        client_area: Size,
        measurer: &impl IntrinsicMeasureProvider,
    ) -> Result<LayoutSnapshot, LayoutError> {
        let mut desired_sizes = HashMap::new();
        let root_constraints = LayoutConstraints::bounded_by(client_area)?;
        measure_node(root, root_constraints, measurer, &mut desired_sizes)?;

        let mut nodes = HashMap::new();
        let root_bounds = Rect::new(0.0, 0.0, client_area.width, client_area.height)?;
        arrange_node(root, root_bounds, &desired_sizes, &mut nodes)?;
        if nodes.len() != desired_sizes.len() {
            return Err(LayoutError::InvalidTree);
        }
        Ok(LayoutSnapshot {
            window_generation: 0,
            tree_revision,
            layout_revision,
            client_area,
            nodes,
        })
    }
}

fn measure_node(
    node: &LayoutNodeSnapshot,
    available: LayoutConstraints,
    measurer: &impl IntrinsicMeasureProvider,
    measured: &mut HashMap<ObjectIdentity, Size>,
) -> Result<Size, LayoutError> {
    if measured.contains_key(&node.identity) {
        return Err(LayoutError::InvalidTree);
    }
    let constraints = node.constraints.intersect(available)?;
    let desired = match node.kind {
        LayoutNodeKind::Leaf if node.children.is_empty() => {
            constraints.clamp(measurer.measure(node.identity, constraints)?)?
        }
        LayoutNodeKind::StackPanel(orientation) => {
            let mut main_total = 0.0;
            let mut cross_max: f64 = 0.0;
            for child in &node.children {
                let child_available =
                    child_constraints(orientation, constraints, child.constraints)?;
                let child_size = measure_node(child, child_available, measurer, measured)?;
                let (main, cross) = axes(orientation, child_size);
                main_total += main;
                if !main_total.is_finite() {
                    return Err(LayoutError::ArithmeticOverflow);
                }
                cross_max = cross_max.max(cross);
            }
            let aggregate = from_axes(orientation, main_total, cross_max)?;
            constraints.clamp(aggregate)?
        }
        LayoutNodeKind::Leaf => return Err(LayoutError::InvalidTree),
    };
    measured
        .try_reserve(1)
        .map_err(|_| LayoutError::AllocationFailed)?;
    if measured.insert(node.identity, desired).is_some() {
        return Err(LayoutError::InvalidTree);
    }
    Ok(desired)
}

fn child_constraints(
    orientation: Orientation,
    parent: LayoutConstraints,
    child: LayoutConstraints,
) -> Result<LayoutConstraints, LayoutError> {
    let (main_parent, cross_parent) = match orientation {
        Orientation::Vertical => (parent.vertical, parent.horizontal),
        Orientation::Horizontal => (parent.horizontal, parent.vertical),
    };
    let (main_child, cross_child) = match orientation {
        Orientation::Vertical => (child.vertical, child.horizontal),
        Orientation::Horizontal => (child.horizontal, child.vertical),
    };
    let main = main_child.intersect(main_parent.with_max(Maximum::Unbounded)?)?;
    let cross = cross_child.intersect(cross_parent)?;
    Ok(match orientation {
        Orientation::Vertical => LayoutConstraints {
            horizontal: cross,
            vertical: main,
        },
        Orientation::Horizontal => LayoutConstraints {
            horizontal: main,
            vertical: cross,
        },
    })
}

fn arrange_node(
    node: &LayoutNodeSnapshot,
    bounds: Rect,
    measured: &HashMap<ObjectIdentity, Size>,
    arranged: &mut HashMap<ObjectIdentity, NodeLayout>,
) -> Result<(), LayoutError> {
    let desired_size = *measured
        .get(&node.identity)
        .ok_or(LayoutError::InvalidTree)?;
    arranged
        .try_reserve(1)
        .map_err(|_| LayoutError::AllocationFailed)?;
    if arranged
        .insert(
            node.identity,
            NodeLayout {
                desired_size,
                final_bounds: bounds,
            },
        )
        .is_some()
    {
        return Err(LayoutError::InvalidTree);
    }
    let LayoutNodeKind::StackPanel(orientation) = node.kind else {
        if !node.children.is_empty() {
            return Err(LayoutError::InvalidTree);
        }
        return Ok(());
    };

    let mut main_offset = 0.0;
    for child in &node.children {
        let child_desired = *measured
            .get(&child.identity)
            .ok_or(LayoutError::InvalidTree)?;
        let (_, available_cross) = rect_axes(orientation, bounds);
        let (desired_main, desired_cross) = axes(orientation, child_desired);
        let (main_constraints, cross_constraints) = match orientation {
            Orientation::Vertical => (child.constraints.vertical, child.constraints.horizontal),
            Orientation::Horizontal => (child.constraints.horizontal, child.constraints.vertical),
        };
        let main_size = main_constraints.clamp(desired_main)?;
        let cross_size = match child.align {
            Align::Stretch => cross_constraints.clamp(available_cross)?,
            Align::Start | Align::Center | Align::End => cross_constraints.clamp(desired_cross)?,
        };
        if !(main_offset + main_size).is_finite() {
            return Err(LayoutError::ArithmeticOverflow);
        }
        let cross_offset = match child.align {
            Align::Start | Align::Stretch => 0.0,
            Align::Center => (available_cross - cross_size) / 2.0,
            Align::End => available_cross - cross_size,
        };
        let child_bounds = match orientation {
            Orientation::Vertical => Rect::new(
                bounds.x + cross_offset,
                bounds.y + main_offset,
                cross_size,
                main_size,
            )?,
            Orientation::Horizontal => Rect::new(
                bounds.x + main_offset,
                bounds.y + cross_offset,
                main_size,
                cross_size,
            )?,
        };
        arrange_node(child, child_bounds, measured, arranged)?;
        main_offset += main_size;
        if !main_offset.is_finite() {
            return Err(LayoutError::ArithmeticOverflow);
        }
    }
    Ok(())
}

fn axes(orientation: Orientation, size: Size) -> (f64, f64) {
    match orientation {
        Orientation::Vertical => (size.height, size.width),
        Orientation::Horizontal => (size.width, size.height),
    }
}

fn from_axes(orientation: Orientation, main: f64, cross: f64) -> Result<Size, LayoutError> {
    match orientation {
        Orientation::Vertical => Size::new(cross, main),
        Orientation::Horizontal => Size::new(main, cross),
    }
}

fn rect_axes(orientation: Orientation, rect: Rect) -> (f64, f64) {
    match orientation {
        Orientation::Vertical => (rect.height, rect.width),
        Orientation::Horizontal => (rect.width, rect.height),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Align, AxisConstraints, IntrinsicMeasureProvider, LayoutConstraints, LayoutEngine,
        LayoutError, LayoutNodeSnapshot, Orientation, Size,
    };
    use crate::handle_registry::registry::ObjectIdentity;
    use std::collections::HashMap;
    use std::num::NonZeroU64;

    struct FakeMeasurer {
        sizes: HashMap<ObjectIdentity, Size>,
        fail_on: Option<ObjectIdentity>,
    }

    impl IntrinsicMeasureProvider for FakeMeasurer {
        fn measure(
            &self,
            identity: ObjectIdentity,
            _: LayoutConstraints,
        ) -> Result<Size, LayoutError> {
            if self.fail_on == Some(identity) {
                return Err(LayoutError::MeasurementFailed);
            }
            self.sizes
                .get(&identity)
                .copied()
                .ok_or(LayoutError::MeasurementFailed)
        }
    }

    fn identity(slot: u64) -> ObjectIdentity {
        ObjectIdentity::new(slot, NonZeroU64::new(1).expect("nonzero generation"))
    }

    fn size(width: f64, height: f64) -> Size {
        Size::new(width, height).expect("valid size")
    }

    fn provider(sizes: &[(ObjectIdentity, Size)]) -> FakeMeasurer {
        FakeMeasurer {
            sizes: sizes.iter().copied().collect(),
            fail_on: None,
        }
    }

    #[test]
    fn layout_engine_measures_and_arranges_vertical_and_horizontal_panels() {
        let a = identity(1);
        let b = identity(2);
        let vertical = LayoutNodeSnapshot::stack_panel(
            identity(3),
            Orientation::Vertical,
            vec![LayoutNodeSnapshot::leaf(a), LayoutNodeSnapshot::leaf(b)],
        );
        let measured = provider(&[(a, size(80.0, 20.0)), (b, size(60.0, 30.0))]);
        let result = LayoutEngine::calculate(&vertical, 7, 11, size(100.0, 100.0), &measured)
            .expect("vertical layout");

        assert_eq!(result.tree_revision, 7);
        assert_eq!(result.layout_revision, 11);
        assert_eq!(
            result.node(identity(3)).unwrap().desired_size,
            size(80.0, 50.0)
        );
        assert_eq!(result.node(a).unwrap().final_bounds.x, 0.0);
        assert_eq!(result.node(a).unwrap().final_bounds.width, 100.0);
        assert_eq!(result.node(b).unwrap().final_bounds.y, 20.0);
        assert_eq!(result.node(b).unwrap().final_bounds.height, 30.0);

        let empty =
            LayoutNodeSnapshot::stack_panel(identity(15), Orientation::Vertical, Vec::new());
        let result = LayoutEngine::calculate(&empty, 7, 12, size(100.0, 100.0), &provider(&[]))
            .expect("empty panel layout");
        assert_eq!(
            result.node(identity(15)).unwrap().desired_size,
            size(0.0, 0.0)
        );
        assert_eq!(result.node(identity(15)).unwrap().final_bounds.width, 100.0);

        let horizontal = LayoutNodeSnapshot::stack_panel(
            identity(4),
            Orientation::Horizontal,
            vec![LayoutNodeSnapshot::leaf(a), LayoutNodeSnapshot::leaf(b)],
        );
        let result = LayoutEngine::calculate(&horizontal, 8, 12, size(200.0, 100.0), &measured)
            .expect("horizontal layout");
        assert_eq!(
            result.node(identity(4)).unwrap().desired_size,
            size(140.0, 30.0)
        );
        assert_eq!(result.node(b).unwrap().final_bounds.x, 80.0);
        assert_eq!(result.node(b).unwrap().final_bounds.height, 100.0);
    }

    #[test]
    fn layout_engine_applies_constraints_and_alignment_without_splitting_main_axis_space() {
        let child_id = identity(5);
        let mut child = LayoutNodeSnapshot::leaf(child_id);
        child.constraints = LayoutConstraints::new(50.0, Some(70.0), 0.0, None).unwrap();
        let root = LayoutNodeSnapshot::stack_panel(identity(6), Orientation::Vertical, vec![child]);
        let result = LayoutEngine::calculate(
            &root,
            1,
            2,
            size(100.0, 100.0),
            &provider(&[(child_id, size(40.0, 20.0))]),
        )
        .expect("layout with max width");
        assert_eq!(
            result.node(identity(6)).unwrap().desired_size,
            size(50.0, 20.0)
        );
        assert_eq!(result.node(child_id).unwrap().final_bounds.width, 70.0);
        assert_eq!(result.node(child_id).unwrap().final_bounds.height, 20.0);

        let mut centered = LayoutNodeSnapshot::leaf(child_id);
        centered.align = Align::Center;
        let root =
            LayoutNodeSnapshot::stack_panel(identity(7), Orientation::Vertical, vec![centered]);
        let result = LayoutEngine::calculate(
            &root,
            1,
            3,
            size(100.0, 100.0),
            &provider(&[(child_id, size(40.0, 20.0))]),
        )
        .expect("centered layout");
        assert_eq!(result.node(child_id).unwrap().final_bounds.x, 30.0);
        assert_eq!(result.node(child_id).unwrap().final_bounds.y, 0.0);
    }

    #[test]
    fn layout_engine_rejects_failed_or_overflowing_pass_without_mutating_previous_snapshot() {
        let a = identity(8);
        let root = LayoutNodeSnapshot::stack_panel(
            identity(9),
            Orientation::Vertical,
            vec![LayoutNodeSnapshot::leaf(a)],
        );
        let previous = LayoutEngine::calculate(
            &root,
            1,
            1,
            size(10.0, 10.0),
            &provider(&[(a, size(5.0, 5.0))]),
        )
        .expect("baseline snapshot");

        let mut failing = provider(&[(a, size(6.0, 6.0))]);
        failing.fail_on = Some(a);
        assert_eq!(
            LayoutEngine::calculate(&root, 2, 2, size(10.0, 10.0), &failing),
            Err(LayoutError::MeasurementFailed)
        );
        assert_eq!(previous.tree_revision, 1);
        assert_eq!(previous.node(a).unwrap().desired_size, size(5.0, 5.0));

        let overflow_root = LayoutNodeSnapshot::stack_panel(
            identity(10),
            Orientation::Vertical,
            vec![
                LayoutNodeSnapshot::leaf(identity(11)),
                LayoutNodeSnapshot::leaf(identity(12)),
            ],
        );
        let huge = provider(&[
            (identity(11), size(1.0, f64::MAX)),
            (identity(12), size(1.0, f64::MAX)),
        ]);
        assert_eq!(
            LayoutEngine::calculate(&overflow_root, 3, 3, size(1.0, f64::MAX), &huge),
            Err(LayoutError::ArithmeticOverflow)
        );
    }

    #[test]
    fn layout_engine_rejects_invalid_geometry_constraints_and_duplicate_identities() {
        assert_eq!(Size::new(f64::NAN, 1.0), Err(LayoutError::InvalidGeometry));
        assert_eq!(
            AxisConstraints::new(2.0, Some(1.0)),
            Err(LayoutError::InvalidConstraints)
        );

        let repeated = identity(13);
        let root = LayoutNodeSnapshot::stack_panel(
            identity(14),
            Orientation::Vertical,
            vec![
                LayoutNodeSnapshot::leaf(repeated),
                LayoutNodeSnapshot::leaf(repeated),
            ],
        );
        assert_eq!(
            LayoutEngine::calculate(
                &root,
                1,
                1,
                size(20.0, 20.0),
                &provider(&[(repeated, size(1.0, 1.0))]),
            ),
            Err(LayoutError::InvalidTree)
        );
    }
}
