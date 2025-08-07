use evdev::{Device};
use std::fs::File;
use std::os::fd::OwnedFd;

const DEFAULT_INPUT_PATH: &str = "/dev/input";
const DEFAULT_CURSOR_PATH: &str = "/usr/share/icons/vantara/cursors/default.png";

pub fn find_mouse_device() -> Device {
    use std::fs::read_dir;

    for entry in read_dir(DEFAULT_INPUT_PATH).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.to_str().unwrap().contains("event") {
                if let Ok(file) = File::open(&path) {
                    let fd = OwnedFd::from(file);
                    if let Ok(dev) = Device::from_fd(fd) {
                        if dev.name().unwrap_or("").to_lowercase().contains("mouse") {
                            println!("[OK] Found mouse device: {:?}", path);
                            return dev;
                        }
                    }
                }
            }
        }
    }
    panic!("Mouse device not found!");
}

pub fn draw_cursor(fb: &mut [u8], fb_width: usize, fb_height: usize, x: usize, y: usize) {
    use image::imageops::FilterType;

    const CURSOR_SIZE: u32 = 22;

    let img = image::io::Reader::open(DEFAULT_CURSOR_PATH)
        .expect("Failed to open cursor image")
        .decode()
        .expect("Failed to decode image")
        .resize_exact(CURSOR_SIZE, CURSOR_SIZE, FilterType::Lanczos3)
        .to_rgba8();

    let (img_width, img_height) = img.dimensions();

    for dy in 0..img_height {
        for dx in 0..img_width {
            let px = x + dx as usize;
            let py = y + dy as usize;

            if px >= fb_width || py >= fb_height {
                continue;
            }

            let rgba = img.get_pixel(dx, dy).0;
            let alpha = rgba[3] as f32 / 255.0;

            let offset = (py * fb_width + px) * 4;

            // Alpha blending
            for i in 0..3 {
                let dst = fb[offset + i] as f32;
                let src = rgba[i] as f32;
                fb[offset + i] = (src * alpha + dst * (1.0 - alpha)) as u8;
            }

            // Biarkan alpha channel asal framebuffer (X)
        }
    }
}

pub fn restore_cursor_area(
    fb: &mut [u8],
    cache: &[u8],
    fb_width: usize,
    fb_height: usize,
    x: usize,
    y: usize,
) {
    const CURSOR_SIZE: usize = 22;

    for dy in 0..CURSOR_SIZE {
        for dx in 0..CURSOR_SIZE {
            let px = x + dx;
            let py = y + dy;

            if px >= fb_width || py >= fb_height {
                continue;
            }

            let offset = (py * fb_width + px) * 4;
            let original_offset = offset;
            fb[offset..offset + 4].copy_from_slice(&cache[original_offset..original_offset + 4]);
        }
    }
}
