//! Linux system-default font resolution, shaping, metrics, and glyph mask realization.

use freemvvm_core::abi_support::{
    FontRequest, GlyphPlacement, GlyphRunSnapshot, Rect, ResolvedFontIdentity, ResolvedGlyphRun,
    ResolvedTextSnapshot, TextMetrics, TextRequest, TextService, TextServiceError,
};
use freetype::bitmap::PixelMode;
use freetype::face::LoadFlag;
use freetype::{Face, Library};
use harfbuzz::sys;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use unicode_segmentation::UnicodeSegmentation;

const FONT_SIZE_DIP: u32 = 14;
const HARFBUZZ_SCALE: i32 = FONT_SIZE_DIP as i32 * 64;
const RESOLVER_PROFILE: &str = "linux-fontconfig-system-sans-serif-v1";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct GlyphCacheKey {
    font_digest: [u8; 32],
    font_size_dip: u32,
    glyph_id: u32,
}

#[derive(Clone, Debug)]
pub(super) struct GlyphMask {
    pub(super) width: u32,
    pub(super) rows: u32,
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) ascent_dip: f64,
    pub(super) pixels: Arc<[u8]>,
}

#[derive(Clone, Debug)]
struct FontCandidate {
    path: PathBuf,
    index: u32,
    family: String,
    style: String,
}

#[derive(Clone, Debug)]
struct FontRun {
    candidate: usize,
    text: String,
}

/// Stateless adapter except for memoized grayscale glyph masks shared with the renderer.
pub(super) struct LinuxTextService {
    glyph_masks: Mutex<HashMap<GlyphCacheKey, Arc<GlyphMask>>>,
}

impl LinuxTextService {
    pub(super) fn new() -> Self {
        Self {
            glyph_masks: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    fn glyph_mask_count(&self) -> usize {
        self.glyph_masks.lock().expect("mask cache lock").len()
    }

    #[cfg(test)]
    fn glyph_mask_pixel_count(&self) -> usize {
        self.glyph_masks
            .lock()
            .expect("mask cache lock")
            .values()
            .map(|mask| mask.pixels.len())
            .sum()
    }

    pub(super) fn glyph_mask(
        &self,
        font_digest: [u8; 32],
        font_size_dip: u32,
        glyph_id: u32,
    ) -> Result<Arc<GlyphMask>, TextServiceError> {
        self.glyph_masks
            .lock()
            .map_err(|_| TextServiceError::RasterizationFailed)?
            .get(&GlyphCacheKey {
                font_digest,
                font_size_dip,
                glyph_id,
            })
            .cloned()
            .ok_or(TextServiceError::RasterizationFailed)
    }
}

impl TextService for LinuxTextService {
    fn resolve_and_shape(
        &self,
        request: &TextRequest,
    ) -> Result<ResolvedTextSnapshot, TextServiceError> {
        if request.font != FontRequest::PlatformDefault {
            return Err(TextServiceError::FontResolutionFailed);
        }
        if request.text.is_empty() {
            return Ok(ResolvedTextSnapshot::new(
                request.revision,
                TextMetrics::new(0.0, 0.0)?,
                Arc::<[ResolvedGlyphRun]>::from([]),
            ));
        }

        let candidates = font_candidates(&request.text)?;
        let library = Library::init().map_err(|_| TextServiceError::Unavailable)?;
        let mut faces: Vec<Option<Face>> = (0..candidates.len()).map(|_| None).collect();
        let runs = choose_font_runs(&request.text, &candidates, &library, &mut faces)?;
        let mut font_digests = HashMap::new();
        for run in &runs {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                font_digests.entry(run.candidate)
            {
                let bytes = fs::read(&candidates[run.candidate].path)
                    .map_err(|_| TextServiceError::FontResolutionFailed)?;
                entry.insert(Sha256::digest(bytes).into());
            }
        }
        let line_height = runs
            .iter()
            .filter_map(|run| faces[run.candidate].as_ref()?.size_metrics())
            .map(|metrics| metrics.height as f64 / 64.0)
            .fold(0.0_f64, f64::max)
            .max(FONT_SIZE_DIP as f64);
        let mut glyph_runs = Vec::new();
        glyph_runs
            .try_reserve(runs.len())
            .map_err(|_| TextServiceError::Unavailable)?;
        let mut pen_x = 0.0;

        for run in runs {
            let candidate = &candidates[run.candidate];
            let digest = *font_digests
                .get(&run.candidate)
                .ok_or(TextServiceError::FontResolutionFailed)?;
            let face = faces[run.candidate]
                .as_ref()
                .ok_or(TextServiceError::FontResolutionFailed)?;
            let run_start = pen_x;
            let placements = shape_run(candidate, &run.text, &mut pen_x)?;
            for glyph in &placements {
                self.cache_mask(digest, face, glyph.glyph_id())?;
            }
            let bounds = Rect::new(run_start, 0.0, pen_x - run_start, line_height)
                .map_err(|_| TextServiceError::InvalidGlyphRun)?;
            let glyphs = GlyphRunSnapshot::new(
                digest,
                FONT_SIZE_DIP as f64,
                bounds,
                Arc::<[GlyphPlacement]>::from(placements),
            )
            .map_err(|_| TextServiceError::InvalidGlyphRun)?;
            glyph_runs.push(ResolvedGlyphRun::new(
                ResolvedFontIdentity {
                    resolver_profile: Arc::from(RESOLVER_PROFILE),
                    family: Arc::from(candidate.family.as_str()),
                    style: Arc::from(candidate.style.as_str()),
                    asset_digest: digest,
                    face_index: candidate.index,
                },
                glyphs,
            )?);
        }

        Ok(ResolvedTextSnapshot::new(
            request.revision,
            TextMetrics::new(pen_x, line_height)?,
            Arc::<[ResolvedGlyphRun]>::from(glyph_runs),
        ))
    }
}

impl LinuxTextService {
    fn cache_mask(
        &self,
        font_digest: [u8; 32],
        face: &Face,
        glyph_id: u32,
    ) -> Result<(), TextServiceError> {
        let key = GlyphCacheKey {
            font_digest,
            font_size_dip: FONT_SIZE_DIP,
            glyph_id,
        };
        let mut cache = self
            .glyph_masks
            .lock()
            .map_err(|_| TextServiceError::RasterizationFailed)?;
        if cache.contains_key(&key) {
            return Ok(());
        }
        face.load_glyph(glyph_id, LoadFlag::RENDER)
            .map_err(|_| TextServiceError::RasterizationFailed)?;
        let bitmap = face.glyph().bitmap();
        let width =
            u32::try_from(bitmap.width()).map_err(|_| TextServiceError::RasterizationFailed)?;
        let rows =
            u32::try_from(bitmap.rows()).map_err(|_| TextServiceError::RasterizationFailed)?;
        let pixels = normalize_bitmap(&bitmap, width, rows)?;
        let slot = face.glyph();
        let ascent_dip = face
            .size_metrics()
            .ok_or(TextServiceError::RasterizationFailed)?
            .ascender as f64
            / 64.0;
        let mask = GlyphMask {
            width,
            rows,
            left: slot.bitmap_left(),
            top: slot.bitmap_top(),
            ascent_dip,
            pixels,
        };
        cache.insert(key, Arc::new(mask));
        Ok(())
    }
}

fn normalize_bitmap(
    bitmap: &freetype::bitmap::Bitmap,
    width: u32,
    rows: u32,
) -> Result<Arc<[u8]>, TextServiceError> {
    let width = usize::try_from(width).map_err(|_| TextServiceError::RasterizationFailed)?;
    let rows = usize::try_from(rows).map_err(|_| TextServiceError::RasterizationFailed)?;
    let size = width
        .checked_mul(rows)
        .ok_or(TextServiceError::RasterizationFailed)?;
    let mut normalized = Vec::new();
    normalized
        .try_reserve_exact(size)
        .map_err(|_| TextServiceError::RasterizationFailed)?;
    if size == 0 {
        return Ok(Arc::from(normalized));
    }
    let pitch = bitmap.pitch();
    let row_stride =
        usize::try_from(pitch.unsigned_abs()).map_err(|_| TextServiceError::RasterizationFailed)?;
    if row_stride == 0 {
        return Err(TextServiceError::RasterizationFailed);
    }
    let source = bitmap.buffer();
    let source_size = row_stride
        .checked_mul(rows)
        .ok_or(TextServiceError::RasterizationFailed)?;
    if source.len() < source_size {
        return Err(TextServiceError::RasterizationFailed);
    }
    let mode = bitmap
        .pixel_mode()
        .map_err(|_| TextServiceError::RasterizationFailed)?;
    let num_grays = bitmap.raw().num_grays;
    for row in 0..rows {
        let source_row = if pitch < 0 { rows - 1 - row } else { row };
        let start = source_row
            .checked_mul(row_stride)
            .ok_or(TextServiceError::RasterizationFailed)?;
        let source_row = &source[start..start + row_stride];
        for column in 0..width {
            let coverage = match mode {
                PixelMode::Gray => {
                    let value = *source_row
                        .get(column)
                        .ok_or(TextServiceError::RasterizationFailed)?;
                    if num_grays > 1 && num_grays != 256 {
                        let gray_levels = u32::try_from(num_grays - 1)
                            .map_err(|_| TextServiceError::RasterizationFailed)?;
                        u8::try_from(u32::from(value) * 255 / gray_levels)
                            .map_err(|_| TextServiceError::RasterizationFailed)?
                    } else {
                        value
                    }
                }
                PixelMode::Mono => {
                    let byte = *source_row
                        .get(column / 8)
                        .ok_or(TextServiceError::RasterizationFailed)?;
                    if byte & (0x80 >> (column % 8)) != 0 {
                        255
                    } else {
                        0
                    }
                }
                _ => return Err(TextServiceError::RasterizationFailed),
            };
            normalized.push(coverage);
        }
    }
    Ok(Arc::from(normalized))
}

fn font_candidates(text: &str) -> Result<Vec<FontCandidate>, TextServiceError> {
    let config = fontconfig::Fontconfig::new().ok_or(TextServiceError::Unavailable)?;
    let mut pattern =
        fontconfig::Pattern::new(&config).map_err(|_| TextServiceError::FontResolutionFailed)?;
    let family = CString::new("sans-serif").map_err(|_| TextServiceError::FontResolutionFailed)?;
    pattern
        .add_string(fontconfig::FC_FAMILY, &family)
        .map_err(|_| TextServiceError::FontResolutionFailed)?;
    pattern
        .add_integer(fontconfig::FC_WEIGHT, fontconfig::FC_WEIGHT_REGULAR)
        .map_err(|_| TextServiceError::FontResolutionFailed)?;
    let mut charset =
        fontconfig::CharSet::new(&config).map_err(|_| TextServiceError::FontResolutionFailed)?;
    for scalar in text.chars() {
        charset
            .add_char(scalar)
            .map_err(|_| TextServiceError::FontResolutionFailed)?;
    }
    pattern
        .add_charset(charset)
        .map_err(|_| TextServiceError::FontResolutionFailed)?;
    let font_set = pattern
        .sort_fonts(fontconfig::UnicodeCoverage::NoTrim)
        .map_err(|_| TextServiceError::FontResolutionFailed)?;

    let mut candidates = Vec::new();
    for pattern in font_set.iter() {
        let Ok(path) = pattern.filename() else {
            continue;
        };
        let Ok(index) = pattern.face_index().and_then(|index| {
            u32::try_from(index).map_err(|_| fontconfig::FontconfigError::Failed)
        }) else {
            continue;
        };
        let family = pattern
            .get_string(fontconfig::FC_FAMILY)
            .or_else(|_| pattern.name())
            .unwrap_or("sans-serif")
            .to_owned();
        let style = pattern
            .get_string(fontconfig::FC_STYLE)
            .unwrap_or("Regular")
            .to_owned();
        let path = PathBuf::from(path);
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        candidates.push(FontCandidate {
            path,
            index,
            family,
            style,
        });
    }
    if candidates.is_empty() {
        return Err(TextServiceError::FontResolutionFailed);
    }
    Ok(candidates)
}

fn choose_font_runs(
    text: &str,
    candidates: &[FontCandidate],
    library: &Library,
    faces: &mut [Option<Face>],
) -> Result<Vec<FontRun>, TextServiceError> {
    group_graphemes_by_candidate(text, |grapheme| {
        let mut selected = None;
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            if faces[candidate_index].is_none() {
                let face = library
                    .new_face(&candidate.path, candidate.index as isize)
                    .ok()?;
                face.set_pixel_sizes(0, FONT_SIZE_DIP).ok()?;
                faces[candidate_index] = Some(face);
            }
            let face = faces[candidate_index].as_ref()?;
            if grapheme.chars().all(|scalar| {
                face.get_char_index(scalar as usize)
                    .is_some_and(|glyph_id| glyph_id != 0)
            }) {
                selected = Some(candidate_index);
                break;
            }
        }
        selected
    })
}

fn group_graphemes_by_candidate(
    text: &str,
    mut select_candidate: impl FnMut(&str) -> Option<usize>,
) -> Result<Vec<FontRun>, TextServiceError> {
    let mut runs: Vec<FontRun> = Vec::new();
    for grapheme in text.graphemes(true) {
        let candidate = select_candidate(grapheme).ok_or(TextServiceError::FontResolutionFailed)?;
        match runs.last_mut() {
            Some(previous) if previous.candidate == candidate => previous.text.push_str(grapheme),
            _ => runs.push(FontRun {
                candidate,
                text: grapheme.to_owned(),
            }),
        }
    }
    Ok(runs)
}

fn shape_run(
    candidate: &FontCandidate,
    text: &str,
    pen_x: &mut f64,
) -> Result<Vec<GlyphPlacement>, TextServiceError> {
    let path = std::os::unix::ffi::OsStrExt::as_bytes(candidate.path.as_os_str());
    let path = CString::new(path).map_err(|_| TextServiceError::ShapingFailed)?;
    // HarfBuzz's C ABI is isolated here. Resource wrappers destroy in reverse dependency order.
    let mut resources = HarfBuzzResources::default();
    // SAFETY: `path` is a live NUL-terminated path for this call; HarfBuzz returns an owned blob.
    let blob = non_null(unsafe { sys::hb_blob_create_from_file(path.as_ptr()) })
        .ok_or(TextServiceError::ShapingFailed)?;
    resources.blob = Some(blob);
    // SAFETY: `blob` is a live owned HarfBuzz object and the call returns an owned face reference.
    let face = non_null(unsafe { sys::hb_face_create(blob.as_ptr(), candidate.index) })
        .ok_or(TextServiceError::ShapingFailed)?;
    resources.face = Some(face);
    // SAFETY: `face` remains live until `HarfBuzzResources` drops the font before the face.
    let font = non_null(unsafe { sys::hb_font_create(face.as_ptr()) })
        .ok_or(TextServiceError::ShapingFailed)?;
    resources.font = Some(font);
    // SAFETY: this constructor creates an owned empty buffer.
    let buffer =
        non_null(unsafe { sys::hb_buffer_create() }).ok_or(TextServiceError::ShapingFailed)?;
    resources.buffer = Some(buffer);
    // SAFETY: all four objects are live, input bytes are valid UTF-8, and feature pointer is null
    // with a zero feature count as required by HarfBuzz.
    unsafe {
        sys::hb_ot_font_set_funcs(font.as_ptr());
        sys::hb_font_set_scale(font.as_ptr(), HARFBUZZ_SCALE, HARFBUZZ_SCALE);
        sys::hb_buffer_add_utf8(
            buffer.as_ptr(),
            text.as_ptr().cast(),
            i32::try_from(text.len()).map_err(|_| TextServiceError::ShapingFailed)?,
            0,
            i32::try_from(text.len()).map_err(|_| TextServiceError::ShapingFailed)?,
        );
        sys::hb_buffer_guess_segment_properties(buffer.as_ptr());
        sys::hb_shape(font.as_ptr(), buffer.as_ptr(), std::ptr::null(), 0);
    }

    let mut info_count = 0;
    let mut position_count = 0;
    // SAFETY: `buffer` is live and both output counts are valid writable pointers.
    let infos = unsafe { sys::hb_buffer_get_glyph_infos(buffer.as_ptr(), &mut info_count) };
    // SAFETY: `buffer` is live and `position_count` is a valid writable pointer.
    let positions =
        unsafe { sys::hb_buffer_get_glyph_positions(buffer.as_ptr(), &mut position_count) };
    if info_count != position_count || (info_count > 0 && (infos.is_null() || positions.is_null()))
    {
        return Err(TextServiceError::ShapingFailed);
    }
    let infos = if info_count == 0 {
        &[]
    } else {
        // SAFETY: HarfBuzz returned a non-null info pointer and count for the live unchanged buffer.
        unsafe { std::slice::from_raw_parts(infos, info_count as usize) }
    };
    let positions = if position_count == 0 {
        &[]
    } else {
        // SAFETY: HarfBuzz returned a non-null positions pointer and matching count for the buffer.
        unsafe { std::slice::from_raw_parts(positions, position_count as usize) }
    };
    let mut glyphs = Vec::new();
    glyphs
        .try_reserve(infos.len())
        .map_err(|_| TextServiceError::Unavailable)?;
    for (info, position) in infos.iter().zip(positions) {
        let x_offset = position.x_offset as f64 / 64.0;
        let y_offset = position.y_offset as f64 / 64.0;
        let advance_x = position.x_advance as f64 / 64.0;
        let advance_y = -(position.y_advance as f64) / 64.0;
        glyphs.push(
            GlyphPlacement::new(
                info.codepoint,
                *pen_x + x_offset,
                -y_offset,
                advance_x,
                advance_y,
            )
            .map_err(|_| TextServiceError::InvalidGlyphRun)?,
        );
        *pen_x += advance_x;
        if !pen_x.is_finite() {
            return Err(TextServiceError::InvalidGlyphRun);
        }
    }
    Ok(glyphs)
}

fn non_null<T>(pointer: *mut T) -> Option<NonNull<T>> {
    NonNull::new(pointer)
}

#[derive(Default)]
struct HarfBuzzResources {
    buffer: Option<NonNull<sys::hb_buffer_t>>,
    font: Option<NonNull<sys::hb_font_t>>,
    face: Option<NonNull<sys::hb_face_t>>,
    blob: Option<NonNull<sys::hb_blob_t>>,
}

impl Drop for HarfBuzzResources {
    fn drop(&mut self) {
        // SAFETY: each pointer is one owned reference; destroy children before their dependencies.
        unsafe {
            if let Some(buffer) = self.buffer {
                sys::hb_buffer_destroy(buffer.as_ptr());
            }
            if let Some(font) = self.font {
                sys::hb_font_destroy(font.as_ptr());
            }
            if let Some(face) = self.face {
                sys::hb_face_destroy(face.as_ptr());
            }
            if let Some(blob) = self.blob {
                sys::hb_blob_destroy(blob.as_ptr());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LinuxTextService, choose_font_runs, font_candidates, group_graphemes_by_candidate,
    };
    use freemvvm_core::abi_support::{TextRequest, TextService};
    use freetype::Library;

    #[test]
    fn linux_text_service_shapes_latin_japanese_and_mixed_corpus() {
        let service = LinuxTextService::new();
        for (revision, text) in ["FreeMvvm 0123", "日本語表示", "FreeMvvm 日本語"]
            .into_iter()
            .enumerate()
        {
            let request = TextRequest::platform_default(revision as u64 + 1, text);
            let snapshot = service
                .resolve_and_shape(&request)
                .expect("system default font resolves and shapes corpus");
            assert_eq!(snapshot.revision, request.revision);
            assert!(snapshot.metrics.width_dip() > 0.0);
            assert!(snapshot.metrics.height_dip() > 0.0);
            assert!(!snapshot.glyph_runs.is_empty());
            for run in snapshot.glyph_runs.iter() {
                assert!(!run.font.family.is_empty());
                assert!(!run.font.style.is_empty());
                assert_eq!(run.glyphs.font_digest(), &run.font.asset_digest);
                assert!(!run.glyphs.glyphs().is_empty());
            }
        }
        assert!(service.glyph_mask_count() > 0);
        assert!(service.glyph_mask_pixel_count() > 0);
    }

    #[test]
    fn linux_text_service_reports_uncovered_scalar_without_empty_success() {
        let service = LinuxTextService::new();
        let request = TextRequest::platform_default(1, "\u{10ffff}");

        assert_eq!(
            service.resolve_and_shape(&request),
            Err(freemvvm_core::abi_support::TextServiceError::FontResolutionFailed)
        );
    }

    #[test]
    fn linux_candidate_list_is_ordered_and_grapheme_assignment_can_cross_faces() {
        let candidates = font_candidates("A日本語").expect("test font configuration available");
        assert!(!candidates.is_empty());

        let library = Library::init().expect("FreeType available");
        let mut faces = (0..candidates.len()).map(|_| None).collect::<Vec<_>>();
        let assigned = choose_font_runs("A日本語", &candidates, &library, &mut faces)
            .expect("CJK and Latin have system coverage");
        assert_eq!(
            assigned
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "A日本語"
        );
        assert!(!assigned.is_empty());
    }

    #[test]
    fn linux_grapheme_fallback_keeps_clusters_intact_and_groups_adjacent_faces() {
        let assigned = group_graphemes_by_candidate("A日本語B", |grapheme| match grapheme {
            "A" | "B" => Some(0),
            "日" | "本" | "語" => Some(1),
            _ => None,
        })
        .expect("every grapheme has an assigned face");

        assert_eq!(assigned.len(), 3);
        assert_eq!(assigned[0].candidate, 0);
        assert_eq!(assigned[0].text, "A");
        assert_eq!(assigned[1].candidate, 1);
        assert_eq!(assigned[1].text, "日本語");
        assert_eq!(assigned[2].candidate, 0);
        assert_eq!(assigned[2].text, "B");
    }

    #[test]
    fn linux_grapheme_fallback_fails_when_no_candidate_covers_a_cluster() {
        assert!(
            group_graphemes_by_candidate("A\u{10ffff}", |grapheme| {
                (grapheme == "A").then_some(0)
            })
            .is_err()
        );
    }
}
