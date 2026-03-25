//! Atrium desktop application entry point.

#[tokio::main]
async fn main() {
    let mut early = atrium_init::early_init()
        .await
        .expect("early initialization failed");

    tracing::info!("Atrium starting");

    let application = early.take_application().expect("Application not available");
    let app = atrium_gui::app::AtriumApp::new(application);
    app.run();
}
