//! Main application window.
//!
//! Thin shell that owns components and delegates rendering.
//! Terminal logic lives in `atrium-terminal`; this module wires it to GPUI.

use gpui::{Context, Entity, FocusHandle, KeyDownEvent, Render, Window, div, prelude::*, px, rgb};

use crate::components::graph_view::GraphViewPanel;
use crate::terminal::rendering;
use crate::theme::ThemeState;
use atrium_core::theme::ThemePalette;
use atrium_terminal::{TerminalSession, terminal_escape_bytes};

// ── State ───────────────────────────────────────────────────────────

pub struct AtriumWindow {
    theme: ThemeState,
    session: Option<TerminalSession>,
    graph_view: Entity<GraphViewPanel>,
    focus: FocusHandle,
    poller_started: bool,
}

// ── Initialization ──────────────────────────────────────────────────

impl AtriumWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Spawn a default terminal session
        let shell = atrium_core::terminal::default_shell();
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from(if cfg!(windows) { "C:\\" } else { "/" }));
        let id = atrium_core::id::TerminalSessionId::new("term-1".to_owned());
        let wid = atrium_core::id::WorkspaceId::new("default".to_owned());

        let session =
            TerminalSession::spawn_standalone(id, wid, cwd, &shell, "Terminal 1", 120, 40).ok();

        let graph_view = cx.new(GraphViewPanel::new);

        Self {
            theme: ThemeState::default(),
            session,
            graph_view,
            focus: cx.focus_handle(),
            poller_started: false,
        }
    }

    /// Attach a pre-spawned terminal session.
    pub fn set_session(&mut self, session: TerminalSession) {
        self.session = Some(session);
    }
}

// ── Polling ─────────────────────────────────────────────────────────

impl AtriumWindow {
    fn ensure_poller(&mut self, cx: &mut Context<Self>) {
        // Only start once
        if self.poller_started {
            return;
        }
        self.poller_started = true;

        // GPUI foreground task
        cx.spawn(async move |this, cx| {
            loop {
                // Sleep on a background thread (never blocks the UI)
                cx.background_spawn(async {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                })
                .await;

                // Back on main thread: tell GPUI to re-render
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break; // Window closed, stop polling
                }
            }
        })
        .detach(); // Fire and forget — runs until window closes
    }
}

// ── Input ───────────────────────────────────────────────────────────

impl AtriumWindow {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.modifiers.platform {
            return;
        }
        let Some(session) = &self.session else { return };

        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control;
        let alt = event.keystroke.modifiers.alt;
        let modes = session.runtime().map(|r| r.modes()).unwrap_or_default();

        if let Some(bytes) = terminal_escape_bytes(key, ctrl, alt, modes) {
            let _ = session.write(&bytes);
        } else if !ctrl && !alt && key.len() <= 4 {
            let _ = session.write(key.as_bytes());
        }

        cx.notify();
    }
}

// ── Rendering ───────────────────────────────────────────────────────
//
// ┌──────────────────────────────────────────────────────────────┐
// │ [Terminal 1]                                                 │ tab bar
// ├──────────────────────────────────────────────────────────────┤
// │ C:\Users\zhangqiang>                                        │
// │ $ dir                                                        │ terminal
// │ ...                                                          │ output
// ├──────────────────────────────────────────────────────────────┤
// │ 1 terminal · v0.1.0                                          │ status
// └──────────────────────────────────────────────────────────────┘

impl Render for AtriumWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = self.theme.palette();

        if !self.focus.is_focused(window) {
            self.focus.focus(window);
        }

        div()
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xFFFFFFu32))
            .text_color(rgb(palette.text_primary))
            .child(self.graph_view.clone())
    }
}

impl AtriumWindow {
    fn render_tab_bar(&self, palette: &ThemePalette) -> gpui::Div {
        let label = self
            .session
            .as_ref()
            .map(|s| s.title().to_owned())
            .unwrap_or_else(|| "No terminal".to_owned());

        div()
            .h(px(32.0))
            .w_full()
            .bg(rgb(palette.chrome_bg))
            .border_b_1()
            .border_color(rgb(palette.border))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(10.0))
                    .py(px(4.0))
                    .text_size(px(12.0))
                    .text_color(rgb(palette.text_primary))
                    .bg(rgb(palette.app_bg))
                    .rounded(px(4.0))
                    .child(label),
            )
    }

    fn render_terminal_area(&self, palette: &ThemePalette) -> gpui::Div {
        match &self.session {
            Some(session) => rendering::render_terminal(session, palette),
            None => div()
                .size_full()
                .bg(rgb(palette.terminal_bg))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(palette.text_muted))
                        .child("No terminal session"),
                ),
        }
    }

    fn render_status_bar(&self, palette: &ThemePalette) -> gpui::Div {
        let count = if self.session.is_some() { 1 } else { 0 };
        div()
            .h(px(24.0))
            .w_full()
            .bg(rgb(palette.chrome_bg))
            .border_t_1()
            .border_color(rgb(palette.border))
            .flex()
            .items_center()
            .px(px(12.0))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(palette.text_muted))
                    .child(format!("{count} terminal(s) · v0.1.0")),
            )
    }
}
