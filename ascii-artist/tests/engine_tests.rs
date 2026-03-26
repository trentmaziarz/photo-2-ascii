use ascii_artist::ascii_engine;
use ascii_artist::state::ColorMode;
use image::{DynamicImage, Rgba, RgbaImage};

fn create_white_image(w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])))
}

fn create_black_image(w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255])))
}

fn create_solid_image(r: u8, g: u8, b: u8, w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([r, g, b, 255])))
}

fn create_gradient_image(w: u32, h: u32) -> DynamicImage {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let val = ((x as f32 / (w.max(1) - 1).max(1) as f32) * 255.0) as u8;
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }
    DynamicImage::ImageRgba8(img)
}

const DEFAULT_RAMP: &str = " .:-=+*#%@";

#[test]
fn test_white_pixel_maps_to_last_char() {
    let img = create_white_image(10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.chars[0][0], '@');
}

#[test]
fn test_black_pixel_maps_to_first_char() {
    let img = create_black_image(10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.chars[0][0], ' ');
}

#[test]
fn test_mid_gray_maps_to_middle_char() {
    let img = create_solid_image(128, 128, 128, 10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
    let ramp: Vec<char> = DEFAULT_RAMP.chars().collect();
    let mid_idx = ((128.0_f32 / 255.0) * 9.0).round() as usize;
    assert_eq!(output.chars[0][0], ramp[mid_idx]);
}

#[test]
fn test_invert_flips_mapping() {
    let img = create_white_image(10, 10);
    let normal = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
    let inverted = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, true, ColorMode::Off);
    assert_eq!(normal.chars[0][0], '@');
    assert_eq!(inverted.chars[0][0], ' ');
}

#[test]
fn test_brightness_positive_shifts_up() {
    let img = create_black_image(10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.5, 1.0, false, ColorMode::Off);
    // Black + 0.5 brightness should not be space anymore
    assert_ne!(output.chars[0][0], ' ');
}

#[test]
fn test_brightness_negative_shifts_down() {
    let img = create_white_image(10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, -0.5, 1.0, false, ColorMode::Off);
    // White - 0.5 brightness should not be '@' anymore
    assert_ne!(output.chars[0][0], '@');
}

#[test]
fn test_contrast_amplifies_differences() {
    let img = create_solid_image(140, 140, 140, 10, 10);
    let low = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 0.5, false, ColorMode::Off);
    let high = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 3.0, false, ColorMode::Off);
    let ramp: Vec<char> = DEFAULT_RAMP.chars().collect();
    let low_idx = ramp.iter().position(|&c| c == low.chars[0][0]).unwrap_or(0);
    let high_idx = ramp
        .iter()
        .position(|&c| c == high.chars[0][0])
        .unwrap_or(0);
    // 140/255 ≈ 0.549 — slightly above 0.5, so high contrast should push up
    assert!(high_idx >= low_idx);
}

#[test]
fn test_custom_ramp_respected() {
    let img = create_white_image(10, 10);
    let output = ascii_engine::convert(&img, "ABC", 10, 0.0, 1.0, false, ColorMode::Off);
    // White = luminance 1.0, should map to last char 'C'
    assert_eq!(output.chars[0][0], 'C');
}

#[test]
fn test_empty_ramp_fallback() {
    let img = create_white_image(10, 10);
    let output = ascii_engine::convert(&img, "", 10, 0.0, 1.0, false, ColorMode::Off);
    // Should use default ramp, white -> '@'
    assert_eq!(output.chars[0][0], '@');
}

#[test]
fn test_single_char_ramp() {
    let img = create_gradient_image(20, 10);
    let output = ascii_engine::convert(&img, "X", 20, 0.0, 1.0, false, ColorMode::Off);
    for row in &output.chars {
        for &ch in row {
            assert_eq!(ch, 'X');
        }
    }
}

#[test]
fn test_ansi_color_nearest_black() {
    let img = create_black_image(10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Ansi16);
    let colors = output.colors.as_ref().expect("should have colors");
    assert_eq!(colors[0][0], [0, 0, 0]);
}

#[test]
fn test_ansi_color_nearest_red() {
    let img = create_solid_image(200, 10, 10, 10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Ansi16);
    let colors = output.colors.as_ref().expect("should have colors");
    let c = colors[0][0];
    // Should match dark red [170, 0, 0] or bright red [255, 85, 85]
    assert!(c == [170, 0, 0] || c == [255, 85, 85]);
}

#[test]
fn test_output_dimensions_match() {
    let img = create_solid_image(128, 128, 128, 200, 100);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 60, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.cols, 60);
    assert_eq!(output.chars.len(), output.rows);
    for row in &output.chars {
        assert_eq!(row.len(), output.cols);
    }
}

#[test]
fn test_aspect_ratio_preserved() {
    // A 200x100 image (2:1 ratio) at 100 columns should produce ~25 rows
    // because rows = (h/w) * cols * 0.5 = (100/200) * 100 * 0.5 = 25
    let img = create_solid_image(128, 128, 128, 200, 100);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 100, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.cols, 100);
    assert_eq!(output.rows, 25);
}

#[test]
fn test_full_rgb_stores_original_colors() {
    let img = create_solid_image(42, 128, 200, 10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::FullRgb);
    let colors = output.colors.as_ref().expect("should have colors");
    assert_eq!(colors[0][0], [42, 128, 200]);
}

#[test]
fn test_color_off_no_colors() {
    let img = create_solid_image(42, 128, 200, 10, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 10, 0.0, 1.0, false, ColorMode::Off);
    assert!(output.colors.is_none());
}

#[test]
fn test_long_ramp_no_overflow() {
    let long_ramp: String = (0..200)
        .map(|i| std::char::from_u32(0x20 + (i % 95)).unwrap_or('?'))
        .collect();
    let img = create_gradient_image(100, 10);
    let output = ascii_engine::convert(&img, &long_ramp, 100, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.cols, 100);
    assert!(!output.chars.is_empty());
}

#[test]
fn test_1x1_image() {
    let img = create_white_image(1, 1);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 1, 0.0, 1.0, false, ColorMode::Off);
    assert_eq!(output.cols, 1);
    assert_eq!(output.rows, 1);
    assert_eq!(output.chars.len(), 1);
    assert_eq!(output.chars[0].len(), 1);
}

#[test]
fn test_gradient_produces_varied_output() {
    let img = create_gradient_image(100, 10);
    let output = ascii_engine::convert(&img, DEFAULT_RAMP, 100, 0.0, 1.0, false, ColorMode::Off);
    // First and last chars of first row should differ (black to white)
    assert_ne!(output.chars[0][0], output.chars[0][output.cols - 1]);
}
