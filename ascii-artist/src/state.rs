use image::DynamicImage;
use std::path::PathBuf;

/// Available color modes for ASCII output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    /// No color — pure grayscale character mapping.
    Off,
    /// Full RGB color per character.
    FullRgb,
    /// Classic 16-color ANSI palette.
    Ansi16,
}

/// The result of converting an image to ASCII art.
pub struct AsciiOutput {
    /// 2D grid of ASCII characters (row-major).
    pub chars: Vec<Vec<char>>,
    /// Optional per-character RGB colors, matching `chars` dimensions.
    pub colors: Option<Vec<Vec<[u8; 3]>>>,
    /// Number of columns in the output.
    pub cols: usize,
    /// Number of rows in the output.
    pub rows: usize,
}

/// Shared application state holding all settings and cached data.
pub struct AppState {
    // Image
    /// The currently loaded source image.
    pub source_image: Option<DynamicImage>,
    /// File path of the loaded image.
    pub image_path: Option<PathBuf>,
    /// Cached egui texture handle for the source image preview.
    pub texture_handle: Option<egui::TextureHandle>,

    // ASCII conversion
    /// Character ramp from lightest to darkest.
    pub char_ramp: String,
    /// Number of columns in the ASCII output.
    pub output_columns: usize,
    /// Whether to auto-fit columns to the available panel width.
    pub auto_fit_columns: bool,
    /// Brightness adjustment (-1.0 to 1.0).
    pub brightness: f32,
    /// Contrast multiplier (0.1 to 3.0).
    pub contrast: f32,
    /// Whether to invert the luminance mapping.
    pub invert: bool,

    // Display
    /// Font size for the ASCII preview.
    pub font_size: f32,
    /// Whether the preview background is dark.
    pub dark_background: bool,
    /// Current color mode.
    pub color_mode: ColorMode,

    // Export
    /// Scale factor for PNG export.
    pub export_scale: f32,

    // Internal
    /// Flag indicating the output needs recomputation.
    pub dirty: bool,
    /// Cached ASCII conversion output.
    pub cached_output: Option<AsciiOutput>,
    /// Status bar message.
    pub status_message: String,
    /// Time taken for last conversion in milliseconds.
    pub conversion_time_ms: f64,
    /// Last error message, if any.
    pub last_error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            source_image: None,
            image_path: None,
            texture_handle: None,
            char_ramp: " .:-=+*#%@".to_string(),
            output_columns: 80,
            auto_fit_columns: true,
            brightness: 0.0,
            contrast: 1.0,
            invert: false,
            font_size: 10.0,
            dark_background: true,
            color_mode: ColorMode::Off,
            export_scale: 1.0,
            dirty: true,
            cached_output: None,
            status_message: String::new(),
            conversion_time_ms: 0.0,
            last_error: None,
        }
    }
}
