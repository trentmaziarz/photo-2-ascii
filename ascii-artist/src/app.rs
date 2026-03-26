use crate::ascii_engine;
use crate::controls;
use crate::image_loader;
use crate::preview;
use crate::state::{AppState, ColorMode};

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
                    self.state.layout_dirty = true;
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

    /// Runs the ASCII conversion if the state is dirty and an image is loaded.
    fn maybe_reconvert(&mut self) {
        if !self.state.dirty {
            return;
        }
        let Some(ref image) = self.state.source_image else {
            return;
        };

        let start = std::time::Instant::now();
        let output = ascii_engine::convert(
            image,
            &self.state.char_ramp,
            self.state.output_columns,
            self.state.brightness,
            self.state.contrast,
            self.state.invert,
            self.state.color_mode,
        );
        self.state.conversion_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        let mode_label = match self.state.color_mode {
            ColorMode::Off => "Grayscale",
            ColorMode::FullRgb => "Full RGB",
            ColorMode::Ansi16 => "ANSI 16",
        };
        self.state.status_message = format!(
            "{} cols × {} rows | {:.1}ms | {}",
            output.cols, output.rows, self.state.conversion_time_ms, mode_label
        );
        self.state.cached_output = Some(output);
        self.state.dirty = false;
        // Engine output changed, so layout jobs must be rebuilt
        self.state.layout_dirty = true;
    }

    /// Rebuilds cached LayoutJobs if display settings changed.
    fn maybe_rebuild_layout(&mut self) {
        // Check if display settings changed without engine reconversion
        let display_changed = self.state.font_size != self.state.cached_layout_font_size
            || self.state.dark_background != self.state.cached_layout_dark_bg
            || self.state.color_mode != self.state.cached_layout_color_mode;

        if !self.state.layout_dirty && !display_changed {
            return;
        }

        if let Some(ref output) = self.state.cached_output {
            self.state.cached_layout_jobs = preview::build_layout_jobs(output, &self.state);
            self.state.cached_layout_font_size = self.state.font_size;
            self.state.cached_layout_dark_bg = self.state.dark_background;
            self.state.cached_layout_color_mode = self.state.color_mode;
            self.state.layout_dirty = false;
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

                ui.separator();

                ui.heading("ASCII Artist");

                if let Some(path) = &self.state.image_path {
                    if let Some(name) = path.file_name() {
                        ui.separator();
                        ui.label(name.to_string_lossy().as_ref());
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export PNG").clicked() {
                        self.state.status_message = "Export PNG: not yet implemented".to_string();
                    }
                    if ui.button("Save .txt").clicked() {
                        self.state.status_message = "Save .txt: not yet implemented".to_string();
                    }
                    if ui.button("Copy to Clipboard").clicked() {
                        self.state.status_message =
                            "Copy to Clipboard: not yet implemented".to_string();
                    }
                });
            });
        });

        if open_clicked {
            self.open_file(ctx);
        }
    }

    /// Renders the status bar at the bottom.
    fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.state.source_image.is_some() {
                    ui.label(&self.state.status_message);
                } else {
                    ui.label("Load an image to begin");
                }

                if let Some(ref err) = self.state.last_error {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
                }
            });
        });
    }

    /// Renders the right panel with controls.
    fn render_controls_panel(&mut self, ctx: &egui::Context) {
        let mut controls_changed = false;
        egui::SidePanel::right("controls")
            .default_width(240.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    controls_changed = controls::show(ui, &mut self.state);
                });
            });
        if controls_changed {
            self.state.dirty = true;
        }
    }

    /// Renders the left panel with the original image.
    fn render_image_panel(&self, ctx: &egui::Context) {
        egui::SidePanel::left("original")
            .default_width(300.0)
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
                    let scale = scale.min(1.0); // Don't upscale beyond original
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

    /// Renders the central panel with the ASCII preview.
    fn render_preview_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Paint background
            let bg_color = if self.state.dark_background {
                egui::Color32::from_gray(20)
            } else {
                egui::Color32::from_gray(240)
            };
            ui.painter()
                .rect_filled(ui.available_rect_before_wrap(), 0.0, bg_color);

            // Auto-fit column calculation
            if self.state.auto_fit_columns && self.state.source_image.is_some() {
                let char_width = self.state.font_size * 0.6;
                let panel_width = ui.available_width() - 16.0;
                let new_cols = (panel_width / char_width).floor() as usize;
                if new_cols != self.state.output_columns && new_cols > 0 {
                    self.state.output_columns = new_cols.max(20);
                    self.state.dirty = true;
                }
            }

            // Reconvert if auto-fit changed columns, then rebuild layout
            self.maybe_reconvert();
            self.maybe_rebuild_layout();

            if self.state.cached_output.is_some() {
                preview::show(ui, &self.state);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Load an image to begin")
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
        // Run conversion before rendering panels
        self.maybe_reconvert();

        // Panel order: top → bottom → right side → left side → central (must be last)
        self.render_toolbar(ctx);
        self.render_status_bar(ctx);
        self.render_controls_panel(ctx);
        self.render_image_panel(ctx);
        self.render_preview_panel(ctx);
    }
}
