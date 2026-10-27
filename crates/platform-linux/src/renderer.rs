use freemvvm_core::abi_support::{DeviceScale, DrawOp, Rect, SceneSnapshot};
#[cfg(target_os = "linux")]
use glow::HasContext;

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
pub(super) struct RenderPlan {
    pub(super) background_rgb: [f32; 3],
    pub(super) commands: Vec<FillCommand>,
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
                    commands.push(FillCommand {
                        bounds: pixel_bounds,
                        clip: active_clip,
                        rgba: *rgba,
                    });
                }
            }
            DrawOp::GlyphRun(_) => return Err(PlanError::UnsupportedOperation),
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
pub(super) struct GlesRenderer {
    gl: glow::Context,
    program: glow::Program,
    vertex_array: glow::VertexArray,
    vertex_buffer: glow::Buffer,
    viewport_uniform: Option<glow::UniformLocation>,
    color_uniform: Option<glow::UniformLocation>,
}

#[cfg(target_os = "linux")]
impl GlesRenderer {
    pub(super) unsafe fn new(
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
        Ok(Self {
            gl,
            program,
            vertex_array,
            vertex_buffer,
            viewport_uniform,
            color_uniform,
        })
    }

    pub(super) fn render(
        &self,
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
            self.gl.clear_color(
                plan.background_rgb[0],
                plan.background_rgb[1],
                plan.background_rgb[2],
                1.0,
            );
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.use_program(Some(self.program));
            self.gl
                .uniform_2_f32(self.viewport_uniform.as_ref(), width as f32, height as f32);
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl.enable_vertex_attrib_array(0);
            self.gl
                .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            self.gl.enable(glow::SCISSOR_TEST);
            for command in plan.commands {
                let clip_bottom = command.clip.y.saturating_add(command.clip.height);
                self.gl.scissor(
                    i32::try_from(command.clip.x).map_err(|_| PlanError::NativeFailure)?,
                    height - i32::try_from(clip_bottom).map_err(|_| PlanError::NativeFailure)?,
                    i32::try_from(command.clip.width).map_err(|_| PlanError::NativeFailure)?,
                    i32::try_from(command.clip.height).map_err(|_| PlanError::NativeFailure)?,
                );
                self.gl.uniform_4_f32(
                    self.color_uniform.as_ref(),
                    command.rgba[0],
                    command.rgba[1],
                    command.rgba[2],
                    command.rgba[3],
                );
                let left = command.bounds.x as f32;
                let top = command.bounds.y as f32;
                let right = command.bounds.x.saturating_add(command.bounds.width) as f32;
                let bottom = command.bounds.y.saturating_add(command.bounds.height) as f32;
                let vertices = [
                    left, top, right, top, right, bottom, left, top, right, bottom, left, bottom,
                ];
                let mut bytes = Vec::with_capacity(vertices.len() * 4);
                for value in vertices {
                    bytes.extend_from_slice(&value.to_ne_bytes());
                }
                self.gl
                    .buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STREAM_DRAW);
                self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            }
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for GlesRenderer {
    fn drop(&mut self) {
        // SAFETY: NativeSurface drops the renderer before unbinding or destroying its EGL context.
        unsafe {
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_program(self.program);
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

#[cfg(test)]
mod tests {
    use super::*;
    use freemvvm_core::abi_support::{Damage, GlyphRunSnapshot, SceneError};
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
            [FillCommand {
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
            }]
        );
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
        assert_eq!(
            commands[0].bounds,
            PixelRect {
                x: 0,
                y: 0,
                width: 80,
                height: 60
            }
        );
        assert_eq!(
            commands[0].clip,
            PixelRect {
                x: 10,
                y: 5,
                width: 40,
                height: 30
            }
        );
        assert_eq!(commands[1].bounds, commands[0].bounds);
        assert_eq!(
            commands[1].clip,
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
        let glyph = GlyphRunSnapshot::new([0; 32], 12.0, rect(0.0, 0.0, 10.0, 10.0), Arc::from([]))
            .expect("valid glyph run");
        let snapshot = scene(scale(1.0, 1.0), vec![DrawOp::GlyphRun(glyph)]);
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
