use crate::image_loader;
use crate::state::{AsciiOutput, ColorMode};
use image::DynamicImage;

/// Standard ANSI 16-color palette: [(R, G, B); 16]
const ANSI_PALETTE: [[u8; 3]; 16] = [
    [0, 0, 0],
    [170, 0, 0],
    [0, 170, 0],
    [170, 85, 0],
    [0, 0, 170],
    [170, 0, 170],
    [0, 170, 170],
    [170, 170, 170],
    [85, 85, 85],
    [255, 85, 85],
    [85, 255, 85],
    [255, 255, 85],
    [85, 85, 255],
    [255, 85, 255],
    [85, 255, 255],
    [255, 255, 255],
];

/// Default character ramp from lightest (space) to darkest (@).
const DEFAULT_RAMP: &str = " .:-=+*#%@";

/// Main conversion function. Produces ASCII output from an image and settings.
///
/// The conversion pipeline:
/// 1. Resize image to target columns (with aspect ratio correction)
/// 2. For each pixel, compute luminance with brightness/contrast adjustments
/// 3. Map luminance to a character from the ramp
/// 4. Optionally capture per-character color data
pub fn convert(
    image: &DynamicImage,
    char_ramp: &str,
    output_columns: usize,
    brightness: f32,
    contrast: f32,
    invert: bool,
    color_mode: ColorMode,
) -> AsciiOutput {
    let ramp: Vec<char> = if char_ramp.is_empty() {
        DEFAULT_RAMP.chars().collect()
    } else {
        char_ramp.chars().collect()
    };
    let ramp_len = ramp.len();

    // Handle zero-dimension images
    if image.width() == 0 || image.height() == 0 {
        return AsciiOutput {
            chars: Vec::new(),
            colors: None,
            cols: 0,
            rows: 0,
        };
    }

    let (rgba, cols, rows) = image_loader::resize_for_ascii(image, output_columns);

    let want_color = color_mode != ColorMode::Off;
    let mut chars = Vec::with_capacity(rows);
    let mut colors: Vec<Vec<[u8; 3]>> = if want_color {
        Vec::with_capacity(rows)
    } else {
        Vec::new()
    };

    for row in 0..rows {
        let mut char_row = Vec::with_capacity(cols);
        let mut color_row = if want_color {
            Vec::with_capacity(cols)
        } else {
            Vec::new()
        };

        for col in 0..cols {
            let pixel = rgba.get_pixel(col as u32, row as u32);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            // Compute luminance (ITU-R BT.601)
            let mut luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;

            // Apply brightness
            luminance += brightness;

            // Apply contrast
            luminance = ((luminance - 0.5) * contrast) + 0.5;

            // Clamp
            luminance = luminance.clamp(0.0, 1.0);

            // Invert
            if invert {
                luminance = 1.0 - luminance;
            }

            // Map to character
            let idx = if ramp_len == 1 {
                0
            } else {
                (luminance * (ramp_len - 1) as f32).round() as usize
            };
            let idx = idx.min(ramp_len - 1);
            char_row.push(ramp[idx]);

            // Store color if needed
            if want_color {
                let rgb = match color_mode {
                    ColorMode::FullRgb => [r, g, b],
                    ColorMode::Ansi16 => nearest_ansi_color(r, g, b),
                    ColorMode::Off => unreachable!(),
                };
                color_row.push(rgb);
            }
        }

        chars.push(char_row);
        if want_color {
            colors.push(color_row);
        }
    }

    AsciiOutput {
        chars,
        colors: if want_color { Some(colors) } else { None },
        cols,
        rows,
    }
}

/// Find the nearest ANSI 16-color palette entry using Euclidean distance in RGB space.
fn nearest_ansi_color(r: u8, g: u8, b: u8) -> [u8; 3] {
    ANSI_PALETTE
        .iter()
        .min_by_key(|&&[pr, pg, pb]| {
            let dr = r as i32 - pr as i32;
            let dg = g as i32 - pg as i32;
            let db = b as i32 - pb as i32;
            dr * dr + dg * dg + db * db
        })
        .copied()
        .unwrap_or([255, 255, 255])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    fn make_solid_image(r: u8, g: u8, b: u8, w: u32, h: u32) -> DynamicImage {
        let mut img = RgbaImage::new(w, h);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([r, g, b, 255]);
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn white_pixel_maps_to_last_char() {
        let img = make_solid_image(255, 255, 255, 10, 10);
        let output = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
        assert_eq!(output.chars[0][0], '@');
    }

    #[test]
    fn black_pixel_maps_to_first_char() {
        let img = make_solid_image(0, 0, 0, 10, 10);
        let output = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
        assert_eq!(output.chars[0][0], ' ');
    }

    #[test]
    fn mid_gray_maps_to_middle_char() {
        let img = make_solid_image(128, 128, 128, 10, 10);
        let output = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
        let mid_idx = ((128.0_f32 / 255.0) * 9.0).round() as usize;
        let expected = DEFAULT_RAMP.chars().nth(mid_idx).unwrap_or(' ');
        assert_eq!(output.chars[0][0], expected);
    }

    #[test]
    fn invert_flips_output() {
        let img = make_solid_image(255, 255, 255, 10, 10);
        let normal = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
        let inverted = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, true, ColorMode::Off);
        assert_ne!(normal.chars[0][0], inverted.chars[0][0]);
        assert_eq!(inverted.chars[0][0], ' ');
    }

    #[test]
    fn brightness_shifts_output() {
        let img = make_solid_image(0, 0, 0, 10, 10);
        let bright = convert(&img, DEFAULT_RAMP, 10, 0.5, 1.0, false, ColorMode::Off);
        // Black + 0.5 brightness should map to a mid-range character
        assert_ne!(bright.chars[0][0], ' ');
    }

    #[test]
    fn contrast_amplifies_differences() {
        let img = make_solid_image(140, 140, 140, 10, 10);
        let low_contrast = convert(&img, DEFAULT_RAMP, 10, 0.0, 0.5, false, ColorMode::Off);
        let high_contrast = convert(&img, DEFAULT_RAMP, 10, 0.0, 3.0, false, ColorMode::Off);
        // High contrast on a slightly-above-mid gray should push it further from center
        let ramp: Vec<char> = DEFAULT_RAMP.chars().collect();
        let low_idx = ramp
            .iter()
            .position(|&c| c == low_contrast.chars[0][0])
            .unwrap_or(0);
        let high_idx = ramp
            .iter()
            .position(|&c| c == high_contrast.chars[0][0])
            .unwrap_or(0);
        assert!(high_idx > low_idx || high_idx == ramp.len() - 1);
    }

    #[test]
    fn empty_ramp_uses_default() {
        let img = make_solid_image(128, 128, 128, 10, 10);
        let output = convert(&img, "", 10, 0.0, 1.0, false, ColorMode::Off);
        assert!(!output.chars.is_empty());
        assert!(output.chars[0][0] != '\0');
    }

    #[test]
    fn single_char_ramp_fills_all() {
        let img = make_solid_image(128, 128, 128, 10, 10);
        let output = convert(&img, "X", 10, 0.0, 1.0, false, ColorMode::Off);
        for row in &output.chars {
            for &ch in row {
                assert_eq!(ch, 'X');
            }
        }
    }

    #[test]
    fn ansi_color_nearest_black() {
        let color = nearest_ansi_color(0, 0, 0);
        assert_eq!(color, [0, 0, 0]);
    }

    #[test]
    fn ansi_color_nearest_white() {
        let color = nearest_ansi_color(255, 255, 255);
        assert_eq!(color, [255, 255, 255]);
    }

    #[test]
    fn ansi_color_nearest_red() {
        let color = nearest_ansi_color(200, 10, 10);
        // Should match bright red [255, 85, 85] or dark red [170, 0, 0]
        assert!(color == [170, 0, 0] || color == [255, 85, 85]);
    }

    #[test]
    fn color_mode_full_rgb_stores_colors() {
        let img = make_solid_image(100, 150, 200, 10, 10);
        let output = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::FullRgb);
        assert!(output.colors.is_some());
        let colors = output.colors.as_ref().unwrap();
        assert_eq!(colors[0][0], [100, 150, 200]);
    }

    #[test]
    fn color_mode_off_no_colors() {
        let img = make_solid_image(100, 150, 200, 10, 10);
        let output = convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
        assert!(output.colors.is_none());
    }

    #[test]
    fn output_dimensions_match() {
        let img = make_solid_image(128, 128, 128, 100, 50);
        let output = convert(&img, DEFAULT_RAMP, 40, 0.0, 1.0, false, ColorMode::Off);
        assert_eq!(output.cols, 40);
        assert_eq!(output.chars.len(), output.rows);
        assert_eq!(output.chars[0].len(), output.cols);
    }
}
