use image::io::Reader as ImageReader;
use image::GenericImageView;

fn draw_wallpaper_to_framebuffer(
    framebuffer: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    image_data: &[u8],
    img_width: usize,
    img_height: usize,
    x_offset: usize,
    y_offset: usize,
) {
    for y in 0..img_height {
        for x in 0..img_width {
            let fb_x = x + x_offset;
            let fb_y = y + y_offset;

            if fb_x >= fb_width || fb_y >= fb_height {
                continue; // Elak keluar sempadan
            }

            let fb_index = (fb_y * fb_width + fb_x) * 4;
            let img_index = (y * img_width + x) * 4;

            // Salin 4 byte: B, G, R, A (atau ikut format imej)
            framebuffer[fb_index..fb_index + 4]
                .copy_from_slice(&image_data[img_index..img_index + 4]);
        }
    }
}
