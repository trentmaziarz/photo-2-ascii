use crate::state::{AppState, ColorMode};

/// Renders the controls side panel. Returns true if any setting changed
/// that requires reconversion.
pub fn show(ui: &mut egui::Ui, state: &mut AppState) -> bool {
    let mut changed = false;

    ui.heading("Controls");
    ui.separator();

    // Character Ramp
    ui.label("Character Ramp (sparse → dense):")
        .on_hover_text("Characters used to represent brightness levels, from lightest to darkest. Supports Unicode (e.g., ░▒▓█).");
    if ui.text_edit_singleline(&mut state.char_ramp).changed() {
        changed = true;
    }
    // Show fallback warning if ramp is empty
    if state.char_ramp.is_empty() {
        state.ramp_fallback_active = true;
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 80),
            "Using default ramp (empty ramp provided)",
        );
    } else {
        state.ramp_fallback_active = false;
    }

    ui.add_space(12.0);

    // Brightness: -1.0 to 1.0
    ui.label("Brightness")
        .on_hover_text("Shifts overall lightness. Positive = brighter, negative = darker.");
    if ui
        .add(egui::Slider::new(&mut state.brightness, -1.0..=1.0).step_by(0.01))
        .changed()
    {
        changed = true;
    }

    // Contrast: 0.1 to 3.0
    ui.label("Contrast")
        .on_hover_text("Amplifies differences between light and dark areas. 1.0 = no change.");
    if ui
        .add(egui::Slider::new(&mut state.contrast, 0.1..=3.0).step_by(0.01))
        .changed()
    {
        changed = true;
    }

    // Invert
    if ui
        .checkbox(&mut state.invert, "Invert")
        .on_hover_text(
            "Swap light and dark character mapping. Useful when switching background color.",
        )
        .changed()
    {
        changed = true;
    }

    ui.add_space(12.0);

    // Font Size: 4.0 to 24.0
    ui.label("Font Size")
        .on_hover_text("Size of characters in the preview. Smaller = more detail visible.");
    if ui
        .add(egui::Slider::new(&mut state.font_size, 4.0..=24.0).step_by(0.5))
        .changed()
    {
        // Font size changes affect auto-fit column count, so mark dirty
        if state.auto_fit_columns {
            changed = true;
        }
    }

    // Color Mode
    let mode_text = match state.color_mode {
        ColorMode::Off => "Off",
        ColorMode::FullRgb => "Full RGB",
        ColorMode::Ansi16 => "ANSI 16",
    };
    egui::ComboBox::from_label("Color Mode")
        .selected_text(mode_text)
        .show_ui(ui, |ui| {
            if ui
                .selectable_value(&mut state.color_mode, ColorMode::Off, "Off")
                .on_hover_text("Grayscale characters only")
                .changed()
            {
                changed = true;
            }
            if ui
                .selectable_value(&mut state.color_mode, ColorMode::FullRgb, "Full RGB")
                .on_hover_text("Each character colored with the original pixel color")
                .changed()
            {
                changed = true;
            }
            if ui
                .selectable_value(&mut state.color_mode, ColorMode::Ansi16, "ANSI 16")
                .on_hover_text("Colors mapped to the classic 16-color ANSI palette")
                .changed()
            {
                changed = true;
            }
        });

    // Background
    if ui
        .checkbox(&mut state.dark_background, "Dark Background")
        .on_hover_text("Toggle between dark and light preview background.")
        .changed()
    {
        // Background change doesn't require reconversion, just redraw
    }

    ui.add_space(12.0);

    // Resolution
    if ui
        .checkbox(&mut state.auto_fit_columns, "Auto-fit to panel")
        .on_hover_text("Automatically adjust column count to fill the preview panel.")
        .changed()
    {
        changed = true;
    }
    if !state.auto_fit_columns {
        let mut cols = state.output_columns as i32;
        if ui
            .add(egui::Slider::new(&mut cols, 20..=300).text("columns"))
            .on_hover_text("Number of characters per row in the ASCII output.")
            .changed()
        {
            state.output_columns = cols as usize;
            changed = true;
        }
    } else {
        ui.label(format!("Columns: {}", state.output_columns));
    }

    ui.add_space(12.0);

    // Export scale
    ui.label("PNG Export Scale").on_hover_text(
        "Multiplier for PNG export resolution. 4× produces a 4× larger, sharper image.",
    );
    ui.add(egui::Slider::new(&mut state.export_scale, 1.0..=4.0).step_by(0.5));

    ui.add_space(16.0);

    // Reset button
    if ui
        .button("Reset to Defaults")
        .on_hover_text("Reset all settings to defaults without reloading the image.")
        .clicked()
    {
        let defaults = AppState::default();
        state.char_ramp = defaults.char_ramp;
        state.brightness = defaults.brightness;
        state.contrast = defaults.contrast;
        state.invert = defaults.invert;
        state.font_size = defaults.font_size;
        state.color_mode = defaults.color_mode;
        state.dark_background = defaults.dark_background;
        state.auto_fit_columns = defaults.auto_fit_columns;
        state.export_scale = defaults.export_scale;
        state.ramp_fallback_active = false;
        changed = true;
    }

    ui.add_space(16.0);
    ui.separator();
    ui.weak("Ctrl+O  Open image");
    ui.weak("Ctrl+C  Copy to clipboard");

    changed
}
