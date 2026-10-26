use drm::control::{Device as ControlDevice, Device as _, ModeTypeFlags, connector};
use gbm::{AsRaw, BufferObjectFlags, Device as GbmDevice, Format};
use khronos_egl as egl;
use std::fs::{self, File, OpenOptions};
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

const EGL_PLATFORM_GBM_KHR: egl::Enum = 0x31D7;
const EGL_WINDOW_BIT: egl::Int = 0x0004;
const EGL_OPENGL_ES3_BIT_KHR: egl::Int = 0x0040;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OutputMode {
    connector_id: u32,
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
    for path in nodes {
        let card = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(file) => Card(file),
            Err(_) => continue,
        };
        let resources = match card.resource_handles() {
            Ok(resources) => resources,
            Err(_) => continue,
        };
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
                        .map(|mode| OutputMode {
                            connector_id: u32::from(handle),
                            width: mode.size().0,
                            height: mode.size().1,
                            preferred: mode.mode_type().contains(ModeTypeFlags::PREFERRED),
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten();
        if let Some(mode) = choose_output(modes) {
            return Ok(SelectedDevice { path, mode });
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "no DRM card has a connected connector with a usable mode",
    ))
}

pub(super) struct NativeSurface {
    egl: egl::DynamicInstance<egl::EGL1_5>,
    display: egl::Display,
    egl_surface: egl::Surface,
    _gbm_surface: gbm::Surface<()>,
    _gbm_device: GbmDevice<Card>,
    _selected: SelectedDevice,
}

impl Drop for NativeSurface {
    fn drop(&mut self) {
        let _ = self.egl.destroy_surface(self.display, self.egl_surface);
        let _ = self.egl.terminate(self.display);
    }
}

pub(super) fn open_surface() -> std::io::Result<NativeSurface> {
    let selected = open_selected_device()?;
    let card = Card(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&selected.path)?,
    );
    let gbm_device = GbmDevice::new(card).map_err(std::io::Error::other)?;
    let width = u32::from(selected.mode.width);
    let height = u32::from(selected.mode.height);
    let gbm_surface = gbm_device.create_surface::<()>(
        width,
        height,
        Format::Xrgb8888,
        BufferObjectFlags::RENDERING | BufferObjectFlags::SCANOUT,
    )?;

    // SAFETY: libEGL.so is loaded dynamically and its EGL entry points are used through the crate.
    let egl =
        unsafe { egl::DynamicInstance::<egl::EGL1_5>::load() }.map_err(std::io::Error::other)?;
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

    Ok(NativeSurface {
        egl,
        display,
        egl_surface,
        _gbm_surface: gbm_surface,
        _gbm_device: gbm_device,
        _selected: selected,
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
                width: 3840,
                height: 2160,
                preferred: true,
            },
            OutputMode {
                connector_id: 3,
                width: 1920,
                height: 1080,
                preferred: true,
            },
            OutputMode {
                connector_id: 1,
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
                width: 1920,
                height: 1080,
                preferred: false,
            },
            OutputMode {
                connector_id: 2,
                width: 2560,
                height: 1440,
                preferred: false,
            },
            OutputMode {
                connector_id: 1,
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
}
