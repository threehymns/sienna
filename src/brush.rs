/// Stateless brush engine — provides optimized dab rendering functions.
/// All methods are static and operate directly on pixel buffers.
/// Pixels are expected in BGRA format.
pub struct BrushEngine;

impl BrushEngine {
    #[inline]
    fn for_each_dab_pixel<F>(
        grid: &mut crate::tile::TileGrid,
        dab: &crate::stroke::DabParams,
        mut f: F,
    ) where
        F: FnMut(&mut crate::tile::Tile, usize, f32),
    {
        let pos = dab.position;
        let size = dab.size;
        let hardness = dab.hardness;
        let strength = dab.opacity * dab.flow;

        let radius = size / 2.0;
        let radius_sq = radius * radius;
        let falloff_start = radius * hardness;
        let falloff_start_sq = falloff_start * falloff_start;
        let falloff_range_inv = 1.0 / (radius - falloff_start).max(0.001);

        // Fast path check
        let inner_radius = (radius - 1.0).max(0.0);
        let inner_radius_sq = inner_radius * inner_radius;
        let fast_limit_sq = falloff_start_sq.min(inner_radius_sq);

        // Pre-clamp to canvas bounds
        let x_start = (pos.x - radius).floor().max(0.0) as u32;
        let y_start = (pos.y - radius).floor().max(0.0) as u32;
        let x_end = (pos.x + radius).ceil().min(grid.width as f32) as u32;
        let y_end = (pos.y + radius).ceil().min(grid.height as f32) as u32;

        let tx_start = x_start / crate::tile::TILE_SIZE;
        let ty_start = y_start / crate::tile::TILE_SIZE;
        let tx_end = x_end.div_ceil(crate::tile::TILE_SIZE);
        let ty_end = y_end.div_ceil(crate::tile::TILE_SIZE);

        for ty in ty_start..ty_end {
            for tx in tx_start..tx_end {
                let x_tile_start = tx * crate::tile::TILE_SIZE;
                let y_tile_start = ty * crate::tile::TILE_SIZE;
                let x_tile_end = (x_tile_start + crate::tile::TILE_SIZE).min(grid.width);
                let y_tile_end = (y_tile_start + crate::tile::TILE_SIZE).min(grid.height);

                let inter_x_start = x_start.max(x_tile_start);
                let inter_y_start = y_start.max(y_tile_start);
                let inter_x_end = x_end.min(x_tile_end);
                let inter_y_end = y_end.min(y_tile_end);

                if inter_x_start >= inter_x_end || inter_y_start >= inter_y_end {
                    continue;
                }

                let tile = grid
                    .tiles
                    .entry(crate::tile::TileCoords::new(tx, ty))
                    .or_insert_with(crate::tile::Tile::new);

                for y in inter_y_start..inter_y_end {
                    let tile_y = y - y_tile_start;
                    let row_base = (tile_y * crate::tile::TILE_SIZE) as usize * 4;
                    let dy = y as f32 + 0.5 - pos.y;
                    let dy_sq = dy * dy;

                    for x in inter_x_start..inter_x_end {
                        let dx = x as f32 + 0.5 - pos.x;
                        let dist_sq = dx * dx + dy_sq;

                        if dist_sq > radius_sq {
                            continue;
                        }

                        let tile_x = x - x_tile_start;
                        let idx = row_base + (tile_x as usize) * 4;

                        let alpha = if dist_sq <= fast_limit_sq {
                            strength
                        } else {
                            let dist = dist_sq.sqrt();
                            let intensity = if dist_sq <= falloff_start_sq {
                                1.0
                            } else {
                                let t = (dist - falloff_start) * falloff_range_inv;
                                (1.0 - t).max(0.0)
                            };
                            let edge_aa = (radius - dist).clamp(0.0, 1.0);
                            strength * intensity * edge_aa
                        };

                        if alpha > 0.0 {
                            f(tile, idx, alpha);
                        }
                    }
                }
            }
        }
    }

    /// Draw a brush dab at `pos` into the pixel buffer.
    /// Uses a max-alpha compositing model within a single stroke:
    /// each dab's contribution is max'd with existing stroke alpha,
    /// preventing darkening on overlapping dabs (like Photoshop's flow behavior).
    #[inline]
    pub fn draw_dab(grid: &mut crate::tile::TileGrid, dab: &crate::stroke::DabParams) {
        let b_src = (dab.color.b * 255.0) as u8;
        let g_src = (dab.color.g * 255.0) as u8;
        let r_src = (dab.color.r * 255.0) as u8;

        Self::for_each_dab_pixel(grid, dab, |tile, idx, dab_alpha| {
            tile.blend_max_alpha(idx, b_src, g_src, r_src, dab_alpha);
        });
    }

    /// Erase dab: writes alpha into the stroke buffer that will be used
    /// to reduce the layer alpha during compositing.
    #[inline]
    pub fn erase_dab(grid: &mut crate::tile::TileGrid, dab: &crate::stroke::DabParams) {
        Self::for_each_dab_pixel(grid, dab, |tile, idx, erase_alpha| {
            tile.erase_max_alpha(idx, erase_alpha);
        });
    }
}
