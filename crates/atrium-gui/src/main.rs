//! Atrium desktop application entry point.

use atrium_gui::app;

fn main() {
    atrium_trace::init();
    tracing::info!("Atrium starting");
    app::run();
}
