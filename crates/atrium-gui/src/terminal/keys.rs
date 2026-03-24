//! Keystroke to terminal escape sequence mapping.

/// Maps GPUI keystrokes to terminal escape sequences.
pub struct KeyMapper;

impl KeyMapper {
    /// Convert a key name to its terminal escape sequence.
    #[must_use]
    pub fn to_escape_sequence(key: &str, ctrl: bool, alt: bool) -> Option<Vec<u8>> {
        if ctrl {
            return Self::ctrl_key(key);
        }

        let seq = match key {
            "enter" | "return" => b"\r".to_vec(),
            "tab" => b"\t".to_vec(),
            "escape" => b"\x1b".to_vec(),
            "backspace" => b"\x7f".to_vec(),
            "delete" => b"\x1b[3~".to_vec(),
            "up" => b"\x1b[A".to_vec(),
            "down" => b"\x1b[B".to_vec(),
            "right" => b"\x1b[C".to_vec(),
            "left" => b"\x1b[D".to_vec(),
            "home" => b"\x1b[H".to_vec(),
            "end" => b"\x1b[F".to_vec(),
            "pageup" => b"\x1b[5~".to_vec(),
            "pagedown" => b"\x1b[6~".to_vec(),
            _ => {
                if alt {
                    let mut seq = vec![0x1b];
                    seq.extend_from_slice(key.as_bytes());
                    seq
                } else {
                    return None;
                }
            }
        };
        Some(seq)
    }

    fn ctrl_key(key: &str) -> Option<Vec<u8>> {
        let ch = key.chars().next()?;
        if ch.is_ascii_lowercase() {
            Some(vec![ch as u8 - b'a' + 1])
        } else {
            None
        }
    }
}
