use gpui::*;

pub mod blend;
mod brush;
mod canvas;
mod document;
pub mod geom;
mod project_modal;
mod sidebar;
mod stroke;
mod tile;
mod tool;
mod ui_components;
mod workspace;

#[cfg(test)]
mod document_test;
#[cfg(test)]
mod geom_test;

#[cfg(test)]
mod blend_test;

use document::Document;
use gpui_component::{Root, Theme, ThemeMode};
use tool::ToolState;
use workspace::{NewProject, Redo, Undo, Workspace};

struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>, anyhow::Error> {
        match path {
            "icons/eye.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/eye.svg"
            )))),
            "icons/eye-slash.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/eye-slash.svg"
            )))),
            "icons/move.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/move.svg"
            )))),
            "icons/brush.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/brush.svg"
            )))),
            "icons/eraser.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/eraser.svg"
            )))),
            "icons/eyedropper.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/eyedropper.svg"
            )))),
            "icons/plus.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/plus.svg"
            )))),
            "icons/minus.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/minus.svg"
            )))),
            "icons/chevron-up.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/chevron-up.svg"
            )))),
            "icons/chevron-down.svg" => Ok(Some(std::borrow::Cow::Borrowed(include_bytes!(
                "../icons/chevron-down.svg"
            )))),
            _ => Ok(None),
        }
    }

    fn list(&self, _path: &str) -> Result<Vec<gpui::SharedString>, anyhow::Error> {
        Ok(vec![
            "icons/eye.svg".into(),
            "icons/eye-slash.svg".into(),
            "icons/move.svg".into(),
            "icons/brush.svg".into(),
            "icons/eraser.svg".into(),
            "icons/eyedropper.svg".into(),
            "icons/plus.svg".into(),
            "icons/minus.svg".into(),
            "icons/chevron-up.svg".into(),
            "icons/chevron-down.svg".into(),
        ])
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);

            cx.bind_keys([
                KeyBinding::new("cmd-z", Undo, None),
                KeyBinding::new("cmd-shift-z", Redo, None),
                KeyBinding::new("cmd-n", NewProject, None),
            ]);

            let document = cx.new(|cx| {
                Document::new(
                    Size {
                        width: 1024,
                        height: 768,
                    },
                    cx,
                )
            });
            let tool_state = cx.new(|_cx| ToolState::new());

            cx.spawn(async move |cx| {
                cx.open_window(WindowOptions::default(), |window, cx| {
                    let view = cx.new(|cx| Workspace::new(document, tool_state, window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open window");
            })
            .detach();
        });
}
