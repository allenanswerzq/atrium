//! GPUI action definitions for the Atrium application.

use gpui::actions;

actions!(
    atrium,
    [
        // Window management
        NewWindow,
        RequestQuit,
        ImmediateQuit,
        ShowAbout,
        // Terminal
        SpawnTerminal,
        CloseActiveTerminal,
        // Workspace
        OpenAddRepository,
        OpenCreateWorktree,
        RefreshWorktrees,
        RefreshChanges,
        // Navigation
        NavigateBack,
        NavigateForward,
        // Layout
        ToggleSidebar,
        // Panels
        ViewLogs,
        OpenCommandPalette,
        OpenThemePicker,
        OpenSettings,
    ]
);
