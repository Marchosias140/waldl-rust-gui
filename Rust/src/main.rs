use eframe::egui;
use reqwest::blocking;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use sha1::{Sha1, Digest};

#[derive(Deserialize)]
struct ApiResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize, Clone)]
struct ImageData {
    path: String,
    thumbs: Thumbs,
}

#[derive(Deserialize, Clone)]
struct Thumbs {
    small: String,
}

struct WallhavenApp {
    query: String,
    results: Vec<ImageData>,
    textures: HashMap<String, egui::TextureHandle>,
    download_dir: PathBuf,
    status: String,
}

impl Default for WallhavenApp {
    fn default() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            textures: HashMap::new(),
            download_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            status: String::new(),
        }
    }
}

impl eframe::App for WallhavenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("wldl (Grid Layout)");

            ui.horizontal(|ui| {
                ui.label("Enter search query:");
                ui.text_edit_singleline(&mut self.query);
                if ui.button("Search").clicked() {
                    self.search();
                }
            });

            ui.label(format!("Download directory: {}", self.download_dir.display()));
            if !self.status.is_empty() {
                ui.label(&self.status);
            }

            let mut to_download: Option<String> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Create a grid with 4 columns
                egui::Grid::new("thumb_grid")
                    .num_columns(4)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        for (i, img) in self.results.iter().enumerate() {
                            if let Some(tex) = self.textures.get(&img.thumbs.small) {
                                if ui
                                    .add_sized([150.0, 100.0], egui::ImageButton::new(tex))
                                    .clicked()
                                {
                                    to_download = Some(img.path.clone());
                                }
                            } else {
                                if let Ok(bytes) =
                                    blocking::get(&img.thumbs.small).and_then(|r| r.bytes())
                                {
                                    if let Ok(dynamic) = image::load_from_memory(&bytes) {
                                        let size =
                                            [dynamic.width() as usize, dynamic.height() as usize];
                                        let rgba = dynamic.to_rgba8();
                                        let color_image =
                                            egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                                        let tex = ctx.load_texture(
                                            &img.thumbs.small,
                                            color_image,
                                            egui::TextureOptions::default(),
                                        );
                                        self.textures.insert(img.thumbs.small.clone(), tex);
                                    }
                                }
                            }

                            // After every 4 thumbnails, end the row
                            if (i + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });

            if let Some(url) = to_download {
                self.download(&url);
            }
        });
    }
}

impl WallhavenApp {
    fn search(&mut self) {
        let url = format!(
            "https://wallhaven.cc/api/v1/search?q={}&sorting=relevance",
            self.query
        );
        if let Ok(resp) = blocking::get(&url) {
            if let Ok(api) = resp.json::<ApiResponse>() {
                self.results = api.data;
                self.textures.clear();
                self.status = format!("Found {} results", self.results.len());
            }
        }
    }

    fn download(&mut self, url: &str) {
        match blocking::get(url) {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(bytes) = resp.bytes() {
                    let mut hasher = Sha1::new();
                    hasher.update(url.as_bytes());
                    let hash = hasher.finalize();
                    let filename = format!("{:x}.jpg", hash);
                    let save_path = self.download_dir.join(filename);

                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let resized = img.resize_exact(
                            3840,
                            2160,
                            image::imageops::FilterType::Lanczos3,
                        );
                        if let Ok(mut file) = File::create(&save_path) {
                            if resized
                                .write_to(&mut file, image::ImageFormat::Jpeg)
                                .is_ok()
                            {
                                self.status =
                                    format!("Saved wallpaper to {}", save_path.display());
                            }
                        }
                    }
                }
            }
            _ => {
                self.status = "Failed to download wallpaper".to_string();
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "wldl",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(WallhavenApp::default()) as Box<dyn eframe::App>)
        }),
    )
}
