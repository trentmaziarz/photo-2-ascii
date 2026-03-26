use ascii_artist::ascii_engine;
use ascii_artist::export;
use ascii_artist::state::AppState;
use image::{DynamicImage, Rgba, RgbaImage};

fn create_solid_image(r: u8, g: u8, b: u8, w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([r, g, b, 255])))
}

fn make_output_and_state() -> (ascii_artist::state::AsciiOutput, AppState) {
    let img = create_solid_image(128, 128, 128, 40, 20);
    let state = AppState::default();
    let output = ascii_engine::convert(
        &img,
        &state.char_ramp,
        20,
        state.brightness,
        state.contrast,
        state.invert,
        state.color_mode,
    );
    (output, state)
}

#[test]
fn test_to_text_matches_chars() {
    let (output, _state) = make_output_and_state();
    let text = export::to_text(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), output.rows);
    for (line, char_row) in lines.iter().zip(output.chars.iter()) {
        let expected: String = char_row.iter().collect();
        assert_eq!(*line, expected);
    }
}

#[test]
fn test_txt_trailing_newline() {
    let (output, _state) = make_output_and_state();
    let text = export::to_text(&output) + "\n";
    assert!(text.ends_with('\n'));
}

#[test]
fn test_txt_save_and_read() {
    let (output, _state) = make_output_and_state();
    let dir = std::env::temp_dir();
    let path = dir.join("ascii_artist_test_output.txt");
    export::save_txt(&output, &path).expect("should save txt");
    let contents = std::fs::read_to_string(&path).expect("should read back");
    let expected = export::to_text(&output) + "\n";
    assert_eq!(contents, expected);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_png_dimensions_at_1x() {
    let (output, state) = make_output_and_state();
    let img = export::render_png(&output, &state, 1.0).expect("should render PNG");

    let font_size = state.font_size * 1.0;
    let expected_width = output.cols as u32 * (font_size * 0.6).ceil() as u32;
    let expected_height = output.rows as u32 * (font_size * 1.2).ceil() as u32;

    assert_eq!(img.width(), expected_width);
    assert_eq!(img.height(), expected_height);
}

#[test]
fn test_png_dimensions_at_4x() {
    let (output, state) = make_output_and_state();
    let img_1x = export::render_png(&output, &state, 1.0).expect("should render 1x");
    let img_4x = export::render_png(&output, &state, 4.0).expect("should render 4x");

    // 4x should be roughly 4x larger in each dimension
    assert!(img_4x.width() > img_1x.width() * 3);
    assert!(img_4x.height() > img_1x.height() * 3);
}

#[test]
fn test_png_background_dark() {
    let (output, mut state) = make_output_and_state();
    state.dark_background = true;
    let img = export::render_png(&output, &state, 1.0).expect("should render PNG");
    // Check a corner pixel is dark background (26, 26, 26, 255)
    let pixel = img.get_pixel(0, 0);
    assert_eq!(pixel[0], 26);
    assert_eq!(pixel[1], 26);
    assert_eq!(pixel[2], 26);
}

#[test]
fn test_png_background_light() {
    let (output, mut state) = make_output_and_state();
    state.dark_background = false;
    let img = export::render_png(&output, &state, 1.0).expect("should render PNG");
    // Check a corner pixel is light background (240, 240, 240, 255)
    let pixel = img.get_pixel(0, 0);
    assert_eq!(pixel[0], 240);
    assert_eq!(pixel[1], 240);
    assert_eq!(pixel[2], 240);
}

#[test]
fn test_png_empty_output_returns_error() {
    let output = ascii_artist::state::AsciiOutput {
        chars: Vec::new(),
        colors: None,
        cols: 0,
        rows: 0,
    };
    let state = AppState::default();
    let result = export::render_png(&output, &state, 1.0);
    assert!(result.is_err());
}
