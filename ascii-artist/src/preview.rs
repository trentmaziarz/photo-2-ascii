use crate::state::{AppState, AsciiOutput, ColorMode};
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

/// Renders the ASCII output in the preview panel.
pub fn show(ui: &mut egui::Ui, output: &AsciiOutput, state: &AppState) {
    let font_id = FontId::monospace(state.font_size);
    let default_color = if state.dark_background {
        Color32::from_gray(200)
    } else {
        Color32::from_gray(30)
    };

    egui::ScrollArea::both().show(ui, |ui| {
        for (row_idx, row) in output.chars.iter().enumerate() {
            let line: String = row.iter().collect();

            match (&state.color_mode, &output.colors) {
                (ColorMode::Off, _) | (_, None) => {
                    // Monochrome: single-color text
                    ui.label(
                        egui::RichText::new(&line)
                            .font(font_id.clone())
                            .color(default_color),
                    );
                }
                (_, Some(colors)) => {
                    // Colored: per-character LayoutJob
                    let mut job = LayoutJob::default();
                    for (col_idx, ch) in row.iter().enumerate() {
                        let [r, g, b] = colors[row_idx][col_idx];
                        let color = Color32::from_rgb(r, g, b);
                        let format = TextFormat {
                            font_id: font_id.clone(),
                            color,
                            ..Default::default()
                        };
                        let s = ch.to_string();
                        job.append(&s, 0.0, format);
                    }
                    ui.label(job);
                }
            }
        }
    });
}
