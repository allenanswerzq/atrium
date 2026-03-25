//! Application bootstrap — window creation and GUI lifecycle.

use gpui::{Bounds, WindowBounds, WindowOptions, size, px, prelude::*};

use super::keybindings;
use super::window::AtriumWindow;

/// Fraction of screen size used for the default window.
const DEFAULT_SIZE_RATIO: f32 = 0.8;
/// Absolute minimum window width in pixels.
const MIN_WIDTH: f32 = 800.0;
/// Absolute minimum window height in pixels.
const MIN_HEIGHT: f32 = 600.0;
const APP_ID: &str = "dev.atrium.app";

/// GPUI Application wrapper with window management.
pub struct AtriumApp {
    inner: gpui::Application,
}

impl AtriumApp {
    /// Create an AtriumApp from a GPUI Application.
    pub fn new(application: gpui::Application) -> Self {
        Self {
            inner: application,
        }
    }

    /// Run the event loop — installs keybindings, opens the window, blocks until close.
    pub fn run(self) {
        self.inner.run(move |cx| {
            keybindings::install(cx);
            Self::open_window(cx);
        });
    }

    fn open_window(cx: &mut gpui::App) {
        // Use 80% of screen size, but no smaller than the minimums
        let displays = cx.displays();
        let (width, height) = displays
            .first()
            .map(|d| {
                let b = d.bounds();
                let w = (f32::from(b.size.width) * DEFAULT_SIZE_RATIO).max(MIN_WIDTH);
                let h = (f32::from(b.size.height) * DEFAULT_SIZE_RATIO).max(MIN_HEIGHT);
                (w, h)
            })
            .unwrap_or((1280.0, 800.0));

        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(MIN_WIDTH), px(MIN_HEIGHT))),
                app_id: Some(APP_ID.to_owned()),
                ..Default::default()
            },
            |_, cx| cx.new(|_cx| AtriumWindow::new()),
        ) {
            tracing::error!(%error, "failed to open Atrium window");
        }
    }
}
