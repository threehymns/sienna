use crate::document::Document;
use gpui::{Size, TestAppContext};

#[gpui::test]
fn test_document_add_layer(cx: &mut TestAppContext) {
    let size = Size {
        width: 100,
        height: 100,
    };
    let mut doc = cx.update(|cx| Document::new(size, cx));

    assert_eq!(doc.layers.len(), 1);
    assert_eq!(doc.active_layer_index, 0);

    cx.update(|cx| doc.add_layer("Layer 2", cx));

    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.active_layer_index, 0);

    // Check if undo works
    cx.update(|cx| doc.undo(cx));
    assert_eq!(doc.layers.len(), 1);

    // Check if redo works
    cx.update(|cx| doc.redo(cx));
    assert_eq!(doc.layers.len(), 2);
}

#[gpui::test]
fn test_document_delete_layer(cx: &mut TestAppContext) {
    let size = Size {
        width: 100,
        height: 100,
    };
    let mut doc = cx.update(|cx| Document::new(size, cx));

    cx.update(|cx| doc.add_layer("Layer 2", cx));
    assert_eq!(doc.layers.len(), 2);

    cx.update(|cx| doc.delete_layer(0, cx));
    assert_eq!(doc.layers.len(), 1);
    assert_eq!(doc.active_layer_index, 0); // Active layer index clamped

    // Undo deletion
    cx.update(|cx| doc.undo(cx));
    assert_eq!(doc.layers.len(), 2);

    // Redo deletion
    cx.update(|cx| doc.redo(cx));
    assert_eq!(doc.layers.len(), 1);
}

#[gpui::test]
fn test_document_move_layer(cx: &mut TestAppContext) {
    let size = Size {
        width: 100,
        height: 100,
    };
    let mut doc = cx.update(|cx| Document::new(size, cx));

    cx.update(|cx| {
        doc.add_layer("Layer 2", cx);
        doc.add_layer("Layer 3", cx);
    });
    assert_eq!(doc.layers.len(), 3);

    doc.move_layer(0, 2);
    assert_eq!(doc.active_layer_index, 2);

    cx.update(|cx| doc.undo(cx));
    assert_eq!(doc.active_layer_index, 0);

    cx.update(|cx| doc.redo(cx));
    assert_eq!(doc.active_layer_index, 2);
}

#[gpui::test]
fn test_document_toggle_visibility(cx: &mut TestAppContext) {
    let size = Size {
        width: 100,
        height: 100,
    };
    let mut doc = cx.update(|cx| Document::new(size, cx));
    let layer = doc.layers[0].clone();

    assert!(cx.update(|cx| layer.read(cx).visible()));

    cx.update(|cx| doc.toggle_visibility(0, cx));
    assert!(!cx.update(|cx| layer.read(cx).visible()));

    cx.update(|cx| doc.undo(cx));
    assert!(cx.update(|cx| layer.read(cx).visible()));

    cx.update(|cx| doc.redo(cx));
    assert!(!cx.update(|cx| layer.read(cx).visible()));
}

#[gpui::test]
fn test_document_set_opacity(cx: &mut TestAppContext) {
    let size = Size {
        width: 100,
        height: 100,
    };
    let mut doc = cx.update(|cx| Document::new(size, cx));
    let layer = doc.layers[0].clone();

    assert_eq!(cx.update(|cx| layer.read(cx).opacity()), 1.0);

    cx.update(|cx| doc.set_opacity(0, 0.5, cx));
    assert_eq!(cx.update(|cx| layer.read(cx).opacity()), 0.5);

    cx.update(|cx| doc.undo(cx));
    assert_eq!(cx.update(|cx| layer.read(cx).opacity()), 1.0);

    cx.update(|cx| doc.redo(cx));
    assert_eq!(cx.update(|cx| layer.read(cx).opacity()), 0.5);
}

#[gpui::test]
fn test_stroke_performance(cx: &mut TestAppContext) {
    let size = Size {
        width: 4096,
        height: 4096,
    };
    let _doc = cx.update(|cx| Document::new(size, cx));

    let start = std::time::Instant::now();
    let mut feed_duration = std::time::Duration::ZERO;
    let mut build_duration = std::time::Duration::ZERO;
    let mut build_count = 0;

    cx.update(|_cx| {
        let mut accumulator = crate::stroke::StrokeAccumulator::begin(
            size.width,
            size.height,
            crate::tile::TileGrid::new(size.width, size.height),
            20.0, // brush_size
            1.0,  // brush_opacity
            1.0,  // brush_flow
            0.5,  // brush_hardness
            0.1,  // brush_spacing
            0.0,  // brush_stabilization
            gpui::Rgba::default(),
            false,
        );
        // Feed 100 points
        for i in 0..100 {
            let pos = gpui::Point {
                x: 10.0 + i as f32 * 5.0,
                y: 10.0 + i as f32 * 5.0,
            };
            let feed_start = std::time::Instant::now();
            let dirty_tiles = accumulator.feed(pos);
            feed_duration += feed_start.elapsed();

            if !dirty_tiles.is_empty() {
                let build_start = std::time::Instant::now();
                for coords in &dirty_tiles {
                    if let Some(tile) = accumulator.stroke_buffer.composited.tiles.get(coords) {
                        tile.build_render_image();
                        build_count += 1;
                    }
                }
                build_duration += build_start.elapsed();
            }
        }
    });

    let elapsed = start.elapsed();
    println!("PERF_RESULT: Stroke of 100 dabs took {:?}", elapsed);
    println!("PERF_RESULT: feed() took {:?}", feed_duration);
    println!(
        "PERF_RESULT: build_render_image() for {} tiles took {:?}",
        build_count, build_duration
    );

    let avg_feed = feed_duration / 100;
    let limit = if cfg!(debug_assertions) {
        std::time::Duration::from_millis(5)
    } else {
        std::time::Duration::from_millis(1)
    };
    assert!(
        avg_feed < limit,
        "Average feed time per dab must be under target ({:?}), got {:?}",
        limit,
        avg_feed
    );

    if build_count > 0 {
        let avg_build = build_duration / build_count;
        let build_limit = if cfg!(debug_assertions) {
            std::time::Duration::from_millis(5)
        } else {
            std::time::Duration::from_millis(1)
        };
        assert!(
            avg_build < build_limit,
            "Average texture build time per dirty tile must be under target ({:?}), got {:?}",
            build_limit,
            avg_build
        );
    }
}

#[test]
fn test_tile_non_transparent_bounds() {
    use crate::tile::Tile;

    let mut tile = Tile::new();
    assert_eq!(tile.non_transparent_bounds, None);

    // Set a pixel in the tile at (10, 20) with alpha > 0
    let idx = ((20 * crate::tile::TILE_SIZE + 10) * 4) as usize;
    tile.pixels[idx + 3] = 255;
    tile.update_bounds();

    assert_eq!(tile.non_transparent_bounds, Some((10, 10, 20, 20)));

    // Set another pixel at (50, 100)
    let idx2 = ((100 * crate::tile::TILE_SIZE + 50) * 4) as usize;
    tile.pixels[idx2 + 3] = 128;
    tile.update_bounds();

    assert_eq!(tile.non_transparent_bounds, Some((10, 50, 20, 100)));

    // Clear the pixels (make them transparent again)
    tile.pixels.fill(0);
    tile.update_bounds();
    assert_eq!(tile.non_transparent_bounds, None);
}
