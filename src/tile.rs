use gpui::RenderImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub const TILE_SIZE: u32 = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tile {
    pub pixels: Vec<u8>, // Size is TILE_SIZE * TILE_SIZE * 4 (BGRA)
    #[serde(default)]
    pub non_transparent_bounds: Option<(u32, u32, u32, u32)>,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; (TILE_SIZE * TILE_SIZE * 4) as usize],
            non_transparent_bounds: None,
        }
    }

    pub fn update_bounds(&mut self) {
        let mut min_x = TILE_SIZE;
        let mut max_x = 0;
        let mut min_y = TILE_SIZE;
        let mut max_y = 0;
        let mut has_content = false;

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let idx = ((y * TILE_SIZE + x) * 4) as usize;
                if idx + 3 < self.pixels.len() && self.pixels[idx + 3] > 0 {
                    has_content = true;
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }
        }

        if has_content {
            self.non_transparent_bounds = Some((min_x, max_x, min_y, max_y));
        } else {
            self.non_transparent_bounds = None;
        }
    }

    #[inline]
    pub fn blend_max_alpha(&mut self, idx: usize, b_src: u8, g_src: u8, r_src: u8, dab_alpha: f32) {
        let new_a = (dab_alpha * 255.0).min(255.0) as u8;
        let existing_a = self.pixels[idx + 3];

        if new_a > existing_a {
            self.pixels[idx] = b_src;
            self.pixels[idx + 1] = g_src;
            self.pixels[idx + 2] = r_src;
            self.pixels[idx + 3] = new_a;
        }
    }

    #[inline]
    pub fn erase_max_alpha(&mut self, idx: usize, erase_alpha: f32) {
        let new_val = (erase_alpha * 255.0).min(255.0) as u8;
        let existing = self.pixels[idx + 3];
        if new_val > existing {
            self.pixels[idx + 3] = new_val;
        }
    }

    pub fn composite_stroke(
        &mut self,
        stroke_tile: &Tile,
        snapshot_tile: Option<&Tile>,
        is_eraser: bool,
    ) {
        if let Some(snap) = snapshot_tile {
            self.pixels.copy_from_slice(&snap.pixels);
        } else {
            self.pixels.fill(0);
        }

        for idx in (0..(TILE_SIZE * TILE_SIZE * 4) as usize).step_by(4) {
            let stroke_a = stroke_tile.pixels[idx + 3] as u32;
            if stroke_a == 0 {
                continue;
            }

            if is_eraser {
                let bg_a = self.pixels[idx + 3] as u32;
                let new_a = (bg_a * (255 - stroke_a)) / 255;
                self.pixels[idx + 3] = new_a as u8;
            } else {
                let bg_b = self.pixels[idx] as u32;
                let bg_g = self.pixels[idx + 1] as u32;
                let bg_r = self.pixels[idx + 2] as u32;
                let bg_a = self.pixels[idx + 3] as u32;

                let one_minus_fg_a = 255 - stroke_a;
                let bg_a_blend = (bg_a * one_minus_fg_a) / 255;
                let out_a = stroke_a + bg_a_blend;

                if out_a > 0 {
                    let fg_b = stroke_tile.pixels[idx] as u32;
                    let fg_g = stroke_tile.pixels[idx + 1] as u32;
                    let fg_r = stroke_tile.pixels[idx + 2] as u32;

                    self.pixels[idx] = ((fg_b * stroke_a + bg_b * bg_a_blend) / out_a) as u8;
                    self.pixels[idx + 1] = ((fg_g * stroke_a + bg_g * bg_a_blend) / out_a) as u8;
                    self.pixels[idx + 2] = ((fg_r * stroke_a + bg_r * bg_a_blend) / out_a) as u8;
                    self.pixels[idx + 3] = out_a as u8;
                } else {
                    self.pixels[idx] = 0;
                    self.pixels[idx + 1] = 0;
                    self.pixels[idx + 2] = 0;
                    self.pixels[idx + 3] = 0;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn composite_layer(
        &mut self,
        source: &Tile,
        blend_mode: crate::blend::BlendMode,
        opacity: f32,
    ) {
        if opacity <= 0.0 {
            return;
        }

        for idx in (0..(TILE_SIZE * TILE_SIZE * 4) as usize).step_by(4) {
            let fg_a = source.pixels[idx + 3];
            if fg_a == 0 {
                continue;
            }

            let bg = (
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            );

            let fg = (
                source.pixels[idx],
                source.pixels[idx + 1],
                source.pixels[idx + 2],
                fg_a,
            );

            let out = crate::blend::composite_pixel(bg, fg, blend_mode, opacity);

            self.pixels[idx] = out.0;
            self.pixels[idx + 1] = out.1;
            self.pixels[idx + 2] = out.2;
            self.pixels[idx + 3] = out.3;
        }
    }

    pub fn build_render_image(&self) -> Arc<RenderImage> {
        let buffer = image::RgbaImage::from_raw(TILE_SIZE, TILE_SIZE, self.pixels.clone()).unwrap();
        let frame = image::Frame::new(buffer);
        Arc::new(RenderImage::new(smallvec::smallvec![frame]))
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileCoords {
    pub x: u32,
    pub y: u32,
}

impl TileCoords {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl From<(u32, u32)> for TileCoords {
    fn from(tuple: (u32, u32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<TileCoords> for (u32, u32) {
    fn from(coords: TileCoords) -> Self {
        (coords.x, coords.y)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TileGrid {
    pub width: u32,
    pub height: u32,
    pub tiles: HashMap<TileCoords, Tile>,
}

impl TileGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
        }
    }

    pub fn tile_coords(x: u32, y: u32) -> TileCoords {
        TileCoords::new(x / TILE_SIZE, y / TILE_SIZE)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let coords = Self::tile_coords(x, y);
        if let Some(tile) = self.tiles.get(&coords) {
            let px = x % TILE_SIZE;
            let py = y % TILE_SIZE;
            let idx = ((py * TILE_SIZE + px) * 4) as usize;
            [
                tile.pixels[idx],
                tile.pixels[idx + 1],
                tile.pixels[idx + 2],
                tile.pixels[idx + 3],
            ]
        } else {
            [0, 0, 0, 0]
        }
    }

    #[allow(dead_code)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let coords = Self::tile_coords(x, y);
        let tile = self.tiles.entry(coords).or_insert_with(Tile::new);
        let px = x % TILE_SIZE;
        let py = y % TILE_SIZE;
        let idx = ((py * TILE_SIZE + px) * 4) as usize;
        tile.pixels[idx] = color[0];
        tile.pixels[idx + 1] = color[1];
        tile.pixels[idx + 2] = color[2];
        tile.pixels[idx + 3] = color[3];
        tile.update_bounds();
    }

    pub fn to_monolithic(&self) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut pixels = vec![0; size];
        for (&coords, tile) in &self.tiles {
            let start_x = coords.x * TILE_SIZE;
            let start_y = coords.y * TILE_SIZE;
            for row in 0..TILE_SIZE {
                let y = start_y + row;
                if y >= self.height {
                    break;
                }
                let dest_row_offset = (y * self.width) as usize * 4;
                let src_row_offset = (row * TILE_SIZE) as usize * 4;
                for col in 0..TILE_SIZE {
                    let x = start_x + col;
                    if x >= self.width {
                        break;
                    }
                    let dest_idx = dest_row_offset + (x as usize) * 4;
                    let src_idx = src_row_offset + (col as usize) * 4;
                    pixels[dest_idx..dest_idx + 4]
                        .copy_from_slice(&tile.pixels[src_idx..src_idx + 4]);
                }
            }
        }
        pixels
    }

    pub fn from_monolithic(width: u32, height: u32, pixels: &[u8]) -> Self {
        let mut grid = Self::new(width, height);
        let cols = width.div_ceil(TILE_SIZE);
        let rows = height.div_ceil(TILE_SIZE);

        for ty in 0..rows {
            for tx in 0..cols {
                let mut tile = Tile::new();
                let start_x = tx * TILE_SIZE;
                let start_y = ty * TILE_SIZE;
                let mut has_any_content = false;

                for row in 0..TILE_SIZE {
                    let y = start_y + row;
                    if y >= height {
                        break;
                    }
                    let src_row_offset = (y * width) as usize * 4;
                    let dest_row_offset = (row * TILE_SIZE) as usize * 4;
                    for col in 0..TILE_SIZE {
                        let x = start_x + col;
                        if x >= width {
                            break;
                        }
                        let src_idx = src_row_offset + (x as usize) * 4;
                        let dest_idx = dest_row_offset + (col as usize) * 4;
                        let pixel = &pixels[src_idx..src_idx + 4];
                        if pixel[3] > 0 || pixel[0] > 0 || pixel[1] > 0 || pixel[2] > 0 {
                            has_any_content = true;
                        }
                        tile.pixels[dest_idx..dest_idx + 4].copy_from_slice(pixel);
                    }
                }

                if has_any_content {
                    tile.update_bounds();
                    grid.tiles.insert(TileCoords::new(tx, ty), tile);
                }
            }
        }
        grid
    }

    pub fn swap_rb_channels(&mut self) {
        for tile in self.tiles.values_mut() {
            for chunk in tile.pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }
        }
    }

    pub fn delta(before: &Self, after: &Self) -> HashMap<TileCoords, (Option<Tile>, Option<Tile>)> {
        let mut diff = HashMap::new();
        let all_keys: std::collections::HashSet<_> = before
            .tiles
            .keys()
            .chain(after.tiles.keys())
            .copied()
            .collect();
        for coords in all_keys {
            let b_tile = before.tiles.get(&coords);
            let a_tile = after.tiles.get(&coords);
            if b_tile != a_tile {
                diff.insert(coords, (b_tile.cloned(), a_tile.cloned()));
            }
        }
        diff
    }
}
