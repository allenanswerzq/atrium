//! Built-in color theme definitions.
//!
//! Provides a focused set of themes shared between all UI surfaces.
//! Each theme defines a palette of colors for the chrome, editor, and terminal.

use serde::Serialize;
use strum::{Display, IntoStaticStr};

/// Available theme variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ThemeKind {
    One,
    Ayu,
    Gruvbox,
    Dracula,
    SolarizedLight,
    SolarizedDark,
    Nord,
    Catppuccin,
    TokyoNight,
    Rosepine,
}

/// A color palette for a theme. Values are `0xRRGGBB` u32s.
#[derive(Debug, Clone, Serialize)]
pub struct ThemePalette {
    pub chrome_bg: u32,
    pub app_bg: u32,
    pub text_primary: u32,
    pub text_muted: u32,
    pub accent: u32,
    pub border: u32,
    pub terminal_bg: u32,
    pub terminal_cursor: u32,
}

impl ThemeKind {
    /// All available themes.
    pub const ALL: &[Self] = &[
        Self::One,
        Self::Ayu,
        Self::Gruvbox,
        Self::Dracula,
        Self::SolarizedLight,
        Self::SolarizedDark,
        Self::Nord,
        Self::Catppuccin,
        Self::TokyoNight,
        Self::Rosepine,
    ];

    /// Human-readable label.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::One => "One Dark",
            Self::Ayu => "Ayu Dark",
            Self::Gruvbox => "Gruvbox Dark",
            Self::Dracula => "Dracula",
            Self::SolarizedLight => "Solarized Light",
            Self::SolarizedDark => "Solarized Dark",
            Self::Nord => "Nord",
            Self::Catppuccin => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::Rosepine => "Rosé Pine",
        }
    }

    /// Whether this is a light theme.
    #[must_use]
    pub fn is_light(&self) -> bool {
        matches!(self, Self::SolarizedLight)
    }
}
