use crate::image_loader;
use crate::state::AppState;

/// Main application struct implementing the eframe::App trait.
#[derive(Default)]
pub struct AsciiApp {
    /// Shared application state.
    pub state: AppState,
}

impl AsciiApp {
    /// Opens a file dialog and loads the selected image.
    fn open_file(&mut self, ctx: &egui::Context) {
        let file = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tif"],
            )
            .pick_file();

        if let Some(path) = file {
            match image_loader::load(&path) {
                Ok(img) => {
                    let flattened = if self.state.dark_background {
                        image_loader::flatten_alpha(&img, [0, 0, 0])
                    } else {
                        image_loader::flatten_alpha(&img, [255, 255, 255])
                    };

                    // Create texture for preview
                    let rgba = flattened.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.as_flat_samples();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let texture =
                        ctx.load_texture("source_image", color_image, egui::TextureOptions::LINEAR);
                    self.state.texture_handle = Some(texture);

                    self.state.image_path = Some(path);
                    self.state.source_image = Some(flattened);
                    self.state.dirty = true;
                    self.state.last_error = None;
                    self.state.status_message = "Image loaded successfully".to_string();
                }
                Err(e) => {
                    self.state.last_error = Some(format!("Failed to load image: {e}"));
                    self.state.source_image = None;
                    self.state.texture_handle = None;
                    self.state.image_path = None;
                }
            }
        }
    }

    /// Renders the toolbar at the top.
    fn render_toolbar(&mut self, ctx: &egui::Context) {
        let mut open_clicked = false;
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load Image").clicked() {
                    open_clicked = true;
                }

                // Keyboard shortcut: Ctrl+O
                if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
                    open_clicked = true;
                }

                if let Some(path) = &self.state.image_path {
                    if let Some(name) = path.file_name() {
                        ui.separator();
                        ui.label(name.to_string_lossy().as_ref());
                    }
                }
            });
        });

        if open_clicked {
            self.open_file(ctx);
        }
    }

    /// Renders the left panel with the original image.
    fn render_image_panel(&self, ctx: &egui::Context) {
        egui::SidePanel::left("original")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Original Image");
                ui.separator();

                // Show error if any
                if let Some(err) = &self.state.last_error {
                    ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
                    return;
                }

                // Show image if loaded
                if let Some(texture) = &self.state.texture_handle {
                    let available = ui.available_size();
                    let tex_size = texture.size_vec2();

                    // Scale to fit panel while preserving aspect ratio
                    let scale = (available.x / tex_size.x).min(available.y / tex_size.y);
                    let display_size = egui::vec2(tex_size.x * scale, tex_size.y * scale);

                    ui.image(egui::load::SizedTexture::new(texture.id(), display_size));

                    // Show image info
                    if let Some(ref img) = self.state.source_image {
                        let filename = self
                            .state
                            .image_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        ui.separator();
                        ui.label(format!("{} — {}×{}", filename, img.width(), img.height()));
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("No image loaded")
                                .size(16.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                }
            });
    }
}

impl eframe::App for AsciiApp {
    /// Called each frame to render the UI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_toolbar(ctx);
        self.render_image_panel(ctx);

        // Central panel: placeholder for ASCII output (Phase 3+)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("ASCII output will appear here")
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    }
}
