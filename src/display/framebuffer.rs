use drm::control::Device as ControlDevice;
use drm::buffer::DrmFourcc;
use crate::graphics::{draw_wallpaper_to_framebuffer, WallpaperMode};

pub fn create_and_draw_framebuffer<T: ControlDevice>(
    dev: &T,
    width: u16,
    height: u16,
    wallpaper_path: &str,
) -> Result<drm::control::framebuffer::Handle, Box<dyn std::error::Error>> {
    let mut dumb = dev.create_dumb_buffer((width.into(), height.into()), DrmFourcc::Xrgb8888, 32)?;
    let fb = dev.add_framebuffer(&dumb, 24, 32)?;
    let mut mapping = dev.map_dumb_buffer(&mut dumb)?;
    let framebuffer = mapping.as_mut();
    let pixels = (width as usize) * (height as usize);

    for i in 0..pixels {
        let offset = i * 4;
        framebuffer[offset] = 0xFF;
        framebuffer[offset + 1] = 0x99;
        framebuffer[offset + 2] = 0x33;
        framebuffer[offset + 3] = 0x00;
    }

    draw_wallpaper_to_framebuffer(
        framebuffer,
        width as usize,
        height as usize,
        wallpaper_path,
        WallpaperMode::Fill,
    )?;

    Ok(fb)
}
