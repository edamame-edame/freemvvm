//! Core-neutral boundary between text controls and platform text realization.

use crate::platform::scene::{GlyphRunSnapshot, SceneError};
use std::sync::Arc;

/// A request for the active platform profile's default font, without OS-specific naming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontRequest {
    PlatformDefault,
}

/// Immutable Core input captured from one text-display revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextRequest {
    pub revision: u64,
    pub text: Arc<str>,
    pub font: FontRequest,
}

impl TextRequest {
    pub fn platform_default(revision: u64, text: impl Into<Arc<str>>) -> Self {
        Self {
            revision,
            text: text.into(),
            font: FontRequest::PlatformDefault,
        }
    }
}

/// Identity of the concrete font face resolved by a platform text service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFontIdentity {
    pub resolver_profile: Arc<str>,
    pub family: Arc<str>,
    pub style: Arc<str>,
    pub asset_digest: [u8; 32],
    pub face_index: u32,
}

/// A shaped run paired with the exact face used to realize its glyphs.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedGlyphRun {
    pub font: ResolvedFontIdentity,
    pub glyphs: GlyphRunSnapshot,
}

impl ResolvedGlyphRun {
    pub fn new(
        font: ResolvedFontIdentity,
        glyphs: GlyphRunSnapshot,
    ) -> Result<Self, TextServiceError> {
        if font.asset_digest != *glyphs.font_digest() {
            return Err(TextServiceError::FontIdentityMismatch);
        }
        Ok(Self { font, glyphs })
    }
}

/// Layout measurement returned from the same realization as the glyph runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    width_dip: f64,
    height_dip: f64,
}

impl TextMetrics {
    pub fn new(width_dip: f64, height_dip: f64) -> Result<Self, TextServiceError> {
        if !width_dip.is_finite() || !height_dip.is_finite() || width_dip < 0.0 || height_dip < 0.0
        {
            return Err(TextServiceError::InvalidMetrics);
        }
        Ok(Self {
            width_dip,
            height_dip,
        })
    }

    pub fn width_dip(self) -> f64 {
        self.width_dip
    }

    pub fn height_dip(self) -> f64 {
        self.height_dip
    }
}

/// One immutable, revision-stamped result shared by measurement and scene generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTextSnapshot {
    pub revision: u64,
    pub metrics: TextMetrics,
    pub glyph_runs: Arc<[ResolvedGlyphRun]>,
}

impl ResolvedTextSnapshot {
    pub fn new(
        revision: u64,
        metrics: TextMetrics,
        glyph_runs: impl Into<Arc<[ResolvedGlyphRun]>>,
    ) -> Self {
        Self {
            revision,
            metrics,
            glyph_runs: glyph_runs.into(),
        }
    }
}

/// Failure to resolve or realize a text request; no partial snapshot is published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextServiceError {
    Unavailable,
    FontResolutionFailed,
    ShapingFailed,
    RevisionMismatch,
    FontIdentityMismatch,
    InvalidGlyphRun,
    InvalidMetrics,
    RasterizationFailed,
}

impl From<SceneError> for TextServiceError {
    fn from(_: SceneError) -> Self {
        Self::InvalidGlyphRun
    }
}

/// Platform implementation for default-font resolution, shaping, and measurement.
pub trait TextService: Send + Sync {
    fn resolve_and_shape(
        &self,
        request: &TextRequest,
    ) -> Result<ResolvedTextSnapshot, TextServiceError>;
}

/// Validates the platform result before Core makes it available to downstream stages.
pub(crate) fn resolve_text(
    service: &dyn TextService,
    request: &TextRequest,
) -> Result<ResolvedTextSnapshot, TextServiceError> {
    let snapshot = service.resolve_and_shape(request)?;
    if snapshot.revision != request.revision {
        return Err(TextServiceError::RevisionMismatch);
    }
    if !snapshot.metrics.width_dip.is_finite()
        || !snapshot.metrics.height_dip.is_finite()
        || snapshot.metrics.width_dip < 0.0
        || snapshot.metrics.height_dip < 0.0
    {
        return Err(TextServiceError::InvalidMetrics);
    }
    if snapshot
        .glyph_runs
        .iter()
        .any(|run| run.font.asset_digest != *run.glyphs.font_digest())
    {
        return Err(TextServiceError::FontIdentityMismatch);
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::{
        FontRequest, ResolvedFontIdentity, ResolvedGlyphRun, ResolvedTextSnapshot, TextMetrics,
        TextRequest, TextService, TextServiceError, resolve_text,
    };
    use crate::platform::scene::{GlyphPlacement, GlyphRunSnapshot, Rect};
    use std::sync::Arc;

    struct FakeTextService {
        profile: &'static str,
        digest: [u8; 32],
        failure: Option<TextServiceError>,
        revision_delta: u64,
    }

    impl TextService for FakeTextService {
        fn resolve_and_shape(
            &self,
            request: &TextRequest,
        ) -> Result<ResolvedTextSnapshot, TextServiceError> {
            if let Some(error) = self.failure {
                return Err(error);
            }
            let identity = ResolvedFontIdentity {
                resolver_profile: Arc::from(self.profile),
                family: Arc::from("Test Sans"),
                style: Arc::from("Regular"),
                asset_digest: self.digest,
                face_index: 0,
            };
            let glyphs = GlyphRunSnapshot::new(
                self.digest,
                14.0,
                Rect::new(0.0, 0.0, 12.0, 16.0).expect("valid bounds"),
                Arc::from([GlyphPlacement::new(1, 0.0, 0.0, 12.0, 0.0).expect("valid glyph")]),
            )?;
            let run = ResolvedGlyphRun::new(identity, glyphs)?;
            Ok(ResolvedTextSnapshot::new(
                request.revision + self.revision_delta,
                TextMetrics::new(12.0, 16.0).expect("valid metrics"),
                Arc::from([run]),
            ))
        }
    }

    #[test]
    fn text_service_request_is_platform_neutral_and_result_records_profile_identity() {
        let request = TextRequest::platform_default(7, Arc::<str>::from("日本語"));
        assert_eq!(request.font, FontRequest::PlatformDefault);
        assert_eq!(&*request.text, "日本語");

        let linux = FakeTextService {
            profile: "test-linux",
            digest: [1; 32],
            failure: None,
            revision_delta: 0,
        };
        let other = FakeTextService {
            profile: "test-other-platform",
            digest: [2; 32],
            failure: None,
            revision_delta: 0,
        };
        let linux_snapshot = resolve_text(&linux, &request).expect("Linux profile resolves");
        let other_snapshot = resolve_text(&other, &request).expect("other profile resolves");

        assert_eq!(linux_snapshot.revision, request.revision);
        assert_eq!(linux_snapshot.metrics, other_snapshot.metrics);
        assert_ne!(
            linux_snapshot.glyph_runs[0].font,
            other_snapshot.glyph_runs[0].font
        );
        assert_eq!(
            linux_snapshot.glyph_runs[0].glyphs.font_digest(),
            &linux_snapshot.glyph_runs[0].font.asset_digest
        );
    }

    #[test]
    fn text_metrics_reject_negative_or_non_finite_dimensions() {
        assert_eq!(
            TextMetrics::new(-1.0, 12.0),
            Err(TextServiceError::InvalidMetrics)
        );
        assert_eq!(
            TextMetrics::new(12.0, f64::INFINITY),
            Err(TextServiceError::InvalidMetrics)
        );
    }

    #[test]
    fn text_service_propagates_resolution_failure_without_a_partial_snapshot() {
        let service = FakeTextService {
            profile: "unavailable",
            digest: [0; 32],
            failure: Some(TextServiceError::FontResolutionFailed),
            revision_delta: 0,
        };
        let request = TextRequest::platform_default(3, "FreeMvvm");

        assert_eq!(
            resolve_text(&service, &request),
            Err(TextServiceError::FontResolutionFailed)
        );
    }

    #[test]
    fn text_service_rejects_result_from_a_different_text_revision() {
        let service = FakeTextService {
            profile: "test-linux",
            digest: [3; 32],
            failure: None,
            revision_delta: 1,
        };
        let request = TextRequest::platform_default(9, "revision");

        assert_eq!(
            resolve_text(&service, &request),
            Err(TextServiceError::RevisionMismatch)
        );
    }

    #[test]
    fn text_service_rejects_a_glyph_run_with_a_different_font_identity() {
        let identity = ResolvedFontIdentity {
            resolver_profile: Arc::from("test-linux"),
            family: Arc::from("Test Sans"),
            style: Arc::from("Regular"),
            asset_digest: [4; 32],
            face_index: 0,
        };
        let glyphs = GlyphRunSnapshot::new(
            [5; 32],
            14.0,
            Rect::new(0.0, 0.0, 1.0, 1.0).expect("valid bounds"),
            Arc::<[GlyphPlacement]>::from([]),
        )
        .expect("valid glyph run");

        assert_eq!(
            ResolvedGlyphRun::new(identity, glyphs),
            Err(TextServiceError::FontIdentityMismatch)
        );
    }
}
