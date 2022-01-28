use ferinth::Ferinth;
use message::{FetchingModContext, Message};
use mod_entry::ModEntry;
use std::{
    fs,
    sync::mpsc::{Receiver, Sender},
};

use eframe::{
    egui::{
        self, style::DebugOptions, Color32, CtxRef, FontData, FontDefinitions, FontFamily,
        ProgressBar, RichText, Style, Vec2, Visuals, Widget,
    },
    epi,
};

use std::borrow::Cow;

use crate::mod_entry::{Source, State};

mod message;
mod mod_entry;

struct UiApp {
    mod_list: Vec<ModEntry>,
    search_buf: String,
    app_rx: Option<Receiver<Message>>,
    fetching_info: Option<FetchingModContext>,
}

impl Default for UiApp {
    fn default() -> Self {
        Self {
            mod_list: Default::default(),
            search_buf: Default::default(),
            app_rx: None,
            fetching_info: None,
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
        self.configure_fonts(ctx);
        self.configure_style(ctx);

        self.scan_mod_folder();

        let mut mod_list = self.mod_list.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Message>();

        self.app_rx = Some(rx);

        tokio::task::spawn(async move {
            struct ModrinthManager {
                client: Ferinth,
            }

            impl ModrinthManager {
                async fn fetch_mod_data(&self, mod_list: &mut Vec<ModEntry>, tx: Sender<Message>) {
                    let list_length = mod_list.len();
                    for (position, entry) in mod_list.iter_mut().enumerate() {
                        tx.send(Message::FetchingMod {
                            context: FetchingModContext {
                                name: entry.display_name.clone(),
                                position,
                                total: list_length,
                            },
                        })
                        .unwrap();

                        let modrinth_id = self.get_modrinth_id(entry.hashes.sha1.as_str()).await;

                        entry.modrinth_id = Some(modrinth_id.clone());

                        match self.client.list_versions(modrinth_id.as_str()).await {
                            Ok(version_data) => {
                                entry.sourced_from = Source::Modrinth;
                                // Assume its outdated unless proven otherwise
                                entry.state = State::Outdated;

                                'outer: for file in &version_data[0].files {
                                    if let Some(hash) = &file.hashes.sha1 {
                                        if hash == &entry.hashes.sha1 {
                                            entry.state = State::Current;
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                            Err(_err) => entry.state = State::Invalid,
                        };
                    }
                }

                async fn get_modrinth_id(&self, mod_hash: &str) -> String {
                    match self.client.get_version_from_file_hash(mod_hash).await {
                        Ok(result) => result.mod_id,
                        Err(_err) => "No".into(),
                    }
                }
            }

            let modrinth = ModrinthManager {
                client: Ferinth::new("Test app"),
            };

            let tx_clone = tx.clone();
            modrinth.fetch_mod_data(&mut mod_list, tx_clone).await;

            tx.send(Message::UpdatedModList { list: mod_list }).unwrap();
        });
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        if let Some(rx) = &self.app_rx {
            match rx.try_recv() {
                Ok(message) => {
                    //A
                    match message {
                        Message::UpdatedModList { list } => {
                            self.fetching_info = None;
                            self.mod_list = list;
                        }
                        Message::FetchingMod { context: mod_name } => {
                            self.fetching_info = Some(mod_name);
                        }
                    }
                }
                Err(err) => {
                    match err {
                        std::sync::mpsc::TryRecvError::Empty => {
                            //Non issue
                        }
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            // Eventually handle
                        }
                    }
                }
            };
        }

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

            if let Some(context) = &self.fetching_info {
                let count = context.position as f32;
                let total = context.total as f32;

                ui.vertical_centered(|ui| {
                    ui.style_mut().spacing.interact_size.y = 20.;

                    ProgressBar::new(count / total)
                        .text(format!("Fetching info for mod \"{}\"", context.name))
                        .ui(ui);
                });

                ui.add_space(5.);
                ctx.request_repaint();
            }

            ui.vertical_centered_justified(|ui| {
                egui::Frame {
                    fill: Color32::from_gray(22),
                    margin: Vec2::new(10., 10.),
                    corner_radius: 4.,
                    ..Default::default()
                }
                .show(ui, |ui| {
                    let filtered_list: Vec<&ModEntry> = self
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
                                        ui.set_height(40.);

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
                                            ui.set_width(45.);
                                            ui.centered_and_justified(|ui| {
                                                let text = match mod_entry.state {
                                                    State::Current => RichText::new(version)
                                                        .color(Color32::from_rgb(50, 255, 50)),
                                                    State::Outdated => RichText::new(version)
                                                        .color(Color32::from_rgb(255, 255, 50)),
                                                    State::Invalid => RichText::new(version)
                                                        .color(Color32::from_rgb(255, 50, 50)),
                                                };

                                                ui.label(text);
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.set_width(60.);

                                            egui::Frame {
                                                margin: Vec2::new(8.0, 0.0),
                                                ..Default::default()
                                            }
                                            .show(
                                                ui,
                                                |ui| {
                                                    ui.set_height(ui.available_height() / 2.0);
                                                    ui.horizontal(|ui| {
                                                        ui.label(mod_entry.modloader.to_string());
                                                    });
                                                },
                                            );

                                            egui::Frame {
                                                margin: Vec2::new(8.0, 0.0),
                                                ..Default::default()
                                            }
                                            .show(
                                                ui,
                                                |ui| {
                                                    ui.set_height(ui.available_height() / 2.0);
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            mod_entry.sourced_from.to_string(),
                                                        );
                                                    });
                                                },
                                            );
                                        });

                                        egui::Frame {
                                            margin: Vec2::splat(10.),
                                            ..Default::default()
                                        }
                                        .show(ui, |ui| {
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
        self.mod_list.clear();

        let dir = fs::canonicalize("./mods/").unwrap();

        let read_dir = fs::read_dir(dir).unwrap();

        for path in read_dir {
            let path = path.unwrap().path();

            let file = fs::File::open(&path).unwrap();

            self.mod_list.append(&mut ModEntry::from_file(file));
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

#[tokio::main(worker_threads = 4)]
async fn main() {
    let app = UiApp::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(970., 300.)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);
}
