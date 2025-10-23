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
    meta: Meta,
}

#[derive(Deserialize)]
struct Meta {
    current_page: u32,
    last_page: u32,
    per_page: u32,
    total: u32,
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
    categories: String,         // "111" by default (General + Anime + People)
    purity: String,             // "100" by default (only SFW)
    ratios: Option<String>,     // None = any aspect ratio; Some("16x9"), Some("21x9"), etc.
    max_pages: u32,             // page number limit 
    results: Vec<ImageData>,
    textures: HashMap<String, egui::TextureHandle>,
    download_dir: PathBuf,
    status: String,
}

impl Default for WallhavenApp {
    fn default() -> Self {
        Self {
            query: String::new(),
            categories: "111".to_string(),
            purity: "100".to_string(),
            ratios: None,
            max_pages: 5, // by default loads 5 pages 
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
            ui.heading("waldl (Grid Layout)");

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.query);
                if ui.button("Search").clicked() {
                    self.search();
                }
            });

            // Categories
            ui.horizontal(|ui| {
                ui.label("Categories:");
                if ui.selectable_label(self.categories == "100", "General").clicked() {
                    self.categories = "100".to_string();
                }
                if ui.selectable_label(self.categories == "010", "Anime").clicked() {
                    self.categories = "010".to_string();
                }
                if ui.selectable_label(self.categories == "001", "People").clicked() {
                    self.categories = "001".to_string();
                }
                if ui.selectable_label(self.categories == "111", "All").clicked() {
                    self.categories = "111".to_string();
                }
            });

            // Purity
            ui.horizontal(|ui| {
                ui.label("Purity:");
                if ui.selectable_label(self.purity == "100", "SFW").clicked() {
                    self.purity = "100".to_string();
                }
                if ui.selectable_label(self.purity == "110", "SFW+Sketchy").clicked() {
                    self.purity = "110".to_string();
                }
                if ui.selectable_label(self.purity == "111", "All").clicked() {
                    self.purity = "111".to_string();
                }
            });

            // Aspect ratio
            ui.horizontal(|ui| {
                ui.label("Aspect ratio:");
                if ui.selectable_label(self.ratios.as_deref() == Some("16x9"), "16:9").clicked() {
                    self.ratios = Some("16x9".to_string());
                }
                if ui.selectable_label(self.ratios.as_deref() == Some("21x9"), "21:9").clicked() {
                    self.ratios = Some("21x9".to_string());
                }
                if ui.selectable_label(self.ratios.is_none(), "Any").clicked() {
                    self.ratios = None;
                }
            });

            // Page number limit
            ui.horizontal(|ui| {
                ui.label("Max pages:");
                let mut pages_str = self.max_pages.to_string();
                if ui.text_edit_singleline(&mut pages_str).changed() {
                    if let Ok(val) = pages_str.parse::<u32>() {
                        self.max_pages = val.max(1).min(50); // avoids extremes
                    }
                }
                if ui.button("Apply").clicked() {
                    // does nothing direct, but works with UX 
                }
            });

            ui.label(format!("Download directory: {}", self.download_dir.display()));
            if !self.status.is_empty() {
                ui.label(&self.status);
            }

            let mut to_download: Option<String> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
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
        let mut all_results = Vec::new();

        // Builds the URL according to the desired parameters 
        let build_url = |page: u32, query: &str, categories: &str, purity: &str, ratios: &Option<String>| {
            let mut url = format!(
                "https://wallhaven.cc/api/v1/search?q={}&categories={}&purity={}&sorting=relevance&page={}",
                query, categories, purity, page
            );
            if let Some(r) = ratios {
                url.push_str(&format!("&ratios={}", r));
            }
            url
        };

        // Page 1
        let url = build_url(1, &self.query, &self.categories, &self.purity, &self.ratios);

        if let Ok(resp) = blocking::get(&url) {
            if let Ok(api) = resp.json::<ApiResponse>() {
                let total_pages = api.meta.last_page;
                all_results.extend(api.data);

                // Goes through the rest of the pages, limited by max_pages 
                let max_pages = self.max_pages.min(total_pages);
                for page in 2..=max_pages {
                    let url = build_url(page, &self.query, &self.categories, &self.purity, &self.ratios);
                    if let Ok(resp) = blocking::get(&url) {
                        if let Ok(api) = resp.json::<ApiResponse>() {
                            all_results.extend(api.data);
                        }
                    }
                }

                self.status = format!(
                    "Found {} results (pages loaded: {}/{}, per page: {})",
                    all_results.len(),
                    max_pages,
                    total_pages,
                    api.meta.per_page
                );
            } else {
                self.status = "Failed to parse API response".to_string();
            }
        } else {
            self.status = "Failed to fetch from Wallhaven".to_string();
        }

        self.results = all_results;
        self.textures.clear();
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
    if let Ok(mut file) = File::create(&save_path) {
        if img.write_to(&mut file, image::ImageFormat::Jpeg).is_ok() {
            self.status = format!("Saved wallpaper to {}", save_path.display());
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
        "waldl",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(WallhavenApp::default()) as Box<dyn eframe::App>)
        }),
    )
}


 
                  
                  
                  
                  
                  
