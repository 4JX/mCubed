use ferinth::Ferinth;
use mc_mod_meta::{common::McMod, fabric::FabricManifest, forge::ForgeManifest};
use std::fs;

use eframe::{
    egui::{
        self, style::DebugOptions, Color32, CtxRef, FontData, FontDefinitions, FontFamily,
        RichText, Style, Vec2, Visuals,
    },
    epi,
};

use std::borrow::Cow;

struct UiApp {
    mod_list: Vec<McMod>,
    search_buf: String,
    modrinth: Ferinth,
}

impl Default for UiApp {
    fn default() -> Self {
        Self {
            mod_list: Default::default(),
            search_buf: Default::default(),
            modrinth: Ferinth::new("Test app"),
        }
    }
}
impl epi::App for UiApp {
    fn name(&self) -> &str {
        "An App"
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.scan_mod_folder();
        mod_thing(&self.mod_list, &self.modrinth);
        self.configure_fonts(ctx);
        self.configure_style(ctx);
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::SidePanel::left("options_panel")
            .resizable(false)
            .max_width(180.)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Placeholder");
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame {
                fill: Color32::from_rgb(50, 50, 50),
                margin: Vec2::new(10., 10.),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    let edit = egui::TextEdit::singleline(&mut self.search_buf).hint_text(
                        RichText::new("Search installed mods").color(Color32::from_rgb(40, 40, 40)),
                    );
                    ui.add(edit);
                });
            });
            ui.add_space(5.);

            ui.vertical_centered_justified(|ui| {
                egui::Frame {
                    fill: Color32::from_gray(22),
                    margin: Vec2::new(10., 10.),
                    corner_radius: 4.,
                    ..Default::default()
                }
                .show(ui, |ui| {
                    let filtered_list: Vec<&McMod> = self
                        .mod_list
                        .iter()
                        .filter(|mod_entry| {
                            mod_entry
                                .display_name
                                .to_lowercase()
                                .contains(self.search_buf.as_str().to_lowercase().as_str())
                        })
                        .collect();

                    if self.mod_list.is_empty() {
                        ui.label("There are no mods to display");
                    } else if filtered_list.is_empty() {
                        ui.label("No mods match your search");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing.y = 10.0;

                            for mod_entry in filtered_list {
                                egui::Frame {
                                    fill: Color32::from_rgb(50, 50, 50),
                                    ..Default::default()
                                }
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.set_height(60.);

                                        ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);

                                        let version = mod_entry.normalized_version();
                                        egui::Frame {
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
                                            ui.set_width(130.);

                                            ui.centered_and_justified(|ui| {
                                                ui.style_mut().visuals.window_corner_radius = 20.;
                                                ui.label(&mod_entry.display_name);
                                            });
                                        });

                                        egui::Frame {
                                            fill: Color32::from_rgb(80, 80, 80),
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
                                            ui.set_width(35.);
                                            ui.centered_and_justified(|ui| {
                                                ui.label(version);
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.set_width(60.);

                                            egui::Frame {
                                                ..Default::default()
                                            }
                                            .show(
                                                ui,
                                                |ui| {
                                                    ui.set_height(30.);
                                                    ui.centered_and_justified(|ui| {
                                                        ui.label(mod_entry.modloader.to_string());
                                                    });
                                                },
                                            );

                                            egui::Frame {
                                                ..Default::default()
                                            }
                                            .show(
                                                ui,
                                                |ui| {
                                                    ui.set_height(30.);
                                                    ui.centered_and_justified(|ui| {
                                                        ui.label(mod_entry.modloader.to_string());
                                                    });
                                                },
                                            );
                                        });

                                        egui::Frame {
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
                                            ui.set_width(130.);
                                            ui.centered_and_justified(|ui| {
                                                ui.label(&mod_entry.id);
                                            });
                                        });

                                        egui::Frame {
                                            margin: Vec2::splat(10.),
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
                                            ui.set_min_width(10.);
                                            ui.centered_and_justified(|ui| {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(),
                                                    |ui| {
                                                        ui.label("Icon placeholder");
                                                    },
                                                );
                                            });
                                        });
                                    });
                                });
                            }
                        });
                    }
                });
            });
        });
    }
}

impl UiApp {
    fn scan_mod_folder(&mut self) {
        let dir = fs::canonicalize("./mods/").unwrap();

        let read_dir = fs::read_dir(dir).unwrap();

        for path in read_dir {
            let fname = path.unwrap().path();

            let file = fs::File::open(&fname).unwrap();

            match mc_mod_meta::get_modloader(&file) {
                Ok(modloader) => match modloader {
                    mc_mod_meta::ModLoader::Forge => {
                        let forge_meta = ForgeManifest::from_file(file).unwrap();
                        for mod_meta in forge_meta.mods {
                            self.mod_list.push(mod_meta.into());
                        }
                    }
                    mc_mod_meta::ModLoader::Fabric => {
                        let mod_meta = FabricManifest::from_file(file).unwrap();

                        self.mod_list.push(mod_meta.into());
                    }
                },
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }

    fn configure_fonts(&self, ctx: &CtxRef) {
        let data = FontData {
            font: Cow::Borrowed(include_bytes!("../fonts/inter/static/Inter-Medium.ttf")),
            index: 0,
        };
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert("Inter-Medium".to_string(), data);
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Heading,
            (FontFamily::Proportional, 16.),
        );
        font_def.family_and_size.insert(
            eframe::egui::TextStyle::Body,
            (FontFamily::Proportional, 12.),
        );
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "Inter-Medium".to_string());
        ctx.set_fonts(font_def);
    }

    fn configure_style(&self, ctx: &CtxRef) {
        let visuals = Visuals {
            override_text_color: Some(Color32::from_gray(255)),
            ..Default::default()
        };
        let style = Style {
            visuals,
            debug: DebugOptions {
                debug_on_hover: false,
                show_expand_width: false,
                show_expand_height: false,
                show_resize: false,
            },
            ..Default::default()
        };
        ctx.set_style(style);
    }
}

async fn mod_thing(mod_list: &Vec<McMod>, modrinth: &Ferinth) {
    for entry in mod_list {
        modrinth.get_mod(&entry.id).await.unwrap();
    }
}

fn main() {
    let app = UiApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(970., 300.)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
