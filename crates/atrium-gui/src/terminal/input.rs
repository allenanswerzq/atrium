//! Terminal input — write, scroll, selection, copy/paste.

/// Terminal input handler.
pub struct TerminalInput;

impl TerminalInput {
    /// Write raw bytes to the terminal backend.
    pub fn write(_session_id: &atrium_core::id::SessionId, _data: &[u8]) {
        // TODO: route to backend
    }

    /// Copy the current selection to the clipboard.
    pub fn copy_selection() -> Option<String> {
        // TODO: implement selection tracking
        None
    }

    /// Paste text into the terminal.
    pub fn paste(_session_id: &atrium_core::id::SessionId, _text: &str) {
        // TODO: route to backend
    }
}
