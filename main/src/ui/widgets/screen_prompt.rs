use eframe::{
    egui::{Area, Context, Frame, Id, Order, Sense, Ui},
    emath::{Align2, Pos2, Vec2},
    epaint::{Color32, Rounding, Shape},
};

use crate::ui::THEME;

lazy_static::lazy_static!(
    pub static ref PROMPT_BASE_ID: Id = Id::new("ui_prompt");
);

#[derive(Clone)]
pub struct ScreenPrompt {
    id: Id,
    prompt_frame: Frame,
    bg_overlay_color: Color32,
    outside_click_closes: bool,
}

#[derive(Clone, Default, Debug, Copy)]
pub struct State {
    is_shown: bool,
}

impl State {
    pub fn shown(&mut self, shown: bool) {
        self.is_shown = shown;
    }
}

impl ScreenPrompt {
    pub fn with_id(id: Id) -> Self {
        Self {
            id,
            prompt_frame: THEME.prompt_frame,
            bg_overlay_color: Color32::from_black_alpha(200),
            outside_click_closes: true,
        }
    }

    pub fn show_with_id(ctx: &Context, toggle_id: impl Into<Id>, shown: bool) {
        ctx.memory()
            .data
            .get_persisted_mut_or_default::<State>(toggle_id.into())
            .is_shown = shown;
    }
}

impl ScreenPrompt {
    pub fn show<R>(&mut self, ctx: &Context, add_contents: impl FnOnce(&mut Ui, &mut State) -> R) {
        let state = ctx.memory().data.get_persisted::<State>(self.id);
        let mut state = state.unwrap_or_default();

        if state.is_shown {
            Area::new("prompt_bg")
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    let screen_rect = ctx.input().screen_rect;

                    ui.allocate_response(screen_rect.size(), Sense::click());

                    ui.painter().add(Shape::rect_filled(
                        screen_rect,
                        Rounding::none(),
                        self.bg_overlay_color,
                    ));

                    let area_res = Area::new("prompt_centered")
                        .fixed_pos(Pos2::ZERO)
                        .anchor(Align2::CENTER_CENTER, Vec2::splat(0.0))
                        .order(Order::Foreground)
                        .show(ctx, |ui| {
                            self.prompt_frame.show(ui, |ui| {
                                add_contents(ui, &mut state);
                            });
                        });

                    if area_res.response.clicked_elsewhere() && self.outside_click_closes {
                        state.is_shown = false
                    };
                });
        }

        ctx.memory().data.insert_persisted(self.id, state);
    }
}
