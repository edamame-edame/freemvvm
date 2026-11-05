#[cfg(any(target_os = "windows", test))]
use std::collections::HashMap;

/// Owns font-face resources for one Windows renderer lifetime.
///
/// The digest is supplied by the shaping/font-profile owner. This catalog never chooses a
/// replacement face: a missing digest is a render failure.
#[cfg(any(target_os = "windows", test))]
pub(crate) struct FontFaceCatalog<F> {
    faces: HashMap<[u8; 32], F>,
}

#[cfg(any(target_os = "windows", test))]
impl<F> Default for FontFaceCatalog<F> {
    fn default() -> Self {
        Self {
            faces: HashMap::new(),
        }
    }
}

#[cfg(any(target_os = "windows", test))]
impl<F> FontFaceCatalog<F> {
    pub(crate) fn register(&mut self, digest: [u8; 32], face: F) -> Result<(), F> {
        match self.faces.entry(digest) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(face);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => Err(face),
        }
    }

    pub(crate) fn resolve(&self, digest: &[u8; 32]) -> Option<&F> {
        self.faces.get(digest)
    }

    pub(crate) fn clear(&mut self) {
        self.faces.clear();
    }
}

#[cfg(target_os = "windows")]
pub(crate) mod directwrite {
    use super::FontFaceCatalog;
    use freemvvm_core::abi_support::{
        FontRequest, GlyphPlacement, GlyphRunSnapshot, Rect, ResolvedFontIdentity,
        ResolvedGlyphRun, ResolvedTextSnapshot, TextMetrics, TextRequest, TextServiceError,
    };
    use sha2::{Digest, Sha256};
    use std::{cell::RefCell, fs, path::Path, ptr, sync::Arc};
    use windows::Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FACE_TYPE_TRUETYPE, DWRITE_FONT_FILE_TYPE_TRUETYPE,
        DWRITE_FONT_SIMULATIONS_NONE, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_NORMAL, DWRITE_GLYPH_RUN, DWRITE_GLYPH_RUN_DESCRIPTION, DWRITE_MATRIX,
        DWRITE_MEASURING_MODE, DWRITE_TEXT_METRICS, DWRITE_WORD_WRAPPING_NO_WRAP,
        DWriteCreateFactory, IDWriteFactory, IDWriteFactory5, IDWriteFontCollection,
        IDWriteFontFace, IDWriteFontFace3, IDWriteFontFile, IDWriteLocalFontFileLoader,
        IDWritePixelSnapping_Impl, IDWriteTextRenderer, IDWriteTextRenderer_Impl,
    };
    use windows::core::implement;
    use windows::core::{ComObject, Error, HRESULT, Interface, PCWSTR};
    use windows_sys::Win32::{
        Globalization::GetUserDefaultLocaleName,
        UI::WindowsAndMessaging::{
            NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SystemParametersInfoW,
        },
    };

    pub(crate) struct FontResources {
        _factory: IDWriteFactory,
        faces: FontFaceCatalog<IDWriteFontFace>,
    }

    impl FontResources {
        pub(crate) fn new() -> Result<Self, ()> {
            // SAFETY: DirectWrite returns a shared factory whose COM lifetime is held by this
            // resource owner and released when it is dropped.
            // SAFETY: DirectWrite initializes the requested factory interface on success.
            let factory =
                unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }.map_err(|_| ())?;
            Ok(Self {
                _factory: factory,
                faces: FontFaceCatalog::default(),
            })
        }

        /// Loads an explicit TrueType file without installing it into the operating system.
        /// `digest` must be the same identity used by the TextService's shaped GlyphRun.
        #[allow(dead_code)] // Called when the Windows TextService supplies an explicit TTF source.
        pub(crate) fn load_ttf(&mut self, digest: [u8; 32], path: &Path) -> Result<(), ()> {
            let path = path.to_str().ok_or(())?;
            let mut wide_path: Vec<u16> = path.encode_utf16().collect();
            wide_path.push(0);
            // SAFETY: wide_path is NUL-terminated and remains alive for the call.
            let file: IDWriteFontFile = unsafe {
                self._factory
                    .CreateFontFileReference(PCWSTR(wide_path.as_ptr()), None)
            }
            .map_err(|_| ())?;

            let mut supported = windows::core::BOOL(0);
            let mut file_type = Default::default();
            let mut face_type = Default::default();
            let mut face_count = 0;
            // SAFETY: each output pointer refers to a live writable local.
            unsafe {
                file.Analyze(
                    &mut supported,
                    &mut file_type,
                    Some(&mut face_type),
                    &mut face_count,
                )
            }
            .map_err(|_| ())?;
            if !supported.as_bool()
                || file_type != DWRITE_FONT_FILE_TYPE_TRUETYPE
                || face_type != DWRITE_FONT_FACE_TYPE_TRUETYPE
                || face_count != 1
            {
                return Err(());
            }

            // SAFETY: the analyzed file is one supported TrueType face; DirectWrite retains its
            // file loader reference through the returned face.
            let face = unsafe {
                self._factory.CreateFontFace(
                    face_type,
                    &[Some(file)],
                    0,
                    DWRITE_FONT_SIMULATIONS_NONE,
                )
            }
            .map_err(|_| ())?;
            self.faces.register(digest, face).map_err(|_| ())
        }

        /// Registers the exact face selected by the platform TextService for a shaped run.
        #[allow(dead_code)] // Called when the Windows TextService resolves a system font face.
        pub(crate) fn register_system_face(
            &mut self,
            digest: [u8; 32],
            face: IDWriteFontFace,
        ) -> Result<(), ()> {
            self.faces.register(digest, face).map_err(|_| ())
        }

        /// Shapes one paragraph and retains the exact DirectWrite faces used by its glyph runs.
        /// The caller and this resource owner must be the same Windows worker.
        #[allow(dead_code)] // Used by isolated DirectWrite resolver tests.
        pub(crate) fn shape_text(
            &mut self,
            text: &str,
            font_size: f64,
            max_width: f64,
            source: FontSource<'_>,
        ) -> Result<Vec<GlyphRunSnapshot>, ()> {
            self.shape_text_detailed(text, font_size, max_width, source)
                .map(|(_, _, runs)| runs.into_iter().map(|run| run.glyphs).collect())
        }

        pub(crate) fn resolve_text(
            &mut self,
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

            let (width, height, glyph_runs) = self
                .shape_text_detailed(
                    &request.text,
                    14.0,
                    f64::from(f32::MAX),
                    FontSource::SystemDefault,
                )
                .map_err(|_| TextServiceError::ShapingFailed)?;
            Ok(ResolvedTextSnapshot::new(
                request.revision,
                TextMetrics::new(width, height)?,
                Arc::<[ResolvedGlyphRun]>::from(glyph_runs),
            ))
        }

        fn shape_text_detailed(
            &mut self,
            text: &str,
            font_size: f64,
            max_width: f64,
            source: FontSource<'_>,
        ) -> Result<(f64, f64, Vec<ResolvedGlyphRun>), ()> {
            if !font_size.is_finite()
                || font_size <= 0.0
                || !max_width.is_finite()
                || max_width <= 0.0
                || font_size > f32::MAX as f64
                || max_width > f32::MAX as f64
            {
                return Err(());
            }
            if text.is_empty() {
                return Ok((0.0, 0.0, Vec::new()));
            }

            let (family_name, collection) = match source {
                FontSource::SystemDefault => (system_message_font_family()?, None),
                FontSource::TrueType(path) => {
                    let collection = self.ttf_collection(path)?;
                    (first_family_name(&collection)?, Some(collection))
                }
            };
            let mut family_wide: Vec<u16> = family_name.encode_utf16().collect();
            family_wide.push(0);
            let locale = user_locale()?;
            let mut locale_wide: Vec<u16> = locale.encode_utf16().collect();
            locale_wide.push(0);
            let collection = collection
                .as_ref()
                .map(|font_collection| font_collection.cast::<IDWriteFontCollection>())
                .transpose()
                .map_err(|_| ())?;
            // SAFETY: both strings are NUL-terminated and remain alive through the call.
            let format = unsafe {
                self._factory.CreateTextFormat(
                    PCWSTR(family_wide.as_ptr()),
                    collection.as_ref(),
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    font_size as f32,
                    PCWSTR(locale_wide.as_ptr()),
                )
            }
            .map_err(|_| ())?;
            // Keep the initial text contract to one paragraph without mandatory soft wrapping.
            // SAFETY: format is a live DirectWrite interface and the enum value is valid.
            unsafe { format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP) }.map_err(|_| ())?;

            let text_wide: Vec<u16> = text.encode_utf16().collect();
            // SAFETY: text_wide and format remain alive for this synchronous layout creation.
            let layout = unsafe {
                self._factory
                    .CreateTextLayout(&text_wide, &format, max_width as f32, f32::MAX)
            }
            .map_err(|_| ())?;
            let mut metrics = DWRITE_TEXT_METRICS::default();
            // SAFETY: metrics is a writable output local and layout is live.
            unsafe { layout.GetMetrics(&mut metrics) }.map_err(|_| ())?;
            let bounds = Rect::new(
                f64::from(metrics.left),
                f64::from(metrics.top),
                f64::from(metrics.widthIncludingTrailingWhitespace),
                f64::from(metrics.height),
            )
            .map_err(|_| ())?;
            let capture = ComObject::new(GlyphCapture {
                bounds,
                faces: RefCell::new(&mut self.faces),
                runs: RefCell::new(Vec::new()),
            });
            let renderer: IDWriteTextRenderer = capture.to_interface();
            // SAFETY: layout and renderer are live; the callback runs synchronously before return.
            unsafe { layout.Draw(None, &renderer, 0.0, 0.0) }.map_err(|_| ())?;
            let runs = capture.get().runs.borrow().clone();
            if runs.is_empty() {
                return Err(());
            }
            let resolved = runs
                .into_iter()
                .map(|run| {
                    ResolvedGlyphRun::new(
                        ResolvedFontIdentity {
                            resolver_profile: Arc::from("windows-directwrite-system-message-font"),
                            family: Arc::from(run.family),
                            style: Arc::from(run.style),
                            asset_digest: *run.glyphs.font_digest(),
                            face_index: run.face_index,
                        },
                        run.glyphs,
                    )
                    .map_err(|_| ())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                f64::from(metrics.widthIncludingTrailingWhitespace),
                f64::from(metrics.height),
                resolved,
            ))
        }

        fn ttf_collection(
            &self,
            path: &Path,
        ) -> Result<windows::Win32::Graphics::DirectWrite::IDWriteFontCollection1, ()> {
            let path_text = path.to_str().ok_or(())?;
            let mut path_wide: Vec<u16> = path_text.encode_utf16().collect();
            path_wide.push(0);
            // SAFETY: path_wide is NUL-terminated and remains alive for the call.
            let file: IDWriteFontFile = unsafe {
                self._factory
                    .CreateFontFileReference(PCWSTR(path_wide.as_ptr()), None)
            }
            .map_err(|_| ())?;
            let mut supported = windows::core::BOOL(0);
            let mut file_type = Default::default();
            let mut face_type = Default::default();
            let mut face_count = 0;
            // SAFETY: all Analyze output pointers refer to live writable locals.
            unsafe {
                file.Analyze(
                    &mut supported,
                    &mut file_type,
                    Some(&mut face_type),
                    &mut face_count,
                )
            }
            .map_err(|_| ())?;
            if !supported.as_bool()
                || file_type != DWRITE_FONT_FILE_TYPE_TRUETYPE
                || face_type != DWRITE_FONT_FACE_TYPE_TRUETYPE
                || face_count != 1
            {
                return Err(());
            }
            let factory5: IDWriteFactory5 = self._factory.cast().map_err(|_| ())?;
            // SAFETY: factory5 is a live DirectWrite interface.
            let builder = unsafe { factory5.CreateFontSetBuilder() }.map_err(|_| ())?;
            // SAFETY: file is a live DirectWrite font file reference.
            unsafe { builder.AddFontFile(&file) }.map_err(|_| ())?;
            // SAFETY: builder is live and owns the added font reference.
            let font_set = unsafe { builder.CreateFontSet() }.map_err(|_| ())?;
            // SAFETY: font_set is live and was created by the same DirectWrite factory.
            let collection =
                unsafe { factory5.CreateFontCollectionFromFontSet(&font_set) }.map_err(|_| ())?;
            Ok(collection)
        }

        pub(crate) fn resolve(&self, digest: &[u8; 32]) -> Option<&IDWriteFontFace> {
            self.faces.resolve(digest)
        }

        pub(crate) fn close(&mut self) {
            self.faces.clear();
        }
    }

    #[allow(dead_code)] // The Runtime scene producer will call the service when text is connected.
    #[derive(Clone, Copy)]
    pub(crate) enum FontSource<'a> {
        SystemDefault,
        TrueType(&'a Path),
    }

    #[implement(IDWriteTextRenderer)]
    struct GlyphCapture<'a> {
        bounds: Rect,
        faces: RefCell<&'a mut FontFaceCatalog<IDWriteFontFace>>,
        runs: RefCell<Vec<CapturedGlyphRun>>,
    }

    #[derive(Clone)]
    struct CapturedGlyphRun {
        glyphs: GlyphRunSnapshot,
        family: String,
        style: String,
        face_index: u32,
    }

    impl IDWritePixelSnapping_Impl for GlyphCapture_Impl<'_> {
        fn IsPixelSnappingDisabled(
            &self,
            _clientdrawingcontext: *const core::ffi::c_void,
        ) -> windows::core::Result<windows::core::BOOL> {
            Ok(windows::core::BOOL(1))
        }

        fn GetCurrentTransform(
            &self,
            _clientdrawingcontext: *const core::ffi::c_void,
            transform: *mut DWRITE_MATRIX,
        ) -> windows::core::Result<()> {
            if transform.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
            }
            // SAFETY: the pointer was checked for null and DirectWrite supplied writable output.
            unsafe {
                transform.write(std::mem::zeroed());
                (*transform).m11 = 1.0;
                (*transform).m22 = 1.0;
            }
            Ok(())
        }

        fn GetPixelsPerDip(
            &self,
            _clientdrawingcontext: *const core::ffi::c_void,
        ) -> windows::core::Result<f32> {
            Ok(1.0)
        }
    }

    impl IDWriteTextRenderer_Impl for GlyphCapture_Impl<'_> {
        fn DrawGlyphRun(
            &self,
            _clientdrawingcontext: *const core::ffi::c_void,
            baselineoriginx: f32,
            baselineoriginy: f32,
            _measuringmode: DWRITE_MEASURING_MODE,
            glyphrun: *const DWRITE_GLYPH_RUN,
            _glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
            _clientdrawingeffect: windows::core::Ref<windows::core::IUnknown>,
        ) -> windows::core::Result<()> {
            if glyphrun.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
            }
            // SAFETY: DirectWrite supplies a valid run pointer for the duration of this callback.
            let run = unsafe { &*glyphrun };
            let face = run
                .fontFace
                .as_ref()
                .ok_or_else(|| Error::from_hresult(HRESULT(0x80004003u32 as i32)))?
                .clone();
            let digest = face_digest(&face)
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            let mut faces = self.faces.borrow_mut();
            if faces.resolve(&digest).is_none() {
                faces
                    .register(digest, face.clone())
                    .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            }
            if run.glyphCount == 0 || run.glyphIndices.is_null() || run.glyphAdvances.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004005u32 as i32)));
            }
            // SAFETY: glyphCount is nonzero and the corresponding pointers were checked above.
            let indices =
                unsafe { std::slice::from_raw_parts(run.glyphIndices, run.glyphCount as usize) };
            // SAFETY: glyphCount is nonzero and DirectWrite supplies one advance per glyph.
            let advances =
                unsafe { std::slice::from_raw_parts(run.glyphAdvances, run.glyphCount as usize) };
            let offsets = if run.glyphOffsets.is_null() {
                vec![Default::default(); run.glyphCount as usize]
            } else {
                // SAFETY: DirectWrite supplies glyphCount offsets when the pointer is non-null.
                unsafe { std::slice::from_raw_parts(run.glyphOffsets, run.glyphCount as usize) }
                    .to_vec()
            };
            let mut cursor_x = f64::from(baselineoriginx);
            let mut glyphs = Vec::with_capacity(run.glyphCount as usize);
            for ((glyph_id, advance), offset) in indices.iter().zip(advances).zip(&offsets) {
                let glyph = GlyphPlacement::new(
                    u32::from(*glyph_id),
                    cursor_x + f64::from(offset.advanceOffset),
                    f64::from(baselineoriginy) - f64::from(offset.ascenderOffset),
                    f64::from(*advance),
                    0.0,
                )
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
                glyphs.push(glyph);
                cursor_x += f64::from(*advance);
            }
            let snapshot =
                GlyphRunSnapshot::new(digest, f64::from(run.fontEmSize), self.bounds, glyphs)
                    .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            let face3: IDWriteFontFace3 = face
                .cast()
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            // SAFETY: face3 is the same live face selected by DirectWrite for this glyph run.
            let family = unsafe { face3.GetFamilyNames() }
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            // SAFETY: face3 is the same live face selected by DirectWrite for this glyph run.
            let style = unsafe { face3.GetFaceNames() }
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            let family = localized_first_string(&family)
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            let style = localized_first_string(&style)
                .map_err(|_| Error::from_hresult(HRESULT(0x80004005u32 as i32)))?;
            // SAFETY: face remains live for this synchronous metadata query.
            let face_index = unsafe { face.GetIndex() };
            self.runs.borrow_mut().push(CapturedGlyphRun {
                glyphs: snapshot,
                family,
                style,
                face_index,
            });
            Ok(())
        }

        fn DrawUnderline(
            &self,
            _: *const core::ffi::c_void,
            _: f32,
            _: f32,
            _: *const windows::Win32::Graphics::DirectWrite::DWRITE_UNDERLINE,
            _: windows::core::Ref<windows::core::IUnknown>,
        ) -> windows::core::Result<()> {
            Err(Error::from_hresult(HRESULT(0x80004001u32 as i32)))
        }

        fn DrawStrikethrough(
            &self,
            _: *const core::ffi::c_void,
            _: f32,
            _: f32,
            _: *const windows::Win32::Graphics::DirectWrite::DWRITE_STRIKETHROUGH,
            _: windows::core::Ref<windows::core::IUnknown>,
        ) -> windows::core::Result<()> {
            Err(Error::from_hresult(HRESULT(0x80004001u32 as i32)))
        }

        fn DrawInlineObject(
            &self,
            _: *const core::ffi::c_void,
            _: f32,
            _: f32,
            _: windows::core::Ref<windows::Win32::Graphics::DirectWrite::IDWriteInlineObject>,
            _: windows::core::BOOL,
            _: windows::core::BOOL,
            _: windows::core::Ref<windows::core::IUnknown>,
        ) -> windows::core::Result<()> {
            Err(Error::from_hresult(HRESULT(0x80004001u32 as i32)))
        }
    }

    fn user_locale() -> Result<String, ()> {
        let mut buffer = [0u16; 85];
        // SAFETY: buffer is writable and its capacity is passed to the OS API.
        let length = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
        if length <= 1 {
            return Err(());
        }
        String::from_utf16(&buffer[..(length - 1) as usize]).map_err(|_| ())
    }

    fn system_message_font_family() -> Result<String, ()> {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        // SAFETY: metrics is initialized and writable with its exact structure size.
        let success = unsafe {
            SystemParametersInfoW(
                SPI_GETNONCLIENTMETRICS,
                metrics.cbSize,
                (&mut metrics as *mut NONCLIENTMETRICSW).cast(),
                0,
            )
        };
        if success == 0 {
            return Err(());
        }
        let name = &metrics.lfMessageFont.lfFaceName;
        let length = name
            .iter()
            .position(|character| *character == 0)
            .ok_or(())?;
        if length == 0 {
            return Err(());
        }
        String::from_utf16(&name[..length]).map_err(|_| ())
    }

    fn first_family_name(
        collection: &windows::Win32::Graphics::DirectWrite::IDWriteFontCollection1,
    ) -> Result<String, ()> {
        // SAFETY: collection is live and its family count is nonzero for a validated TTF set.
        let family = unsafe { collection.GetFontFamily(0) }.map_err(|_| ())?;
        // SAFETY: family is a live DirectWrite interface.
        let names = unsafe { family.GetFamilyNames() }.map_err(|_| ())?;
        // SAFETY: names is live and index zero is a valid localized-name entry.
        let length = unsafe { names.GetStringLength(0) }.map_err(|_| ())? as usize;
        let mut buffer = vec![0u16; length + 1];
        // SAFETY: buffer has room for the string and its terminator.
        unsafe { names.GetString(0, &mut buffer) }.map_err(|_| ())?;
        String::from_utf16(&buffer[..length]).map_err(|_| ())
    }

    fn localized_first_string(
        names: &windows::Win32::Graphics::DirectWrite::IDWriteLocalizedStrings,
    ) -> Result<String, ()> {
        // SAFETY: names is live and index zero is a valid localized-name entry.
        let length = unsafe { names.GetStringLength(0) }.map_err(|_| ())? as usize;
        let mut buffer = vec![0u16; length + 1];
        // SAFETY: buffer has room for the string and its terminator.
        unsafe { names.GetString(0, &mut buffer) }.map_err(|_| ())?;
        String::from_utf16(&buffer[..length]).map_err(|_| ())
    }

    fn face_digest(face: &IDWriteFontFace) -> Result<[u8; 32], ()> {
        let mut count = 0;
        // SAFETY: count is a writable output pointer and None requests only the file count.
        unsafe { face.GetFiles(&mut count, None) }.map_err(|_| ())?;
        if count == 0 {
            return Err(());
        }
        let mut files = vec![None; count as usize];
        // SAFETY: files has count writable entries for DirectWrite to populate.
        unsafe { face.GetFiles(&mut count, Some(files.as_mut_ptr())) }.map_err(|_| ())?;
        let mut hash = Sha256::new();
        let mut found_file = false;
        for file in files.into_iter().flatten() {
            let path = local_font_file_path(&file)?;
            let bytes = fs::read(path).map_err(|_| ())?;
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
            found_file = true;
        }
        if !found_file {
            return Err(());
        }
        // SAFETY: face remains live for this synchronous metadata query.
        hash.update(unsafe { face.GetIndex() }.to_le_bytes());
        Ok(hash.finalize().into())
    }

    fn local_font_file_path(file: &IDWriteFontFile) -> Result<std::path::PathBuf, ()> {
        // SAFETY: file is a live DirectWrite font file.
        let loader = unsafe { file.GetLoader() }.map_err(|_| ())?;
        let loader: IDWriteLocalFontFileLoader = loader.cast().map_err(|_| ())?;
        let mut key = ptr::null_mut();
        let mut key_size = 0;
        // SAFETY: both output locals receive the key owned by the live file reference.
        unsafe { file.GetReferenceKey(&mut key, &mut key_size) }.map_err(|_| ())?;
        if key.is_null() || key_size == 0 {
            return Err(());
        }
        // SAFETY: key and key_size were returned by the live font file above.
        let path_length =
            unsafe { loader.GetFilePathLengthFromKey(key, key_size) }.map_err(|_| ())? as usize;
        let mut path_wide = vec![0u16; path_length + 1];
        // SAFETY: path_wide provides space for the path and its NUL terminator.
        unsafe { loader.GetFilePathFromKey(key, key_size, &mut path_wide) }.map_err(|_| ())?;
        let path = String::from_utf16(&path_wide[..path_length]).map_err(|_| ())?;
        Ok(path.into())
    }

    #[cfg(test)]
    fn system_font_file_path(factory: &IDWriteFactory) -> Result<std::path::PathBuf, ()> {
        let mut collection = None;
        // SAFETY: collection is a writable output and factory is a live DirectWrite interface.
        unsafe { factory.GetSystemFontCollection(&mut collection, false) }.map_err(|_| ())?;
        let collection = collection.ok_or(())?;
        // SAFETY: collection is a live DirectWrite interface.
        let family_count = unsafe { collection.GetFontFamilyCount() };
        for family_index in 0..family_count {
            // SAFETY: family_index is bounded by GetFontFamilyCount.
            let family = unsafe { collection.GetFontFamily(family_index) }.map_err(|_| ())?;
            // SAFETY: family is a live DirectWrite interface.
            let font = unsafe {
                family.GetFirstMatchingFont(
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                )
            }
            .map_err(|_| ())?;
            // SAFETY: font is a live DirectWrite interface.
            let face = unsafe { font.CreateFontFace() }.map_err(|_| ())?;
            let mut file_count = 0;
            // SAFETY: file_count is writable and None requests only the file count.
            unsafe { face.GetFiles(&mut file_count, None) }.map_err(|_| ())?;
            let mut files = vec![None; file_count as usize];
            // SAFETY: files contains file_count writable slots.
            unsafe { face.GetFiles(&mut file_count, Some(files.as_mut_ptr())) }.map_err(|_| ())?;
            for file in files.into_iter().flatten() {
                let mut supported = windows::core::BOOL(0);
                let mut file_type = Default::default();
                let mut face_type = Default::default();
                let mut face_count = 0;
                // SAFETY: all Analyze output pointers refer to writable locals.
                unsafe {
                    file.Analyze(
                        &mut supported,
                        &mut file_type,
                        Some(&mut face_type),
                        &mut face_count,
                    )
                }
                .map_err(|_| ())?;
                if supported.as_bool()
                    && file_type == DWRITE_FONT_FILE_TYPE_TRUETYPE
                    && face_type == DWRITE_FONT_FACE_TYPE_TRUETYPE
                    && face_count == 1
                {
                    return local_font_file_path(&file);
                }
            }
        }
        Err(())
    }

    #[cfg(test)]
    mod tests {
        use super::{FontResources, FontSource};
        use std::path::Path;

        #[test]
        #[ignore = "manual Windows profile smoke; OS font selection is not a CI oracle"]
        fn platform_windows_text_service_shapes_system_text_and_registers_exact_faces() {
            let mut resources = FontResources::new().expect("DirectWrite factory");
            let runs = resources
                .shape_text("FreeMvvm 日本語", 16.0, 512.0, FontSource::SystemDefault)
                .expect("system UI font shapes supported text");
            assert!(!runs.is_empty());
            for run in &runs {
                assert!(!run.glyphs().is_empty());
                assert_eq!(run.text_color_rgba(), [0.0, 0.0, 0.0, 1.0]);
                assert!(resources.resolve(run.font_digest()).is_some());
            }
        }

        #[test]
        fn platform_windows_text_service_rejects_missing_and_non_font_files() {
            let mut resources = FontResources::new().expect("DirectWrite factory");
            assert!(
                resources
                    .shape_text(
                        "text",
                        16.0,
                        512.0,
                        FontSource::TrueType(Path::new("missing-font.ttf")),
                    )
                    .is_err()
            );
            let executable = std::env::current_exe().expect("test executable path");
            assert!(
                resources
                    .shape_text("text", 16.0, 512.0, FontSource::TrueType(&executable))
                    .is_err()
            );
        }

        #[test]
        #[ignore = "manual Windows profile smoke; TTF source is not a CI oracle"]
        fn platform_windows_text_service_shapes_explicit_system_ttf_without_installing_it() {
            let mut resources = FontResources::new().expect("DirectWrite factory");
            let path = super::system_font_file_path(&resources._factory)
                .expect("Windows system font collection has a local font file");
            let runs = resources
                .shape_text("FreeMvvm", 16.0, 512.0, FontSource::TrueType(&path))
                .expect("explicit TTF shapes supported text");
            assert!(!runs.is_empty());
            for run in &runs {
                assert!(resources.resolve(run.font_digest()).is_some());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FontFaceCatalog;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct Face(Arc<AtomicUsize>);

    impl Drop for Face {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn platform_windows_text_catalog_resolves_exact_digest_and_releases_faces_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut catalog = FontFaceCatalog::default();
        catalog
            .register([1; 32], Face(Arc::clone(&drops)))
            .expect("first registration succeeds");
        assert!(catalog.resolve(&[1; 32]).is_some());
        assert!(catalog.resolve(&[2; 32]).is_none());

        let duplicate = catalog.register([1; 32], Face(Arc::clone(&drops)));
        assert!(duplicate.is_err());
        drop(duplicate);
        assert_eq!(drops.load(Ordering::SeqCst), 1);

        catalog.clear();
        assert_eq!(drops.load(Ordering::SeqCst), 2);
        catalog.clear();
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
