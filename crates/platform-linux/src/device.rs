use crate::renderer::{GlesRenderer, PlanError};
use drm::Device as DrmDevice;
use drm::control::{
    Device as ControlDevice, Event, ModeTypeFlags, PageFlipFlags, connector, crtc, framebuffer,
};
use freemvvm_core::abi_support::{DeviceScale, FrameTicket, SceneSnapshot, WindowClientArea};
use gbm::{AsRaw, BufferObject, BufferObjectFlags, Device as GbmDevice, Format};
use khronos_egl as egl;
use std::ffi::c_void;
use std::fs::{self, File, OpenOptions};
use std::os::fd::AsFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const EGL_PLATFORM_GBM_KHR: egl::Enum = 0x31D7;
const EGL_WINDOW_BIT: egl::Int = 0x0004;
const EGL_OPENGL_ES3_BIT_KHR: egl::Int = 0x0040;
const EGL_CONTEXT_CLIENT_VERSION: egl::Int = egl::CONTEXT_CLIENT_VERSION;
const EGL_OPENGL_ES_API: egl::Enum = egl::OPENGL_ES_API;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OutputMode {
    connector_id: u32,
    mode_index: usize,
    width: u16,
    height: u16,
    preferred: bool,
}

fn choose_output(modes: impl IntoIterator<Item = OutputMode>) -> Option<OutputMode> {
    let modes = modes.into_iter().collect::<Vec<_>>();
    modes
        .iter()
        .copied()
        .filter(|mode| mode.preferred)
        .min_by_key(|mode| mode.connector_id)
        .or_else(|| {
            modes.into_iter().max_by(|left, right| {
                u32::from(left.width)
                    .saturating_mul(u32::from(left.height))
                    .cmp(&u32::from(right.width).saturating_mul(u32::from(right.height)))
                    .then_with(|| right.connector_id.cmp(&left.connector_id))
            })
        })
}

fn card_nodes(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut nodes = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let suffix = name.strip_prefix("card")?.parse::<u32>().ok()?;
            Some((suffix, entry.path()))
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|(suffix, _)| *suffix);
    Ok(nodes.into_iter().map(|(_, path)| path).collect())
}

fn first_usable_output<N>(
    nodes: impl IntoIterator<Item = N>,
    mut inspect: impl FnMut(&N) -> Option<OutputMode>,
) -> Option<(N, OutputMode)> {
    nodes
        .into_iter()
        .find_map(|node| inspect(&node).map(|mode| (node, mode)))
}

struct Card(File);

impl AsFd for Card {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl drm::Device for Card {}
impl ControlDevice for Card {}

struct SelectedDevice {
    path: PathBuf,
    mode: OutputMode,
}

fn open_selected_device() -> std::io::Result<SelectedDevice> {
    let nodes = card_nodes(Path::new("/dev/dri"))?;
    first_usable_output(nodes, |path| {
        let card = Card(OpenOptions::new().read(true).write(true).open(path).ok()?);
        let resources = card.resource_handles().ok()?;
        let modes = resources
            .connectors()
            .iter()
            .filter_map(|handle| {
                let info = card.get_connector(*handle, false).ok()?;
                if info.state() != connector::State::Connected {
                    return None;
                }
                Some(
                    info.modes()
                        .iter()
                        .enumerate()
                        .map(|(mode_index, mode)| OutputMode {
                            connector_id: u32::from(*handle),
                            mode_index,
                            width: mode.size().0,
                            height: mode.size().1,
                            preferred: mode.mode_type().contains(ModeTypeFlags::PREFERRED),
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten();
        choose_output(modes)
    })
    .map(|(path, mode)| SelectedDevice { path, mode })
    .ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no DRM card has a connected connector with a usable mode",
        )
    })
}

pub(super) struct NativeSurface {
    egl: egl::DynamicInstance<egl::EGL1_5>,
    display: egl::Display,
    egl_surface: egl::Surface,
    context: egl::Context,
    renderer: Option<GlesRenderer>,
    size: (u32, u32),
    gbm_surface: gbm::Surface<()>,
    gbm_device: GbmDevice<Card>,
    connector: connector::Handle,
    crtc: crtc::Handle,
    mode: drm::control::Mode,
    previous_crtc: crtc::Info,
    previous_connectors: Vec<connector::Handle>,
    front_buffer: Option<ScanoutBuffer>,
    pending_buffer: Option<(FrameTicket, ScanoutBuffer)>,
    retired_buffers: Vec<ScanoutBuffer>,
    master_acquired: bool,
}

struct ScanoutBuffer {
    _buffer: BufferObject<()>,
    framebuffer: framebuffer::Handle,
}

pub(super) enum Submission {
    Complete,
    Pending,
}

impl Drop for NativeSurface {
    fn drop(&mut self) {
        let _ = self.close_surface();
        self.renderer.take();
        let _ = self.egl.make_current(self.display, None, None, None);
        let _ = self.egl.destroy_context(self.display, self.context);
        let _ = self.egl.destroy_surface(self.display, self.egl_surface);
        let _ = self.egl.terminate(self.display);
    }
}

impl NativeSurface {
    pub(super) fn client_area(&self) -> WindowClientArea {
        WindowClientArea {
            width_dip: f64::from(self.size.0),
            height_dip: f64::from(self.size.1),
        }
    }

    /// The direct-KMS profile maps the selected mode's numeric pixel extent to the same DIP extent.
    pub(super) fn device_scale(&self) -> DeviceScale {
        DeviceScale::new(1.0, 1.0).expect("valid Linux KMS profile scale")
    }

    pub(super) fn close_surface(&mut self) -> Result<(), ()> {
        if self.master_acquired && (self.front_buffer.is_some() || self.pending_buffer.is_some()) {
            self.gbm_device
                .set_crtc(
                    self.crtc,
                    self.previous_crtc.framebuffer(),
                    self.previous_crtc.position(),
                    &self.previous_connectors,
                    self.previous_crtc.mode(),
                )
                .map_err(|_| ())?;
        }
        if let Some((_, pending)) = self.pending_buffer.as_ref() {
            self.gbm_device
                .destroy_framebuffer(pending.framebuffer)
                .map_err(|_| ())?;
            self.pending_buffer.take();
        }
        if let Some(front) = self.front_buffer.as_ref() {
            self.gbm_device
                .destroy_framebuffer(front.framebuffer)
                .map_err(|_| ())?;
            self.front_buffer.take();
        }
        while let Some(retired) = self.retired_buffers.last() {
            self.gbm_device
                .destroy_framebuffer(retired.framebuffer)
                .map_err(|_| ())?;
            self.retired_buffers.pop();
        }
        if self.master_acquired {
            self.gbm_device.release_master_lock().map_err(|_| ())?;
            self.master_acquired = false;
        }
        Ok(())
    }

    pub(super) fn render(
        &mut self,
        ticket: FrameTicket,
        scene: &SceneSnapshot,
    ) -> Result<Submission, PlanError> {
        self.egl
            .make_current(
                self.display,
                Some(self.egl_surface),
                Some(self.egl_surface),
                Some(self.context),
            )
            .map_err(|_| PlanError::NativeFailure)?;
        self.renderer
            .as_mut()
            .ok_or(PlanError::NativeFailure)?
            .render(scene, self.size)?;
        self.egl
            .swap_buffers(self.display, self.egl_surface)
            .map_err(|_| PlanError::NativeFailure)?;
        // SAFETY: exactly one front-buffer lock follows the successful eglSwapBuffers above.
        let buffer = unsafe {
            self.gbm_surface
                .lock_front_buffer()
                .map_err(|_| PlanError::NativeFailure)?
        };
        let framebuffer = self
            .gbm_device
            .add_framebuffer(&buffer, 24, 32)
            .map_err(|_| PlanError::NativeFailure)?;
        let scanout = ScanoutBuffer {
            _buffer: buffer,
            framebuffer,
        };
        if self.front_buffer.is_none() {
            if self.gbm_device.acquire_master_lock().is_err() {
                let _ = self.gbm_device.destroy_framebuffer(framebuffer);
                drop(scanout);
                return Err(PlanError::NativeFailure);
            }
            self.master_acquired = true;
            if self
                .gbm_device
                .set_crtc(
                    self.crtc,
                    Some(framebuffer),
                    (0, 0),
                    &[self.connector],
                    Some(self.mode),
                )
                .is_err()
            {
                let _ = self.gbm_device.destroy_framebuffer(framebuffer);
                drop(scanout);
                let _ = self.gbm_device.release_master_lock();
                self.master_acquired = false;
                return Err(PlanError::NativeFailure);
            }
            self.front_buffer = Some(scanout);
            return Ok(Submission::Complete);
        }
        if self.pending_buffer.is_some() {
            let _ = self.gbm_device.destroy_framebuffer(framebuffer);
            drop(scanout);
            return Err(PlanError::NativeFailure);
        }
        if self
            .gbm_device
            .page_flip(self.crtc, framebuffer, PageFlipFlags::EVENT, None)
            .is_err()
        {
            let _ = self.gbm_device.destroy_framebuffer(framebuffer);
            drop(scanout);
            return Err(PlanError::NativeFailure);
        }
        self.pending_buffer = Some((ticket, scanout));
        Ok(Submission::Pending)
    }

    pub(super) fn wait_present(&mut self) -> Result<Option<FrameTicket>, FrameTicket> {
        let Some((ticket, _)) = self.pending_buffer.as_ref() else {
            return Ok(None);
        };
        let ticket = *ticket;
        let events = match self.gbm_device.receive_events() {
            Ok(events) => events,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(_) => return Err(ticket),
        };
        for event in events {
            if let Event::PageFlip(event) = event {
                if event.crtc != self.crtc {
                    continue;
                }
                let Some((completed_ticket, next_front)) = self.pending_buffer.take() else {
                    return Err(ticket);
                };
                let previous = self.front_buffer.replace(next_front);
                if let Some(previous) = previous {
                    if self
                        .gbm_device
                        .destroy_framebuffer(previous.framebuffer)
                        .is_err()
                    {
                        self.retired_buffers.push(previous);
                        return Ok(Some(completed_ticket));
                    }
                    drop(previous);
                }
                return Ok(Some(completed_ticket));
            }
        }
        Ok(None)
    }
}

pub(super) fn open_surface(
    text_service: Arc<super::text_service::LinuxTextService>,
) -> std::io::Result<NativeSurface> {
    let selected = open_selected_device()?;
    let card = Card(
        OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&selected.path)?,
    );
    let gbm_device = GbmDevice::new(card).map_err(std::io::Error::other)?;
    let resources = gbm_device.resource_handles()?;
    let connector = resources
        .connectors()
        .iter()
        .copied()
        .find(|handle| u32::from(*handle) == selected.mode.connector_id)
        .ok_or_else(|| std::io::Error::other("selected connector disappeared"))?;
    let connector_info = gbm_device.get_connector(connector, false)?;
    let mode = *connector_info
        .modes()
        .get(selected.mode.mode_index)
        .ok_or_else(|| std::io::Error::other("selected mode disappeared"))?;
    let mut selected_crtc = None;
    for encoder_handle in connector_info.encoders() {
        let encoder = gbm_device.get_encoder(*encoder_handle)?;
        let possible_crtcs = resources.filter_crtcs(encoder.possible_crtcs());
        if let Some(current) = encoder.crtc().filter(|crtc| possible_crtcs.contains(crtc)) {
            selected_crtc = Some(current);
            break;
        }
        selected_crtc = possible_crtcs.into_iter().find(|crtc| {
            gbm_device
                .get_crtc(*crtc)
                .is_ok_and(|info| info.mode().is_none())
        });
        if selected_crtc.is_some() {
            break;
        }
    }
    let crtc = selected_crtc.ok_or_else(|| {
        std::io::Error::other("connected output has no compatible unoccupied CRTC")
    })?;
    let previous_crtc = gbm_device.get_crtc(crtc)?;
    let previous_connectors = resources
        .connectors()
        .iter()
        .copied()
        .filter(|handle| {
            gbm_device
                .get_connector(*handle, false)
                .ok()
                .and_then(|info| info.current_encoder())
                .and_then(|encoder| gbm_device.get_encoder(encoder).ok())
                .is_some_and(|encoder| encoder.crtc() == Some(crtc))
        })
        .collect::<Vec<_>>();
    let width = u32::from(selected.mode.width);
    let height = u32::from(selected.mode.height);
    let gbm_surface = gbm_device.create_surface::<()>(
        width,
        height,
        Format::Xrgb8888,
        BufferObjectFlags::RENDERING | BufferObjectFlags::SCANOUT,
    )?;

    // SAFETY: libEGL.so is loaded dynamically and its EGL entry points are used through the crate.
    let egl = unsafe { egl::DynamicInstance::<egl::EGL1_5>::load_required() }
        .map_err(std::io::Error::other)?;
    // SAFETY: GBM returns a valid native display handle whose owner outlives the EGL display.
    let display = unsafe {
        egl.get_platform_display(
            EGL_PLATFORM_GBM_KHR,
            gbm_device.as_raw() as egl::NativeDisplayType,
            &[egl::ATTRIB_NONE],
        )
    }
    .map_err(std::io::Error::other)?;
    if let Err(error) = egl.initialize(display) {
        return Err(std::io::Error::other(error));
    }
    let config = match egl.choose_first_config(
        display,
        &[
            egl::SURFACE_TYPE,
            EGL_WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            EGL_OPENGL_ES3_BIT_KHR,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::NONE,
        ],
    ) {
        Ok(Some(config)) => config,
        Ok(None) => {
            let _ = egl.terminate(display);
            return Err(std::io::Error::other("no compatible EGL config"));
        }
        Err(error) => {
            let _ = egl.terminate(display);
            return Err(std::io::Error::other(error));
        }
    };
    // SAFETY: The GBM surface is live and retained in NativeSurface for longer than the EGL surface.
    let egl_surface = match unsafe {
        egl.create_window_surface(
            display,
            config,
            gbm_surface.as_raw() as usize as egl::NativeWindowType,
            None,
        )
    } {
        Ok(surface) => surface,
        Err(error) => {
            let _ = egl.terminate(display);
            return Err(std::io::Error::other(error));
        }
    };

    if let Err(error) = egl.bind_api(EGL_OPENGL_ES_API) {
        let _ = egl.destroy_surface(display, egl_surface);
        let _ = egl.terminate(display);
        return Err(std::io::Error::other(error));
    }
    let context = match egl.create_context(
        display,
        config,
        None,
        &[EGL_CONTEXT_CLIENT_VERSION, 3, egl::NONE],
    ) {
        Ok(context) => context,
        Err(error) => {
            let _ = egl.destroy_surface(display, egl_surface);
            let _ = egl.terminate(display);
            return Err(std::io::Error::other(error));
        }
    };
    if let Err(error) =
        egl.make_current(display, Some(egl_surface), Some(egl_surface), Some(context))
    {
        let _ = egl.destroy_context(display, context);
        let _ = egl.destroy_surface(display, egl_surface);
        let _ = egl.terminate(display);
        return Err(std::io::Error::other(error));
    }
    // SAFETY: EGL has made the ES3 context current on this thread. The returned GL entry points
    // are only used while this NativeSurface's context is current.
    let renderer = match unsafe {
        GlesRenderer::new(Arc::clone(&text_service), |name| {
            egl.get_proc_address(name)
                .map_or(std::ptr::null(), |address| {
                    address as *const () as *const c_void
                })
        })
    } {
        Ok(renderer) => renderer,
        Err(error) => {
            let _ = egl.make_current(display, None, None, None);
            let _ = egl.destroy_context(display, context);
            let _ = egl.destroy_surface(display, egl_surface);
            let _ = egl.terminate(display);
            return Err(std::io::Error::other(error));
        }
    };

    Ok(NativeSurface {
        egl,
        display,
        egl_surface,
        context,
        renderer: Some(renderer),
        size: (width, height),
        gbm_surface,
        gbm_device,
        connector,
        crtc,
        mode,
        previous_crtc,
        previous_connectors,
        front_buffer: None,
        pending_buffer: None,
        retired_buffers: Vec::new(),
        master_acquired: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_device_prefers_lowest_connector_with_preferred_mode() {
        let selected = choose_output([
            OutputMode {
                connector_id: 9,
                mode_index: 0,
                width: 3840,
                height: 2160,
                preferred: true,
            },
            OutputMode {
                connector_id: 3,
                mode_index: 0,
                width: 1920,
                height: 1080,
                preferred: true,
            },
            OutputMode {
                connector_id: 1,
                mode_index: 0,
                width: 800,
                height: 600,
                preferred: false,
            },
        ]);
        assert_eq!(selected.map(|mode| mode.connector_id), Some(3));
    }

    #[test]
    fn platform_device_uses_largest_area_then_lowest_connector_without_preferred_mode() {
        let selected = choose_output([
            OutputMode {
                connector_id: 8,
                mode_index: 0,
                width: 1920,
                height: 1080,
                preferred: false,
            },
            OutputMode {
                connector_id: 2,
                mode_index: 0,
                width: 2560,
                height: 1440,
                preferred: false,
            },
            OutputMode {
                connector_id: 1,
                mode_index: 0,
                width: 2560,
                height: 1440,
                preferred: false,
            },
        ]);
        assert_eq!(selected.map(|mode| mode.connector_id), Some(1));
    }

    #[test]
    fn platform_device_sorts_card_nodes_by_numeric_suffix() {
        let root = std::env::temp_dir().join(format!("freemvvm-dri-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture");
        for name in ["card10", "card2", "renderD128", "card0"] {
            File::create(root.join(name)).expect("create node fixture");
        }
        let names = card_nodes(&root)
            .expect("list nodes")
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_owned))
            .collect::<Vec<_>>();
        assert_eq!(names, ["card0", "card2", "card10"]);
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn platform_device_skips_unusable_cards_until_first_connected_output() {
        let attempted = std::cell::RefCell::new(Vec::new());
        let selected = first_usable_output([0, 1, 2], |card| {
            attempted.borrow_mut().push(*card);
            match card {
                0 | 1 => None, // permission/resource failure and no connected mode
                2 => Some(OutputMode {
                    connector_id: 4,
                    mode_index: 0,
                    width: 1280,
                    height: 720,
                    preferred: true,
                }),
                _ => unreachable!(),
            }
        });
        assert_eq!(attempted.into_inner(), [0, 1, 2]);
        assert_eq!(selected.map(|(card, _)| card), Some(2));
    }
}
