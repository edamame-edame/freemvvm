#[cfg(target_os = "linux")]
use crate::text_service::{GlyphMask, LinuxTextService};
#[cfg(target_os = "linux")]
use freemvvm_core::abi_support::GlyphPlacement;
use freemvvm_core::abi_support::{DeviceScale, DrawOp, Rect, SceneSnapshot};
#[cfg(target_os = "linux")]
use glow::HasContext;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PixelRect {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FillCommand {
    pub(super) bounds: PixelRect,
    pub(super) clip: PixelRect,
    pub(super) rgba: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GlyphRunCommand {
    pub(super) clip: PixelRect,
    pub(super) rgba: [f32; 4],
    pub(super) run: freemvvm_core::abi_support::GlyphRunSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum RenderCommand {
    Fill(FillCommand),
    GlyphRun(GlyphRunCommand),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RenderPlan {
    pub(super) background_rgb: [f32; 3],
    pub(super) commands: Vec<RenderCommand>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PlanError {
    UnsupportedOperation,
    NativeFailure,
}

pub(super) fn plan(
    scene: &SceneSnapshot,
    surface_size: (u32, u32),
) -> Result<RenderPlan, PlanError> {
    let viewport = PixelRect {
        x: 0,
        y: 0,
        width: surface_size.0,
        height: surface_size.1,
    };
    let mut active_clip = viewport;
    let mut commands = Vec::new();
    for operation in scene.operations() {
        match operation {
            DrawOp::Clip(rect) => {
                active_clip = to_pixel_rect(*rect, scene.device_scale(), surface_size);
            }
            DrawOp::FillRect { bounds, rgba } => {
                let pixel_bounds = to_pixel_rect(*bounds, scene.device_scale(), surface_size);
                if pixel_bounds.width != 0
                    && pixel_bounds.height != 0
                    && active_clip.width != 0
                    && active_clip.height != 0
                {
                    commands.push(RenderCommand::Fill(FillCommand {
                        bounds: pixel_bounds,
                        clip: active_clip,
                        rgba: *rgba,
                    }));
                }
            }
            DrawOp::GlyphRun { run, rgba } => {
                if run.glyphs().is_empty() {
                    return Err(PlanError::UnsupportedOperation);
                }
                if active_clip.width != 0 && active_clip.height != 0 {
                    commands.push(RenderCommand::GlyphRun(GlyphRunCommand {
                        clip: active_clip,
                        rgba: *rgba,
                        run: run.clone(),
                    }));
                }
            }
        }
    }
    Ok(RenderPlan {
        background_rgb: scene.background_rgb(),
        commands,
    })
}

fn to_pixel_rect(rect: Rect, scale: DeviceScale, surface_size: (u32, u32)) -> PixelRect {
    let left = (rect.x() * scale.x())
        .round()
        .clamp(0.0, f64::from(surface_size.0));
    let top = (rect.y() * scale.y())
        .round()
        .clamp(0.0, f64::from(surface_size.1));
    let right = ((rect.x() + rect.width()) * scale.x())
        .round()
        .clamp(0.0, f64::from(surface_size.0));
    let bottom = ((rect.y() + rect.height()) * scale.y())
        .round()
        .clamp(0.0, f64::from(surface_size.1));
    let x = left as u32;
    let y = top as u32;
    let right = (right as u32).max(x);
    let bottom = (bottom as u32).max(y);
    PixelRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    }
}

#[cfg(target_os = "linux")]
const GLYPH_ATLAS_SIZE: u32 = 2048;

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct GlyphAtlasKey {
    digest: [u8; 32],
    font_size_64: u32,
    scale_x: u64,
    scale_y: u64,
    glyph_id: u32,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AtlasEntry {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct AtlasPacker {
    x: u32,
    y: u32,
    row_height: u32,
}

#[cfg(target_os = "linux")]
impl AtlasPacker {
    fn allocate(&mut self, width: u32, height: u32) -> Result<AtlasEntry, PlanError> {
        let padded_width = width.checked_add(2).ok_or(PlanError::NativeFailure)?;
        let padded_height = height.checked_add(2).ok_or(PlanError::NativeFailure)?;
        if padded_width > GLYPH_ATLAS_SIZE || padded_height > GLYPH_ATLAS_SIZE {
            return Err(PlanError::NativeFailure);
        }
        if self.x + padded_width > GLYPH_ATLAS_SIZE {
            self.x = 0;
            self.y = self
                .y
                .checked_add(self.row_height)
                .ok_or(PlanError::NativeFailure)?;
            self.row_height = 0;
        }
        if self.y + padded_height > GLYPH_ATLAS_SIZE {
            return Err(PlanError::NativeFailure);
        }
        let entry = AtlasEntry {
            x: self.x + 1,
            y: self.y + 1,
            width,
            height,
        };
        self.x += padded_width;
        self.row_height = self.row_height.max(padded_height);
        Ok(entry)
    }
}

#[cfg(target_os = "linux")]
pub(super) struct GlesRenderer {
    gl: glow::Context,
    program: glow::Program,
    glyph_program: glow::Program,
    vertex_array: glow::VertexArray,
    vertex_buffer: glow::Buffer,
    atlas_texture: glow::Texture,
    atlas_packer: AtlasPacker,
    atlas_entries: HashMap<GlyphAtlasKey, AtlasEntry>,
    text_service: Arc<LinuxTextService>,
    viewport_uniform: Option<glow::UniformLocation>,
    color_uniform: Option<glow::UniformLocation>,
    glyph_viewport_uniform: Option<glow::UniformLocation>,
    glyph_color_uniform: Option<glow::UniformLocation>,
    glyph_atlas_uniform: Option<glow::UniformLocation>,
}

#[cfg(target_os = "linux")]
impl GlesRenderer {
    pub(super) unsafe fn new(
        text_service: Arc<LinuxTextService>,
        mut load: impl FnMut(&str) -> *const std::ffi::c_void,
    ) -> Result<Self, String> {
        // SAFETY: the caller supplies the current EGL context's GLES entrypoint loader.
        let gl = unsafe { glow::Context::from_loader_function(|name| load(name)) };
        let vertex_shader = compile_shader(
            &gl,
            glow::VERTEX_SHADER,
            "#version 300 es\nprecision highp float;\nlayout(location=0) in vec2 a_position;\nuniform vec2 u_viewport;\nvoid main() { vec2 p = a_position / u_viewport; gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0); }",
        )?;
        let fragment_shader = match compile_shader(
            &gl,
            glow::FRAGMENT_SHADER,
            "#version 300 es\nprecision mediump float;\nuniform vec4 u_color;\nout vec4 out_color;\nvoid main() { out_color = u_color; }",
        ) {
            Ok(shader) => shader,
            Err(error) => {
                // SAFETY: this shader was created by this context and remains live here.
                unsafe { gl.delete_shader(vertex_shader) };
                return Err(error);
            }
        };
        // SAFETY: the current EGL context is valid for GLES resource creation.
        let program = match unsafe { gl.create_program() } {
            Ok(program) => program,
            Err(error) => {
                // SAFETY: both shaders were created by this context and remain live here.
                unsafe {
                    gl.delete_shader(vertex_shader);
                    gl.delete_shader(fragment_shader);
                }
                return Err(error);
            }
        };
        // SAFETY: the program and both shaders are valid handles from this context.
        unsafe {
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
        }
        // SAFETY: the program was created by the current context.
        let linked = unsafe { gl.get_program_link_status(program) };
        // SAFETY: the program was created by the current context.
        let link_log = (!linked).then(|| unsafe { gl.get_program_info_log(program) });
        // SAFETY: both shader handles were created by this context and can now be released.
        unsafe {
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
        }
        if let Some(error) = link_log {
            // SAFETY: the program was created by this context and failed linking.
            unsafe { gl.delete_program(program) };
            return Err(error);
        }
        // SAFETY: the current EGL context is valid for GLES resource creation.
        let vertex_array = match unsafe { gl.create_vertex_array() } {
            Ok(vertex_array) => vertex_array,
            Err(error) => {
                // SAFETY: the program was created by this context and remains live here.
                unsafe { gl.delete_program(program) };
                return Err(error);
            }
        };
        // SAFETY: the current EGL context is valid for GLES resource creation.
        let vertex_buffer = match unsafe { gl.create_buffer() } {
            Ok(buffer) => buffer,
            Err(error) => {
                // SAFETY: both resources were created by this context and remain live here.
                unsafe {
                    gl.delete_vertex_array(vertex_array);
                    gl.delete_program(program);
                }
                return Err(error);
            }
        };
        // SAFETY: the linked program is valid in the current context.
        let viewport_uniform = unsafe { gl.get_uniform_location(program, "u_viewport") };
        // SAFETY: the linked program is valid in the current context.
        let color_uniform = unsafe { gl.get_uniform_location(program, "u_color") };
        let glyph_program = match link_program(
            &gl,
            "#version 300 es\nprecision highp float;\nlayout(location=0) in vec2 a_position;\nlayout(location=1) in vec2 a_uv;\nuniform vec2 u_viewport;\nout vec2 v_uv;\nvoid main() { vec2 p = a_position / u_viewport; gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0); v_uv = a_uv; }",
            "#version 300 es\nprecision mediump float;\nin vec2 v_uv;\nuniform vec4 u_color;\nuniform sampler2D u_atlas;\nout vec4 out_color;\nvoid main() { float coverage = texture(u_atlas, v_uv).r; out_color = vec4(u_color.rgb, u_color.a * coverage); }",
        ) {
            Ok(program) => program,
            Err(error) => {
                // SAFETY: these renderer resources were created by this still-current context.
                unsafe {
                    gl.delete_buffer(vertex_buffer);
                    gl.delete_vertex_array(vertex_array);
                    gl.delete_program(program);
                }
                return Err(error);
            }
        };
        // SAFETY: the linked glyph program is valid in the current context.
        let glyph_viewport_uniform =
            unsafe { gl.get_uniform_location(glyph_program, "u_viewport") };
        // SAFETY: the linked glyph program is valid in the current context.
        let glyph_color_uniform = unsafe { gl.get_uniform_location(glyph_program, "u_color") };
        // SAFETY: the linked glyph program is valid in the current context.
        let glyph_atlas_uniform = unsafe { gl.get_uniform_location(glyph_program, "u_atlas") };
        // SAFETY: the current GLES context is valid for texture allocation.
        let atlas_texture = match unsafe { gl.create_texture() } {
            Ok(texture) => texture,
            Err(error) => {
                // SAFETY: these renderer resources were created by this still-current context.
                unsafe {
                    gl.delete_program(glyph_program);
                    gl.delete_buffer(vertex_buffer);
                    gl.delete_vertex_array(vertex_array);
                    gl.delete_program(program);
                }
                return Err(error);
            }
        };
        // SAFETY: the atlas texture is a current-context object and has bounded dimensions.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(atlas_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                GLYPH_ATLAS_SIZE as i32,
                GLYPH_ATLAS_SIZE as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            if gl.get_error() != glow::NO_ERROR {
                gl.delete_texture(atlas_texture);
                gl.delete_program(glyph_program);
                gl.delete_buffer(vertex_buffer);
                gl.delete_vertex_array(vertex_array);
                gl.delete_program(program);
                return Err("failed to allocate GLES glyph atlas".into());
            }
        }
        Ok(Self {
            gl,
            program,
            glyph_program,
            vertex_array,
            vertex_buffer,
            atlas_texture,
            atlas_packer: AtlasPacker::default(),
            atlas_entries: HashMap::new(),
            text_service,
            viewport_uniform,
            color_uniform,
            glyph_viewport_uniform,
            glyph_color_uniform,
            glyph_atlas_uniform,
        })
    }

    pub(super) fn render(
        &mut self,
        scene: &SceneSnapshot,
        surface_size: (u32, u32),
    ) -> Result<(), PlanError> {
        let plan = plan(scene, surface_size)?;
        let width = i32::try_from(surface_size.0).map_err(|_| PlanError::NativeFailure)?;
        let height = i32::try_from(surface_size.1).map_err(|_| PlanError::NativeFailure)?;
        if width == 0 || height == 0 {
            return Err(PlanError::NativeFailure);
        }
        // SAFETY: every GL handle belongs to this current context; viewport and scissor values
        // have been clamped to the selected mode's bounded pixel dimensions.
        unsafe {
            self.gl.viewport(0, 0, width, height);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.disable(glow::BLEND);
            self.gl.bind_texture(glow::TEXTURE_2D, None);
            self.gl.use_program(None);
            self.gl.clear_color(
                plan.background_rgb[0],
                plan.background_rgb[1],
                plan.background_rgb[2],
                1.0,
            );
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl.enable(glow::SCISSOR_TEST);
            for command in plan.commands {
                match command {
                    RenderCommand::Fill(command) => {
                        let clip_bottom = command.clip.y.saturating_add(command.clip.height);
                        self.gl.scissor(
                            i32::try_from(command.clip.x).map_err(|_| PlanError::NativeFailure)?,
                            height
                                - i32::try_from(clip_bottom)
                                    .map_err(|_| PlanError::NativeFailure)?,
                            i32::try_from(command.clip.width)
                                .map_err(|_| PlanError::NativeFailure)?,
                            i32::try_from(command.clip.height)
                                .map_err(|_| PlanError::NativeFailure)?,
                        );
                        self.gl.use_program(Some(self.program));
                        self.gl.uniform_2_f32(
                            self.viewport_uniform.as_ref(),
                            width as f32,
                            height as f32,
                        );
                        self.gl.uniform_4_f32(
                            self.color_uniform.as_ref(),
                            command.rgba[0],
                            command.rgba[1],
                            command.rgba[2],
                            command.rgba[3],
                        );
                        self.gl.enable_vertex_attrib_array(0);
                        self.gl
                            .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
                        let left = command.bounds.x as f32;
                        let top = command.bounds.y as f32;
                        let right = command.bounds.x.saturating_add(command.bounds.width) as f32;
                        let bottom = command.bounds.y.saturating_add(command.bounds.height) as f32;
                        let vertices = [
                            left, top, right, top, right, bottom, left, top, right, bottom, left,
                            bottom,
                        ];
                        let mut bytes = Vec::with_capacity(vertices.len() * 4);
                        for value in vertices {
                            bytes.extend_from_slice(&value.to_ne_bytes());
                        }
                        self.gl
                            .buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STREAM_DRAW);
                        self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
                    }
                    RenderCommand::GlyphRun(command) => {
                        self.draw_glyph_run(&command, scene.device_scale(), width, height)?;
                    }
                }
            }
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }
        self.check_gl_error()
    }

    fn draw_glyph_run(
        &mut self,
        command: &GlyphRunCommand,
        scale: DeviceScale,
        width: i32,
        height: i32,
    ) -> Result<(), PlanError> {
        if command.run.font_size() != 14.0 {
            return Err(PlanError::UnsupportedOperation);
        }
        let clip_bottom = command.clip.y.saturating_add(command.clip.height);
        // SAFETY: renderer handles belong to the current context; command clip is surface-clamped.
        unsafe {
            self.gl.scissor(
                i32::try_from(command.clip.x).map_err(|_| PlanError::NativeFailure)?,
                height - i32::try_from(clip_bottom).map_err(|_| PlanError::NativeFailure)?,
                i32::try_from(command.clip.width).map_err(|_| PlanError::NativeFailure)?,
                i32::try_from(command.clip.height).map_err(|_| PlanError::NativeFailure)?,
            );
            self.gl.enable(glow::BLEND);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.use_program(Some(self.glyph_program));
            self.gl.uniform_2_f32(
                self.glyph_viewport_uniform.as_ref(),
                width as f32,
                height as f32,
            );
            self.gl.uniform_4_f32(
                self.glyph_color_uniform.as_ref(),
                command.rgba[0],
                command.rgba[1],
                command.rgba[2],
                command.rgba[3],
            );
            self.gl.uniform_1_i32(self.glyph_atlas_uniform.as_ref(), 0);
            self.gl.active_texture(glow::TEXTURE0);
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.atlas_texture));
            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);
            self.gl
                .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            self.gl
                .vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
        }
        for glyph in command.run.glyphs() {
            let key = GlyphAtlasKey {
                digest: *command.run.font_digest(),
                font_size_64: (command.run.font_size() * 64.0).round() as u32,
                scale_x: scale.x().to_bits(),
                scale_y: scale.y().to_bits(),
                glyph_id: glyph.glyph_id(),
            };
            let mask = self
                .text_service
                .glyph_mask(*command.run.font_digest(), 14, glyph.glyph_id())
                .map_err(|_| PlanError::NativeFailure)?;
            let entry = if let Some(entry) = self.atlas_entries.get(&key) {
                *entry
            } else {
                if mask.width == 0 || mask.rows == 0 {
                    continue;
                }
                let size = usize::try_from(mask.width)
                    .ok()
                    .and_then(|width| {
                        usize::try_from(mask.rows)
                            .ok()
                            .and_then(|rows| width.checked_mul(rows))
                    })
                    .ok_or(PlanError::NativeFailure)?;
                if mask.pixels.len() != size {
                    return Err(PlanError::NativeFailure);
                }
                let entry = self.atlas_packer.allocate(mask.width, mask.rows)?;
                // SAFETY: texture belongs to the current context and mask is tightly packed R8 data.
                unsafe {
                    self.gl
                        .bind_texture(glow::TEXTURE_2D, Some(self.atlas_texture));
                    self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
                    self.gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        entry.x as i32,
                        entry.y as i32,
                        mask.width as i32,
                        mask.rows as i32,
                        glow::RED,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(Some(&mask.pixels)),
                    );
                    self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
                }
                self.check_gl_error()?;
                self.atlas_entries.insert(key, entry);
                entry
            };
            self.draw_glyph_quad(glyph, &mask, entry, scale)?;
        }
        // SAFETY: restore shared state for later draw commands.
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, None);
            self.gl.disable(glow::BLEND);
        }
        self.check_gl_error()
    }

    fn draw_glyph_quad(
        &self,
        glyph: &GlyphPlacement,
        mask: &GlyphMask,
        entry: AtlasEntry,
        scale: DeviceScale,
    ) -> Result<(), PlanError> {
        let (glyph_x, glyph_y) = glyph.position();
        let x = ((glyph_x + f64::from(mask.left)) * scale.x()) as f32;
        let y = ((glyph_y + mask.ascent_dip - f64::from(mask.top)) * scale.y()) as f32;
        let right = x + mask.width as f32 * scale.x() as f32;
        let bottom = y + mask.rows as f32 * scale.y() as f32;
        let atlas_size = GLYPH_ATLAS_SIZE as f32;
        let u0 = entry.x as f32 / atlas_size;
        // OpenGL's first uploaded texel row is at the low-V edge of the texture.
        let v0 = entry.y as f32 / atlas_size;
        let u1 = (entry.x + entry.width) as f32 / atlas_size;
        let v1 = (entry.y + entry.height) as f32 / atlas_size;
        let vertices = [
            x, y, u0, v0, right, y, u1, v0, right, bottom, u1, v1, x, y, u0, v0, right, bottom, u1,
            v1, x, bottom, u0, v1,
        ];
        let mut bytes = Vec::with_capacity(vertices.len() * 4);
        for value in vertices {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        // SAFETY: current context owns the bound renderer vertex buffer/VAO.
        unsafe {
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STREAM_DRAW);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
        self.check_gl_error()
    }

    fn check_gl_error(&self) -> Result<(), PlanError> {
        // SAFETY: renderer is only submitted while its owning EGL context is current.
        let mut error = unsafe { self.gl.get_error() };
        if error == glow::NO_ERROR {
            return Ok(());
        }
        while error != glow::NO_ERROR {
            // SAFETY: drain all queued errors so the retry path begins from a clean context.
            error = unsafe { self.gl.get_error() };
        }
        Err(PlanError::NativeFailure)
    }
}

#[cfg(target_os = "linux")]
impl Drop for GlesRenderer {
    fn drop(&mut self) {
        // SAFETY: NativeSurface drops the renderer before unbinding or destroying its EGL context.
        unsafe {
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_texture(self.atlas_texture);
            self.gl.delete_program(self.program);
            self.gl.delete_program(self.glyph_program);
        }
    }
}

#[cfg(target_os = "linux")]
fn compile_shader(gl: &glow::Context, kind: u32, source: &str) -> Result<glow::Shader, String> {
    // SAFETY: the caller has a current GLES context and supplies a valid shader type/source.
    unsafe {
        let shader = gl.create_shader(kind)?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if gl.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            let error = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            Err(error)
        }
    }
}

#[cfg(target_os = "linux")]
fn link_program(
    gl: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> Result<glow::Program, String> {
    let vertex = compile_shader(gl, glow::VERTEX_SHADER, vertex_source)?;
    let fragment = match compile_shader(gl, glow::FRAGMENT_SHADER, fragment_source) {
        Ok(fragment) => fragment,
        Err(error) => {
            // SAFETY: the shader was created by the current context.
            unsafe { gl.delete_shader(vertex) };
            return Err(error);
        }
    };
    // SAFETY: the current context is valid for program creation.
    let program = match unsafe { gl.create_program() } {
        Ok(program) => program,
        Err(error) => {
            // SAFETY: both shader handles belong to the current context.
            unsafe {
                gl.delete_shader(vertex);
                gl.delete_shader(fragment);
            }
            return Err(error);
        }
    };
    // SAFETY: the program and shaders were created by the current context.
    unsafe {
        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.link_program(program);
    }
    // SAFETY: this program belongs to the current context.
    let linked = unsafe { gl.get_program_link_status(program) };
    // SAFETY: this program belongs to the current context.
    let log = (!linked).then(|| unsafe { gl.get_program_info_log(program) });
    // SAFETY: shaders have been linked into the program and are no longer needed.
    unsafe {
        gl.delete_shader(vertex);
        gl.delete_shader(fragment);
    }
    if let Some(error) = log {
        // SAFETY: this program belongs to the current context.
        unsafe { gl.delete_program(program) };
        return Err(error);
    }
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemvvm_core::abi_support::{Damage, GlyphPlacement, GlyphRunSnapshot, SceneError};
    #[cfg(target_os = "linux")]
    use freemvvm_core::abi_support::{TextRequest, TextService};
    use std::sync::Arc;

    fn scale(x: f64, y: f64) -> DeviceScale {
        DeviceScale::new(x, y).expect("valid test scale")
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect::new(x, y, width, height).expect("valid test rectangle")
    }

    fn scene(scale: DeviceScale, operations: Vec<DrawOp>) -> SceneSnapshot {
        SceneSnapshot::new(
            1,
            scale,
            [0.1, 0.2, 0.3],
            Arc::from(operations),
            Damage::Full,
        )
        .expect("valid test scene")
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn glyph_atlas_packer_rejects_capacity_exhaustion_without_wrapping() {
        let mut packer = AtlasPacker::default();
        let entry = packer
            .allocate(GLYPH_ATLAS_SIZE - 2, GLYPH_ATLAS_SIZE - 2)
            .expect("one full-size glyph mask fits with padding");
        assert_eq!(entry.x, 1);
        assert_eq!(entry.y, 1);
        assert_eq!(entry.width, GLYPH_ATLAS_SIZE - 2);
        assert_eq!(entry.height, GLYPH_ATLAS_SIZE - 2);
        assert_eq!(packer.allocate(1, 1), Err(PlanError::NativeFailure));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires headless EGL/GLES software rendering"]
    fn headless_gles_renders_core_scale_background_clip_and_fill_pixels() {
        use freemvvm_core::abi_support::{Damage, DeviceScale, DrawOp, Rect, SceneSnapshot};

        // SAFETY: the dynamic EGL loader resolves the process EGL implementation.
        let egl = unsafe { khronos_egl::DynamicInstance::<khronos_egl::EGL1_5>::load_required() }
            .expect("load EGL");
        // SAFETY: a null native display requests the platform's default EGL display; the
        // Docker test sets EGL_PLATFORM=surfaceless before the process starts.
        let display = unsafe { egl.get_display(std::ptr::null_mut()) }.expect("EGL display");
        egl.initialize(display).expect("initialize EGL");
        egl.bind_api(khronos_egl::OPENGL_ES_API)
            .expect("bind GLES API");
        let config = egl
            .choose_first_config(
                display,
                &[
                    khronos_egl::SURFACE_TYPE,
                    khronos_egl::PBUFFER_BIT,
                    khronos_egl::RENDERABLE_TYPE,
                    khronos_egl::OPENGL_ES3_BIT,
                    khronos_egl::RED_SIZE,
                    8,
                    khronos_egl::GREEN_SIZE,
                    8,
                    khronos_egl::BLUE_SIZE,
                    8,
                    khronos_egl::NONE,
                ],
            )
            .expect("choose pbuffer config")
            .expect("available pbuffer config");
        let surface = egl
            .create_pbuffer_surface(
                display,
                config,
                &[
                    khronos_egl::WIDTH,
                    8,
                    khronos_egl::HEIGHT,
                    8,
                    khronos_egl::NONE,
                ],
            )
            .expect("create pbuffer");
        let context = egl
            .create_context(
                display,
                config,
                None,
                &[khronos_egl::CONTEXT_CLIENT_VERSION, 3, khronos_egl::NONE],
            )
            .expect("create GLES 3 context");
        egl.make_current(display, Some(surface), Some(surface), Some(context))
            .expect("make GLES context current");
        let text_service = Arc::new(LinuxTextService::new());
        // SAFETY: this EGL context is current and provides the GLES entrypoints.
        let mut renderer = unsafe {
            GlesRenderer::new(Arc::clone(&text_service), |name| {
                egl.get_proc_address(name)
                    .map_or(std::ptr::null(), |function| {
                        function as *const () as *const std::ffi::c_void
                    })
            })
        }
        .expect("create renderer");
        // SAFETY: the current context is valid and the renderer string is a GLES parameter.
        let renderer_name = unsafe { renderer.gl.get_parameter_string(glow::RENDERER) };
        println!("headless GLES renderer: {renderer_name}");
        let renderer_name_lower = renderer_name.to_ascii_lowercase();
        assert!(
            ["llvmpipe", "softpipe", "swrast"]
                .iter()
                .any(|name| renderer_name_lower.contains(name)),
            "expected a Mesa software renderer, got {renderer_name}"
        );
        let scene = SceneSnapshot::new(
            1,
            DeviceScale::new(2.0, 2.0).expect("scale"),
            [0.0, 0.0, 1.0],
            Arc::from([
                DrawOp::Clip(Rect::new(1.0, 1.0, 2.0, 2.0).expect("clip")),
                DrawOp::FillRect {
                    bounds: Rect::new(0.0, 0.0, 4.0, 4.0).expect("fill bounds"),
                    rgba: [1.0, 0.0, 0.0, 1.0],
                },
            ]),
            Damage::Full,
        )
        .expect("scene");
        renderer.render(&scene, (8, 8)).expect("render scene");

        let mut center = [0_u8; 4];
        let mut outside_clip = [0_u8; 4];
        // SAFETY: the EGL pbuffer is current and each destination is a writable RGBA pixel.
        unsafe {
            renderer.gl.read_pixels(
                3,
                3,
                1,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut center)),
            );
            renderer.gl.read_pixels(
                1,
                1,
                1,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut outside_clip)),
            );
        }
        assert_eq!(center, [255, 0, 0, 255]);
        assert_eq!(outside_clip, [0, 0, 255, 255]);

        drop(renderer);
        egl.make_current(display, None, None, None)
            .expect("unbind GLES context");
        egl.destroy_context(display, context)
            .expect("destroy context");
        egl.destroy_surface(display, surface)
            .expect("destroy pbuffer");
        egl.terminate(display).expect("terminate EGL");
    }

    #[test]
    fn platform_present_scales_logical_edges_and_clamps_to_surface() {
        let snapshot = scene(
            scale(1.5, 2.0),
            vec![DrawOp::FillRect {
                bounds: rect(-2.0, 3.0, 12.0, 8.0),
                rgba: [0.2, 0.3, 0.4, 1.0],
            }],
        );
        assert_eq!(
            plan(&snapshot, (10, 12)).expect("render plan").commands,
            [RenderCommand::Fill(FillCommand {
                bounds: PixelRect {
                    x: 0,
                    y: 6,
                    width: 10,
                    height: 6,
                },
                clip: PixelRect {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 12,
                },
                rgba: [0.2, 0.3, 0.4, 1.0],
            })]
        );
    }

    #[test]
    fn glyph_plan_preserves_core_color_clip_and_device_scale() {
        let glyph = GlyphPlacement::new(7, 2.0, 3.0, 8.0, 0.0).expect("glyph placement");
        let run = GlyphRunSnapshot::new(
            [0x2a; 32],
            14.0,
            rect(2.0, 0.0, 8.0, 20.0),
            Arc::<[GlyphPlacement]>::from([glyph]),
        )
        .expect("glyph run");
        let snapshot = scene(
            scale(2.0, 1.5),
            vec![
                DrawOp::Clip(rect(1.0, 2.0, 9.0, 11.0)),
                DrawOp::GlyphRun {
                    run,
                    rgba: [0.0, 0.0, 0.0, 1.0],
                },
            ],
        );
        assert_eq!(
            plan(&snapshot, (40, 30))
                .expect("glyph render plan")
                .commands,
            [RenderCommand::GlyphRun(GlyphRunCommand {
                clip: PixelRect {
                    x: 2,
                    y: 3,
                    width: 18,
                    height: 17
                },
                rgba: [0.0, 0.0, 0.0, 1.0],
                run: match &snapshot.operations()[1] {
                    DrawOp::GlyphRun { run, .. } => run.clone(),
                    _ => unreachable!(),
                },
            })]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires headless EGL/GLES software rendering"]
    fn headless_gles_renders_glyph_runs_with_device_scale_and_clip() {
        use freemvvm_core::abi_support::{DeviceScale, DrawOp, Rect, SceneSnapshot};

        // SAFETY: the dynamic EGL loader resolves the process EGL implementation.
        let egl = unsafe { khronos_egl::DynamicInstance::<khronos_egl::EGL1_5>::load_required() }
            .expect("load EGL");
        // SAFETY: a null native display requests the platform's default EGL display.
        let display = unsafe { egl.get_display(std::ptr::null_mut()) }.expect("EGL display");
        egl.initialize(display).expect("initialize EGL");
        egl.bind_api(khronos_egl::OPENGL_ES_API)
            .expect("bind GLES API");
        let config = egl
            .choose_first_config(
                display,
                &[
                    khronos_egl::SURFACE_TYPE,
                    khronos_egl::PBUFFER_BIT,
                    khronos_egl::RENDERABLE_TYPE,
                    khronos_egl::OPENGL_ES3_BIT,
                    khronos_egl::RED_SIZE,
                    8,
                    khronos_egl::GREEN_SIZE,
                    8,
                    khronos_egl::BLUE_SIZE,
                    8,
                    khronos_egl::NONE,
                ],
            )
            .expect("choose pbuffer config")
            .expect("available pbuffer config");
        let surface = egl
            .create_pbuffer_surface(
                display,
                config,
                &[
                    khronos_egl::WIDTH,
                    128,
                    khronos_egl::HEIGHT,
                    48,
                    khronos_egl::NONE,
                ],
            )
            .expect("create pbuffer");
        let context = egl
            .create_context(
                display,
                config,
                None,
                &[khronos_egl::CONTEXT_CLIENT_VERSION, 3, khronos_egl::NONE],
            )
            .expect("create GLES 3 context");
        egl.make_current(display, Some(surface), Some(surface), Some(context))
            .expect("make GLES context current");

        let text_service = Arc::new(LinuxTextService::new());
        let shaped = text_service
            .resolve_and_shape(&TextRequest::platform_default(1, Arc::<str>::from("A")))
            .expect("resolve glyph mask through the Platform TextService");
        let run = shaped.glyph_runs[0].glyphs.clone();
        let scene = SceneSnapshot::new(
            1,
            DeviceScale::new(2.0, 1.5).expect("anisotropic scale"),
            [0.0, 0.0, 1.0],
            Arc::from([
                DrawOp::Clip(Rect::new(0.0, 0.0, 28.0, 24.0).expect("client clip")),
                DrawOp::GlyphRun {
                    run: run.clone(),
                    rgba: [0.0, 0.0, 0.0, 1.0],
                },
            ]),
            Damage::Full,
        )
        .expect("valid glyph scene");
        // SAFETY: EGL has made the ES3 context current; entrypoints are used only on this thread.
        let mut renderer = unsafe {
            GlesRenderer::new(Arc::clone(&text_service), |name| {
                egl.get_proc_address(name)
                    .map_or(std::ptr::null(), |address| {
                        address as *const () as *const std::ffi::c_void
                    })
            })
        }
        .expect("create atlas renderer");
        renderer
            .render(&scene, (128, 48))
            .expect("render resolved glyphs");

        let mut pixels = vec![0_u8; 128 * 48 * 4];
        // SAFETY: the pbuffer is current and the destination covers its complete RGBA surface.
        unsafe {
            renderer.gl.read_pixels(
                0,
                0,
                128,
                48,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );
        }
        let pixel = |x: usize, y: usize| &pixels[(y * 128 + x) * 4..(y * 128 + x + 1) * 4];
        assert!((0..128).any(|x| {
            (0..48).any(|y| x < 56 && (12..48).contains(&y) && pixel(x, y) != [0, 0, 255, 255])
        }));
        assert!((0..48).all(|y| (56..128).all(|x| pixel(x, y) == [0, 0, 255, 255])));

        // A queued GLES error during atlas-backed rendering must fail the frame.
        // SAFETY: the renderer's EGL GLES context remains current on this test thread.
        unsafe { renderer.gl.enable(0xffff) };
        assert_eq!(
            renderer.render(&scene, (128, 48)),
            Err(PlanError::NativeFailure)
        );

        let missing = GlyphRunSnapshot::new(
            [0xee; 32],
            14.0,
            run.bounds(),
            Arc::<[GlyphPlacement]>::from(run.glyphs()),
        )
        .expect("valid but unresolved asset identity");
        let missing_scene = SceneSnapshot::new(
            2,
            scene.device_scale(),
            scene.background_rgb(),
            Arc::from([
                DrawOp::Clip(Rect::new(0.0, 0.0, 28.0, 24.0).expect("client clip")),
                DrawOp::GlyphRun {
                    run: missing,
                    rgba: [0.0, 0.0, 0.0, 1.0],
                },
            ]),
            Damage::Full,
        )
        .expect("missing asset scene");
        assert_eq!(
            renderer.render(&missing_scene, (128, 48)),
            Err(PlanError::NativeFailure)
        );

        drop(renderer);
        egl.make_current(display, None, None, None)
            .expect("unbind GLES context");
        egl.destroy_context(display, context)
            .expect("destroy context");
        egl.destroy_surface(display, surface)
            .expect("destroy surface");
        egl.terminate(display).expect("terminate EGL");
    }

    #[test]
    fn platform_present_clip_replaces_effective_clip_without_resizing_fill() {
        let snapshot = scene(
            scale(1.0, 1.0),
            vec![
                DrawOp::Clip(rect(10.0, 5.0, 40.0, 30.0)),
                DrawOp::FillRect {
                    bounds: rect(0.0, 0.0, 80.0, 60.0),
                    rgba: [1.0, 0.0, 0.0, 1.0],
                },
                DrawOp::Clip(rect(60.0, 0.0, 20.0, 20.0)),
                DrawOp::FillRect {
                    bounds: rect(0.0, 0.0, 80.0, 60.0),
                    rgba: [0.0, 1.0, 0.0, 1.0],
                },
            ],
        );
        let commands = plan(&snapshot, (80, 60)).expect("render plan").commands;
        assert_eq!(commands.len(), 2);
        let RenderCommand::Fill(first) = &commands[0] else {
            panic!("first draw is a fill");
        };
        let RenderCommand::Fill(second) = &commands[1] else {
            panic!("second draw is a fill");
        };
        assert_eq!(
            first.bounds,
            PixelRect {
                x: 0,
                y: 0,
                width: 80,
                height: 60
            }
        );
        assert_eq!(
            first.clip,
            PixelRect {
                x: 10,
                y: 5,
                width: 40,
                height: 30
            }
        );
        assert_eq!(second.bounds, first.bounds);
        assert_eq!(
            second.clip,
            PixelRect {
                x: 60,
                y: 0,
                width: 20,
                height: 20
            }
        );
    }

    #[test]
    fn platform_present_rejects_glyphs_instead_of_silently_presenting_them() {
        let glyph = GlyphRunSnapshot::new([0; 32], 14.0, rect(0.0, 0.0, 10.0, 10.0), Arc::from([]))
            .expect("valid glyph run");
        let snapshot = scene(
            scale(1.0, 1.0),
            vec![DrawOp::GlyphRun {
                run: glyph,
                rgba: [0.0, 0.0, 0.0, 1.0],
            }],
        );
        assert_eq!(
            plan(&snapshot, (100, 100)),
            Err(PlanError::UnsupportedOperation)
        );
    }

    #[test]
    fn platform_present_plan_uses_core_background_color_for_surface_clear() {
        let snapshot = scene(scale(1.0, 1.0), Vec::new());
        let render_plan = plan(&snapshot, (20, 10)).expect("render plan");
        assert_eq!(render_plan.background_rgb, [0.1, 0.2, 0.3]);
        assert!(render_plan.commands.is_empty());
    }

    #[test]
    fn platform_present_scale_validation_remains_a_core_scene_contract() {
        assert_eq!(DeviceScale::new(0.0, 1.0), Err(SceneError::InvalidScale));
    }
}
