use evdev::{Device, InputEvent};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::fd::OwnedFd;

pub fn find_mouse_device() -> Device {
    use std::fs::read_dir;

    for entry in read_dir("/dev/input").unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.to_str().unwrap().contains("event") {
                if let Ok(file) = File::open(&path) {
                    let fd = OwnedFd::from(file);
                    if let Ok(mut dev) = Device::from_fd(fd) {
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
    let size = 10;
    for dy in 0..size {
        for dx in 0..size {
            let px = x + dx;
            let py = y + dy;

            if px >= fb_width || py >= fb_height {
                continue;
            }

            let offset = (py * fb_width + px) * 4;
            fb[offset] = 0xFF;        // Blue
            fb[offset + 1] = 0xFF;    // Green
            fb[offset + 2] = 0xFF;    // Red
            fb[offset + 3] = 0x00;    // X
        }
    }
}

pub fn restore_cursor_area(fb: &mut [u8], fb_width: usize, fb_height: usize, x: usize, y: usize) {
    let size = 10;
    for dy in 0..size {
        for dx in 0..size {
            let px = x + dx;
            let py = y + dy;

            if px >= fb_width || py >= fb_height {
                continue;
            }

            let offset = (py * fb_width + px) * 4;

            // Latar belakang asal - samakan dengan warna asal kau
            fb[offset] = 0xFF;        // Blue
            fb[offset + 1] = 0x99;    // Green
            fb[offset + 2] = 0x33;    // Red
            fb[offset + 3] = 0x00;    // X
        }
    }
}
