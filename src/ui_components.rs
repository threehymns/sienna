use gpui::*;
use gpui_component::Icon;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::slider::{Slider, SliderState};
use gpui_component::{ActiveTheme, Selectable, Sizable, h_flex, v_flex};

pub fn tool_button(
    icon_path: &'static str,
    id: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .icon(Icon::empty().path(icon_path))
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
    icon_path: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .icon(Icon::empty().path(icon_path))
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

pub fn paint_checkerboard(
    window: &mut Window,
    visible_bounds: Bounds<Pixels>,
    layer_origin: Point<Pixels>,
    check_size: f32,
    color_light: Rgba,
    color_dark: Rgba,
) {
    if visible_bounds.size.width > px(0.0) && visible_bounds.size.height > px(0.0) {
        window.paint_quad(fill(visible_bounds, color_light));

        let check_px = px(check_size);

        let rel_left = visible_bounds.origin.x - layer_origin.x;
        let rel_top = visible_bounds.origin.y - layer_origin.y;
        let rel_right = rel_left + visible_bounds.size.width;
        let rel_bottom = rel_top + visible_bounds.size.height;

        let first_col = (rel_left / check_px).floor() as i32;
        let first_row = (rel_top / check_px).floor() as i32;
        let last_col = (rel_right / check_px).ceil() as i32;
        let last_row = (rel_bottom / check_px).ceil() as i32;

        for row in first_row..last_row {
            for col in first_col..last_col {
                if (row + col) % 2 != 0 {
                    let check_bounds = Bounds {
                        origin: Point {
                            x: layer_origin.x + px(col as f32 * check_size),
                            y: layer_origin.y + px(row as f32 * check_size),
                        },
                        size: Size {
                            width: check_px,
                            height: check_px,
                        },
                    };
                    let clipped = check_bounds.intersect(&visible_bounds);
                    if clipped.size.width > px(0.0) && clipped.size.height > px(0.0) {
                        window.paint_quad(fill(clipped, color_dark));
                    }
                }
            }
        }
    }
}
