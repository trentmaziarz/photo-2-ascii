use crate::state::{AppState, AsciiOutput, ColorMode};
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

/// Builds cached LayoutJobs from the ASCII output and current display settings.
/// Each row becomes one LayoutJob. Consecutive characters with the same color
/// are batched into a single `append()` call for performance.
pub fn build_layout_jobs(output: &AsciiOutput, state: &AppState) -> Vec<LayoutJob> {
    let font_id = FontId::monospace(state.font_size);
    let default_color = if state.dark_background {
        Color32::from_gray(200)
    } else {
        Color32::from_gray(30)
    };

    let use_color = state.color_mode != ColorMode::Off && output.colors.is_some();

    output
        .chars
        .iter()
        .enumerate()
        .map(|(row_idx, row)| {
            let mut job = LayoutJob::default();

            if !use_color {
                // Monochrome: entire row in one append
                let line: String = row.iter().collect();
                let format = TextFormat {
                    font_id: font_id.clone(),
                    color: default_color,
                    ..Default::default()
                };
                job.append(&line, 0.0, format);
            } else if let Some(colors) = &output.colors {
                // Colored: batch consecutive same-color characters
                let color_row = &colors[row_idx];
                let mut batch_start = 0;

                while batch_start < row.len() {
                    let batch_color = color_row[batch_start];
                    let mut batch_end = batch_start + 1;

                    // Extend batch while color matches
                    while batch_end < row.len() && color_row[batch_end] == batch_color {
                        batch_end += 1;
                    }

                    let text: String = row[batch_start..batch_end].iter().collect();
                    let [r, g, b] = batch_color;
                    let color = Color32::from_rgb(r, g, b);
                    let format = TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    };
                    job.append(&text, 0.0, format);

                    batch_start = batch_end;
                }
            }

            job
        })
        .collect()
}

/// Renders the ASCII output in the preview panel using cached LayoutJobs.
pub fn show(ui: &mut egui::Ui, state: &AppState) {
    egui::ScrollArea::both().show(ui, |ui| {
        for job in &state.cached_layout_jobs {
            ui.label(job.clone());
        }
    });
}
