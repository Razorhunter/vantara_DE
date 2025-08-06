use std::error::Error;
use image::imageops::FilterType;

pub enum WallpaperMode {
    Fit,
    Fill,
    Stretch,
    Center,
}

pub fn draw_wallpaper_to_framebuffer(
    framebuffer: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    image_path: &str,
    mode: WallpaperMode,
) -> Result<(), Box<dyn Error>> {
    let img = image::ImageReader::open(image_path)
        .map_err(|e| format!("Failed to open image: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?
        .to_rgba8();
    let (img_width, img_height) = img.dimensions();

    let (new_width, new_height, offset_x, offset_y, resized) = match mode {
        WallpaperMode::Fit => {
            let scale_x = fb_width as f32 / img_width as f32;
            let scale_y = fb_height as f32 / img_height as f32;
            let scale = scale_x.min(scale_y);

            let new_w = (img_width as f32 * scale).round() as u32;
            let new_h = (img_height as f32 * scale).round() as u32;

            if new_w == 0 || new_h == 0 {
                return Err("Image too small after scaling".into());
            }

            let resized = image::imageops::resize(&img, new_w, new_h, FilterType::Lanczos3);
            let offset_x = fb_width.saturating_sub(new_w as usize) / 2;
            let offset_y = fb_height.saturating_sub(new_h as usize) / 2;
            (new_w, new_h, offset_x, offset_y, resized)
        }

        WallpaperMode::Fill => {
            let scale_x = fb_width as f32 / img_width as f32;
            let scale_y = fb_height as f32 / img_height as f32;
            let scale = scale_x.max(scale_y);

            let new_w = (img_width as f32 * scale).round() as u32;
            let new_h = (img_height as f32 * scale).round() as u32;

            if new_w == 0 || new_h == 0 {
                return Err("Image too small after scaling".into());
            }

            let resized = image::imageops::resize(&img, new_w, new_h, FilterType::Lanczos3);
            let offset_x = fb_width.saturating_sub(new_w as usize) / 2;
            let offset_y = fb_height.saturating_sub(new_h as usize) / 2;
            (new_w, new_h, offset_x, offset_y, resized)
        }

        WallpaperMode::Stretch => {
            let resized = image::imageops::resize(&img, fb_width as u32, fb_height as u32, FilterType::Lanczos3);
            (fb_width as u32, fb_height as u32, 0, 0, resized)
        }

        WallpaperMode::Center => {
            let offset_x = fb_width.saturating_sub(img_width as usize) / 2;
            let offset_y = fb_height.saturating_sub(img_height as usize) / 2;
            (img_width, img_height, offset_x, offset_y, img)
        }
    };

    let image_data = resized.into_raw();

    for y in 0..new_height {
        for x in 0..new_width {
            let fb_x = x as usize + offset_x;
            let fb_y = y as usize + offset_y;

            if fb_x >= fb_width || fb_y >= fb_height {
                continue;
            }

            let fb_index = (fb_y * fb_width + fb_x) * 4;
            let img_index = ((y * new_width + x) * 4) as usize;

            // RGBA → XRGB8888
            framebuffer[fb_index + 0] = image_data[img_index + 2]; // B
            framebuffer[fb_index + 1] = image_data[img_index + 1]; // G
            framebuffer[fb_index + 2] = image_data[img_index + 0]; // R
            framebuffer[fb_index + 3] = 0x00;                      // X (Alpha tak guna)
        }
    }

    Ok(())
}
