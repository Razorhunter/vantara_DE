mod display;
mod graphics;
mod input;

use display::drm_device::DrmDeviceWrapper;
use display::find_best_display_setup;
use display::utils::find_drm_card;
use drm::control::Device as ControlDevice;
use drm::buffer::DrmFourcc;
use graphics::{draw_wallpaper_to_framebuffer, WallpaperMode};
use std::thread;
use std::time::{Instant, Duration};

use evdev::EventType;

const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;
const DEFAULT_WALLPAPER_PATH: &str = "/usr/share/backgrounds/default-wallpapers.jpg";

fn main() -> display::MyResult<()> {
    //Initialize Display
    let path = find_drm_card().expect("/dev/dri/cardX not found!");
    let drm = DrmDeviceWrapper::open(&path)?;
    let res = drm.resource_handles()?;
    let setup = find_best_display_setup(&drm, &res).expect("Failed to find connector/encoder/crtc");

    let mode = setup.connector.modes()
        .iter()
        .find(|m| m.mode_type().contains(drm::control::ModeTypeFlags::PREFERRED))
        .unwrap_or_else(|| &setup.connector.modes()[0]);

    let (width, height) = mode.size();
    let mut dumb = drm.create_dumb_buffer((width.into(), height.into()), DrmFourcc::Xrgb8888, 32)?;
    let fb = drm.add_framebuffer(&dumb, 24, 32)?;
    let mut mapping = drm.map_dumb_buffer(&mut dumb)?;
    let framebuffer = mapping.as_mut();
    let pixels = (width as usize) * (height as usize);

    match draw_wallpaper_to_framebuffer(
        framebuffer,
        width as usize,
        height as usize,
        DEFAULT_WALLPAPER_PATH,
        WallpaperMode::Fill,
    ) {
        Ok(()) => {}
        Err(e) => {
            for i in 0..pixels {
                let offset = i * 4;
                framebuffer[offset] = 0xFF;
                framebuffer[offset + 1] = 0x99;
                framebuffer[offset + 2] = 0x33;
                framebuffer[offset + 3] = 0x00;
            }
        }
    }

    drm.set_crtc(setup.crtc, Some(fb), (0, 0), &[setup.connector.handle()], Some(*mode))?;

    //Initialize cursor
    let mut dev = input::cursor::find_mouse_device();
    let mut cursor_x = width as i32 / 2;
    let mut cursor_y = height as i32 / 2;
    let mut prev_cursor_pos = (cursor_x, cursor_y);
    let background_cache = framebuffer.to_vec();
    let mut last = Instant::now();

    loop {
        for ev in dev.fetch_events().unwrap() {
            match ev.event_type() {
                EventType::RELATIVE => {
                    match ev.code() {
                        REL_X => {
                            cursor_x += ev.value();
                        }
                        REL_Y => {
                            cursor_y += ev.value();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // Clamp cursor within screen bounds
            cursor_x = cursor_x.clamp(0, width as i32 - 1);
            cursor_y = cursor_y.clamp(0, height as i32 - 1);
        }

        input::cursor::restore_cursor_area(framebuffer, &background_cache, width as usize, height as usize, prev_cursor_pos.0 as usize, prev_cursor_pos.1 as usize);
        input::cursor::draw_cursor(framebuffer, width as usize, height as usize, cursor_x as usize, cursor_y as usize);
        drm.set_crtc(setup.crtc, Some(fb), (0, 0), &[setup.connector.handle()], Some(*mode))?;
        prev_cursor_pos = (cursor_x, cursor_y);

        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }
}
