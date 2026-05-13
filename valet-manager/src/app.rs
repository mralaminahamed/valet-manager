use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::commands::AppCommand;
use crate::events::AppEvent;
use crate::php::detector;
use crate::php::{fpm_manager, extension_manager, ini_manager, switcher};
use crate::php::types::IniType;
use crate::services::monitor::ServiceStatus;
use crate::state::app_state::{AppState, Panel};
use crate::ui::{panels::{dashboard, php_versions, php_extensions, php_ini}, sidebar, theme};
use crate::valet::{config_reader, site_scanner, watcher, variant};
use crate::nginx::site_manager as nginx_manager;
use crate::ui::panels::{sites, parks, nginx};

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
                AppEvent::IniLoaded { version: _, ini_type: _, sections, raw } => {
                    self.state.ini_sections = sections;
                    self.state.ini_raw = raw;
                }
                AppEvent::ExtensionsLoaded { version: _, extensions } => {
                    self.state.extensions = extensions;
                }
                AppEvent::IniSaved => {
                    self.state.ini_last_saved = Some(chrono::Local::now().to_rfc3339());
                }
                AppEvent::PhpSwitched(_)
                | AppEvent::PhpFpmStatusChanged { .. }
                | AppEvent::SiteIsolated { .. }
                | AppEvent::SiteUnisolated(_)
                | AppEvent::PhpVersionInstalled(_)
                | AppEvent::PhpVersionRemoved(_) => {}
                // Phase 4
                AppEvent::CreatorOutputLine(line) => {
                    self.state.creator.add_output_line(line);
                }
                AppEvent::CreatorComplete { .. } => {
                    self.state.creator.is_running = false;
                }
                AppEvent::CreatorFailed(msg) => {
                    self.state.creator.is_running = false;
                    self.state.creator.error = Some(msg);
                }
                // Phase 3
                AppEvent::SitesRefreshed(sites) => {
                    self.state.site_count = sites.len();
                    self.state.sites = sites;
                }
                AppEvent::NginxConfigLoaded { site, content } => {
                    self.state.nginx_configs.insert(site.clone(), content);
                    self.state.nginx_selected = Some(site);
                }
                AppEvent::NginxReloaded => {}
                AppEvent::ParksUpdated(parks) => {
                    self.state.parks = parks;
                }
            }
        }
    }
}

impl eframe::App for ValetManagerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        theme::apply_dark(&ctx);
        self.drain_events();
        ctx.request_repaint_after(Duration::from_secs(1));

        // Keyboard shortcuts
        ui.input(|i| {
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
        egui::Panel::top("titlebar")
            .exact_size(38.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::Colors::DEEP_BG)
                    .stroke(egui::Stroke::new(0.5, theme::Colors::BORDER)),
            )
            .show_inside(ui, |ui| {
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
                    // ── LEFT: GNOME-style window control buttons ──────────
                    ui.add_space(10.0);

                    let btn_size = egui::vec2(22.0, 22.0);
                    let btn_defs: &[(egui::Color32, egui::Color32, &str)] = &[
                        (theme::Colors::CARD, theme::Colors::TEXT_SECONDARY, "—"),
                        (theme::Colors::CARD, theme::Colors::TEXT_SECONDARY, "⬜"),
                        (theme::Colors::CLOSE_BTN_BG, theme::Colors::DANGER, "✕"),
                    ];

                    let mut close_clicked = false;
                    for (idx, (bg, icon_color, icon)) in btn_defs.iter().enumerate() {
                        let (rect, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());
                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter();
                            let center = rect.center();
                            let radius = 11.0_f32;
                            painter.circle(
                                center,
                                radius,
                                *bg,
                                egui::Stroke::new(0.5, theme::Colors::BORDER_MED),
                            );
                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                *icon,
                                egui::FontId::proportional(10.0),
                                *icon_color,
                            );
                        }
                        if idx == 2 && resp.clicked() {
                            close_clicked = true;
                        }
                        if idx < btn_defs.len() - 1 {
                            ui.add_space(8.0);
                        }
                    }

                    if close_clicked {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    // ── CENTER: title painted via LayoutJob for multi-color ──
                    let title_rect = ui.max_rect();
                    let panel_name = panel_display_name(&self.state.ui.active_panel);
                    let font_id = egui::FontId::proportional(13.0);

                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        "Valet Manager ",
                        0.0,
                        egui::text::TextFormat {
                            font_id: font_id.clone(),
                            color: theme::Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );
                    job.append(
                        "· ",
                        0.0,
                        egui::text::TextFormat {
                            font_id: font_id.clone(),
                            color: theme::Colors::TEXT_TERTIARY,
                            ..Default::default()
                        },
                    );
                    job.append(
                        panel_name,
                        0.0,
                        egui::text::TextFormat {
                            font_id: font_id.clone(),
                            color: theme::Colors::TEXT_PRIMARY,
                            ..Default::default()
                        },
                    );

                    let galley = ui.ctx().fonts_mut(|f| f.layout_job(job));
                    let galley_size = galley.rect.size();
                    let title_pos = egui::pos2(
                        title_rect.center().x - galley_size.x / 2.0,
                        title_rect.center().y - galley_size.y / 2.0,
                    );
                    ui.painter().galley(title_pos, galley, egui::Color32::WHITE);

                    // ── RIGHT: PHP version & valet variant ───────────────
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
        egui::Panel::left("sidebar")
            .exact_size(220.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(theme::Colors::DEEP_BG))
            .show_inside(ui, |ui| {
                sidebar::render(ui, &self.state, &self.cmd_tx);
            });

        // ── Content panel ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::Colors::SURFACE))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(4.0);
                    match self.state.ui.active_panel {
                        Panel::Dashboard => {
                            dashboard::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::PhpVersions => {
                            php_versions::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::PhpExtensions => {
                            php_extensions::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::PhpIni => {
                            php_ini::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::Sites => {
                            sites::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::Parks => {
                            parks::render(ui, &self.state, &self.cmd_tx);
                        }
                        Panel::Nginx => {
                            nginx::render(ui, &self.state, &self.cmd_tx);
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

fn panel_display_name(panel: &Panel) -> &'static str {
    match panel {
        Panel::Dashboard     => "Dashboard",
        Panel::PhpVersions   => "PHP Versions",
        Panel::PhpExtensions => "PHP Extensions",
        Panel::PhpIni        => "PHP INI",
        Panel::Sites         => "Sites",
        Panel::Parks         => "Parks",
        Panel::Nginx         => "Nginx",
        Panel::Settings      => "Settings",
        Panel::PhpInfo       => "phpinfo()",
        Panel::PhpCompat     => "PHP Compat",
        Panel::Proxies       => "Proxies",
        Panel::Dnsmasq       => "DNS",
        Panel::SslCerts      => "SSL Certs",
        Panel::Database      => "Database",
        Panel::EnvEditor     => ".env Editor",
        Panel::Artisan       => "Artisan",
        Panel::QueueWorkers  => "Queue Workers",
        Panel::Xdebug        => "Xdebug",
        Panel::MailCatcher   => "Mailpit",
        Panel::Sharing       => "Sharing",
        Panel::Drivers       => "Drivers",
        Panel::Logs          => "Logs",
        Panel::History       => "History",
        Panel::Diagnostics   => "Diagnostics",
        Panel::AppCreator    => "App Creator",
    }
}

pub async fn run_dispatcher(
    mut rx: mpsc::Receiver<AppCommand>,
    tx: mpsc::Sender<AppEvent>,
    state: Arc<RwLock<AppState>>,
    cmd_tx: mpsc::Sender<AppCommand>,
) {
    // Kick off initial detection on first start
    {
        let tx = tx.clone();
        let state = Arc::clone(&state);
        tokio::spawn(initial_detection(tx, state));
    }

    // Start file watcher (waits briefly for valet detection, then starts)
    {
        let cmd_tx_watch = cmd_tx.clone();
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            // Give initial_detection a moment to populate valet_paths
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let valet_paths = {
                let s = state.read().await;
                s.valet_paths.clone()
            };
            if let Some(paths) = valet_paths {
                watcher::start_watcher(paths, cmd_tx_watch).await;
            }
        });
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
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let valet_paths = {
                        let s = state_clone.read().await;
                        s.valet_paths.clone()
                    };
                    if let Some(paths) = valet_paths {
                        match switcher::switch_global(&ver, &paths).await {
                            Ok(()) => { let _ = tx.send(AppEvent::PhpSwitched(ver)).await; }
                            Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                        }
                    } else {
                        let _ = tx.send(AppEvent::Error("Valet not detected".to_string())).await;
                    }
                });
            }
            AppCommand::OpenPanel(panel) => {
                state.write().await.ui.active_panel = panel;
            }
            AppCommand::OpenCommandPalette => {
                // Phase 3+
            }
            AppCommand::StartPhpFpm(ver) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match fpm_manager::start(&ver).await {
                        Ok(()) => {
                            let _ = tx.send(AppEvent::PhpFpmStatusChanged {
                                version: ver,
                                status: ServiceStatus::Running,
                            }).await;
                        }
                        Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                    }
                });
            }
            AppCommand::StopPhpFpm(ver) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match fpm_manager::stop(&ver).await {
                        Ok(()) => {
                            let _ = tx.send(AppEvent::PhpFpmStatusChanged {
                                version: ver,
                                status: ServiceStatus::Stopped,
                            }).await;
                        }
                        Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                    }
                });
            }
            AppCommand::RestartPhpFpm(ver) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match fpm_manager::restart(&ver).await {
                        Ok(()) => {}
                        Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                    }
                });
            }
            AppCommand::EnableExtension { version, extension } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = extension_manager::enable(&version, &extension).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                    }
                });
            }
            AppCommand::DisableExtension { version, extension } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = extension_manager::disable(&version, &extension).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                    }
                });
            }
            AppCommand::LoadExtensions(ver) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let exts = extension_manager::list(&ver).await;
                    let _ = tx.send(AppEvent::ExtensionsLoaded {
                        version: ver,
                        extensions: exts,
                    }).await;
                });
            }
            AppCommand::LoadIni { version, ini_type } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let path = match ini_type {
                        IniType::Cli => format!("/etc/php/{}/cli/php.ini", version),
                        IniType::Fpm => format!("/etc/php/{}/fpm/php.ini", version),
                    };
                    match tokio::fs::read_to_string(&path).await {
                        Ok(raw) => {
                            let sections = ini_manager::parse(&raw);
                            let _ = tx.send(AppEvent::IniLoaded {
                                version,
                                ini_type,
                                sections,
                                raw,
                            }).await;
                        }
                        Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                    }
                });
            }
            AppCommand::SaveIni { version, ini_type, content } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match ini_manager::save(&version, &ini_type, &content).await {
                        Ok(()) => { let _ = tx.send(AppEvent::IniSaved).await; }
                        Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                    }
                });
            }
            AppCommand::IsolateSite { site, version } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let path = std::path::Path::new(&site);
                    if let Err(e) = switcher::isolate_site(path, &version).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                    } else {
                        let _ = tx.send(AppEvent::SiteIsolated {
                            site: site.clone(),
                            version,
                        }).await;
                    }
                });
            }
            AppCommand::UnisolateSite(site) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let path = std::path::Path::new(&site);
                    if let Err(e) = switcher::unisolate_site(path).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                    } else {
                        let _ = tx.send(AppEvent::SiteUnisolated(site)).await;
                    }
                });
            }
            AppCommand::InstallPhpVersion(ver) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "Install PHP {} not yet implemented",
                    ver
                ))).await;
            }
            AppCommand::RemovePhpVersion(ver) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "Remove PHP {} not yet implemented",
                    ver
                ))).await;
            }
            // Phase 3 — Sites
            AppCommand::RefreshSites => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let (valet_paths, _valet_variant) = {
                        let s = state.read().await;
                        (s.valet_paths.clone(), s.valet_variant.clone())
                    };
                    if let Some(paths) = valet_paths {
                        let config = config_reader::load(&paths).await.unwrap_or_default();
                        // Update tld and parks in state
                        {
                            let mut s = state.write().await;
                            s.tld = config.tld.clone();
                            s.parks = config.paths.iter().map(std::path::PathBuf::from).collect();
                        }
                        let sites = site_scanner::scan_all(&paths, &config).await;
                        let _ = tx.send(AppEvent::SitesRefreshed(sites)).await;
                        let parks: Vec<std::path::PathBuf> =
                            config.paths.iter().map(std::path::PathBuf::from).collect();
                        let _ = tx.send(AppEvent::ParksUpdated(parks)).await;
                    }
                });
            }
            AppCommand::ParkDirectory(path) => {
                let _ = tx.send(AppEvent::Error(
                    "ParkDirectory requires valet CLI — run 'valet park' manually".to_string(),
                )).await;
                drop(path);
            }
            AppCommand::ForgetDirectory(path) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "ForgetDirectory({}) not yet implemented",
                    path.display()
                ))).await;
            }
            AppCommand::LinkSite { name, path } => {
                let _ = tx.send(AppEvent::Error(format!(
                    "LinkSite({name}) not yet implemented"
                ))).await;
                drop(path);
            }
            AppCommand::UnlinkSite(name) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "UnlinkSite({name}) not yet implemented"
                ))).await;
            }
            AppCommand::SecureSite(name) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "SecureSite({name}) not yet implemented"
                ))).await;
            }
            AppCommand::UnsecureSite(name) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "UnsecureSite({name}) not yet implemented"
                ))).await;
            }
            AppCommand::ToggleFavoriteSite(name) => {
                let mut s = state.write().await;
                if let Some(site) = s.sites.iter_mut().find(|s| s.name == name) {
                    site.is_favorite = !site.is_favorite;
                }
            }
            AppCommand::OpenSiteInBrowser(url) => {
                let _ = tokio::process::Command::new("xdg-open").arg(&url).spawn();
            }
            AppCommand::OpenSiteInEditor(name) => {
                let editor_path = {
                    let s = state.read().await;
                    if let Some(site) = s.sites.iter().find(|s| s.name == name) {
                        site.path.to_string_lossy().to_string()
                    } else {
                        name.clone()
                    }
                };
                let _ = tokio::process::Command::new("xdg-open").arg(&editor_path).spawn();
            }
            AppCommand::SaveNginxConfig { site, content } => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let valet_paths = {
                        let s = state_clone.read().await;
                        s.valet_paths.clone()
                    };
                    if let Some(paths) = valet_paths {
                        match nginx_manager::write_site_config(&site, &content, &paths).await {
                            Ok(()) => {
                                let _ = tx.send(AppEvent::NginxConfigLoaded { site, content }).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                });
            }
            AppCommand::ReloadNginx => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match nginx_manager::reload().await {
                        Ok(()) => {
                            let _ = tx.send(AppEvent::NginxReloaded).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            // Phase 4 — App Creator
            AppCommand::CreateApp(req) => {
                let _ = tx.send(AppEvent::Error(format!(
                    "CreateApp({}) not yet implemented",
                    req.type_id
                ))).await;
            }
            AppCommand::CancelCreation => {}
            AppCommand::CreatorStepBack => {}
            AppCommand::CreatorSelectType { type_id: _ } => {}
            AppCommand::CreatorUpdateField { key: _, value: _ } => {}
            AppCommand::CreatorNextStep => {}
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
