use ascii_artist::ascii_engine;
use ascii_artist::export;
use ascii_artist::image_loader;
use ascii_artist::state::{AppState, ColorMode};
use image::{DynamicImage, Rgba, RgbaImage};

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

#[test]
fn test_full_pipeline_produces_output() {
    // Simulate: load -> flatten alpha -> convert -> export text
    let img = create_gradient_image(200, 100);
    let flattened = image_loader::flatten_alpha(&img, [0, 0, 0]);

    let state = AppState::default();
    let output = ascii_engine::convert(
        &flattened,
        &state.char_ramp,
        80,
        state.brightness,
        state.contrast,
        state.invert,
        state.color_mode,
    );

    assert!(output.cols > 0);
    assert!(output.rows > 0);

    let text = export::to_text(&output);
    assert!(!text.is_empty());
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), output.rows);
}

#[test]
fn test_settings_change_produces_different_output() {
    let img = create_gradient_image(100, 50);

    let output_normal =
        ascii_engine::convert(&img, " .:-=+*#%@", 80, 0.0, 1.0, false, ColorMode::Off);
    let output_inverted =
        ascii_engine::convert(&img, " .:-=+*#%@", 80, 0.0, 1.0, true, ColorMode::Off);
    let output_bright =
        ascii_engine::convert(&img, " .:-=+*#%@", 80, 0.5, 1.0, false, ColorMode::Off);
    let output_high_contrast =
        ascii_engine::convert(&img, " .:-=+*#%@", 80, 0.0, 3.0, false, ColorMode::Off);

    let text_normal = export::to_text(&output_normal);
    let text_inverted = export::to_text(&output_inverted);
    let text_bright = export::to_text(&output_bright);
    let text_high_contrast = export::to_text(&output_high_contrast);

    assert_ne!(text_normal, text_inverted);
    assert_ne!(text_normal, text_bright);
    assert_ne!(text_normal, text_high_contrast);
}

#[test]
fn test_reload_image_updates_output() {
    let img1 = create_solid_image(0, 0, 0, 100, 50);
    let img2 = create_solid_image(255, 255, 255, 100, 50);

    let output1 = ascii_engine::convert(&img1, " .:-=+*#%@", 40, 0.0, 1.0, false, ColorMode::Off);
    let output2 = ascii_engine::convert(&img2, " .:-=+*#%@", 40, 0.0, 1.0, false, ColorMode::Off);

    let text1 = export::to_text(&output1);
    let text2 = export::to_text(&output2);

    // Black image should be all spaces, white should be all @
    assert_ne!(text1, text2);
    assert!(text1.chars().all(|c| c == ' ' || c == '\n'));
    assert!(text2.chars().all(|c| c == '@' || c == '\n'));
}

#[test]
fn test_color_mode_changes_preserve_chars() {
    let img = create_gradient_image(100, 50);

    let output_off = ascii_engine::convert(&img, " .:-=+*#%@", 60, 0.0, 1.0, false, ColorMode::Off);
    let output_rgb =
        ascii_engine::convert(&img, " .:-=+*#%@", 60, 0.0, 1.0, false, ColorMode::FullRgb);
    let output_ansi =
        ascii_engine::convert(&img, " .:-=+*#%@", 60, 0.0, 1.0, false, ColorMode::Ansi16);

    // Character grids should be identical regardless of color mode
    assert_eq!(output_off.chars, output_rgb.chars);
    assert_eq!(output_off.chars, output_ansi.chars);

    // Color data should differ
    assert!(output_off.colors.is_none());
    assert!(output_rgb.colors.is_some());
    assert!(output_ansi.colors.is_some());
}

#[test]
fn test_flatten_alpha_removes_transparency() {
    // Create a semi-transparent red image
    let mut img = RgbaImage::new(10, 10);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 0, 0, 128]); // 50% transparent red
    }
    let dynamic = DynamicImage::ImageRgba8(img);

    // Flatten onto white background
    let flattened = image_loader::flatten_alpha(&dynamic, [255, 255, 255]);
    let rgba = flattened.to_rgba8();
    let pixel = rgba.get_pixel(0, 0);

    // Should be a blend of red and white, fully opaque
    assert_eq!(pixel[3], 255);
    // Red channel: 255 * 0.502 + 255 * 0.498 ≈ 255 (both contribute red)
    // Green channel: 0 * 0.502 + 255 * 0.498 ≈ 127
    assert!(pixel[1] > 100 && pixel[1] < 160); // green component from white bg
}

#[test]
fn test_png_export_full_pipeline() {
    let img = create_gradient_image(100, 50);
    let flattened = image_loader::flatten_alpha(&img, [0, 0, 0]);
    let state = AppState::default();
    let output = ascii_engine::convert(
        &flattened,
        &state.char_ramp,
        40,
        state.brightness,
        state.contrast,
        state.invert,
        state.color_mode,
    );

    let png = export::render_png(&output, &state, 1.0);
    assert!(png.is_ok());
    let png = png.unwrap();
    assert!(png.width() > 0);
    assert!(png.height() > 0);
}

#[test]
fn test_resize_for_ascii_dimensions() {
    let img = create_solid_image(128, 128, 128, 1920, 1080);
    let (resized, cols, rows) = image_loader::resize_for_ascii(&img, 120);
    assert_eq!(cols, 120);
    assert_eq!(resized.width(), 120);
    assert_eq!(resized.height() as usize, rows);
    // Aspect ratio check: 1080/1920 * 120 * 0.5 ≈ 34
    assert!(rows > 25 && rows < 45);
}
