use gpui::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Move,
    Brush,
    Eraser,
    ColorPicker,
}

pub enum ToolEvent {
    ColorChanged(Rgba),
}

impl EventEmitter<ToolEvent> for ToolState {}

pub struct ToolState {
    pub active_tool: Tool,
    pub brush_size: f32,
    pub brush_opacity: f32,
    pub brush_hardness: f32,
    pub active_color: Rgba,
    pub last_mouse_pos: Option<Point<Pixels>>,
    pub pixels_before: Option<Vec<u8>>,
}

impl ToolState {
    pub fn new() -> Self {
        Self {
            active_tool: Tool::Brush,
            brush_size: 10.0,
            brush_opacity: 1.0,
            brush_hardness: 1.0,
            active_color: rgba(0xffffffff),
            last_mouse_pos: None,
            pixels_before: None,
        }
    }
}
