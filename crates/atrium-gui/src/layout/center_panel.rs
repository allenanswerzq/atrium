//! Center panel — tab bar and content dispatch (terminal, diff, file view, logs).

use gpui::{Div, div, prelude::*, px, rgb};

use atrium_core::theme::ThemePalette;

/// Center panel rendering component.
pub struct CenterPanel;

impl CenterPanel {
    // ┌─────────────────────────────────────────┐
    // │ [Term 1] [Term 2] [Logs]            [+] │  ← tab bar
    // ├─────────────────────────────────────────┤
    // │                                         │
    // │   Terminal output / Diff / File view     │  ← content
    // │   or empty state: "Press Cmd+T"          │
    // │                                         │
    // └─────────────────────────────────────────┘
    /// Render the center panel with a tab bar and content area.
    pub fn render(palette: &ThemePalette, terminal_count: usize) -> Div {
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .child(Self::render_tab_bar(palette, terminal_count))
            .child(Self::render_content(palette, terminal_count))
    }

    fn render_tab_bar(palette: &ThemePalette, terminal_count: usize) -> Div {
        let mut bar = div()
            .h(px(32.0))
            .w_full()
            .bg(rgb(palette.chrome_bg))
            .border_b_1()
            .border_color(rgb(palette.border))
            .flex()
            .items_center()
            .px(px(8.0))
            .gap(px(2.0));

        if terminal_count == 0 {
            bar = bar.child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(palette.text_muted))
                    .child("No terminals"),
            );
        } else {
            for i in 0..terminal_count {
                let label = format!("Terminal {}", i + 1);
                bar = bar.child(
                    div()
                        .px(px(10.0))
                        .py(px(4.0))
                        .text_size(px(12.0))
                        .text_color(rgb(palette.text_primary))
                        .bg(rgb(palette.app_bg))
                        .rounded(px(4.0))
                        .child(label),
                );
            }
        }

        // + button
        bar = bar.child(
            div()
                .ml_auto()
                .px(px(6.0))
                .py(px(2.0))
                .text_size(px(14.0))
                .text_color(rgb(palette.text_muted))
                .hover(|s| s.text_color(rgb(palette.text_primary)))
                .cursor_pointer()
                .child("+"),
        );

        bar
    }

    fn render_content(palette: &ThemePalette, terminal_count: usize) -> Div {
        let content = div()
            .flex_1()
            .w_full()
            .bg(rgb(palette.app_bg))
            .flex()
            .items_center()
            .justify_center();

        if terminal_count == 0 {
            content.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(32.0))
                            .text_color(rgb(palette.text_muted))
                            .child("\u{f120}"),
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(rgb(palette.text_muted))
                            .child("Press Cmd+T to open a terminal"),
                    ),
            )
        } else {
            content.child(
                div()
                    .size_full()
                    .bg(rgb(palette.terminal_bg))
                    .p(px(8.0))
                    .text_size(px(14.0))
                    .text_color(rgb(palette.text_primary))
                    .child("Terminal output will render here"),
            )
        }
    }
}
