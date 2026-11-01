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
    use std::path::Path;
    use windows::Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FACE_TYPE_TRUETYPE, DWRITE_FONT_FILE_TYPE_TRUETYPE,
        DWRITE_FONT_SIMULATIONS_NONE, DWriteCreateFactory, IDWriteFactory, IDWriteFontFace,
        IDWriteFontFile,
    };
    use windows::core::PCWSTR;

    pub(crate) struct FontResources {
        _factory: IDWriteFactory,
        faces: FontFaceCatalog<IDWriteFontFace>,
    }

    impl FontResources {
        pub(crate) fn new() -> Result<Self, ()> {
            // SAFETY: DirectWrite returns a shared factory whose COM lifetime is held by this
            // resource owner and released when it is dropped.
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

        pub(crate) fn resolve(&self, digest: &[u8; 32]) -> Option<&IDWriteFontFace> {
            self.faces.resolve(digest)
        }

        pub(crate) fn close(&mut self) {
            self.faces.clear();
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
