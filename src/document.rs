use gpui::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CanvasTransform {
    pub offset: Point<f32>,
    pub scale: f32,
}

impl Default for CanvasTransform {
    fn default() -> Self {
        Self {
            offset: Point::default(),
            scale: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocumentData {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LayerData>,
    pub transform: CanvasTransform,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LayerData {
    Raster {
        name: String,
        visible: bool,
        opacity: f32,
        tiles: crate::tile::TileGrid,
        #[serde(default)]
        blend_mode: crate::blend::BlendMode,
    },
}

#[derive(Clone)]
pub enum Action {
    Paint {
        layer_index: usize,
        changed_tiles: std::collections::HashMap<
            crate::tile::TileCoords,
            (Option<crate::tile::Tile>, Option<crate::tile::Tile>),
        >,
    },
    AddLayer {
        index: usize,
        layer_data: LayerData,
    },
    DeleteLayer {
        index: usize,
        layer_data: LayerData,
    },
    MoveLayer {
        from: usize,
        to: usize,
    },
    ToggleVisibility {
        index: usize,
        before: bool,
    },
    SetOpacity {
        index: usize,
        before: f32,
        after: f32,
    },
}

pub struct Document {
    pub size: Size<u32>,
    pub layers: Vec<Entity<Layer>>,
    pub active_layer_index: usize,
    pub transform: CanvasTransform,
    pub undo_stack: Vec<Action>,
    pub redo_stack: Vec<Action>,
    pub composited_cache: std::collections::HashMap<crate::tile::TileCoords, Arc<RenderImage>>,
    pub pending_composited_tiles: std::collections::HashMap<crate::tile::TileCoords, usize>,
    pub dirty_composited_tiles: std::collections::HashSet<crate::tile::TileCoords>,
    pub stroke_composited_cache:
        std::collections::HashMap<crate::tile::TileCoords, Arc<RenderImage>>,
    pub cache_version: usize,
}

impl Document {
    pub fn new(size: Size<u32>, cx: &mut App) -> Self {
        let first_layer = cx.new(|_cx| Layer::Raster(RasterLayer::new(size, "Background")));
        Self {
            size,
            layers: vec![first_layer],
            active_layer_index: 0,
            transform: CanvasTransform::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            composited_cache: std::collections::HashMap::new(),
            pending_composited_tiles: std::collections::HashMap::new(),
            dirty_composited_tiles: std::collections::HashSet::new(),
            stroke_composited_cache: std::collections::HashMap::new(),
            cache_version: 0,
        }
    }

    pub fn clear_composited_cache(&mut self) {
        self.cache_version += 1;
        self.composited_cache.clear();
        self.pending_composited_tiles.clear();
        self.dirty_composited_tiles.clear();
        self.stroke_composited_cache.clear();
    }

    #[allow(dead_code)]
    pub fn resolve_composited_tile(
        &mut self,
        coords: crate::tile::TileCoords,
        cx: &mut App,
        doc_weak: &WeakEntity<Self>,
        active_stroke_tile: Option<crate::tile::Tile>,
    ) -> Option<Arc<RenderImage>> {
        let is_dirty = self.dirty_composited_tiles.contains(&coords);
        let in_cache = self.composited_cache.contains_key(&coords);
        let is_stroke_active = active_stroke_tile.is_some();

        if is_stroke_active {
            if let Some(cached) = self.stroke_composited_cache.get(&coords) {
                return Some(cached.clone());
            }
        } else {
            if in_cache && !is_dirty {
                return self.composited_cache.get(&coords).cloned();
            }
        }

        let current_version = self.cache_version;

        if self.pending_composited_tiles.get(&coords) == Some(&current_version) {
            // Fallback to base cache while we wait
            return self.composited_cache.get(&coords).cloned();
        }

        // We need to re-composite
        let mut visible_tiles = Vec::new();
        for (i, layer_entity) in self.layers.iter().enumerate() {
            let layer = layer_entity.read(cx);
            match layer {
                Layer::Raster(r) => {
                    if !r.visible || r.opacity <= 0.0 {
                        continue;
                    }
                    if i == self.active_layer_index && is_stroke_active {
                        visible_tiles.push((
                            active_stroke_tile.clone().unwrap(),
                            r.blend_mode,
                            r.opacity,
                        ));
                    } else if let Some(tile) = r.tiles.tiles.get(&coords) {
                        visible_tiles.push((tile.clone(), r.blend_mode, r.opacity));
                    }
                }
            }
        }

        if visible_tiles.is_empty() {
            if !is_stroke_active {
                self.dirty_composited_tiles.remove(&coords);
                self.composited_cache.remove(&coords);
            }
            return None;
        }

        self.pending_composited_tiles
            .insert(coords, current_version);
        let doc_weak = doc_weak.clone();

        cx.spawn(move |cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let render_image = cx
                    .background_spawn(async move {
                        let mut base_tile = crate::tile::Tile::new();
                        for (tile, blend_mode, opacity) in visible_tiles {
                            base_tile.composite_layer(&tile, blend_mode, opacity);
                        }
                        base_tile.build_render_image()
                    })
                    .await;

                let _ =
                    doc_weak.update(&mut cx, |doc: &mut Document, cx: &mut Context<Document>| {
                        if doc.pending_composited_tiles.get(&coords) == Some(&current_version) {
                            if is_stroke_active {
                                doc.stroke_composited_cache
                                    .insert(coords, render_image.clone());
                            } else {
                                doc.composited_cache.insert(coords, render_image.clone());
                                doc.dirty_composited_tiles.remove(&coords);
                            }
                            doc.pending_composited_tiles.remove(&coords);
                            cx.notify();
                        }
                    });
            }
        })
        .detach();

        self.composited_cache.get(&coords).cloned()
    }

    pub fn from_data(data: DocumentData, cx: &mut App) -> Self {
        let size = Size {
            width: data.width,
            height: data.height,
        };
        let layers = data
            .layers
            .into_iter()
            .map(|l_data| {
                cx.new(|_cx| match l_data {
                    LayerData::Raster {
                        name,
                        visible,
                        opacity,
                        blend_mode,
                        tiles,
                    } => Layer::Raster(RasterLayer {
                        name,
                        visible,
                        opacity,
                        blend_mode,
                        tiles,
                        render_cache: std::collections::HashMap::new(),
                        pending_textures: std::collections::HashSet::new(),
                    }),
                })
            })
            .collect();

        Self {
            size,
            layers,
            active_layer_index: 0,
            transform: data.transform,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            composited_cache: std::collections::HashMap::new(),
            pending_composited_tiles: std::collections::HashMap::new(),
            dirty_composited_tiles: std::collections::HashSet::new(),
            stroke_composited_cache: std::collections::HashMap::new(),
            cache_version: 0,
        }
    }

    pub fn to_data(&self, cx: &App) -> DocumentData {
        let layers = self
            .layers
            .iter()
            .map(|l_entity| match l_entity.read(cx) {
                Layer::Raster(r) => LayerData::Raster {
                    name: r.name.clone(),
                    visible: r.visible,
                    opacity: r.opacity,
                    blend_mode: r.blend_mode,
                    tiles: r.tiles.clone(),
                },
            })
            .collect();

        DocumentData {
            width: self.size.width,
            height: self.size.height,
            layers,
            transform: self.transform,
        }
    }

    pub fn add_layer(&mut self, name: &str, cx: &mut App) {
        let new_layer = cx.new(|_cx| Layer::Raster(RasterLayer::new(self.size, name)));
        self.layers.insert(0, new_layer.clone());
        let layer_data = match new_layer.read(cx) {
            Layer::Raster(r) => LayerData::Raster {
                name: r.name.clone(),
                visible: r.visible,
                opacity: r.opacity,
                blend_mode: r.blend_mode,
                tiles: r.tiles.clone(),
            },
        };
        self.undo_stack.push(Action::AddLayer {
            index: 0,
            layer_data,
        });
        self.redo_stack.clear();
        self.active_layer_index = 0;
        self.clear_composited_cache();
    }

    pub fn delete_layer(&mut self, index: usize, cx: &App) {
        if self.layers.len() > 1 {
            let layer_entity = self.layers.remove(index);
            let layer_data = match layer_entity.read(cx) {
                Layer::Raster(r) => LayerData::Raster {
                    name: r.name.clone(),
                    visible: r.visible,
                    opacity: r.opacity,
                    blend_mode: r.blend_mode,
                    tiles: r.tiles.clone(),
                },
            };
            self.undo_stack
                .push(Action::DeleteLayer { index, layer_data });
            self.redo_stack.clear();
            if self.active_layer_index >= self.layers.len() {
                self.active_layer_index = self.layers.len() - 1;
            }
            self.clear_composited_cache();
        }
    }

    pub fn toggle_visibility(&mut self, index: usize, cx: &mut App) {
        if let Some(layer_entity) = self.layers.get(index) {
            let before = layer_entity.read(cx).visible();
            layer_entity.update(cx, |layer, cx| {
                match layer {
                    Layer::Raster(r) => r.visible = !r.visible,
                }
                cx.notify();
            });
            self.undo_stack
                .push(Action::ToggleVisibility { index, before });
            self.redo_stack.clear();
            self.clear_composited_cache();
        }
    }

    pub fn set_opacity(&mut self, index: usize, opacity: f32, cx: &mut App) {
        if let Some(layer_entity) = self.layers.get(index) {
            let before = layer_entity.read(cx).opacity();
            layer_entity.update(cx, |layer, cx| {
                match layer {
                    Layer::Raster(r) => r.opacity = opacity,
                }
                cx.notify();
            });
            self.undo_stack.push(Action::SetOpacity {
                index,
                before,
                after: opacity,
            });
            self.redo_stack.clear();
            self.clear_composited_cache();
        }
    }

    pub fn move_layer(&mut self, from: usize, to: usize) {
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        self.undo_stack.push(Action::MoveLayer { from, to });
        self.redo_stack.clear();
        self.active_layer_index = to;
        self.clear_composited_cache();
    }

    pub fn undo(&mut self, cx: &mut App) {
        if let Some(action) = self.undo_stack.pop() {
            match action.clone() {
                Action::Paint {
                    layer_index,
                    changed_tiles,
                } => {
                    if let Some(layer_entity) = self.layers.get(layer_index) {
                        layer_entity.update(cx, |layer, cx| {
                            let Layer::Raster(raster) = layer;
                            for (coords, (before_tile, _)) in &changed_tiles {
                                if let Some(tile) = before_tile {
                                    raster.tiles.tiles.insert(*coords, tile.clone());
                                } else {
                                    raster.tiles.tiles.remove(coords);
                                }
                                raster.render_cache.remove(coords);
                            }
                            cx.notify();
                        });
                    }
                    for coords in changed_tiles.keys() {
                        self.dirty_composited_tiles.insert(*coords);
                        self.pending_composited_tiles.remove(coords);
                        self.composited_cache.remove(coords);
                    }
                }
                Action::AddLayer { index, .. } => {
                    self.layers.remove(index);
                    if self.active_layer_index >= self.layers.len() {
                        self.active_layer_index = self.layers.len() - 1;
                    }
                    self.clear_composited_cache();
                }
                Action::DeleteLayer { index, layer_data } => {
                    let layer = cx.new(|_cx| match layer_data {
                        LayerData::Raster {
                            name,
                            visible,
                            opacity,
                            blend_mode,
                            tiles,
                        } => Layer::Raster(RasterLayer {
                            name,
                            visible,
                            opacity,
                            blend_mode,
                            tiles,
                            render_cache: std::collections::HashMap::new(),
                            pending_textures: std::collections::HashSet::new(),
                        }),
                    });
                    self.layers.insert(index, layer);
                    self.active_layer_index = index;
                    self.clear_composited_cache();
                }
                Action::MoveLayer { from, to } => {
                    let layer = self.layers.remove(to);
                    self.layers.insert(from, layer);
                    self.active_layer_index = from;
                    self.clear_composited_cache();
                }
                Action::ToggleVisibility { index, before } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.visible = before,
                            }
                            cx.notify();
                        });
                    }
                    self.clear_composited_cache();
                }
                Action::SetOpacity { index, before, .. } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.opacity = before,
                            }
                            cx.notify();
                        });
                    }
                    self.clear_composited_cache();
                }
            }
            self.redo_stack.push(action);
        }
    }

    pub fn redo(&mut self, cx: &mut App) {
        if let Some(action) = self.redo_stack.pop() {
            match action.clone() {
                Action::Paint {
                    layer_index,
                    changed_tiles,
                } => {
                    if let Some(layer_entity) = self.layers.get(layer_index) {
                        layer_entity.update(cx, |layer, cx| {
                            let Layer::Raster(raster) = layer;
                            for (coords, (_, after_tile)) in &changed_tiles {
                                if let Some(tile) = after_tile {
                                    raster.tiles.tiles.insert(*coords, tile.clone());
                                } else {
                                    raster.tiles.tiles.remove(coords);
                                }
                                raster.render_cache.remove(coords);
                            }
                            cx.notify();
                        });
                    }
                    for coords in changed_tiles.keys() {
                        self.dirty_composited_tiles.insert(*coords);
                        self.pending_composited_tiles.remove(coords);
                        self.composited_cache.remove(coords);
                    }
                }
                Action::AddLayer { index, layer_data } => {
                    let layer = cx.new(|_cx| match layer_data {
                        LayerData::Raster {
                            name,
                            visible,
                            opacity,
                            blend_mode,
                            tiles,
                        } => Layer::Raster(RasterLayer {
                            name,
                            visible,
                            opacity,
                            blend_mode,
                            tiles,
                            render_cache: std::collections::HashMap::new(),
                            pending_textures: std::collections::HashSet::new(),
                        }),
                    });
                    self.layers.insert(index, layer);
                    self.active_layer_index = index;
                    self.clear_composited_cache();
                }
                Action::DeleteLayer { index, .. } => {
                    self.layers.remove(index);
                    if self.active_layer_index >= self.layers.len() {
                        self.active_layer_index = self.layers.len() - 1;
                    }
                    self.clear_composited_cache();
                }
                Action::MoveLayer { from, to } => {
                    let layer = self.layers.remove(from);
                    self.layers.insert(to, layer);
                    self.active_layer_index = to;
                    self.clear_composited_cache();
                }
                Action::ToggleVisibility { index, before } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.visible = !before,
                            }
                            cx.notify();
                        });
                    }
                    self.clear_composited_cache();
                }
                Action::SetOpacity { index, after, .. } => {
                    if let Some(layer_entity) = self.layers.get(index) {
                        layer_entity.update(cx, |layer, cx| {
                            match layer {
                                Layer::Raster(r) => r.opacity = after,
                            }
                            cx.notify();
                        });
                    }
                    self.clear_composited_cache();
                }
            }
            self.undo_stack.push(action);
        }
    }

    pub fn active_layer(&self) -> Option<&Entity<Layer>> {
        self.layers.get(self.active_layer_index)
    }
}

pub enum Layer {
    Raster(RasterLayer),
}

impl Layer {
    pub fn visible(&self) -> bool {
        match self {
            Layer::Raster(r) => r.visible,
        }
    }

    pub fn opacity(&self) -> f32 {
        match self {
            Layer::Raster(r) => r.opacity,
        }
    }

    #[allow(dead_code)]
    pub fn resolve_texture(
        &mut self,
        coords: crate::tile::TileCoords,
        cx: &mut App,
        layer_weak: &WeakEntity<Self>,
    ) -> Option<Arc<RenderImage>> {
        match self {
            Layer::Raster(r) => {
                if let Some(img) = r.render_cache.get(&coords) {
                    return Some(img.clone());
                }

                if r.pending_textures.contains(&coords) {
                    return None;
                }

                if let Some(tile) = r.tiles.tiles.get(&coords).cloned() {
                    r.pending_textures.insert(coords);
                    let layer_weak = layer_weak.clone();
                    cx.spawn(move |cx: &mut AsyncApp| {
                        let mut cx = cx.clone();
                        async move {
                            let render_image = cx
                                .background_spawn(async move { tile.build_render_image() })
                                .await;

                            let _ = layer_weak.update(
                                &mut cx,
                                |layer: &mut Layer, cx: &mut Context<Layer>| {
                                    let Layer::Raster(r) = layer;
                                    r.render_cache.insert(coords, render_image);
                                    r.pending_textures.remove(&coords);
                                    cx.notify();
                                },
                            );
                        }
                    })
                    .detach();
                }

                None
            }
        }
    }

    pub fn resolve_thumbnail(
        &mut self,
        coords: crate::tile::TileCoords,
        cx: &mut App,
        layer_weak: &WeakEntity<Self>,
    ) -> Option<Arc<RenderImage>> {
        match self {
            Layer::Raster(r) => {
                if let Some(img) = r.render_cache.get(&coords) {
                    return Some(img.clone());
                }

                if r.pending_textures.contains(&coords) {
                    return None;
                }

                if let Some(tile) = r.tiles.tiles.get(&coords).cloned() {
                    r.pending_textures.insert(coords);
                    let layer_weak = layer_weak.clone();
                    cx.spawn(move |cx: &mut AsyncApp| {
                        let mut cx = cx.clone();
                        async move {
                            let render_image = cx
                                .background_spawn(async move { tile.build_render_image() })
                                .await;

                            let _ = layer_weak.update(
                                &mut cx,
                                |layer: &mut Layer, cx: &mut Context<Layer>| {
                                    let Layer::Raster(r) = layer;
                                    r.render_cache.insert(coords, render_image);
                                    r.pending_textures.remove(&coords);
                                    cx.notify();
                                },
                            );
                        }
                    })
                    .detach();
                }

                None
            }
        }
    }

    pub fn tile_keys(&self) -> Vec<crate::tile::TileCoords> {
        match self {
            Layer::Raster(r) => r.tiles.tiles.keys().cloned().collect(),
        }
    }
}

pub struct RasterLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: crate::blend::BlendMode,
    pub tiles: crate::tile::TileGrid,
    pub render_cache: std::collections::HashMap<crate::tile::TileCoords, Arc<RenderImage>>,
    pub pending_textures: std::collections::HashSet<crate::tile::TileCoords>,
}

impl RasterLayer {
    pub fn new(size: Size<u32>, name: &str) -> Self {
        Self {
            name: name.to_string(),
            visible: true,
            opacity: 1.0,
            blend_mode: crate::blend::BlendMode::Normal,
            tiles: crate::tile::TileGrid::new(size.width, size.height),
            render_cache: std::collections::HashMap::new(),
            pending_textures: std::collections::HashSet::new(),
        }
    }
}
