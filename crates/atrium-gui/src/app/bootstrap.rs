//! Application bootstrap — font loading, menus, window creation.

use gpui::{Application, App, Bounds, WindowBounds, WindowOptions, size, px, prelude::*};

use super::keybindings;
use super::window::AtriumWindow;

const DEFAULT_WIDTH: f32 = 1460.0;
const DEFAULT_HEIGHT: f32 = 900.0;
const MIN_WIDTH: f32 = 1180.0;
const MIN_HEIGHT: f32 = 760.0;

/// Run the GPUI application. This blocks until the window closes.
pub fn run() {
    let application = Application::new();
    application.run(move |cx: &mut App| {
        keybindings::install(cx);
        open_window(cx);
    });
}

fn open_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(DEFAULT_WIDTH), px(DEFAULT_HEIGHT)), cx);
    if let Err(error) = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(MIN_WIDTH), px(MIN_HEIGHT))),
            app_id: Some("dev.atrium.app".to_owned()),
            ..Default::default()
        },
        |_, cx| cx.new(|_cx| AtriumWindow::new()),
    ) {
        tracing::error!(%error, "failed to open Atrium window");
    }
}
