use freemvvm_core::abi_support::{DrawOp, GlyphRunSnapshot, Rect, SceneSnapshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PixelRect {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

pub(crate) trait DrawTarget {
    fn begin(&mut self) -> Result<(), ()>;
    fn clear(&mut self, rgb: [f32; 3]) -> Result<(), ()>;
    fn set_clip(&mut self, rect: PixelRect) -> Result<(), ()>;
    fn fill_rect(&mut self, rect: PixelRect, rgba: [f32; 4]) -> Result<(), ()>;
    fn draw_glyph_run(
        &mut self,
        run: &GlyphRunSnapshot,
        rgba: [f32; 4],
        scale: (f64, f64),
    ) -> Result<(), ()>;
    fn finish(&mut self) -> Result<(), ()>;
}

pub(crate) fn render_scene(target: &mut impl DrawTarget, scene: &SceneSnapshot) -> Result<(), ()> {
    target.begin()?;
    let result = (|| {
        target.clear(scene.background_rgb())?;
        for operation in scene.operations() {
            match operation {
                DrawOp::Clip(rect) => target.set_clip(to_pixels(*rect, scene)?)?,
                DrawOp::FillRect { bounds, rgba } => {
                    target.fill_rect(to_pixels(*bounds, scene)?, *rgba)?
                }
                DrawOp::GlyphRun { run, rgba } => {
                    let scale = scene.device_scale();
                    target.draw_glyph_run(run, *rgba, (scale.x(), scale.y()))?;
                }
            }
        }
        Ok(())
    })();
    let finish = target.finish();
    result.and(finish)
}

fn to_pixels(rect: Rect, scene: &SceneSnapshot) -> Result<PixelRect, ()> {
    let scale = scene.device_scale();
    let left = (rect.x() * scale.x()) as f32;
    let top = (rect.y() * scale.y()) as f32;
    let right = ((rect.x() + rect.width()) * scale.x()) as f32;
    let bottom = ((rect.y() + rect.height()) * scale.y()) as f32;
    if [left, top, right, bottom]
        .iter()
        .all(|value| value.is_finite())
    {
        Ok(PixelRect {
            left,
            top,
            right,
            bottom,
        })
    } else {
        Err(())
    }
}

#[cfg(target_os = "windows")]
pub(crate) mod direct2d {
    use super::{DrawTarget, PixelRect, render_scene};
    use crate::text::directwrite::{FontResources, FontSource};
    use freemvvm_core::abi_support::GlyphRunSnapshot;
    use freemvvm_core::abi_support::{PlatformOperationError, SceneSnapshot};
    use std::path::Path;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D_RECT_F, D2D_SIZE_U, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
    };
    use windows::Win32::Graphics::Direct2D::{
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
        D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
        D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
        D2D1_RENDER_TARGET_USAGE_NONE, D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget,
    };
    use windows::Win32::Graphics::DirectWrite::{
        DWRITE_GLYPH_RUN, DWRITE_MEASURING_MODE_NATURAL, IDWriteFontFace,
    };

    pub(crate) struct HwndRenderer {
        target: ID2D1HwndRenderTarget,
        text: FontResources,
        active_clip: bool,
    }

    impl HwndRenderer {
        pub(crate) fn new(
            hwnd: HWND,
            width_px: u32,
            height_px: u32,
        ) -> Result<Self, PlatformOperationError> {
            // SAFETY: the factory and target are COM interfaces owned by this window worker.
            let factory: ID2D1Factory =
                unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }
                    .map_err(|_| PlatformOperationError)?;
            let properties = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT::default(),
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let hwnd_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U {
                    width: width_px,
                    height: height_px,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            // SAFETY: hwnd is live and owned by the calling worker; property structs are valid.
            let target = unsafe { factory.CreateHwndRenderTarget(&properties, &hwnd_properties) }
                .map_err(|_| PlatformOperationError)?;
            Ok(Self {
                target,
                text: FontResources::new().map_err(|_| PlatformOperationError)?,
                active_clip: false,
            })
        }

        #[allow(dead_code)] // The profile/TextService integration supplies fonts in a later unit.
        pub(crate) fn load_ttf(&mut self, digest: [u8; 32], path: &Path) -> Result<(), ()> {
            self.text.load_ttf(digest, path)
        }

        #[allow(dead_code)] // The profile/TextService integration supplies faces in a later unit.
        pub(crate) fn register_system_face(
            &mut self,
            digest: [u8; 32],
            face: IDWriteFontFace,
        ) -> Result<(), ()> {
            self.text.register_system_face(digest, face)
        }

        pub(crate) fn resize(&mut self, width_px: u32, height_px: u32) -> Result<(), ()> {
            let size = D2D_SIZE_U {
                width: width_px,
                height: height_px,
            };
            // SAFETY: target is live and called only by its owning window worker.
            unsafe { self.target.Resize(&size) }.map_err(|_| ())
        }

        pub(crate) fn render(&mut self, scene: &SceneSnapshot) -> Result<(), ()> {
            render_scene(self, scene)
        }

        #[allow(dead_code)] // Called by the Windows Control/Scene text connector when it lands.
        pub(crate) fn shape_text(
            &mut self,
            text: &str,
            font_size: f64,
            max_width: f64,
            source: FontSource<'_>,
        ) -> Result<Vec<GlyphRunSnapshot>, ()> {
            self.text.shape_text(text, font_size, max_width, source)
        }
    }

    impl DrawTarget for HwndRenderer {
        fn begin(&mut self) -> Result<(), ()> {
            // SAFETY: target is used serially on the worker that created it.
            unsafe { self.target.BeginDraw() };
            Ok(())
        }

        fn clear(&mut self, rgb: [f32; 3]) -> Result<(), ()> {
            let color = D2D1_COLOR_F {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: 1.0,
            };
            // SAFETY: color is a live value for the duration of the call.
            unsafe { self.target.Clear(Some(&color)) };
            Ok(())
        }

        fn set_clip(&mut self, rect: PixelRect) -> Result<(), ()> {
            if self.active_clip {
                // SAFETY: active_clip tracks the matching PushAxisAlignedClip call.
                unsafe { self.target.PopAxisAlignedClip() };
            }
            let rect = D2D_RECT_F {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            };
            // SAFETY: rect is valid for the duration of the call; clip is replaced, not stacked.
            unsafe {
                self.target.PushAxisAlignedClip(
                    &rect,
                    windows::Win32::Graphics::Direct2D::D2D1_ANTIALIAS_MODE_ALIASED,
                )
            };
            self.active_clip = true;
            Ok(())
        }

        fn fill_rect(&mut self, rect: PixelRect, rgba: [f32; 4]) -> Result<(), ()> {
            let color = D2D1_COLOR_F {
                r: rgba[0],
                g: rgba[1],
                b: rgba[2],
                a: rgba[3],
            };
            // SAFETY: brush and rectangle are owned for the call; the brush is retained by D2D.
            let brush =
                unsafe { self.target.CreateSolidColorBrush(&color, None) }.map_err(|_| ())?;
            let rect = D2D_RECT_F {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            };
            // SAFETY: target is in BeginDraw/EndDraw and both arguments are valid for this call.
            unsafe { self.target.FillRectangle(&rect, &brush) };
            Ok(())
        }

        fn draw_glyph_run(
            &mut self,
            run: &GlyphRunSnapshot,
            rgba: [f32; 4],
            scale: (f64, f64),
        ) -> Result<(), ()> {
            let face = self.text.resolve(run.font_digest()).ok_or(())?;
            let color = D2D1_COLOR_F {
                r: rgba[0],
                g: rgba[1],
                b: rgba[2],
                a: rgba[3],
            };
            // SAFETY: the brush is a COM resource retained through this draw call.
            let brush =
                unsafe { self.target.CreateSolidColorBrush(&color, None) }.map_err(|_| ())?;

            for glyph in run.glyphs() {
                let glyph_id = u16::try_from(glyph.glyph_id()).map_err(|_| ())?;
                let (x, y) = glyph.position();
                let baseline = windows_numerics::Vector2 {
                    X: (x * scale.0) as f32,
                    Y: (y * scale.1) as f32,
                };
                if !baseline.X.is_finite() || !baseline.Y.is_finite() {
                    return Err(());
                }
                let mut glyph_run = DWRITE_GLYPH_RUN {
                    fontFace: std::mem::ManuallyDrop::new(Some(face.clone())),
                    fontEmSize: (run.font_size() * scale.1) as f32,
                    glyphCount: 1,
                    glyphIndices: &glyph_id,
                    glyphAdvances: std::ptr::null(),
                    glyphOffsets: std::ptr::null(),
                    isSideways: false.into(),
                    bidiLevel: 0,
                };
                if !glyph_run.fontEmSize.is_finite() || glyph_run.fontEmSize <= 0.0 {
                    // SAFETY: this run initialized the manually managed COM face reference.
                    unsafe { std::mem::ManuallyDrop::drop(&mut glyph_run.fontFace) };
                    return Err(());
                }
                // SAFETY: glyph arrays and face remain alive through the synchronous Direct2D call.
                unsafe {
                    self.target.DrawGlyphRun(
                        baseline,
                        &glyph_run,
                        &brush,
                        DWRITE_MEASURING_MODE_NATURAL,
                    )
                };
                // SAFETY: release the face reference retained in the ABI struct exactly once.
                unsafe { std::mem::ManuallyDrop::drop(&mut glyph_run.fontFace) };
            }
            Ok(())
        }

        fn finish(&mut self) -> Result<(), ()> {
            if self.active_clip {
                // SAFETY: active_clip tracks the matching PushAxisAlignedClip call.
                unsafe { self.target.PopAxisAlignedClip() };
                self.active_clip = false;
            }
            // SAFETY: balances BeginDraw on the same worker thread.
            unsafe { self.target.EndDraw(None, None) }.map_err(|_| ())
        }
    }

    impl Drop for HwndRenderer {
        fn drop(&mut self) {
            self.text.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DrawTarget, PixelRect, render_scene};
    use freemvvm_core::abi_support::{
        Damage, DeviceScale, DrawOp, GlyphPlacement, GlyphRunSnapshot, Rect, SceneSnapshot,
    };
    use std::sync::Arc;

    #[derive(Default)]
    struct RecordingTarget {
        calls: Vec<String>,
        rects: Vec<PixelRect>,
    }

    impl DrawTarget for RecordingTarget {
        fn begin(&mut self) -> Result<(), ()> {
            self.calls.push("begin".into());
            Ok(())
        }
        fn clear(&mut self, rgb: [f32; 3]) -> Result<(), ()> {
            self.calls.push(format!("clear:{rgb:?}"));
            Ok(())
        }
        fn set_clip(&mut self, rect: PixelRect) -> Result<(), ()> {
            self.calls.push("clip".into());
            self.rects.push(rect);
            Ok(())
        }
        fn fill_rect(&mut self, rect: PixelRect, rgba: [f32; 4]) -> Result<(), ()> {
            self.calls.push(format!("fill:{rgba:?}"));
            self.rects.push(rect);
            Ok(())
        }
        fn draw_glyph_run(
            &mut self,
            run: &GlyphRunSnapshot,
            rgba: [f32; 4],
            scale: (f64, f64),
        ) -> Result<(), ()> {
            self.calls.push(format!(
                "glyph:{:?}:{:?}:{scale:?}:{:?}",
                run.font_digest(),
                rgba,
                run.glyphs()
            ));
            Ok(())
        }
        fn finish(&mut self) -> Result<(), ()> {
            self.calls.push("finish".into());
            Ok(())
        }
    }

    fn scene(ops: Vec<DrawOp>) -> SceneSnapshot {
        SceneSnapshot::new(
            5,
            DeviceScale::new(2.0, 1.5).unwrap(),
            [0.1, 0.2, 0.3],
            Arc::from(ops),
            Damage::Full,
        )
        .unwrap()
    }

    #[test]
    fn platform_windows_render_preserves_scene_order_background_scale_clip_and_fill() {
        let rect = Rect::new(3.0, 4.0, 10.0, 8.0).unwrap();
        let mut target = RecordingTarget::default();
        render_scene(
            &mut target,
            &scene(vec![
                DrawOp::Clip(rect),
                DrawOp::FillRect {
                    bounds: rect,
                    rgba: [0.4, 0.5, 0.6, 0.7],
                },
            ]),
        )
        .unwrap();

        assert_eq!(target.calls[0], "begin");
        assert_eq!(target.calls[1], "clear:[0.1, 0.2, 0.3]");
        assert_eq!(
            &target.calls[2..],
            ["clip", "fill:[0.4, 0.5, 0.6, 0.7]", "finish"]
        );
        assert_eq!(
            target.rects[0],
            PixelRect {
                left: 6.0,
                top: 6.0,
                right: 26.0,
                bottom: 18.0
            }
        );
        assert_eq!(target.rects[1], target.rects[0]);
    }

    #[test]
    fn platform_windows_render_forwards_glyph_run_digest_color_glyphs_and_scale() {
        let placement = GlyphPlacement::new(23, 1.0, 2.0, 4.0, 0.0).unwrap();
        let glyphs = GlyphRunSnapshot::new(
            [7; 32],
            12.0,
            Rect::new(0.0, 0.0, 4.0, 4.0).unwrap(),
            Arc::from([placement]),
        )
        .unwrap();
        let mut target = RecordingTarget::default();
        render_scene(
            &mut target,
            &scene(vec![DrawOp::GlyphRun {
                run: glyphs,
                rgba: [0.0, 0.0, 0.0, 1.0],
            }]),
        )
        .unwrap();
        assert_eq!(target.calls[0], "begin");
        assert_eq!(target.calls[1], "clear:[0.1, 0.2, 0.3]");
        assert!(target.calls[2].contains("[7, 7, 7, 7"));
        assert!(target.calls[2].contains("[0.0, 0.0, 0.0, 1.0]"));
        assert!(target.calls[2].contains("(2.0, 1.5)"));
        assert_eq!(target.calls[3], "finish");
    }

    #[test]
    fn platform_windows_render_rejects_non_finite_pixel_conversion() {
        let rect = Rect::new(f64::MAX / 2.0, 0.0, 1.0, 1.0).unwrap();
        let mut target = RecordingTarget::default();
        assert!(
            render_scene(
                &mut target,
                &scene(vec![DrawOp::FillRect {
                    bounds: rect,
                    rgba: [1.0; 4]
                }])
            )
            .is_err()
        );
        assert_eq!(target.calls, ["begin", "clear:[0.1, 0.2, 0.3]", "finish"]);
    }

    #[test]
    fn platform_windows_render_finishes_failed_draw_without_reporting_success() {
        struct FailingTarget(Vec<&'static str>);
        impl DrawTarget for FailingTarget {
            fn begin(&mut self) -> Result<(), ()> {
                self.0.push("begin");
                Ok(())
            }
            fn clear(&mut self, _: [f32; 3]) -> Result<(), ()> {
                self.0.push("clear");
                Ok(())
            }
            fn set_clip(&mut self, _: PixelRect) -> Result<(), ()> {
                Ok(())
            }
            fn fill_rect(&mut self, _: PixelRect, _: [f32; 4]) -> Result<(), ()> {
                self.0.push("fill-failed");
                Err(())
            }
            fn draw_glyph_run(
                &mut self,
                _: &GlyphRunSnapshot,
                _: [f32; 4],
                _: (f64, f64),
            ) -> Result<(), ()> {
                self.0.push("glyph-failed");
                Err(())
            }
            fn finish(&mut self) -> Result<(), ()> {
                self.0.push("finish");
                Ok(())
            }
        }

        let mut target = FailingTarget(Vec::new());
        assert!(
            render_scene(
                &mut target,
                &scene(vec![DrawOp::FillRect {
                    bounds: Rect::new(0.0, 0.0, 5.0, 6.0).unwrap(),
                    rgba: [1.0; 4],
                }])
            )
            .is_err()
        );
        assert_eq!(target.0, ["begin", "clear", "fill-failed", "finish"]);
    }

    #[test]
    fn platform_windows_text_draw_failure_finishes_batch_and_fails_frame() {
        let run = GlyphRunSnapshot::new(
            [9; 32],
            12.0,
            Rect::new(0.0, 0.0, 4.0, 4.0).unwrap(),
            Arc::from([]),
        )
        .unwrap();
        struct FailingTextTarget(Vec<&'static str>);
        impl DrawTarget for FailingTextTarget {
            fn begin(&mut self) -> Result<(), ()> {
                self.0.push("begin");
                Ok(())
            }
            fn clear(&mut self, _: [f32; 3]) -> Result<(), ()> {
                self.0.push("clear");
                Ok(())
            }
            fn set_clip(&mut self, _: PixelRect) -> Result<(), ()> {
                Ok(())
            }
            fn fill_rect(&mut self, _: PixelRect, _: [f32; 4]) -> Result<(), ()> {
                Ok(())
            }
            fn draw_glyph_run(
                &mut self,
                _: &GlyphRunSnapshot,
                _: [f32; 4],
                _: (f64, f64),
            ) -> Result<(), ()> {
                self.0.push("font-face-missing");
                Err(())
            }
            fn finish(&mut self) -> Result<(), ()> {
                self.0.push("finish");
                Ok(())
            }
        }

        let mut target = FailingTextTarget(Vec::new());
        assert!(
            render_scene(
                &mut target,
                &scene(vec![DrawOp::GlyphRun {
                    run,
                    rgba: [0.0, 0.0, 0.0, 1.0],
                }])
            )
            .is_err()
        );
        assert_eq!(target.0, ["begin", "clear", "font-face-missing", "finish"]);
    }
}
