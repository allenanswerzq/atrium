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
    pub terminal_fg: u32,
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

    /// Returns the color palette for this theme.
    #[must_use]
    pub fn palette(self) -> ThemePalette {
        match self {
            Self::One => ThemePalette {
                chrome_bg: 0x3b414d,
                app_bg: 0x282c33,
                text_primary: 0xc8ccd4,
                text_muted: 0x838994,
                accent: 0x74ade8,
                border: 0x363c46,
                terminal_bg: 0x282c34,
                terminal_fg: 0xc8ccd4,
                terminal_cursor: 0xebdbb2,
            },
            Self::Ayu => ThemePalette {
                chrome_bg: 0x313337,
                app_bg: 0x0d1016,
                text_primary: 0xbfbdb6,
                text_muted: 0x8a8986,
                accent: 0x5ac1fe,
                border: 0x3f4043,
                terminal_bg: 0x0d1016,
                terminal_fg: 0xbfbdb6,
                terminal_cursor: 0xbfbdb6,
            },
            Self::Gruvbox => ThemePalette {
                chrome_bg: 0x3c3836,
                app_bg: 0x282828,
                text_primary: 0xebdbb2,
                text_muted: 0xa89984,
                accent: 0x83a598,
                border: 0x504945,
                terminal_bg: 0x282828,
                terminal_fg: 0xebdbb2,
                terminal_cursor: 0xebdbb2,
            },
            Self::Dracula => ThemePalette {
                chrome_bg: 0x343746,
                app_bg: 0x282a36,
                text_primary: 0xf8f8f2,
                text_muted: 0x9ea0b0,
                accent: 0xbd93f9,
                border: 0x44475a,
                terminal_bg: 0x282a36,
                terminal_fg: 0xf8f8f2,
                terminal_cursor: 0xf8f8f2,
            },
            Self::SolarizedLight => ThemePalette {
                chrome_bg: 0xeee8d5,
                app_bg: 0xfdf6e3,
                text_primary: 0x586e75,
                text_muted: 0x93a1a1,
                accent: 0x268bd2,
                border: 0xd8cfba,
                terminal_bg: 0xfdf6e3,
                terminal_fg: 0x586e75,
                terminal_cursor: 0x586e75,
            },
            Self::SolarizedDark => ThemePalette {
                chrome_bg: 0x073642,
                app_bg: 0x002b36,
                text_primary: 0x839496,
                text_muted: 0x657b83,
                accent: 0x268bd2,
                border: 0x0a4353,
                terminal_bg: 0x002b36,
                terminal_fg: 0x839496,
                terminal_cursor: 0x839496,
            },
            Self::Nord => ThemePalette {
                chrome_bg: 0x3b4252,
                app_bg: 0x2e3440,
                text_primary: 0xd8dee9,
                text_muted: 0x81a1c1,
                accent: 0x88c0d0,
                border: 0x434c5e,
                terminal_bg: 0x2e3440,
                terminal_fg: 0xd8dee9,
                terminal_cursor: 0xd8dee9,
            },
            Self::Catppuccin => ThemePalette {
                chrome_bg: 0x313244,
                app_bg: 0x1e1e2e,
                text_primary: 0xcdd6f4,
                text_muted: 0xa6adc8,
                accent: 0x89b4fa,
                border: 0x45475a,
                terminal_bg: 0x1e1e2e,
                terminal_fg: 0xcdd6f4,
                terminal_cursor: 0xcdd6f4,
            },
            Self::TokyoNight => ThemePalette {
                chrome_bg: 0x1f2335,
                app_bg: 0x1a1b26,
                text_primary: 0xc0caf5,
                text_muted: 0x565f89,
                accent: 0x7aa2f7,
                border: 0x292e42,
                terminal_bg: 0x1a1b26,
                terminal_fg: 0xc0caf5,
                terminal_cursor: 0xc0caf5,
            },
            Self::Rosepine => ThemePalette {
                chrome_bg: 0x26233a,
                app_bg: 0x191724,
                text_primary: 0xe0def4,
                text_muted: 0x908caa,
                accent: 0xc4a7e7,
                border: 0x2a273f,
                terminal_bg: 0x191724,
                terminal_fg: 0xe0def4,
                terminal_cursor: 0xe0def4,
            },
        }
    }
}
