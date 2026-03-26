use image::{DynamicImage, ImageError, RgbaImage};
use std::path::Path;

/// Loads an image from disk. Supports PNG, JPEG, BMP, GIF (first frame), WebP, TIFF.
pub fn load(path: &Path) -> Result<DynamicImage, ImageError> {
    image::open(path)
}

/// Composites transparent images onto a background color.
/// Call this immediately after loading for any image that may have alpha.
pub fn flatten_alpha(image: &DynamicImage, bg_color: [u8; 3]) -> DynamicImage {
    let rgba = image.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    let mut output = RgbaImage::new(w, h);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;
        let r = (pixel[0] as f32 * alpha + bg_color[0] as f32 * inv_alpha) as u8;
        let g = (pixel[1] as f32 * alpha + bg_color[1] as f32 * inv_alpha) as u8;
        let b = (pixel[2] as f32 * alpha + bg_color[2] as f32 * inv_alpha) as u8;
        output.put_pixel(x, y, image::Rgba([r, g, b, 255]));
    }

    DynamicImage::ImageRgba8(output)
}

/// Resizes an image to the target column count for ASCII conversion.
/// Accounts for the ~2:1 character cell aspect ratio.
/// Returns (resized_image, cols, rows).
pub fn resize_for_ascii(image: &DynamicImage, target_cols: usize) -> (RgbaImage, usize, usize) {
    let (w, h) = (image.width() as f64, image.height() as f64);
    let cols = target_cols.max(1);
    let rows = ((h / w) * cols as f64 * 0.5).round() as usize;
    let rows = rows.max(1);
    let resized = image.resize_exact(
        cols as u32,
        rows as u32,
        image::imageops::FilterType::Triangle,
    );
    (resized.to_rgba8(), cols, rows)
}
