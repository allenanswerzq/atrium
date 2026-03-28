//! Atrium desktop application entry point.
//!
//! GPUI requires Application::new() to be called from the main thread
//! without an async runtime. So we init synchronously, then run the GUI.

fn main() {
    // Phase 1: synchronous early init (tracing, context)
    atrium_trace::init();
    tracing::info!("Atrium starting");

    // Phase 2: create GPUI Application and run event loop
    let app = atrium_gui::app::AtriumApp::create();
    app.run();
}
