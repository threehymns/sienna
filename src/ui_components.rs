use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::slider::{Slider, SliderState};
use gpui_component::{ActiveTheme, Selectable, Sizable, h_flex, v_flex};

pub fn tool_button(
    label: &'static str,
    id: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .label(label)
        .selected(active)
        .on_click(on_click)
}

pub fn menu_button(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id).label(label).ghost().on_click(on_click)
}

pub fn icon_button(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .label(label)
        .small()
        .ghost()
        .on_click(on_click)
}

pub fn property_slider(
    label: &'static str,
    state: &Entity<SliderState>,
    value: String,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(div().text_size(px(12.)).child(label))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(cx.theme().muted_foreground)
                        .child(value),
                ),
        )
        .child(Slider::new(state))
}
