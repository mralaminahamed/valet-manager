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
mod cli_tools;
mod creator;
mod tray;
mod notifications;
mod history;
mod deep_link;
mod updater;
mod env_file;
mod artisan;
mod database;
mod ssl;
mod mail;
mod queue;
mod site_config;
mod wordpress;
mod laravel;
mod http_servers;
mod phpmyadmin;
mod version_registry;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use state::app_state::AppState;
use commands::AppCommand;
use events::AppEvent;

fn main() -> eframe::Result {
    // Parse a deep-link argument before any tokio/UI setup.
    let initial_deeplink: Option<AppCommand> = std::env::args()
        .nth(1)
        .filter(|a| a.starts_with("valet-manager://"))
        .map(|a| deep_link::parse(&a))
        .and_then(|l| deep_link::to_command(&l));

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

    // Queue a deep-link command at startup if one was provided.
    if let Some(cmd) = initial_deeplink {
        let _ = cmd_tx.try_send(cmd);
    }

    // Phase 11 — detect installed HTTP servers at startup.
    let _ = cmd_tx.try_send(AppCommand::DetectInstalledServers);

    // Phase 12 — probe for phpMyAdmin install at startup.
    let _ = cmd_tx.try_send(AppCommand::CheckPhpMyAdminInstalled);

    // Phase 14 — preload cached version registry on startup; refresh if stale/missing.
    let cmd_tx_vr = cmd_tx.clone();
    let event_tx_vr = event_tx.clone();
    rt.spawn(async move {
        match version_registry::cache::load().await {
            Ok(c) if !c.is_stale() => {
                let _ = event_tx_vr
                    .send(AppEvent::VersionRegistryRefreshed(c.registry))
                    .await;
            }
            _ => {
                let _ = cmd_tx_vr.try_send(AppCommand::RefreshVersionRegistry);
            }
        }
    });

    // ── GTK init required by tray-icon on Linux (must happen before build_tray) ──
    #[cfg(target_os = "linux")]
    gtk::init().ok();

    // ── System tray (Linux: needs DISPLAY/WAYLAND_DISPLAY; gracefully no-op otherwise) ──
    let (tray_tx, mut tray_rx) = mpsc::channel::<tray::TrayEvent>(8);
    let _tray = if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        match tray::build_tray("8.3", &[], tray::HealthState::Ok) {
            Ok(t) => {
                tray::spawn_menu_drain(tray_tx);
                Some(t)
            }
            Err(e) => {
                tracing::warn!("tray icon init failed: {e}");
                None
            }
        }
    } else {
        tracing::info!("no display server detected — skipping tray icon");
        None
    };

    // Tray event → AppCommand bridge
    let cmd_tx_tray = cmd_tx.clone();
    rt.spawn(async move {
        while let Some(ev) = tray_rx.recv().await {
            match ev {
                tray::TrayEvent::Open => { /* no-op for now */ }
                tray::TrayEvent::Quit => std::process::exit(0),
                tray::TrayEvent::RestartServices => {
                    let _ = cmd_tx_tray.send(commands::AppCommand::RefreshAll).await;
                }
                tray::TrayEvent::SwitchPhp(v) => {
                    let _ = cmd_tx_tray.send(commands::AppCommand::SwitchGlobalPhp(v)).await;
                }
            }
        }
    });

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
