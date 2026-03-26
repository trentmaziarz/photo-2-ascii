use crate::state::{AppState, AsciiOutput, ColorMode};
use ab_glyph::{FontArc, PxScale};
use image::RgbaImage;
use imageproc::drawing::draw_text_mut;
use std::path::Path;

/// Embedded monospace font for PNG export.
const FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

/// Converts AsciiOutput to a plain text string (one row per line).
pub fn to_text(output: &AsciiOutput) -> String {
    output
        .chars
        .iter()
        .map(|row| row.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Copies ASCII text to the system clipboard.
pub fn copy_to_clipboard(output: &AsciiOutput) -> Result<(), String> {
    let text = to_text(output);
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Could not access clipboard: {e}"))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Could not copy to clipboard: {e}"))
}

/// Saves ASCII text to a .txt file.
pub fn save_txt(output: &AsciiOutput, path: &Path) -> Result<(), String> {
    let text = to_text(output) + "\n";
    std::fs::write(path, text).map_err(|e| format!("Could not save file: {e}"))
}

/// Renders ASCII art to a PNG image buffer.
pub fn render_png(output: &AsciiOutput, state: &AppState, scale: f32) -> Result<RgbaImage, String> {
    let font =
        FontArc::try_from_slice(FONT_BYTES).map_err(|e| format!("Could not load font: {e}"))?;

    let font_size = state.font_size * scale;
    let char_width = (font_size * 0.6).ceil() as u32;
    let char_height = (font_size * 1.2).ceil() as u32;

    if output.cols == 0 || output.rows == 0 {
        return Err("No ASCII output to render".to_string());
    }

    let img_width = output.cols as u32 * char_width;
    let img_height = output.rows as u32 * char_height;

    // Guard against absurdly large images
    let pixel_count = img_width as u64 * img_height as u64;
    if pixel_count > 100_000_000 {
        return Err(format!(
            "Output image too large: {}×{} ({:.0}M pixels). Reduce columns or export scale.",
            img_width,
            img_height,
            pixel_count as f64 / 1_000_000.0
        ));
    }

    // Fill background
    let bg = if state.dark_background {
        image::Rgba([26, 26, 26, 255])
    } else {
        image::Rgba([240, 240, 240, 255])
    };
    let mut img = RgbaImage::from_pixel(img_width, img_height, bg);

    let default_color = if state.dark_background {
        image::Rgba([200, 200, 200, 255])
    } else {
        image::Rgba([30, 30, 30, 255])
    };

    let px_scale = PxScale::from(font_size);

    for (row_idx, row) in output.chars.iter().enumerate() {
        let y = row_idx as i32 * char_height as i32;

        for (col_idx, ch) in row.iter().enumerate() {
            let x = col_idx as i32 * char_width as i32;

            let color = match (&state.color_mode, &output.colors) {
                (ColorMode::Off, _) | (_, None) => default_color,
                (_, Some(colors)) => {
                    let [r, g, b] = colors[row_idx][col_idx];
                    image::Rgba([r, g, b, 255])
                }
            };

            let s = ch.to_string();
            draw_text_mut(&mut img, color, x, y, px_scale, &font, &s);
        }
    }

    Ok(img)
}

/// Saves the rendered PNG to disk.
pub fn save_png(output: &AsciiOutput, state: &AppState, path: &Path) -> Result<(), String> {
    let img = render_png(output, state, state.export_scale)?;
    img.save(path)
        .map_err(|e| format!("Could not save PNG: {e}"))
}
