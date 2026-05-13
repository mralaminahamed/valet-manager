mod app;
mod config;
mod commands;
mod events;
mod valet;
mod system;
mod php;
mod services;
mod state;
mod ui;
mod nginx;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use state::app_state::AppState;
use commands::AppCommand;
use events::AppEvent;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("valet_manager=info".parse().unwrap()),
        )
        .init();

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>(64);
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(64);
    let shared_state = Arc::new(RwLock::new(AppState::default()));

    // Service poller: poll_services sends Vec<ManagedService> on svc_tx;
    // bridge task converts to AppEvent::ServiceStatusUpdated on event_tx.
    let (svc_tx, mut svc_rx) = mpsc::channel::<Vec<services::monitor::ManagedService>>(4);
    let event_tx_svc = event_tx.clone();
    rt.spawn(async move {
        while let Some(svcs) = svc_rx.recv().await {
            if event_tx_svc.send(AppEvent::ServiceStatusUpdated(svcs)).await.is_err() {
                break;
            }
        }
    });
    rt.spawn(services::monitor::poll_services(
        vec!["nginx".to_string(), "dnsmasq".to_string(), "mailpit".to_string()],
        svc_tx,
    ));

    // Command dispatcher
    let state_clone = Arc::clone(&shared_state);
    let event_tx_clone = event_tx.clone();
    let cmd_tx_clone = cmd_tx.clone();
    rt.spawn(app::run_dispatcher(cmd_rx, event_tx_clone, state_clone, cmd_tx_clone));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 560.0])
            .with_decorations(false)
            .with_title("Valet Manager"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Valet Manager",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::ValetManagerApp::new(cc, cmd_tx, event_rx)))),
    )
}
