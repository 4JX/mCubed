use eframe::egui::{
    style::{Margin, Selection, WidgetVisuals, Widgets},
    Color32, Frame, Rounding, Stroke, Visuals,
};

pub struct AppTheme {
    pub colors: Colors,
    pub visuals: Visuals,
    pub default_panel_frame: Frame,
}

impl AppTheme {
    pub fn mod_card_status(&self) -> &VersionStatusTheme {
        &self.colors.mod_card.version_status
    }

    pub fn mod_card_source(&self) -> &SourceTheme {
        &self.colors.mod_card.source
    }

    pub fn mod_card_modloader(&self) -> &ModloaderTheme {
        &self.colors.mod_card.modloader
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        let colors = Colors::default();

        let widgets = Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: colors.gray,                                 // window background
                bg_stroke: Stroke::new(1.0, colors.gray), // separators, indentation lines, windows outlines
                fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                bg_fill: colors.dark_gray, // button background
                bg_stroke: Default::default(),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                bg_fill: Color32::from_gray(70),
                bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                rounding: Rounding::same(3.0),
                expansion: 1.0,
            },
            active: WidgetVisuals {
                bg_fill: Color32::from_gray(55),
                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                rounding: Rounding::same(2.0),
                expansion: 1.0,
            },
            open: WidgetVisuals {
                bg_fill: Color32::from_gray(27),
                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
        };

        let selection = Selection {
            bg_fill: colors.light_gray,
            ..Default::default()
        };

        let visuals = Visuals {
            dark_mode: true,
            override_text_color: Some(colors.white),
            widgets,
            selection,
            extreme_bg_color: colors.darker_gray,
            ..Default::default()
        };

        let default_panel_frame = Frame {
            margin: Margin::same(8.0),
            fill: colors.gray,
            ..Default::default()
        };

        Self {
            colors: Default::default(),
            visuals,
            default_panel_frame,
        }
    }
}

pub struct Colors {
    pub white: Color32,
    pub gray: Color32,
    pub dark_gray: Color32,
    pub darker_gray: Color32,
    pub light_gray: Color32,
    pub lighter_gray: Color32,
    pub error_message: Color32,
    pub mod_card: ModCardTheme,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            white: Color32::from_rgb(255, 255, 255),
            gray: Color32::from_rgb(58, 58, 58),
            dark_gray: Color32::from_rgb(38, 38, 38),
            darker_gray: Color32::from_rgb(22, 22, 22),
            light_gray: Color32::from_rgb(85, 85, 85),
            lighter_gray: Color32::from_rgb(120, 120, 120),
            error_message: Color32::from_rgb(211, 80, 80),
            mod_card: Default::default(),
        }
    }
}

pub struct ModCardTheme {
    pub update_button: Color32,
    pub update_button_background: Color32,
    pub delete_button: Color32,
    pub version_status: VersionStatusTheme,
    pub source: SourceTheme,
    pub modloader: ModloaderTheme,
}

pub struct VersionStatusTheme {
    pub up_to_date: Color32,
    pub outdated: Color32,
    pub invalid: Color32,
}

impl Default for ModCardTheme {
    fn default() -> Self {
        Self {
            update_button: Color32::from_rgb(198, 101, 243),
            update_button_background: Color32::from_rgba_premultiplied(198, 101, 243, 50),
            delete_button: Color32::from_rgb(243, 101, 101),
            version_status: Default::default(),
            source: Default::default(),
            modloader: Default::default(),
        }
    }
}

impl Default for VersionStatusTheme {
    fn default() -> Self {
        Self {
            up_to_date: Color32::from_rgb(89, 230, 98),
            outdated: Color32::from_rgb(248, 241, 73),
            invalid: Color32::from_rgb(232, 72, 72),
        }
    }
}

pub struct SourceTheme {
    pub local: Color32,
    pub curseforge: Color32,
    pub modrinth: Color32,
}

impl Default for SourceTheme {
    fn default() -> Self {
        Self {
            local: Color32::from_rgb(90, 176, 255),
            curseforge: Color32::from_rgb(255, 128, 87),
            // modrinth: Color32::from_rgb(150, 229, 90),
            modrinth: Color32::from_rgb(162, 227, 112),
        }
    }
}
pub struct ModloaderTheme {
    pub forge: Color32,
    pub fabric: Color32,
    pub forge_and_fabric: Color32,
}

impl Default for ModloaderTheme {
    fn default() -> Self {
        Self {
            forge: Color32::from_rgb(233, 175, 110),
            fabric: Color32::from_rgb(232, 221, 186),
            forge_and_fabric: Color32::from_rgb(234, 201, 123),
        }
    }
}
