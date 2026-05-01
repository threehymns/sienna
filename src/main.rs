use gpui::*;

mod workspace;
mod document;
mod tool;
mod canvas;

use workspace::{Workspace, Undo, Redo, NewProject};
use document::Document;
use tool::ToolState;
use gpui_component::{Root, Theme, ThemeMode};

fn main() {
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        cx.bind_keys([
            KeyBinding::new("cmd-z", Undo, None),
            KeyBinding::new("cmd-shift-z", Redo, None),
            KeyBinding::new("cmd-n", NewProject, None),
        ]);

        let document = cx.new(|cx| Document::new(Size { width: 1024, height: 768 }, cx));
        let tool_state = cx.new(|_cx| ToolState::new());

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| Workspace::new(document, tool_state, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            }).expect("failed to open window");
        }).detach();
    });
}
