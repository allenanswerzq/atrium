//! Theme management and GPUI color conversions.

use atrium_core::theme::{ThemeKind, ThemePalette};

/// Active theme state.
#[derive(Debug, Clone)]
pub struct ThemeState {
    kind: ThemeKind,
}

impl ThemeState {
    /// Create with the given theme.
    pub fn with_kind(kind: ThemeKind) -> Self {
        Self { kind }
    }

    /// Current theme kind.
    pub fn kind(&self) -> ThemeKind {
        self.kind
    }

    /// Set a new theme.
    pub fn set_kind(&mut self, kind: ThemeKind) {
        self.kind = kind;
    }

    /// Get the active color palette.
    pub fn palette(&self) -> ThemePalette {
        self.kind.palette()
    }
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            kind: ThemeKind::One,
        }
    }
}
