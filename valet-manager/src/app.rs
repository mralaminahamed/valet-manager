use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::commands::AppCommand;
use crate::events::AppEvent;
use crate::php::detector;
use crate::state::app_state::{AppState, Panel};
use crate::ui::{panels::dashboard, sidebar, theme};
use crate::valet::variant;

pub struct ValetManagerApp {
    state: AppState,
    cmd_tx: mpsc::Sender<AppCommand>,
    event_rx: mpsc::Receiver<AppEvent>,
}

impl ValetManagerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        cmd_tx: mpsc::Sender<AppCommand>,
        event_rx: mpsc::Receiver<AppEvent>,
    ) -> Self {
        theme::apply_dark(&cc.egui_ctx);
        Self { state: AppState::default(), cmd_tx, event_rx }
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::ServiceStatusUpdated(services) => {
                    self.state.services = services;
                }
                AppEvent::PhpVersionsRefreshed(versions) => {
                    if let Some(active) = versions.iter().find(|v| v.is_active) {
                        self.state.active_php = active.version.clone();
                    }
                    self.state.php_versions = versions;
                }
                AppEvent::ValetDetected(v, paths) => {
                    self.state.valet_variant = Some(v);
                    self.state.valet_paths = Some(paths);
                }
                AppEvent::Error(msg) => {
                    self.state.ui.last_error = Some(msg);
                }
                AppEvent::PhpSwitched(_) |
                AppEvent::PhpFpmStatusChanged { .. } |
                AppEvent::ExtensionsLoaded { .. } |
                AppEvent::IniLoaded { .. } |
                AppEvent::IniSaved => {}
            }
        }
    }
}

impl eframe::App for ValetManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_dark(ctx);
        self.drain_events();
        ctx.request_repaint_after(Duration::from_secs(1));

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::K) && i.modifiers.ctrl {
                let _ = self.cmd_tx.try_send(AppCommand::OpenCommandPalette);
            }
            if i.key_pressed(egui::Key::R) && i.modifiers.ctrl {
                let _ = self.cmd_tx.try_send(AppCommand::RefreshAll);
            }
            if i.key_pressed(egui::Key::Comma) && i.modifiers.ctrl {
                let _ = self.cmd_tx.try_send(AppCommand::OpenPanel(Panel::Settings));
            }
        });

        // ── Titlebar ─────────────────────────────────────────────────────
        egui::TopBottomPanel::top("titlebar")
            .exact_height(36.0)
            .frame(egui::Frame::NONE.fill(theme::Colors::DEEP_BG))
            .show(ctx, |ui| {
                // Enable window drag from titlebar
                let drag_resp = ui.interact(
                    ui.max_rect(),
                    ui.id().with("drag"),
                    egui::Sense::click_and_drag(),
                );
                if drag_resp.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    // Three traffic-light dots
                    for (i, color) in [
                        theme::Colors::DANGER,
                        theme::Colors::WARNING,
                        theme::Colors::ACCENT,
                    ]
                    .iter()
                    .enumerate()
                    {
                        let pos = ui.min_rect().left_center()
                            + egui::vec2(16.0 + i as f32 * 12.0, 0.0);
                        ui.painter().circle_filled(pos, 5.0, *color);
                    }

                    // Center title
                    let title_rect = ui.max_rect();
                    ui.painter().text(
                        title_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Valet Manager",
                        egui::FontId::proportional(12.0),
                        theme::Colors::TEXT_TERTIARY,
                    );

                    // Right info
                    let right_text = format!(
                        "PHP {} · {}",
                        self.state.active_php,
                        self.state
                            .valet_variant
                            .as_ref()
                            .map(|v| v.display_name())
                            .unwrap_or("Valet"),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(right_text)
                                .size(11.0)
                                .color(theme::Colors::TEXT_TERTIARY),
                        );
                    });
                });
            });

        // ── Sidebar ───────────────────────────────────────────────────────
        egui::SidePanel::left("sidebar")
            .exact_width(196.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(theme::Colors::DEEP_BG))
            .show(ctx, |ui| {
                sidebar::render(ui, &self.state, &self.cmd_tx);
            });

        // ── Content panel ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::Colors::SURFACE))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    match self.state.ui.active_panel {
                        Panel::Dashboard => {
                            dashboard::render(ui, &self.state, &self.cmd_tx);
                        }
                        _ => stub_panel(ui, &self.state.ui.active_panel),
                    }
                });
            });
    }
}

fn stub_panel(ui: &mut egui::Ui, panel: &Panel) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(format!("{:?}", panel))
                .size(14.0)
                .color(theme::Colors::TEXT_SECONDARY),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Coming in a future phase")
                .size(12.0)
                .color(theme::Colors::TEXT_TERTIARY),
        );
    });
}

pub async fn run_dispatcher(
    mut rx: mpsc::Receiver<AppCommand>,
    tx: mpsc::Sender<AppEvent>,
    state: Arc<RwLock<AppState>>,
) {
    // Kick off initial detection on first start
    {
        let tx = tx.clone();
        let state = Arc::clone(&state);
        tokio::spawn(initial_detection(tx, state));
    }

    while let Some(cmd) = rx.recv().await {
        match cmd {
            AppCommand::RefreshAll => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(initial_detection(tx, state));
            }
            AppCommand::RefreshServiceStatus => {
                // poll_services handles its own loop; no action needed
            }
            AppCommand::SwitchGlobalPhp(ver) => {
                let _ = tx
                    .send(AppEvent::Error(format!(
                        "SwitchGlobalPhp({}) not yet implemented",
                        ver
                    )))
                    .await;
            }
            AppCommand::OpenPanel(panel) => {
                state.write().await.ui.active_panel = panel;
            }
            AppCommand::OpenCommandPalette => {
                // Phase 3+
            }
            AppCommand::StartPhpFpm(_) |
            AppCommand::StopPhpFpm(_) |
            AppCommand::RestartPhpFpm(_) |
            AppCommand::EnableExtension { .. } |
            AppCommand::DisableExtension { .. } |
            AppCommand::LoadExtensions(_) |
            AppCommand::LoadIni { .. } |
            AppCommand::SaveIni { .. } => {
                // implemented in Task 9
            }
        }
    }
}

async fn initial_detection(tx: mpsc::Sender<AppEvent>, _state: Arc<RwLock<AppState>>) {
    if let Ok(v) = variant::detect_valet_variant().await {
        let paths = variant::for_variant(&v);
        let _ = tx.send(AppEvent::ValetDetected(v, paths)).await;
    }
    if let Ok(versions) = detector::detect_installed_versions().await {
        let _ = tx.send(AppEvent::PhpVersionsRefreshed(versions)).await;
    }
}
