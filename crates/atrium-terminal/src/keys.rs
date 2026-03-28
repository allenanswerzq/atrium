//! Keystroke → terminal escape sequence mapping.
//!
//! Pure functions, no UI framework dependency.

use crate::styled::Modes;

/// Convert a key name + modifiers to terminal escape bytes.
///
/// Returns `None` if the key should be handled by the IME layer
/// or is a platform shortcut.
pub fn terminal_escape_bytes(
    key: &str,
    ctrl: bool,
    alt: bool,
    _modes: Modes,
) -> Option<Vec<u8>> {
    if ctrl {
        return ctrl_byte(key);
    }

    let seq: Option<&[u8]> = match key {
        "enter" | "return" => Some(b"\r"),
        "tab" => Some(b"\t"),
        "escape" => Some(b"\x1b"),
        "backspace" => Some(b"\x7f"),
        "delete" => Some(b"\x1b[3~"),
        "up" => Some(b"\x1b[A"),
        "down" => Some(b"\x1b[B"),
        "right" => Some(b"\x1b[C"),
        "left" => Some(b"\x1b[D"),
        "home" => Some(b"\x1b[H"),
        "end" => Some(b"\x1b[F"),
        "pageup" => Some(b"\x1b[5~"),
        "pagedown" => Some(b"\x1b[6~"),
        "insert" => Some(b"\x1b[2~"),
        "f1" => Some(b"\x1bOP"),
        "f2" => Some(b"\x1bOQ"),
        "f3" => Some(b"\x1bOR"),
        "f4" => Some(b"\x1bOS"),
        "f5" => Some(b"\x1b[15~"),
        "f6" => Some(b"\x1b[17~"),
        "f7" => Some(b"\x1b[18~"),
        "f8" => Some(b"\x1b[19~"),
        "f9" => Some(b"\x1b[20~"),
        "f10" => Some(b"\x1b[21~"),
        "f11" => Some(b"\x1b[23~"),
        "f12" => Some(b"\x1b[24~"),
        "space" if ctrl => Some(b"\x00"),
        _ => None,
    };

    if let Some(bytes) = seq {
        return Some(bytes.to_vec());
    }

    // Alt + character → ESC prefix
    if alt && key.len() == 1 {
        let mut bytes = vec![0x1b];
        bytes.extend_from_slice(key.as_bytes());
        return Some(bytes);
    }

    None
}

fn ctrl_byte(key: &str) -> Option<Vec<u8>> {
    let ch = key.chars().next()?;
    match ch {
        'a'..='z' => Some(vec![ch as u8 - b'a' + 1]),
        'A'..='Z' => Some(vec![ch as u8 - b'A' + 1]),
        '@' => Some(vec![0x00]),
        '[' => Some(vec![0x1b]),
        '\\' => Some(vec![0x1c]),
        ']' => Some(vec![0x1d]),
        '^' => Some(vec![0x1e]),
        '_' => Some(vec![0x1f]),
        '?' => Some(vec![0x7f]),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn modes() -> Modes { Modes::default() }

    #[test]
    fn basic_keys() {
        assert_eq!(terminal_escape_bytes("enter", false, false, modes()), Some(b"\r".to_vec()));
        assert_eq!(terminal_escape_bytes("tab", false, false, modes()), Some(b"\t".to_vec()));
        assert_eq!(terminal_escape_bytes("escape", false, false, modes()), Some(b"\x1b".to_vec()));
    }

    #[test]
    fn ctrl_c() {
        assert_eq!(terminal_escape_bytes("c", true, false, modes()), Some(vec![0x03]));
    }

    #[test]
    fn arrows() {
        assert_eq!(terminal_escape_bytes("up", false, false, modes()), Some(b"\x1b[A".to_vec()));
        assert_eq!(terminal_escape_bytes("down", false, false, modes()), Some(b"\x1b[B".to_vec()));
    }

    #[test]
    fn alt_char() {
        assert_eq!(terminal_escape_bytes("x", false, true, modes()), Some(b"\x1bx".to_vec()));
    }

    #[test]
    fn plain_char_returns_none() {
        assert_eq!(terminal_escape_bytes("a", false, false, modes()), None);
    }
}
