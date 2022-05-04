use back::{
    messages::{BackendError, CheckProgress, ToBackend, ToFrontend},
    mod_entry::ModLoader,
    settings::SettingsBuilder,
    Back, GameVersion,
};
use parking_lot::Once;

use std::{collections::HashMap, thread};

use crossbeam_channel::{Receiver, Sender};

use eframe::{
    egui::{
        style::{DebugOptions, Margin},
        Align, CentralPanel, ComboBox, Context, Frame, InnerResponse, Label, Layout, ProgressBar,
        RichText, Rounding, ScrollArea, SidePanel, Style, TextEdit, Vec2, Widget,
    },
    epaint::TextureHandle,
    CreationContext,
};

use self::{app_theme::AppTheme, image_utils::ImageTextures, mod_card::ModCard};

mod app_theme;
mod image_utils;
mod misc;
mod mod_card;
mod text_utils;

static SET_LEFT_PANEL_BOTTOM_BUTTONS_WIDTH: Once = Once::new();
const ICON_RESIZE_QUALITY: u32 = 128;

#[derive(Default)]
pub struct MCubedAppUI {
    // UI
    theme: AppTheme,
    search_buf: String,
    add_mod_buf: String,
    images: ImageTextures,
    mod_images: HashMap<String, TextureHandle>,

    // Data
    mod_list: Vec<ModCard>,
    game_version_list: Vec<GameVersion>,
    selected_version: Option<GameVersion>,
    selected_modloader: ModLoader,
    backend_context: BackendContext,

    // Data transferring
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,

    // Misc sizes to combat immediate mode shenanigans
    left_panel_bottom_buttons_width: f32,
}

#[derive(Default)]
struct BackendContext {
    check_for_update_progress: Option<CheckProgress>,
    backend_errors: Vec<BackendError>,
}

impl MCubedAppUI {
    pub fn new(cc: &CreationContext) -> Self {
        let mut new_app = Self::default();

        new_app.configure_style(&cc.egui_ctx);
        new_app.images.load_images(&cc.egui_ctx);

        let (front_tx, front_rx) = crossbeam_channel::unbounded();
        let (back_tx, back_rx) = crossbeam_channel::unbounded();

        let frame_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            Back::new(None, back_tx, front_rx, frame_clone).init();
        });

        new_app.front_tx = Some(front_tx);
        new_app.back_rx = Some(back_rx);

        if let Some(sender) = &new_app.front_tx {
            sender.send(ToBackend::Startup).unwrap();
        }

        SettingsBuilder::new()
            .icon_resize_size(ICON_RESIZE_QUALITY)
            .apply();

        new_app
    }
}

impl eframe::App for MCubedAppUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::SetVersionMetadata { manifest } => {
                        self.selected_version = Some(manifest.versions[0].clone());
                        self.game_version_list = manifest.versions;
                    }
                    ToFrontend::UpdateModList { mod_list } => {
                        self.backend_context.check_for_update_progress = None;
                        self.mod_images.clear();
                        self.mod_list = mod_list
                            .into_iter()
                            .map(|entry| ModCard::new(entry, ctx))
                            .collect();
                        ctx.request_repaint();
                    }
                    ToFrontend::CheckForUpdatesProgress { progress } => {
                        self.backend_context.check_for_update_progress = Some(progress);
                    }
                    ToFrontend::BackendError { error } => {
                        self.backend_context.backend_errors.push(error);
                    }
                },
                Err(err) => {
                    let _ = err;
                }
            }
        }

        self.render_side_panel(ctx);

        self.render_central_panel(ctx);
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        self.update_backend_list();
    }
}

impl MCubedAppUI {
    fn render_side_panel(&mut self, ctx: &Context) -> InnerResponse<()> {
        SidePanel::left("options_panel")
            .frame(self.theme.default_panel_frame)
            .resizable(false)
            .max_width(240.)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);

                ui.horizontal(|ui| {
                    ui.label("Game Version");

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ComboBox::from_id_source("version-combo")
                            .icon(misc::combobox_icon_fn)
                            .selected_text(
                                if let Some(selected_value) = self.selected_version.as_ref() {
                                    selected_value.id.as_str()
                                } else if self.game_version_list.is_empty() {
                                    "Loading..."
                                } else {
                                    self.selected_version = Some(self.game_version_list[0].clone());
                                    self.selected_version.as_ref().unwrap().id.as_str()
                                },
                            )
                            .show_ui(ui, |ui| {
                                for version in &self.game_version_list {
                                    ui.selectable_value(
                                        &mut self.selected_version,
                                        Some(version.clone()),
                                        &version.id,
                                    );
                                }
                            });
                    });
                });

                Frame {
                    fill: self.theme.colors.light_gray,
                    inner_margin: Margin::same(10.0),
                    rounding: Rounding::same(4.),
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    // Fill the side panel
                    ui.set_width(ui.available_width());

                    ui.horizontal(|ui| {
                        let edit = TextEdit::singleline(&mut self.add_mod_buf).hint_text(
                            RichText::new("Modrinth ID or Slug").color(self.theme.colors.gray),
                        );

                        ui.add_sized(Vec2::new(130.0, ui.available_height()), edit);

                        if ui.button("Fetch Mod").clicked() {
                            if let Some(tx) = &self.front_tx {
                                if let Some(version) = &self.selected_version {
                                    tx.send(ToBackend::AddMod {
                                        modrinth_id: self.add_mod_buf.clone(),
                                        game_version: version.id.clone(),
                                        modloader: self.selected_modloader,
                                    })
                                    .unwrap();
                                } else {
                                    tx.send(ToBackend::AddMod {
                                        modrinth_id: self.add_mod_buf.clone(),
                                        game_version: self.game_version_list[0].id.clone(),
                                        modloader: self.selected_modloader,
                                    })
                                    .unwrap();
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.selected_modloader, ModLoader::Forge, "Forge");
                        ui.radio_value(&mut self.selected_modloader, ModLoader::Fabric, "Fabric");
                    });
                });

                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    ui.set_max_width(self.left_panel_bottom_buttons_width);
                    let horizontal_res = ui.horizontal(|ui| {
                        let rescan_folder_button_res = ui.button("Re-scan Folder");

                        if rescan_folder_button_res.clicked() {
                            self.update_backend_list();
                        };

                        let refresh_button_res = ui.button("Refresh");
                        if refresh_button_res.clicked() {
                            if let Some(tx) = &self.front_tx {
                                if let Some(version) = &self.selected_version {
                                    tx.send(ToBackend::CheckForUpdates {
                                        game_version: version.id.clone(),
                                    })
                                    .unwrap();
                                } else {
                                    tx.send(ToBackend::CheckForUpdates {
                                        game_version: self.game_version_list[0].id.clone(),
                                    })
                                    .unwrap();
                                }
                            }
                        }
                    });

                    SET_LEFT_PANEL_BOTTOM_BUTTONS_WIDTH.call_once(|| {
                        self.left_panel_bottom_buttons_width = horizontal_res.response.rect.width();
                    });
                });
            })
    }

    fn render_central_panel(&mut self, ctx: &Context) -> InnerResponse<()> {
        CentralPanel::default()
            .frame(self.theme.default_panel_frame)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);
                ui.horizontal(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        let edit = TextEdit::singleline(&mut self.search_buf).hint_text(
                            RichText::new("Search installed mods").color(self.theme.colors.gray),
                        );
                        ui.add(edit);
                    });
                });

                if !self.backend_context.backend_errors.is_empty() {
                    ScrollArea::vertical().show(ui, |ui| {
                        self.backend_context.backend_errors.retain(|error| {
                            let mut retain = true;

                            Frame {
                                fill: self.theme.colors.error_message,
                                inner_margin: Margin::same(6.0),
                                rounding: Rounding::same(4.),
                                ..Frame::default()
                            }
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(Label::new(&error.message).wrap(true))
                                        .on_hover_text(error.error.to_string());
                                    ui.with_layout(Layout::right_to_left(), |ui| {
                                        if ui.button("Close").clicked() {
                                            retain = false;
                                        }
                                    });
                                });
                            });

                            retain
                        });
                    });
                }

                if let Some(progress) = &self.backend_context.check_for_update_progress {
                    let count = progress.position as f32 + 1.0;
                    let total = progress.total_len as f32;

                    ui.vertical_centered(|ui| {
                        ui.style_mut().spacing.interact_size.y = 20.;

                        ProgressBar::new(count / total)
                            .text(format!(
                                "({}/{}) Fetching info for mod \"{}\"",
                                count, total, progress.name,
                            ))
                            .ui(ui);
                    });
                }

                ui.vertical_centered_justified(|ui| {
                    Frame {
                        fill: self.theme.colors.darker_gray,
                        inner_margin: Margin::same(10.0),
                        rounding: Rounding::same(4.),
                        ..Frame::default()
                    }
                    .show(ui, |ui| {
                        ui.set_height(ui.available_height());

                        if self.mod_list.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("There are no mods to display");
                            });
                        } else {
                            let search_results_exist = self.mod_list.iter().any(|mod_card| {
                                mod_card
                                    .entry()
                                    .display_name
                                    .to_lowercase()
                                    .contains(self.search_buf.to_lowercase().as_str())
                            });

                            if !search_results_exist && !self.search_buf.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No mods match your search");
                                });
                            } else {
                                ScrollArea::vertical().show(ui, |ui| {
                                    ui.style_mut().spacing.item_spacing.y = 10.0;
                                    for mod_card in &mut self.mod_list {
                                        // Skip the entries that are not within the filtered list
                                        if !mod_card
                                            .entry()
                                            .display_name
                                            .to_lowercase()
                                            .contains(self.search_buf.to_lowercase().as_str())
                                        {
                                            continue;
                                        }

                                        mod_card.show(
                                            ui,
                                            &self.theme,
                                            &mut self.images,
                                            &self.front_tx,
                                        );
                                    }
                                });
                            }
                        }
                    });
                });
            })
    }
}

impl MCubedAppUI {
    fn configure_style(&self, ctx: &Context) {
        let style = Style {
            text_styles: text_utils::default_text_styles(),
            visuals: self.theme.visuals.clone(),
            debug: DebugOptions {
                debug_on_hover: false,
                show_expand_width: false,
                show_expand_height: false,
                show_resize: false,
            },
            ..Style::default()
        };

        ctx.set_fonts(text_utils::get_font_def());
        ctx.set_style(style);
    }

    fn update_backend_list(&self) {
        if let Some(tx) = &self.front_tx {
            tx.send(ToBackend::UpdateBackendList {
                mod_list: self.mod_list.iter().map(ModCard::entry).cloned().collect(),
            })
            .unwrap();

            tx.send(ToBackend::Shutdown).unwrap();
        }
    }
}
