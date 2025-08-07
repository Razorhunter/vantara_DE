pub mod drm_device;
pub mod utils;

use drm::control::{self, ResourceHandles, Device as ControlDevice};

pub type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct DisplaySetup {
    pub connector: control::connector::Info,
    pub encoder: control::encoder::Info,
    pub crtc: control::crtc::Handle,
}

pub fn find_best_display_setup<T: ControlDevice>(
    dev: &T,
    res: &ResourceHandles,
) -> Option<DisplaySetup> {
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
