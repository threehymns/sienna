use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use gpui::RenderImage;

pub const TILE_SIZE: u32 = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tile {
    pub pixels: Vec<u8>, // Size is TILE_SIZE * TILE_SIZE * 4 (BGRA)
}

impl Tile {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; (TILE_SIZE * TILE_SIZE * 4) as usize],
        }
    }

    #[inline]
    pub fn blend_max_alpha(&mut self, idx: usize, b_src: u8, g_src: u8, r_src: u8, dab_alpha: f32) {
        let existing_a = self.pixels[idx + 3] as f32 / 255.0;
        let new_a = dab_alpha.min(1.0);

        if new_a > existing_a {
            self.pixels[idx] = b_src;
            self.pixels[idx + 1] = g_src;
            self.pixels[idx + 2] = r_src;
            self.pixels[idx + 3] = (new_a * 255.0) as u8;
        }
    }

    #[inline]
    pub fn erase_max_alpha(&mut self, idx: usize, erase_alpha: f32) {
        let existing = self.pixels[idx + 3] as f32 / 255.0;
        let new_val = erase_alpha.min(1.0);
        if new_val > existing {
            self.pixels[idx + 3] = (new_val * 255.0) as u8;
        }
    }

    pub fn composite_stroke(&mut self, stroke_tile: &Tile, snapshot_tile: Option<&Tile>, is_eraser: bool) {
        for idx in (0..(TILE_SIZE * TILE_SIZE * 4) as usize).step_by(4) {
            let stroke_a = stroke_tile.pixels[idx + 3] as u32;
            if stroke_a == 0 {
                if let Some(snap) = snapshot_tile {
                    self.pixels[idx..idx + 4]
                        .copy_from_slice(&snap.pixels[idx..idx + 4]);
                }
                continue;
            }

            if is_eraser {
                let (bg_b, bg_g, bg_r, bg_a) = if let Some(snap) = snapshot_tile {
                    (
                        snap.pixels[idx],
                        snap.pixels[idx + 1],
                        snap.pixels[idx + 2],
                        snap.pixels[idx + 3] as u32,
                    )
                } else {
                    (0, 0, 0, 0)
                };

                let new_a = (bg_a * (255 - stroke_a)) / 255;
                self.pixels[idx] = bg_b;
                self.pixels[idx + 1] = bg_g;
                self.pixels[idx + 2] = bg_r;
                self.pixels[idx + 3] = new_a as u8;
            } else {
                let (bg_b, bg_g, bg_r, bg_a) = if let Some(snap) = snapshot_tile {
                    (
                        snap.pixels[idx] as u32,
                        snap.pixels[idx + 1] as u32,
                        snap.pixels[idx + 2] as u32,
                        snap.pixels[idx + 3] as u32,
                    )
                } else {
                    (0, 0, 0, 0)
                };

                let one_minus_fg_a = 255 - stroke_a;
                let bg_a_blend = (bg_a * one_minus_fg_a) / 255;
                let out_a = stroke_a + bg_a_blend;

                if out_a > 0 {
                    let fg_b = stroke_tile.pixels[idx] as u32;
                    let fg_g = stroke_tile.pixels[idx + 1] as u32;
                    let fg_r = stroke_tile.pixels[idx + 2] as u32;

                    self.pixels[idx] =
                        ((fg_b * stroke_a + bg_b * bg_a_blend) / out_a) as u8;
                    self.pixels[idx + 1] =
                        ((fg_g * stroke_a + bg_g * bg_a_blend) / out_a) as u8;
                    self.pixels[idx + 2] =
                        ((fg_r * stroke_a + bg_r * bg_a_blend) / out_a) as u8;
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

    pub fn build_render_image(&self) -> Arc<RenderImage> {
        let buffer = image::RgbaImage::from_raw(
            TILE_SIZE,
            TILE_SIZE,
            self.pixels.clone(),
        )
        .unwrap();
        let frame = image::Frame::new(buffer);
        Arc::new(RenderImage::new(smallvec::smallvec![frame]))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TileGrid {
    pub width: u32,
    pub height: u32,
    pub tiles: HashMap<(u32, u32), Tile>,
}

impl TileGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
        }
    }

    pub fn tile_coords(x: u32, y: u32) -> (u32, u32) {
        (x / TILE_SIZE, y / TILE_SIZE)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let (tx, ty) = Self::tile_coords(x, y);
        if let Some(tile) = self.tiles.get(&(tx, ty)) {
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
        let (tx, ty) = Self::tile_coords(x, y);
        let tile = self.tiles.entry((tx, ty)).or_insert_with(Tile::new);
        let px = x % TILE_SIZE;
        let py = y % TILE_SIZE;
        let idx = ((py * TILE_SIZE + px) * 4) as usize;
        tile.pixels[idx] = color[0];
        tile.pixels[idx + 1] = color[1];
        tile.pixels[idx + 2] = color[2];
        tile.pixels[idx + 3] = color[3];
    }

    pub fn to_monolithic(&self) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut pixels = vec![0; size];
        for (&(tx, ty), tile) in &self.tiles {
            let start_x = tx * TILE_SIZE;
            let start_y = ty * TILE_SIZE;
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
                    grid.tiles.insert((tx, ty), tile);
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

    pub fn delta(before: &Self, after: &Self) -> HashMap<(u32, u32), (Option<Tile>, Option<Tile>)> {
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
