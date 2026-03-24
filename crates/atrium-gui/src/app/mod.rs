//! Application shell — window, bootstrap, actions, keybindings.

pub mod actions;
pub mod bootstrap;
pub mod keybindings;
pub mod window;

pub use bootstrap::run;
