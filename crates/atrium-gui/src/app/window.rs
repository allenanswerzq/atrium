//! Main application window — thin shell that owns components.

use gpui::{Context, Render, Window, div, prelude::*, rgb};

use crate::layout::{LayoutState, Sidebar, CenterPanel, TopBar, RightPane, StatusBar};
use crate::state::UiState;
use crate::terminal::TerminalManager;
use crate::theme::ThemeState;
use crate::workspace::WorkspaceState;

/// The top-level window entity. Owns component handles, not raw fields.
pub struct AtriumWindow {
    pub(crate) theme: ThemeState,
    pub(crate) layout: LayoutState,
    pub(crate) ui_state: UiState,
    pub(crate) workspace: WorkspaceState,
    pub(crate) terminals: TerminalManager,
}

impl AtriumWindow {
    pub fn new() -> Self {
        Self {
            theme: ThemeState::default(),
            layout: LayoutState::default(),
            ui_state: UiState::default(),
            workspace: WorkspaceState::default(),
            terminals: TerminalManager::default(),
        }
    }
}

// ┌──────────────────────────────────────────────────────────────────┐
// │ TopBar                                                          │
// │  ◀ ▶   [back/fwd nav]          Atrium          [actions] ⚙     │
// ├────────┬───────────────────────────────────┬───────────────────┤
// │Sidebar │ CenterPanel                       │ RightPane         │
// │        │ ┌─────────────────────────────┐   │ ┌───────────────┐ │
// │REPOS   │ │ Tab bar: [Term 1] [Term 2] +│   │ │Changes│Files  │ │
// │        │ ├─────────────────────────────┤   │ ├───────────────┤ │
// │ repo-a │ │                             │   │ │               │ │
// │ repo-b │ │  Terminal / Diff / Logs     │   │ │ changed files │ │
// │        │ │  content area               │   │ │ or file tree  │ │
// │        │ │                             │   │ │               │ │
// │        │ └─────────────────────────────┘   │ └───────────────┘ │
// ├────────┴───────────────────────────────────┴───────────────────┤
// │ StatusBar                                                       │
// │  Windows · 2 terminals                                  v0.1.0 │
// └──────────────────────────────────────────────────────────────────┘
impl Render for AtriumWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = self.theme.palette();
        let nav = &self.workspace.navigation;
        let terminal_count = self.terminals.count();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(palette.app_bg))
            .text_color(rgb(palette.text_primary))
            // Top bar
            .child(TopBar::render(&palette, nav.can_go_back(), nav.can_go_forward()))
            // Main body
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .when(self.layout.sidebar_visible(), |el| {
                        el.child(Sidebar::render(
                            &palette,
                            self.layout.sidebar_width(),
                            self.workspace.repositories.roots(),
                        ))
                    })
                    .child(CenterPanel::render(&palette, terminal_count))
                    .child(RightPane::render(&palette, self.layout.right_pane_width())),
            )
            // Status bar
            .child(StatusBar::render(&palette, terminal_count))
    }
}
