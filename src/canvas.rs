use gpui::*;
use crate::document::{Document, Layer};
use crate::tool::ToolState;
use std::sync::Arc;

use crate::workspace::Workspace;

pub struct CanvasElement {
    workspace: WeakEntity<Workspace>,
    document: Entity<Document>,
    tool_state: Entity<ToolState>,
}

impl CanvasElement {
    pub fn new(workspace: WeakEntity<Workspace>, document: Entity<Document>, tool_state: Entity<ToolState>) -> Self {
        Self {
            workspace,
            document,
            tool_state,
        }
    }
}

pub struct CanvasLayoutState {
    #[allow(dead_code)]
    layout_id: LayoutId,
}

pub struct CanvasPrepaintState {
    hitbox: Hitbox,
}

impl Element for CanvasElement {
    type RequestLayoutState = CanvasLayoutState;
    type PrepaintState = CanvasPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let layout_id = window.request_layout(
            Style {
                size: Size {
                    width: relative(1.0).into(),
                    height: relative(1.0).into(),
                },
                ..Default::default()
            },
            vec![],
            cx,
        );
        (layout_id, CanvasLayoutState { layout_id })
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        let workspace = self.workspace.clone();
        let hitbox_clone = hitbox.clone();
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                let _ = cx.update(|cx| {
                    workspace.update(cx, |workspace, cx| {
                        workspace.set_canvas_hitbox(hitbox_clone, cx);
                    })
                });
            }
        }).detach();
        CanvasPrepaintState { hitbox }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_quad(fill(bounds, rgb(0x1a1a1a)));

        let (doc_size, transform) = {
            let doc = self.document.read(cx);
            (doc.size, doc.transform)
        };
        
        let layer_origin = bounds.origin + transform.offset.map(|v| px(v));
        let layer_size = doc_size.map(|v| px(v as f32 * transform.scale));
        let layer_bounds = Bounds {
            origin: layer_origin,
            size: layer_size,
        };

        let check_size_base = 32.0;
        let check_size = check_size_base * transform.scale;
        
        if check_size > 4.0 {
            let visible_canvas = bounds.intersect(&layer_bounds);
            if visible_canvas.size.width > px(0.0) && visible_canvas.size.height > px(0.0) {
                window.paint_quad(fill(visible_canvas, rgb(0xeeeeee)));

                let start_x = ((visible_canvas.origin.x - layer_origin.x).to_f64() as f32 / check_size).floor() as i32;
                let start_y = ((visible_canvas.origin.y - layer_origin.y).to_f64() as f32 / check_size).floor() as i32;
                let end_x = ((visible_canvas.origin.x + visible_canvas.size.width - layer_origin.x).to_f64() as f32 / check_size).ceil() as i32;
                let end_y = ((visible_canvas.origin.y + visible_canvas.size.height - layer_origin.y).to_f64() as f32 / check_size).ceil() as i32;

                for r in start_y..end_y {
                    for c in start_x..end_x {
                        if (r + c) % 2 != 0 {
                            let check_origin = layer_origin + Point { 
                                x: px(c as f32 * check_size), 
                                y: px(r as f32 * check_size) 
                            };
                            let check_bounds = Bounds {
                                origin: check_origin,
                                size: Size { width: px(check_size), height: px(check_size) },
                            };
                            let visible_check = check_bounds.intersect(&layer_bounds);
                            if visible_check.size.width > px(0.0) && visible_check.size.height > px(0.0) {
                                window.paint_quad(fill(visible_check, rgb(0xcccccc)));
                            }
                        }
                    }
                }
            }
        } else {
            let visible_canvas = layer_bounds.intersect(&bounds);
            if visible_canvas.size.width > px(0.0) && visible_canvas.size.height > px(0.0) {
                window.paint_quad(fill(visible_canvas, rgb(0xdddddd)));
            }
        }

        let layer_entities = self.document.read(cx).layers.clone();
        for layer_entity in layer_entities {
            let layer = layer_entity.read(cx);
            let Layer::Raster(raster) = layer;
            if raster.visible {
                if let Some(render_image) = &raster.render_cache {
                    let render_image: Arc<RenderImage> = render_image.clone();
                    let _ = window.paint_image(
                        layer_bounds,
                        Corners::default(),
                        render_image,
                        0,
                        false,
                    );
                }
            }
        }
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
