use std::hash::Hash;

use eframe::{
    egui::{Area, Context, Frame, Id, InnerResponse, Sense, Ui, Layout},
    emath::{Pos2, Align},
    epaint::{Color32, Rounding, Shape},
};

use crate::ui::THEME;

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
    pub fn shown(&mut self, shown: bool) { self.is_shown = shown; }
}

impl ScreenPrompt {
    const PROMPT_BASE_ID: &'static str = "ui_prompt";

    pub fn new(name: impl Hash) -> Self {
        Self {
            id: Id::new(Self::PROMPT_BASE_ID).with(name),
            prompt_frame: THEME.prompt_frame,
            bg_overlay_color: Color32::from_black_alpha(200),
            outside_click_closes: false,
        }
    }

    pub fn set_shown(ctx: &Context, name: impl Hash, shown: bool) {
        ctx.memory()
            .data
            .get_persisted_mut_or_default::<State>(Id::new(Self::PROMPT_BASE_ID).with(name))
            .is_shown = shown;
    }
}

impl ScreenPrompt {
    pub fn show<R>(
        &mut self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui, &mut State) -> R,
    ) -> Option<InnerResponse<R>> {
        let state = ctx.memory().data.get_persisted::<State>(self.id);
        let mut state = state.unwrap_or_default();

        let res = if state.is_shown {
            let area_res = Area::new("prompt_bg").fixed_pos(Pos2::ZERO).show(ctx, |ui| {
                let screen_rect = ctx.input().screen_rect;

                ui.allocate_response(screen_rect.size(), Sense::hover());

                ui.painter()
                    .add(Shape::rect_filled(screen_rect, Rounding::none(), self.bg_overlay_color));

                let mut inner_rect = screen_rect.shrink(30.0);
           
                inner_rect.max -= self.prompt_frame.inner_margin.right_bottom() ;

                let mut child_ui = ui.child_ui(inner_rect, Layout::top_down(Align::Center));
              
                let InnerResponse { inner, response } = self.prompt_frame.show(&mut child_ui, |ui| {
                    ui.set_min_size(inner_rect.size());
                    ui.set_max_size(inner_rect.size());
                    
                    add_contents(ui, &mut state)
                });

          
                if response.clicked_elsewhere() && self.outside_click_closes {
                    state.is_shown = false;
                };

                inner
            });
            Some(area_res)
        } else {
            None
        };

        ctx.memory().data.insert_persisted(self.id, state);

        res
    }
}
