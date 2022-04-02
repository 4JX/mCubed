use eframe::{
    egui::{style::WidgetVisuals, Ui},
    emath::{Rect, Vec2},
    epaint::Shape,
};

pub fn combobox_icon_fn(ui: &Ui, rect: Rect, visuals: &WidgetVisuals, _is_open: bool) {
    let rect = Rect::from_center_size(
        rect.center(),
        Vec2::new(rect.width() * 0.6, rect.height() * 0.4),
    );

    ui.painter().add(Shape::convex_polygon(
        vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
        visuals.fg_stroke.color,
        visuals.fg_stroke,
    ));
}
