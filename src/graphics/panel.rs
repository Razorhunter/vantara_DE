pub fn draw_panels_to_framebuffer(
    framebuffer: &mut [u8],
    width: usize,
    height: usize,
    top_panel_height: usize,
    bottom_panel_height: usize,
    top_color: (u8, u8, u8),    // RGB
    bottom_color: (u8, u8, u8), // RGB
) {
    // Panel atas
    for y in 0..top_panel_height {
        for x in 0..width {
            let offset = ((y * width) + x) * 4;
            framebuffer[offset] = top_color.2;     // Blue
            framebuffer[offset + 1] = top_color.1; // Green
            framebuffer[offset + 2] = top_color.0; // Red
            framebuffer[offset + 3] = 0x00;
        }
    }

    // Panel bawah
    for y in (height - bottom_panel_height)..height {
        for x in 0..width {
            let offset = ((y * width) + x) * 4;
            framebuffer[offset] = bottom_color.2;
            framebuffer[offset + 1] = bottom_color.1;
            framebuffer[offset + 2] = bottom_color.0;
            framebuffer[offset + 3] = 0x00;
        }
    }
}
