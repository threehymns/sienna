use gpui::*;
use std::sync::Arc;

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

pub enum StrokeUpdate {
    Tiles(std::collections::HashMap<(u32, u32), (crate::tile::Tile, Arc<RenderImage>)>),
    Finished(crate::tile::TileGrid, crate::tile::TileGrid),
}

#[allow(dead_code)]
pub struct ActiveStroke {
    pub tx_points: Option<tokio::sync::mpsc::UnboundedSender<Point<f32>>>,
    pub width: u32,
    pub height: u32,
    pub composited_tiles: std::collections::HashMap<(u32, u32), crate::tile::Tile>,
    pub render_cache: std::collections::HashMap<(u32, u32), Arc<RenderImage>>,
    pub final_tiles: Option<(crate::tile::TileGrid, crate::tile::TileGrid)>,
}

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
    /// The active stroke — present only while painting.
    pub active_stroke: Option<ActiveStroke>,
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
