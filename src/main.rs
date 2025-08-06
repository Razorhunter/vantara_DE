mod display;
mod graphics;
mod input;

use display::drm_device::DrmDeviceWrapper;
use display::find_best_display_setup;
use display::framebuffer::{create_and_draw_framebuffer};
use display::utils::find_drm_card;

use drm::control::Device;
use std::thread;
use std::time::Duration;

const DEFAULT_WALLPAPER_PATH: &str = "/usr/share/backgrounds/default-wallpaper.jpg";

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

    let fb = create_and_draw_framebuffer(&drm, width, height, DEFAULT_WALLPAPER_PATH)?;

    drm.set_crtc(setup.crtc, Some(fb), (0, 0), &[setup.connector.handle()], Some(*mode))?;

    //Initialize cursor
    let mut dev = input::cursor::find_mouse_device();

    let mut cursor_x = width as i32 / 2;
    let mut cursor_y = height as i32 / 2;

    let mut prev_cursor_pos = (cursor_x, cursor_y);

    loop {
        for ev in dev.fetch_events().unwrap() {
            match ev.kind() {
                InputEventKind::RelAxis(axis) => match axis {
                    evdev::RelativeAxisType::REL_X => {
                        cursor_x += ev.value();
                    }
                    evdev::RelativeAxisType::REL_Y => {
                        cursor_y += ev.value();
                    }
                    _ => {}
                },
                _ => {}
            }

            // Clamp cursor within screen bounds
            cursor_x = cursor_x.clamp(0, width as i32 - 1);
            cursor_y = cursor_y.clamp(0, height as i32 - 1);
        }

        input::cursor::restore_cursor_area(framebuffer, width as usize, height as usize, prev_cursor_pos.0 as usize, prev_cursor_pos.1 as usize);

        input::cursor::draw_cursor(framebuffer, width as usize, height as usize, cursor_x as usize, cursor_y as usize);

        prev_cursor_pos = (cursor_x, cursor_y);

        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }
}
