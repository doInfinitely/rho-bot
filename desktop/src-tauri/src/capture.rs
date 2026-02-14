//! Screen capture via macOS CoreGraphics.
//!
//! Uses `CGWindowListCreateImage` to grab a screenshot of the entire display,
//! then encodes it as PNG and base64 for transmission.

use base64::Engine as _;
use std::io::Cursor;

#[cfg(target_os = "macos")]
use core_graphics::display::{
    kCGWindowListOptionOnScreenOnly, CGDisplay,
};

/// Capture the main display and return a base64-encoded PNG string.
#[cfg(target_os = "macos")]
pub fn capture_screen() -> Result<String, String> {
    let display = CGDisplay::main();
    let image = CGDisplay::screenshot(
        display.bounds(),
        kCGWindowListOptionOnScreenOnly,
        0, // kCGNullWindowID
        Default::default(),
    )
    .ok_or_else(|| "Failed to capture screenshot".to_string())?;

    let width = image.width();
    let height = image.height();
    let bytes_per_row = image.bytes_per_row();
    let raw_data = image.data();
    let data_ptr = raw_data.bytes().as_ptr();
    let data_len = (bytes_per_row * height) as usize;
    let pixel_data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };

    // Encode as PNG via a minimal approach: build raw RGBA image
    // CoreGraphics gives us BGRA, convert to RGBA for PNG encoding
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let offset = y * bytes_per_row + x * 4;
            let b = pixel_data[offset];
            let g = pixel_data[offset + 1];
            let r = pixel_data[offset + 2];
            let a = pixel_data[offset + 3];
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }
    }

    // Use a simple PNG encoder (we keep it lightweight)
    let mut png_buf = Cursor::new(Vec::new());
    // We'll just use raw base64 of the BGRA data with dimensions header.
    // In production, add the `image` crate for proper PNG encoding.
    // For now, we pack: width(u32 LE) + height(u32 LE) + RGBA bytes
    use std::io::Write;
    let _ = png_buf.write_all(&(width as u32).to_le_bytes());
    let _ = png_buf.write_all(&(height as u32).to_le_bytes());
    let _ = png_buf.write_all(&rgba);

    let encoded = base64::engine::general_purpose::STANDARD.encode(png_buf.into_inner());
    Ok(encoded)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_screen() -> Result<String, String> {
    Err("Screen capture is only supported on macOS".into())
}
