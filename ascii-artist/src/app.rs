use crate::state::AppState;

/// Main application struct implementing the eframe::App trait.
#[derive(Default)]
#[allow(dead_code)]
pub struct AsciiApp {
    /// Shared application state.
    pub state: AppState,
}

impl eframe::App for AsciiApp {
    /// Called each frame to render the UI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Load an image to begin")
                        .size(20.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    }
}
