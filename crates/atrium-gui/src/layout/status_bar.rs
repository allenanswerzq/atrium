//! Status bar — bottom bar with connection status, memory usage, etc.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;
use atrium_platform::{Os, Platform};

const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Status bar rendering component.
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar.
    pub fn render(palette: &ThemePalette, terminal_count: usize) -> Div {
        let platform = Platform::current();
        let platform_label = match platform.os {
            Os::Windows => "Windows",
            Os::MacOs => "macOS",
            Os::Linux => "Linux",
        };

        div()
            .h(px(STATUS_BAR_HEIGHT))
            .w_full()
            .bg(rgb(palette.chrome_bg))
            .border_t_1()
            .border_color(rgb(palette.border))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .text_size(px(11.0))
                    .text_color(rgb(palette.text_muted))
                    .child(platform_label)
                    .child("\u{00B7}")
                    .child(format!("{} terminal{}",
                        terminal_count,
                        if terminal_count == 1 { "" } else { "s" }
                    )),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(palette.text_muted))
                    .child("v0.1.0"),
            )
    }
}
