use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState, MaskPattern};
use std::sync::Arc;

pub type OnCreateProjectFn = dyn Fn(u32, u32, &mut Window, &mut App) + 'static;
pub type OnCancelProjectFn = dyn Fn(&mut Window, &mut App) + 'static;

pub struct NewProjectModal {
    width_input: Entity<InputState>,
    height_input: Entity<InputState>,
    on_create: Arc<OnCreateProjectFn>,
    on_cancel: Arc<OnCancelProjectFn>,
}

impl NewProjectModal {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        on_create: impl Fn(u32, u32, &mut Window, &mut App) + 'static,
        on_cancel: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            width_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value("1024")
                    .mask_pattern(MaskPattern::Number {
                        separator: None,
                        fraction: None,
                    })
            }),
            height_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value("768")
                    .mask_pattern(MaskPattern::Number {
                        separator: None,
                        fraction: None,
                    })
            }),
            on_create: Arc::new(on_create),
            on_cancel: Arc::new(on_cancel),
        }
    }
}

impl Render for NewProjectModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(300.))
                    .p(px(16.))
                    .bg(cx.theme().muted)
                    .rounded(px(8.))
                    .shadow_md()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(FontWeight::BOLD)
                            .child("New Project"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child("Width (px)")
                                    .child(div().w(px(80.)).child(Input::new(&self.width_input))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child("Height (px)")
                                    .child(div().w(px(80.)).child(Input::new(&self.height_input))),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(Button::new("cancel-btn").label("Cancel").ghost().on_click({
                                let on_cancel = self.on_cancel.clone();
                                move |_, window: &mut Window, cx: &mut App| (on_cancel)(window, cx)
                            }))
                            .child(
                                Button::new("create-btn")
                                    .label("Create")
                                    .primary()
                                    .on_click({
                                        let width_input = self.width_input.clone();
                                        let height_input = self.height_input.clone();
                                        let on_create = self.on_create.clone();
                                        move |_, window: &mut Window, cx: &mut App| {
                                            let w = width_input
                                                .read(cx)
                                                .value()
                                                .parse::<u32>()
                                                .unwrap_or(1024);
                                            let h = height_input
                                                .read(cx)
                                                .value()
                                                .parse::<u32>()
                                                .unwrap_or(768);
                                            (on_create)(w, h, window, cx);
                                        }
                                    }),
                            ),
                    ),
            )
    }
}
