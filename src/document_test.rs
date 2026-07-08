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
fn test_stroke_performance(cx: &mut TestAppContext) {
    let size = Size {
        width: 1024,
        height: 768,
    };
    let _doc = cx.update(|cx| Document::new(size, cx));
    let initial_pixels = vec![0u8; (size.width * size.height * 4) as usize];

    let start = std::time::Instant::now();
    let mut feed_duration = std::time::Duration::ZERO;
    let mut build_duration = std::time::Duration::ZERO;

    cx.update(|_cx| {
        let mut accumulator = crate::stroke::StrokeAccumulator::begin(
            size.width,
            size.height,
            initial_pixels.clone(),
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
            let placed = accumulator.feed(pos);
            feed_duration += feed_start.elapsed();

            if placed {
                let build_start = std::time::Instant::now();
                accumulator.stroke_buffer.build_render_image();
                build_duration += build_start.elapsed();
            }
        }
    });

    let elapsed = start.elapsed();
    println!("PERF_RESULT: Stroke of 100 dabs took {:?}", elapsed);
    println!("PERF_RESULT: feed() took {:?}", feed_duration);
    println!(
        "PERF_RESULT: build_render_image() took {:?}",
        build_duration
    );
}
