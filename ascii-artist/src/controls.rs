use crate::state::{AppState, ColorMode};

/// Renders the controls side panel. Returns true if any setting changed
/// that requires reconversion.
pub fn show(ui: &mut egui::Ui, state: &mut AppState) -> bool {
    let mut changed = false;

    ui.heading("Controls");
    ui.separator();

    // Character Ramp
    ui.label("Character Ramp (sparse → dense):");
    if ui.text_edit_singleline(&mut state.char_ramp).changed() {
        changed = true;
    }

    ui.add_space(12.0);

    // Brightness: -1.0 to 1.0
    ui.label("Brightness");
    if ui
        .add(egui::Slider::new(&mut state.brightness, -1.0..=1.0).step_by(0.01))
        .changed()
    {
        changed = true;
    }

    // Contrast: 0.1 to 3.0
    ui.label("Contrast");
    if ui
        .add(egui::Slider::new(&mut state.contrast, 0.1..=3.0).step_by(0.01))
        .changed()
    {
        changed = true;
    }

    // Invert
    if ui.checkbox(&mut state.invert, "Invert").changed() {
        changed = true;
    }

    ui.add_space(12.0);

    // Font Size: 4.0 to 24.0
    ui.label("Font Size");
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
                .changed()
            {
                changed = true;
            }
            if ui
                .selectable_value(&mut state.color_mode, ColorMode::FullRgb, "Full RGB")
                .changed()
            {
                changed = true;
            }
            if ui
                .selectable_value(&mut state.color_mode, ColorMode::Ansi16, "ANSI 16")
                .changed()
            {
                changed = true;
            }
        });

    // Background
    if ui
        .checkbox(&mut state.dark_background, "Dark Background")
        .changed()
    {
        // Background change doesn't require reconversion, just redraw
    }

    ui.add_space(12.0);

    // Resolution
    if ui
        .checkbox(&mut state.auto_fit_columns, "Auto-fit to panel")
        .changed()
    {
        changed = true;
    }
    if !state.auto_fit_columns {
        let mut cols = state.output_columns as i32;
        if ui
            .add(egui::Slider::new(&mut cols, 20..=300).text("columns"))
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
    ui.label("PNG Export Scale");
    ui.add(egui::Slider::new(&mut state.export_scale, 1.0..=4.0).step_by(0.5));

    ui.add_space(16.0);

    // Reset button
    if ui.button("Reset to Defaults").clicked() {
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
        changed = true;
    }

    changed
}
