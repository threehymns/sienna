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
