use crate::stroke::StrokeAccumulator;
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
    pub brush_flow: f32,
    pub brush_hardness: f32,
    pub brush_spacing: f32,
    pub brush_stabilization: f32,
    pub active_color: Rgba,
    pub last_mouse_pos: Option<Point<Pixels>>,
    /// The active stroke accumulator — present only while painting.
    pub active_stroke: Option<StrokeAccumulator>,
}

impl ToolState {
    pub fn new() -> Self {
        Self {
            active_tool: Tool::Brush,
            brush_size: 10.0,
            brush_opacity: 1.0,
            brush_flow: 1.0,
            brush_hardness: 0.8,
            brush_spacing: 0.1,
            brush_stabilization: 0.0,
            active_color: rgba(0xffffffff),
            last_mouse_pos: None,
            active_stroke: None,
        }
    }
}
