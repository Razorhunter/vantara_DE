mod wallpaper;

use drm::control::{self, ResourceHandles, Device as ControlDevice};
use drm::Device as BasicDevice;
use std::fs::File;
use std::os::fd::{AsFd, BorrowedFd};
use drm::buffer::DrmFourcc;
use std::fs;
use std::io::Result;
use nix::sys::stat::Mode;
use nix::fcntl::{open as nix_open, OFlag};
use std::os::unix::io::FromRawFd;
use std::os::fd::IntoRawFd;
use std::thread;
use std::time::Duration;

pub struct DisplaySetup {
    pub connector: control::connector::Info,
    pub encoder: control::encoder::Info,
    pub crtc: control::crtc::Handle,
}

/// PEMBALUT struct untuk `drm-rs`
struct DrmDeviceWrapper(File);
impl AsFd for DrmDeviceWrapper {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl BasicDevice for DrmDeviceWrapper {}
impl ControlDevice for DrmDeviceWrapper {}

const DEFAULT_GPU_DRIVE_PATH: &str = "/dev/dri";

fn main() -> Result<()> {
    let path = find_drm_card().expect("/dev/dri/cardX not found!");
    println!("[OK] Found GPU device: {}", path);

    let fd = nix_open(std::path::Path::new(&path), OFlag::O_RDWR, Mode::empty())
        .expect("Failed to open GPU device with O_RDWR");

    let file = unsafe { File::from_raw_fd(fd.into_raw_fd()) };

    let drm = DrmDeviceWrapper(file);

    let res = drm.resource_handles().unwrap();

    let setup = find_best_display_setup(&drm, &res).expect("Failed to find connector/encoder/crtc");

    let mode = setup.connector.modes()
        .iter()
        .find(|m| m.mode_type().contains(control::ModeTypeFlags::PREFERRED))
        .unwrap_or_else(|| &setup.connector.modes()[0]);

    let (width, height) = mode.size();
    println!("[OK] Display setup: {}x{} @ {}Hz", width, height, mode.vrefresh());

    let mut dumb = drm.create_dumb_buffer((width.into(), height.into()), DrmFourcc::Xrgb8888, 32)?;

    let fb = drm.add_framebuffer(&dumb, 24, 32)?;

    let mut mapping = drm.map_dumb_buffer(&mut dumb)?;

    let framebuffer = mapping.as_mut();
    println!("[OK] Framebuffer size: {} bytes", framebuffer.len());

    let pixels = (width as usize) * (height as usize);
    println!("[OK] Framebuffer pixels: {}", pixels);

    // for i in 0..pixels {
    //     let offset = i * 4;
    //     framebuffer[offset] = 0xFF;        // Blue (255)
    //     framebuffer[offset + 1] = 0x99;    // Green (153)
    //     framebuffer[offset + 2] = 0x33;    // Red (51)
    //     framebuffer[offset + 3] = 0x00;    // X / Alpha (tak guna)
    // }

    drm.set_crtc(setup.crtc, Some(fb), (0, 0), &[setup.connector.handle()], Some(*mode))?;

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn find_drm_card() -> Option<String> {
    let entries = fs::read_dir(DEFAULT_GPU_DRIVE_PATH).ok()?;
    for entry in entries {
        let path = entry.ok()?.path();
        if path.file_name()?.to_string_lossy().starts_with("card") {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}

pub fn find_best_display_setup<T: ControlDevice>(dev: &T, res: &ResourceHandles,) -> Option<DisplaySetup> {
    for &conn_handle in res.connectors() {
        let conn_info = dev.get_connector(conn_handle, true).ok()?;

        if conn_info.state() != control::connector::State::Connected {
            continue;
        }

        let enc_handle = if let Some(e) = conn_info.current_encoder() {
            e
        } else if let Some(e) = conn_info.encoders().get(0) {
            *e
        } else {
            continue;
        };

        let encoder = dev.get_encoder(enc_handle).ok()?;

        if let Some(crtc) = encoder.crtc() {
            return Some(DisplaySetup {
                connector: conn_info,
                encoder,
                crtc,
            });
        }

        let bitmask = unsafe { std::mem::transmute::<_, u32>(encoder.possible_crtcs()) };

        for (i, &crtc_handle) in res.crtcs().iter().enumerate() {
            if (bitmask & (1 << i)) != 0 {
                return Some(DisplaySetup {
                    connector: conn_info,
                    encoder,
                    crtc: crtc_handle,
                });
            }
        }
    }

    println!("[WARN] No usable display setup found.");
    None
}
