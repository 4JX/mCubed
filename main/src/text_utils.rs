use std::{borrow::Cow, collections::BTreeMap};

use eframe::egui::{FontData, FontDefinitions, FontFamily, FontId, RichText, TextStyle};

fn add_font(font_def: &mut FontDefinitions, font: FontData, font_name: &str) {
    font_def.font_data.insert(font_name.into(), font);

    font_def
        .families
        .insert(FontFamily::Name(font_name.into()), vec![font_name.into()]);
}

pub fn get_font_def() -> FontDefinitions {
    let mut font_def = FontDefinitions::default();

    let inter_medium = FontData {
        font: Cow::Borrowed(include_bytes!("../fonts/inter/static/Inter-Medium.ttf")),
        index: 0,
    };

    let inter_semi_bold = FontData {
        font: Cow::Borrowed(include_bytes!("../fonts/inter/static/Inter-SemiBold.ttf")),
        index: 0,
    };

    let inter_bold = FontData {
        font: Cow::Borrowed(include_bytes!("../fonts/inter/static/Inter-Bold.ttf")),
        index: 0,
    };

    let inter_extra_bold = FontData {
        font: Cow::Borrowed(include_bytes!("../fonts/inter/static/Inter-ExtraBold.ttf")),
        index: 0,
    };

    add_font(&mut font_def, inter_medium, "Inter-Medium");
    add_font(&mut font_def, inter_semi_bold, "Inter-SemiBold");
    add_font(&mut font_def, inter_bold, "Inter-Bold");
    add_font(&mut font_def, inter_extra_bold, "Inter-ExtraBold");

    font_def
}

type TextStyles = BTreeMap<TextStyle, FontId>;

fn insert_style(text_styles: &mut TextStyles, style_name: &str, font_name: &str, size: f32) {
    text_styles.insert(
        TextStyle::Name(style_name.into()),
        FontId::new(size, FontFamily::Name(font_name.into())),
    );
}

pub fn default_text_styles() -> TextStyles {
    let mut text_styles = BTreeMap::new();

    // Default styles
    text_styles.insert(
        TextStyle::Small,
        FontId::new(10.0, FontFamily::Name("Inter-Medium".into())),
    );
    text_styles.insert(
        TextStyle::Body,
        FontId::new(14.0, FontFamily::Name("Inter-Medium".into())),
    );
    text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Name("Inter-Medium".into())),
    );
    text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, FontFamily::Name("Inter-Medium".into())),
    );
    text_styles.insert(
        TextStyle::Monospace,
        FontId::new(14.0, FontFamily::Monospace),
    );

    // Custom
    insert_style(&mut text_styles, "Mod-Version", "Inter-ExtraBold", 12.0);
    insert_style(&mut text_styles, "Mod-Card-Data", "Inter-Bold", 9.0);
    insert_style(&mut text_styles, "Update-Button", "Inter-SemiBold", 9.0);

    text_styles
}

pub fn mod_version_text(text: impl Into<String>) -> RichText {
    RichText::new(text).text_style(TextStyle::Name("Mod-Version".into()))
}

pub fn mod_card_data_text(text: impl Into<String>) -> RichText {
    RichText::new(text).text_style(TextStyle::Name("Mod-Card-Data".into()))
}

pub fn update_button_text(text: impl Into<String>) -> RichText {
    RichText::new(text).text_style(TextStyle::Name("Update-Button".into()))
}
