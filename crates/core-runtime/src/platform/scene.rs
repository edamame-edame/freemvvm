use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceScale {
    x: f64,
    y: f64,
}

impl DeviceScale {
    pub fn new(x: f64, y: f64) -> Result<Self, SceneError> {
        if !x.is_finite() || !y.is_finite() || x <= 0.0 || y <= 0.0 {
            return Err(SceneError::InvalidScale);
        }
        Ok(Self { x, y })
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, SceneError> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width < 0.0
            || height < 0.0
        {
            return Err(SceneError::InvalidGeometry);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub fn x(self) -> f64 {
        self.x
    }

    pub fn y(self) -> f64 {
        self.y
    }

    pub fn width(self) -> f64 {
        self.width
    }

    pub fn height(self) -> f64 {
        self.height
    }

    fn right(self) -> Option<f64> {
        let right = self.x + self.width;
        right.is_finite().then_some(right)
    }

    fn bottom(self) -> Option<f64> {
        let bottom = self.y + self.height;
        bottom.is_finite().then_some(bottom)
    }

    fn union(self, other: Self) -> Option<Self> {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right()?.max(other.right()?);
        let bottom = self.bottom()?.max(other.bottom()?);
        Self::new(x, y, right - x, bottom - y).ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Damage {
    Full,
    Rect(Rect),
}

impl Default for Damage {
    fn default() -> Self {
        Self::Full
    }
}

impl Damage {
    pub fn rect(rect: Rect) -> Self {
        Self::Rect(rect)
    }

    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::Full, _) | (_, Self::Full) => Self::Full,
            (Self::Rect(left), Self::Rect(right)) => {
                left.union(right).map_or(Self::Full, Self::Rect)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPlacement {
    glyph_id: u32,
    x: f64,
    y: f64,
    advance_x: f64,
    advance_y: f64,
}

impl GlyphPlacement {
    pub fn new(
        glyph_id: u32,
        x: f64,
        y: f64,
        advance_x: f64,
        advance_y: f64,
    ) -> Result<Self, SceneError> {
        if !x.is_finite() || !y.is_finite() || !advance_x.is_finite() || !advance_y.is_finite() {
            return Err(SceneError::InvalidGlyphRun);
        }
        Ok(Self {
            glyph_id,
            x,
            y,
            advance_x,
            advance_y,
        })
    }

    pub fn glyph_id(self) -> u32 {
        self.glyph_id
    }

    pub fn position(self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn advance(self) -> (f64, f64) {
        (self.advance_x, self.advance_y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRunSnapshot {
    font_digest: [u8; 32],
    font_size: f64,
    bounds: Rect,
    glyphs: Arc<[GlyphPlacement]>,
}

impl GlyphRunSnapshot {
    pub fn new(
        font_digest: [u8; 32],
        font_size: f64,
        bounds: Rect,
        glyphs: impl Into<Arc<[GlyphPlacement]>>,
    ) -> Result<Self, SceneError> {
        if !font_size.is_finite() || font_size <= 0.0 {
            return Err(SceneError::InvalidGlyphRun);
        }
        Ok(Self {
            font_digest,
            font_size,
            bounds,
            glyphs: glyphs.into(),
        })
    }

    pub fn font_digest(&self) -> &[u8; 32] {
        &self.font_digest
    }

    pub fn font_size(&self) -> f64 {
        self.font_size
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn glyphs(&self) -> &[GlyphPlacement] {
        &self.glyphs
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawOp {
    Clip(Rect),
    FillRect { bounds: Rect, rgba: [f32; 4] },
    GlyphRun(GlyphRunSnapshot),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneError {
    InvalidGeometry,
    InvalidColor,
    InvalidGlyphRun,
    InvalidScale,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneSnapshot {
    layout_revision: u64,
    device_scale: DeviceScale,
    background_rgb: [f32; 3],
    operations: Arc<[DrawOp]>,
    damage: Damage,
}

impl SceneSnapshot {
    pub fn new(
        layout_revision: u64,
        device_scale: DeviceScale,
        background_rgb: [f32; 3],
        operations: impl Into<Arc<[DrawOp]>>,
        damage: Damage,
    ) -> Result<Self, SceneError> {
        let operations = operations.into();
        if background_rgb
            .iter()
            .any(|channel| !channel.is_finite() || !(0.0..=1.0).contains(channel))
            || operations.iter().any(|operation| {
                matches!(operation, DrawOp::FillRect { rgba, .. } if rgba.iter().any(|channel| !channel.is_finite()))
            })
        {
            return Err(SceneError::InvalidColor);
        }
        Ok(Self {
            layout_revision,
            device_scale,
            background_rgb,
            operations,
            damage,
        })
    }

    pub fn layout_revision(&self) -> u64 {
        self.layout_revision
    }

    pub fn device_scale(&self) -> DeviceScale {
        self.device_scale
    }

    pub fn background_rgb(&self) -> [f32; 3] {
        self.background_rgb
    }

    pub fn operations(&self) -> &[DrawOp] {
        &self.operations
    }

    pub fn damage(&self) -> Damage {
        self.damage
    }

    pub(crate) fn with_damage(&self, damage: Damage) -> Self {
        Self {
            layout_revision: self.layout_revision,
            device_scale: self.device_scale,
            background_rgb: self.background_rgb,
            operations: Arc::clone(&self.operations),
            damage,
        }
    }

    pub(crate) fn with_background_rgb(&self, background_rgb: [f32; 3]) -> Self {
        debug_assert!(
            background_rgb
                .iter()
                .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
        );
        Self {
            layout_revision: self.layout_revision,
            device_scale: self.device_scale,
            background_rgb,
            operations: Arc::clone(&self.operations),
            damage: self.damage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Damage, DeviceScale, DrawOp, GlyphPlacement, GlyphRunSnapshot, Rect, SceneError,
        SceneSnapshot,
    };
    use std::sync::Arc;

    #[test]
    fn platform_frame_damage_union_keeps_bounding_rect_and_overflow_falls_back_to_full() {
        let left = Rect::new(2.0, 4.0, 5.0, 3.0).expect("valid left damage");
        let right = Rect::new(5.0, 1.0, 4.0, 8.0).expect("valid right damage");
        assert_eq!(
            Damage::Rect(left).union(Damage::Rect(right)),
            Damage::Rect(Rect::new(2.0, 1.0, 7.0, 8.0).expect("union rect"))
        );
        assert_eq!(Damage::Full.union(Damage::Rect(left)), Damage::Full);

        let near_limit = Rect::new(f64::MAX, 0.0, f64::MAX, 1.0).expect("finite rect");
        assert_eq!(
            Damage::Rect(near_limit).union(Damage::Rect(left)),
            Damage::Full
        );
    }

    #[test]
    fn platform_frame_scene_snapshot_owns_immutable_operations_and_rejects_invalid_values() {
        let bounds = Rect::new(0.0, 0.0, 10.0, 8.0).expect("valid bounds");
        let glyph = GlyphPlacement::new(23, 1.0, 2.0, 4.0, 0.0).expect("valid glyph");
        let run = GlyphRunSnapshot::new([7; 32], 12.0, bounds, Arc::from([glyph]))
            .expect("valid glyph run");
        let snapshot = SceneSnapshot::new(
            9,
            DeviceScale::new(1.0, 1.0).expect("valid scale"),
            [0.05, 0.1, 0.15],
            Arc::from([
                DrawOp::Clip(bounds),
                DrawOp::FillRect {
                    bounds,
                    rgba: [0.1, 0.2, 0.3, 1.0],
                },
                DrawOp::GlyphRun(run),
            ]),
            Damage::Rect(bounds),
        )
        .expect("valid snapshot");
        let clone = snapshot.clone();

        assert_eq!(snapshot.layout_revision(), 9);
        assert_eq!(snapshot.device_scale().x(), 1.0);
        assert_eq!(snapshot.device_scale().y(), 1.0);
        assert_eq!(snapshot.background_rgb(), [0.05, 0.1, 0.15]);
        assert_eq!(snapshot.operations().len(), 3);
        assert_eq!(snapshot.operations(), clone.operations());
        assert_eq!(snapshot.damage(), Damage::Rect(bounds));
        let redamaged = snapshot.with_damage(Damage::Full);
        assert_eq!(redamaged.device_scale(), snapshot.device_scale());
        assert_eq!(redamaged.background_rgb(), snapshot.background_rgb());
        assert_eq!(redamaged.damage(), Damage::Full);

        assert_eq!(
            Rect::new(0.0, 0.0, f64::INFINITY, 1.0),
            Err(SceneError::InvalidGeometry)
        );
        assert_eq!(
            SceneSnapshot::new(
                10,
                DeviceScale::new(1.0, 1.0).expect("valid scale"),
                [0.0, 0.0, 0.0],
                Arc::from([DrawOp::FillRect {
                    bounds,
                    rgba: [f32::NAN, 0.0, 0.0, 1.0],
                }]),
                Damage::Full,
            ),
            Err(SceneError::InvalidColor)
        );
        assert_eq!(
            SceneSnapshot::new(
                11,
                DeviceScale::new(1.0, 1.0).expect("valid scale"),
                [0.0, 1.1, 0.0],
                Arc::from([]),
                Damage::Full,
            ),
            Err(SceneError::InvalidColor)
        );
        assert_eq!(
            SceneSnapshot::new(
                12,
                DeviceScale::new(1.0, 1.0).expect("valid scale"),
                [f32::NAN, 0.0, 0.0],
                Arc::from([]),
                Damage::Full,
            ),
            Err(SceneError::InvalidColor)
        );
        assert_eq!(
            DeviceScale::new(f64::NAN, 1.0),
            Err(SceneError::InvalidScale)
        );
        assert_eq!(DeviceScale::new(1.0, 0.0), Err(SceneError::InvalidScale));
        assert_eq!(
            DeviceScale::new(1.0, f64::INFINITY),
            Err(SceneError::InvalidScale)
        );
    }
}
