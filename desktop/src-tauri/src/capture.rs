//! Screen capture via macOS CoreGraphics.
//!
//! Uses `CGWindowListCreateImage` to grab a screenshot of the entire display,
//! then encodes it as PNG and base64 for transmission.

use base64::Engine as _;

#[cfg(target_os = "macos")]
use core_graphics::display::{kCGWindowListOptionOnScreenOnly, CGDisplay};

#[cfg(target_os = "macos")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

/// Check whether Screen Recording permission has been granted.
#[cfg(target_os = "macos")]
pub fn has_screen_recording_permission() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
pub fn has_screen_recording_permission() -> bool {
    true
}

/// Capture the main display and return a base64-encoded PNG string.
#[cfg(target_os = "macos")]
pub fn capture_screen() -> Result<String, String> {
    // Quick permission check — no UI, no dialogs, just a bool.
    if !has_screen_recording_permission() {
        return Err("Screen Recording permission not granted".into());
    }

    let display = CGDisplay::main();
    let cg_image = CGDisplay::screenshot(
        display.bounds(),
        kCGWindowListOptionOnScreenOnly,
        0, // kCGNullWindowID
        Default::default(),
    )
    .ok_or_else(|| "CGDisplay::screenshot returned nil".to_string())?;

    let width = cg_image.width() as u32;
    let height = cg_image.height() as u32;
    let bytes_per_row = cg_image.bytes_per_row();

    if width == 0 || height == 0 {
        return Err("Screenshot has zero dimensions".into());
    }

    let raw_data = cg_image.data();
    let actual_len = raw_data.bytes().len();
    let expected_len = bytes_per_row * height as usize;

    if actual_len < expected_len {
        return Err(format!(
            "Screenshot data too small: {} bytes < expected {} ({}x{}, bpr={})",
            actual_len, expected_len, width, height, bytes_per_row
        ));
    }

    if bytes_per_row < (width as usize) * 4 {
        return Err(format!(
            "bytes_per_row ({}) < width*4 ({})",
            bytes_per_row,
            (width as usize) * 4
        ));
    }

    let data_ptr = raw_data.bytes().as_ptr();
    let pixel_data = unsafe { std::slice::from_raw_parts(data_ptr, expected_len) };

    // CoreGraphics gives us BGRA; convert to RGBA for the `image` crate
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        for x in 0..width as usize {
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

    // Encode as a real PNG using the `image` crate
    let img_buf: image::RgbaImage =
        image::ImageBuffer::from_raw(width, height, rgba)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;

    let dynamic_img = image::DynamicImage::ImageRgba8(img_buf);

    let mut png_buf = std::io::Cursor::new(Vec::new());
    dynamic_img
        .write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding failed: {}", e))?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(png_buf.into_inner());
    Ok(encoded)
}

/// Get the dimensions of the main display.
#[cfg(target_os = "macos")]
pub fn screen_size() -> (f64, f64) {
    let display = CGDisplay::main();
    let bounds = display.bounds();
    (bounds.size.width, bounds.size.height)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_screen() -> Result<String, String> {
    Err("Screen capture is only supported on macOS".into())
}

#[cfg(not(target_os = "macos"))]
pub fn screen_size() -> (f64, f64) {
    (1920.0, 1080.0)
}
