use eframe::{
    egui::{
        style::{Margin, Selection, WidgetVisuals, Widgets},
        Color32, Frame, Rounding, Stroke, Visuals,
    },
    emath::Vec2,
};

pub struct AppTheme {
    pub colors: Colors,
    pub visuals: Visuals,
    pub default_panel_frame: Frame,
    pub prompt_frame: Frame,
    pub spacing: Spacing,
    pub rounding: RoundingTypes,
    pub image_size: ImageSize,
    pub margin: MarginSize,
}

impl AppTheme {
    pub const fn mod_card_source(&self) -> &SourceTheme { &self.colors.mod_card.source }

    pub const fn mod_card_modloader(&self) -> &ModloaderTheme { &self.colors.mod_card.modloader }
}

impl Default for AppTheme {
    fn default() -> Self {
        let colors = Colors::default();

        let widgets = Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: colors.gray,                                 // window background
                bg_stroke: Stroke::new(1.0, colors.dark_gray),        // separators, indentation lines, windows outlines
                fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                rounding: Rounding::same(2.0),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                bg_fill: colors.dark_gray, // button background
                bg_stroke: Stroke::default(),
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
            ..Selection::default()
        };

        let visuals = Visuals {
            dark_mode: true,
            override_text_color: Some(colors.white),
            widgets,
            selection,
            extreme_bg_color: colors.darker_gray,
            ..Visuals::default()
        };

        let margin = MarginSize::default();

        let default_panel_frame = Frame {
            inner_margin: margin.frame_margin,
            fill: colors.gray,
            ..Frame::default()
        };

        let rounding = RoundingTypes::default();
        let prompt_frame = default_panel_frame.rounding(rounding.big);

        Self {
            colors,
            visuals,
            default_panel_frame,
            prompt_frame,
            spacing: Spacing::default(),
            rounding,
            image_size: ImageSize::default(),
            margin,
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
            mod_card: ModCardTheme::default(),
        }
    }
}

pub struct ModCardTheme {
    pub update_button: Color32,
    pub update_button_background: Color32,
    pub delete_button: Color32,
    pub source: SourceTheme,
    pub modloader: ModloaderTheme,
    pub mod_status_icon_background: Color32,
}

impl Default for ModCardTheme {
    fn default() -> Self {
        Self {
            update_button: Color32::from_rgb(198, 101, 243),
            update_button_background: Color32::from_rgba_premultiplied(198, 101, 243, 50),
            delete_button: Color32::from_rgb(243, 101, 101),
            source: SourceTheme::default(),
            modloader: ModloaderTheme::default(),
            mod_status_icon_background: Color32::from_gray(32),
        }
    }
}

pub struct SourceTheme {
    pub none: Color32,
    pub local: Color32,
    pub curseforge: Color32,
    pub modrinth: Color32,
}

impl Default for SourceTheme {
    fn default() -> Self {
        Self {
            none: Color32::from_gray(220),
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
}

impl Default for ModloaderTheme {
    fn default() -> Self {
        Self {
            forge: Color32::from_rgb(233, 175, 110),
            fabric: Color32::from_rgb(232, 221, 186),
        }
    }
}

pub struct Spacing {
    pub large: f32,
    pub medium: f32,
    pub small: f32,
    pub widget_spacing: Vec2,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            large: 10.0,
            medium: 5.0,
            small: 2.0,
            widget_spacing: Vec2::splat(8.0),
        }
    }
}

pub struct RoundingTypes {
    pub small: Rounding,
    pub big: Rounding,
}

impl Default for RoundingTypes {
    fn default() -> Self {
        Self {
            small: Rounding::same(2.0),
            big: Rounding::same(4.0),
        }
    }
}

pub struct ImageSize {
    pub mod_card_status: Vec2,
    pub mod_card_data: Vec2,
    pub mod_card_icon: Vec2,
    pub settings_heading: Vec2,
}

impl Default for ImageSize {
    fn default() -> Self {
        Self {
            mod_card_status: Vec2::splat(12.0),
            mod_card_data: Vec2::splat(10.0),
            mod_card_icon: Vec2::splat(26.0),
            settings_heading: Vec2::splat(16.0),
        }
    }
}

pub struct MarginSize {
    pub frame_margin: Margin,
}

impl Default for MarginSize {
    fn default() -> Self {
        Self {
            frame_margin: Margin::same(8.0),
        }
    }
}
