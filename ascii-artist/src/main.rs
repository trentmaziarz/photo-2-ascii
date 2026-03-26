#![windows_subsystem = "windows"]

mod app;
mod ascii_engine;
mod controls;
mod export;
mod image_loader;
mod preview;
mod state;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("ASCII Artist"),
        ..Default::default()
    };
    eframe::run_native(
        "ASCII Artist",
        options,
        Box::new(|_cc| Ok(Box::new(app::AsciiApp::default()))),
    )
}
