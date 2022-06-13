use std::thread;

use back::{
    messages::{BackendError, ToBackend, ToFrontend},
    mod_file::{FileState, ModLoader},
    settings::SettingsBuilder,
    Back, GameVersion,
};
use crossbeam_channel::{Receiver, Sender};
use eframe::{
    egui::{
        style::{DebugOptions, Margin},
        Align, CentralPanel, ComboBox, Context, Frame, ImageButton, InnerResponse, Label, Layout, RichText, ScrollArea,
        SidePanel, Spinner, Style, TextEdit, Vec2, Widget,
    },
    CreationContext,
};
use once_cell::sync::Lazy;
use parking_lot::Once;

use self::{
    app_theme::AppTheme, image_utils::ImageTextures, mod_card::FileCard, settings::SettingsUi,
    widgets::screen_prompt::ScreenPrompt,
};

mod app_theme;
mod image_utils;
mod misc;
mod mod_card;
mod settings;
mod text_utils;
mod widgets;

static SET_LEFT_PANEL_BOTTOM_BUTTONS_WIDTH: Once = Once::new();
const ICON_RESIZE_QUALITY: u32 = 128;

static THEME: Lazy<AppTheme> = Lazy::new(AppTheme::default);

pub struct MCubedAppUI {
    // UI
    search_buf: String,
    add_mod_buf: String,
    images: ImageTextures,

    // Data
    mod_list: Vec<FileCard>,
    game_version_list: Vec<GameVersion>,
    selected_version: Option<GameVersion>,
    selected_modloader: ModLoader,
    backend_context: BackendContext,

    // Data transferring
    front_tx: Sender<ToBackend>,
    back_rx: Receiver<ToFrontend>,

    // Misc sizes to combat immediate mode shenanigans
    left_panel_bottom_buttons_width: f32,
}

#[derive(Default)]
struct BackendContext {
    checking_for_updates: bool,
    backend_errors: Vec<BackendError>,
}

impl MCubedAppUI {
    pub fn new(cc: &CreationContext) -> Self {
        let (front_tx, front_rx) = crossbeam_channel::unbounded();
        let (back_tx, back_rx) = crossbeam_channel::unbounded();

        let frame_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            Back::new(back_tx, front_rx, frame_clone).init();
        });

        SettingsBuilder::from_current()
            .icon_resize_size(ICON_RESIZE_QUALITY)
            .apply();

        Self::configure_style(&cc.egui_ctx);

        let new_app = Self {
            search_buf: String::default(),
            add_mod_buf: String::default(),
            images: ImageTextures::new(&cc.egui_ctx),
            mod_list: Vec::default(),
            game_version_list: Vec::default(),
            selected_version: Option::default(),
            selected_modloader: ModLoader::default(),
            backend_context: BackendContext::default(),
            front_tx,
            back_rx,
            left_panel_bottom_buttons_width: f32::default(),
        };

        new_app.front_tx.send(ToBackend::Startup).unwrap();

        new_app
    }
}

impl eframe::App for MCubedAppUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        match self.back_rx.try_recv() {
            Ok(message) => match message {
                ToFrontend::SetVersionMetadata { manifest } => {
                    self.selected_version = Some(manifest.versions[0].clone());
                    self.game_version_list = manifest.versions;
                }
                ToFrontend::UpdateModList { mod_list } => {
                    self.backend_context.checking_for_updates = false;
                    self.mod_list = mod_list.into_iter().map(|file| FileCard::new(file, ctx)).collect();
                    ctx.request_repaint();
                }
                ToFrontend::BackendError { error } => {
                    self.backend_context.backend_errors.push(error);
                }
            },
            Err(err) => {
                let _ = err;
            }
        }

        ScreenPrompt::new("settings").show(ctx, |ui, state| {
            ScrollArea::vertical().show(ui, |ui| {
                SettingsUi::show(ui, &self.images);
            });

            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    state.shown(false);
                }
            });
        });

        self.render_side_panel(ctx);

        self.render_central_panel(ctx);
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        self.front_tx
            .send(ToBackend::UpdateBackendList {
                mod_list: self.mod_list.iter().map(FileCard::mod_file).cloned().collect(),
            })
            .unwrap();

        self.front_tx.send(ToBackend::Shutdown).unwrap();
    }
}

impl MCubedAppUI {
    fn render_side_panel(&mut self, ctx: &Context) -> InnerResponse<()> {
        SidePanel::left("options_panel")
            .frame(THEME.default_panel_frame)
            .resizable(false)
            .max_width(240.)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = THEME.spacing.widget_spacing;

                ui.horizontal(|ui| {
                    ui.label("Game Version");

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ComboBox::from_id_source("version-combo")
                            .icon(misc::combobox_icon_fn)
                            .selected_text(if let Some(selected_value) = self.selected_version.as_ref() {
                                selected_value.id.as_str()
                            } else if self.game_version_list.is_empty() {
                                "Loading..."
                            } else {
                                self.selected_version = Some(self.game_version_list[0].clone());
                                self.selected_version.as_ref().unwrap().id.as_str()
                            })
                            .show_ui(ui, |ui| {
                                for version in &self.game_version_list {
                                    ui.selectable_value(&mut self.selected_version, Some(version.clone()), &version.id);
                                }
                            });
                    });
                });

                Frame {
                    fill: THEME.colors.light_gray,
                    inner_margin: Margin::same(10.0),
                    rounding: THEME.rounding.big,
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    // Fill the side panel
                    ui.set_width(ui.available_width());

                    ui.horizontal(|ui| {
                        let edit = TextEdit::singleline(&mut self.add_mod_buf)
                            .hint_text(RichText::new("Modrinth ID or Slug").color(THEME.colors.gray));

                        ui.add_sized(Vec2::new(130.0, ui.available_height()), edit);

                        if ui.button("Fetch Mod").clicked() {
                            if let Some(version) = &self.selected_version {
                                if !self.add_mod_buf.is_empty() {
                                    self.front_tx
                                        .send(ToBackend::AddMod {
                                            modrinth_id: self.add_mod_buf.clone(),
                                            game_version: version.id.clone(),
                                            modloader: self.selected_modloader,
                                        })
                                        .unwrap();
                                }
                            };
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
                        let rescan_folder_button_res = ui.button("Scan Folder");

                        if rescan_folder_button_res.clicked() {
                            self.front_tx
                                .send(ToBackend::UpdateBackendList {
                                    mod_list: self.mod_list.iter().map(FileCard::mod_file).cloned().collect(),
                                })
                                .unwrap();

                            self.front_tx.send(ToBackend::ScanFolder).unwrap();
                        };

                        let refresh_button_res = ui.button("Refresh");
                        if refresh_button_res.clicked() {
                            if let Some(version) = &self.selected_version {
                                self.backend_context.checking_for_updates = true;
                                self.front_tx
                                    .send(ToBackend::CheckForUpdates {
                                        game_version: version.id.clone(),
                                    })
                                    .unwrap();
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
            .frame(THEME.default_panel_frame)
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = THEME.spacing.widget_spacing;
                let button = ImageButton::new(&self.images.settings, Vec2::splat(12.0));

                ui.horizontal(|ui| {
                    if self.backend_context.checking_for_updates {
                        ui.horizontal(|ui| {
                            Spinner::new().size(14.0).ui(ui);
                            ui.label("Checking for updates");
                        });
                    };

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.add(button).clicked() {
                            ScreenPrompt::set_shown(ctx, "settings", true);
                        };

                        if self
                            .mod_list
                            .iter()
                            .any(|card| card.mod_file().data.state == FileState::Outdated)
                            && ui.button("Update All").clicked()
                        {
                            self.front_tx.send(ToBackend::UpdateAll).unwrap()
                        };
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical_centered_justified(|ui| {
                        let edit = TextEdit::singleline(&mut self.search_buf)
                            .hint_text(RichText::new("Search installed mods").color(THEME.colors.gray));
                        ui.add(edit);
                    });
                });

                if !self.backend_context.backend_errors.is_empty() {
                    ScrollArea::vertical().show(ui, |ui| {
                        self.backend_context.backend_errors.retain(|error| {
                            let mut retain = true;

                            Frame {
                                fill: THEME.colors.error_message,
                                inner_margin: Margin::same(6.0),
                                rounding: THEME.rounding.big,
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

                ui.vertical_centered_justified(|ui| {
                    Frame {
                        fill: THEME.colors.darker_gray,
                        inner_margin: Margin::same(10.0),
                        rounding: THEME.rounding.big,
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
                                mod_card.mod_file().entries.iter().any(|entry| {
                                    entry
                                        .display_name
                                        .to_lowercase()
                                        .contains(self.search_buf.to_lowercase().as_str())
                                })
                            });

                            if !search_results_exist && !self.search_buf.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label("No mods match your search");
                                });
                            } else {
                                ScrollArea::vertical().show(ui, |ui| {
                                    ui.style_mut().spacing.item_spacing.y = THEME.spacing.large;
                                    for file_card in &mut self.mod_list {
                                        file_card.show(&self.search_buf, ui, &self.front_tx, &self.images);
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
    fn configure_style(ctx: &Context) {
        let style = Style {
            text_styles: text_utils::default_text_styles(),
            visuals: THEME.visuals.clone(),
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
}
