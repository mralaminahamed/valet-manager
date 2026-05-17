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
use crate::ui::panels::{sites, parks, nginx, app_creator, proxies, dnsmasq, sharing, logs, diagnostics, settings, onboarding, phpinfo, compat, history, env_editor, artisan as artisan_panel, database as database_panel, ssl_certs, xdebug as xdebug_panel, mail_catcher, queue as queue_panel, site_config as site_config_panel};
use crate::ui::command_palette;
use crate::ui::components::toast::{self, ToastType};

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
                // Phase 5 — Proxies
                AppEvent::ProxiesRefreshed(list) => {
                    self.state.proxies = list;
                }
                AppEvent::ProxyAdded(_) | AppEvent::ProxyRemoved(_) => {
                    // Caller dispatches a follow-up RefreshProxies; nothing to do here.
                }
                AppEvent::ProxyProbed { domain, result } => {
                    self.state.proxy_status.insert(domain, result);
                }
                // Phase 5 — Dnsmasq
                AppEvent::TldChanged(new_tld) => {
                    self.state.tld = new_tld;
                }
                AppEvent::DnsOutputLine(line) => {
                    self.state.dns_tester_output.push(line);
                    if self.state.dns_tester_output.len() > 1000 {
                        let overflow = self.state.dns_tester_output.len() - 1000;
                        self.state.dns_tester_output.drain(0..overflow);
                    }
                }
                // Phase 5 — Sharing
                AppEvent::SharingStarted(session) => {
                    self.state.share_active = Some(session);
                }
                AppEvent::SharingStopped => {
                    self.state.share_active = None;
                }
                AppEvent::SharingOutputLine(line) => {
                    if self.state.share_active.is_some() {
                        if let Some(url) = crate::valet::sharing::parse_public_url(&line.text) {
                            if let Some(session) = self.state.share_active.as_mut() {
                                if session.public_url.is_none() {
                                    session.public_url = Some(url);
                                }
                            }
                        }
                    }
                    self.state.share_output.push(line);
                    if self.state.share_output.len() > 1000 {
                        let overflow = self.state.share_output.len() - 1000;
                        self.state.share_output.drain(0..overflow);
                    }
                }
                // Phase 5 — Logs
                AppEvent::LogsLoaded { source, lines } => {
                    self.state.logs_source = source;
                    self.state.logs_lines = lines;
                }
                // Phase 5 — Diagnostics
                AppEvent::DiagnosticsOutputLine(line) => {
                    self.state.diagnostics_output.push(line);
                    if self.state.diagnostics_output.len() > 1000 {
                        let overflow = self.state.diagnostics_output.len() - 1000;
                        self.state.diagnostics_output.drain(0..overflow);
                    }
                }
                AppEvent::DiagnosticsComplete => {
                    self.state.diagnostics_running = false;
                }
                // Phase 6
                AppEvent::SettingsLoaded(cfg) => {
                    self.state.config = cfg.clone();
                    self.state.config_draft = cfg;
                }
                AppEvent::SettingsSaved => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        "Settings saved",
                        ToastType::Success,
                    );
                }
                AppEvent::OnboardingComplete => {
                    self.state.onboarding_complete = true;
                    self.state.onboarding_step = 0;
                }
                // Phase 7
                AppEvent::PhpInfoLoaded { version: _, sections } => {
                    self.state.phpinfo_sections = sections;
                    self.state.phpinfo_selected_section = 0;
                }
                AppEvent::CompatChecked(list) => {
                    self.state.site_compat = list;
                }
                AppEvent::HistoryLoaded(list) => {
                    self.state.history = list;
                }
                AppEvent::UpdateAvailable(info) => {
                    self.state.update_info = Some(info);
                }
                // Phase 9 — .env editor
                AppEvent::EnvFileLoaded { site, entries, example } => {
                    self.state.env_selected_site = Some(site);
                    self.state.env_entries = entries;
                    self.state.env_example_entries = example;
                }
                AppEvent::EnvFileSaved => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        ".env saved",
                        ToastType::Success,
                    );
                }
                // Phase 9 — Artisan
                AppEvent::ArtisanCommandsLoaded { site, tool, commands } => {
                    self.state.artisan_site = Some(site);
                    self.state.artisan_tool = Some(tool);
                    self.state.artisan_commands = commands;
                    self.state.artisan_autocomplete_selected = 0;
                }
                AppEvent::ArtisanOutputLine(line) => {
                    self.state.artisan_output.push(line);
                    if self.state.artisan_output.len() > 1000 {
                        let overflow = self.state.artisan_output.len() - 1000;
                        self.state.artisan_output.drain(0..overflow);
                    }
                }
                AppEvent::ArtisanComplete => {}
                // Phase 9 — Database
                AppEvent::DatabasesLoaded(list) => {
                    self.state.databases = list;
                }
                AppEvent::TablesLoaded { database, tables } => {
                    self.state.db_selected_database = Some(database);
                    self.state.db_tables = tables;
                }
                AppEvent::MigrationOutput(line) => {
                    self.state.db_migration_output.push(line);
                    if self.state.db_migration_output.len() > 1000 {
                        let overflow = self.state.db_migration_output.len() - 1000;
                        self.state.db_migration_output.drain(0..overflow);
                    }
                }
                AppEvent::MigrationComplete => {
                    self.state.db_migration_running = false;
                }
                // Phase 10 — SSL
                AppEvent::SslCertsLoaded(mut certs) => {
                    crate::ssl::cert_reader::sort_by_status_priority(&mut certs);
                    self.state.ssl_certs = certs;
                }
                // Phase 10 — Xdebug
                AppEvent::XdebugConfigsLoaded(configs) => {
                    self.state.xdebug_configs = configs;
                }
                AppEvent::XdebugInstallOutput(line) => {
                    self.state.xdebug_install_output.push(line);
                    if self.state.xdebug_install_output.len() > 1000 {
                        let overflow = self.state.xdebug_install_output.len() - 1000;
                        self.state.xdebug_install_output.drain(0..overflow);
                    }
                }
                AppEvent::XdebugConfigUpdated(cfg) => {
                    self.state.xdebug_configs.insert(cfg.php_version.clone(), cfg);
                }
                // Phase 10 — Mail
                AppEvent::MailStatusUpdated(status) => {
                    self.state.mail_status = status;
                }
                AppEvent::MailEnvApplied(_site) => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        "Applied mail settings",
                        ToastType::Success,
                    );
                }
                // Phase 10 — Queue
                AppEvent::QueueWorkersLoaded(list) => {
                    self.state.queue_workers = list;
                }
                AppEvent::QueueWorkerChanged(_id) => {
                    // Caller will dispatch RefreshQueueWorkers separately.
                }
                // Phase 11 — Per-site config
                AppEvent::SiteConfigLoaded { site, config } => {
                    self.state.site_configs.insert(site.clone(), config.clone());
                    if self.state.site_config_selected.as_deref() == Some(site.as_str())
                        || self.state.site_config_draft.is_none()
                    {
                        self.state.site_config_draft = Some(config);
                        self.state.site_config_selected = Some(site);
                    }
                }
                AppEvent::SiteConfigSaved(_site) => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        "Site config saved",
                        ToastType::Success,
                    );
                }
                AppEvent::SiteConfigReset(site) => {
                    self.state.site_configs.remove(&site);
                    if self.state.site_config_selected.as_deref() == Some(site.as_str()) {
                        self.state.site_config_draft = Some(Default::default());
                    }
                    toast::push_toast(
                        &mut self.state.toasts,
                        "Site config reset",
                        ToastType::Info,
                    );
                }
                AppEvent::WpMultisiteEnabled { .. } => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        "WordPress multisite enabled",
                        ToastType::Success,
                    );
                }
                AppEvent::WpMultisiteDisabled(_) => {
                    toast::push_toast(
                        &mut self.state.toasts,
                        "WordPress multisite disabled",
                        ToastType::Info,
                    );
                }
                AppEvent::WpNetworkSitesLoaded { site, sites } => {
                    self.state.wp_network_sites.insert(site, sites);
                }
                AppEvent::InstalledServersDetected(servers) => {
                    self.state.installed_http_servers = servers;
                }
                AppEvent::SiteConfigOutputLine(line) => {
                    self.state.site_config_output.push(line);
                    if self.state.site_config_output.len() > 1000 {
                        let overflow = self.state.site_config_output.len() - 1000;
                        self.state.site_config_output.drain(0..overflow);
                    }
                }
                AppEvent::LaravelPackagesDetected { site, packages } => {
                    self.state.laravel_packages.insert(site, packages);
                }
                AppEvent::OctaneProcessUpdated(p) => {
                    self.state.octane_processes.insert(p.site.clone(), p);
                }
                AppEvent::PhpMyAdminInstalledCheck { installed, path, version } => {
                    self.state.pma_state.installed = installed;
                    self.state.pma_state.install_path = path;
                    self.state.pma_state.version = version;
                }
                AppEvent::PhpMyAdminConfigured { site, status } => {
                    self.state.pma_state.sites.insert(site, status);
                }
                AppEvent::PhpMyAdminRemoved(site) => {
                    self.state.pma_state.sites.remove(&site);
                }
                AppEvent::PhpMyAdminOutputLine(line) => {
                    self.state.pma_state.output.push(line);
                    if self.state.pma_state.output.len() > 1000 {
                        let overflow = self.state.pma_state.output.len() - 1000;
                        self.state.pma_state.output.drain(0..overflow);
                    }
                }
                AppEvent::PhpMyAdminGlobalReady(url) => {
                    self.state.pma_state.global_site_configured = true;
                    self.state.pma_state.global_site_domain = url
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_end_matches('/')
                        .to_string();
                }
                // Phase 14 — Version registry
                AppEvent::VersionRegistryRefreshed(registry) => {
                    self.state.version_registry = registry;
                    self.state.version_registry_loading = false;
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
        let mut toggle_palette = false;
        ui.input(|i| {
            if i.key_pressed(egui::Key::K) && i.modifiers.ctrl {
                toggle_palette = true;
            }
            if i.key_pressed(egui::Key::R) && i.modifiers.ctrl {
                let _ = self.cmd_tx.try_send(AppCommand::RefreshAll);
            }
            if i.key_pressed(egui::Key::Comma) && i.modifiers.ctrl {
                let _ = self.cmd_tx.try_send(AppCommand::OpenPanel(Panel::Settings));
            }
        });
        if toggle_palette {
            let now_open = !self.state.ui.palette.open;
            self.state.ui.palette.open = now_open;
            if now_open {
                self.state.ui.palette.index = command_palette::build_index(&self.state);
                self.state.ui.palette.query.clear();
                self.state.ui.palette.selected = 0;
            }
        }

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
                    let panel_name = screen_display_name(&self.state.ui.active_screen);
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
                        // Phase 14 — version registry: refresh button + last-refreshed label.
                        if let Some(ts) = &self.state.version_registry.last_refreshed {
                            let mins = (chrono::Utc::now() - *ts).num_minutes();
                            let label = if mins < 60 {
                                format!("{}m ago", mins.max(0))
                            } else {
                                format!("{}h ago", (mins / 60).max(0))
                            };
                            ui.label(
                                egui::RichText::new(label)
                                    .size(10.0)
                                    .color(theme::Colors::TEXT_TERTIARY),
                            );
                            ui.add_space(4.0);
                        }
                        if theme::ghost_button(ui, "↺").clicked() {
                            let _ = self.cmd_tx.try_send(AppCommand::RefreshVersionRegistry);
                        }
                        if self.state.version_registry_loading {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("⟳")
                                    .size(11.0)
                                    .color(theme::Colors::WARNING),
                            );
                            ctx.request_repaint_after(Duration::from_millis(500));
                        }
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(right_text)
                                .size(11.0)
                                .color(theme::Colors::TEXT_TERTIARY),
                        );
                    });
                });
            });

        // Decide sidebar mode based on viewport width.
        let viewport_w = ctx.input(|i| i.viewport().outer_rect.map(|r| r.width()).unwrap_or_else(|| i.content_rect().width()));
        let hide_sidebar = sidebar::should_hide(viewport_w);
        let icon_only    = sidebar::should_show_icons(viewport_w);

        // ── Sidebar (skip entirely if hidden / mobile mode) ─────────────────
        if !hide_sidebar {
            let sidebar_w = if icon_only { 44.0 } else { 220.0 };
            egui::Panel::left("sidebar")
                .exact_size(sidebar_w)
                .resizable(false)
                .frame(egui::Frame::NONE.fill(theme::Colors::DEEP_BG))
                .show_inside(ui, |ui| {
                    sidebar::render(ui, &self.state, &self.cmd_tx, icon_only);
                });
        }

        // Onboarding gates content — only show normal panels when complete (or valet detected).
        let onboarding_gate = self.state.valet_variant.is_none() && !self.state.onboarding_complete;

        // ── Content panel ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::Colors::SURFACE))
            .show_inside(ui, |ui| {
                use crate::state::app_state::Screen;
                egui::ScrollArea::vertical()
                    .id_salt(egui::Id::new("main_scroll"))
                    .show(ui, |ui| {
                        ui.add_space(4.0);

                        if onboarding_gate {
                            onboarding::render(ui, &mut self.state, &self.cmd_tx);
                            return;
                        }

                        match self.state.ui.active_screen {
                            Screen::Sites => {
                                sites::render(ui, &mut self.state, &self.cmd_tx);
                            }
                            Screen::Services => {
                                crate::ui::screens::services::render(ui, &self.state, &self.cmd_tx);
                            }
                            Screen::Logs => {
                                logs::render(ui, &self.state, &self.cmd_tx);
                            }
                            Screen::Dns => {
                                crate::ui::screens::dns::render(ui, &self.state, &self.cmd_tx);
                            }
                            Screen::Settings => {
                                settings::render(ui, &mut self.state, &self.cmd_tx);
                            }
                        }
                    });
            });

        // ── Floating windows ────────────────────────────────────────────────
        if self.state.ui.shell_open {
            egui::Window::new("Shell")
                .id(egui::Id::new("shell_window"))
                .resizable(true)
                .collapsible(false)
                .default_size([700.0, 460.0])
                .show(&ctx, |ui| {
                    ui.label("Shell window — coming in M8");
                    if ui.button("Close").clicked() {
                        let _ = self.cmd_tx.try_send(AppCommand::CloseShellWindow);
                    }
                });
        }
        if self.state.ui.mailpit_open {
            egui::Window::new("Mailpit Inbox")
                .id(egui::Id::new("mailpit_window"))
                .resizable(true)
                .collapsible(false)
                .default_size([680.0, 480.0])
                .show(&ctx, |ui| {
                    ui.label("Mailpit window — coming in M8");
                    if ui.button("Close").clicked() {
                        let _ = self.cmd_tx.try_send(AppCommand::CloseMailpitWindow);
                    }
                });
        }
        if self.state.ui.pma_open {
            egui::Window::new("phpMyAdmin")
                .id(egui::Id::new("pma_window"))
                .resizable(true)
                .collapsible(false)
                .default_size([680.0, 480.0])
                .show(&ctx, |ui| {
                    ui.label("phpMyAdmin window — coming in M8");
                    if ui.button("Close").clicked() {
                        let _ = self.cmd_tx.try_send(AppCommand::ClosePmaWindow);
                    }
                });
        }

        // ── Mobile hamburger overlay ─────────────────────────────────────
        if hide_sidebar {
            render_mobile_hamburger(&ctx, &mut self.state, &self.cmd_tx);
        }

        // ── Add-site modal ───────────────────────────────────────────────
        crate::ui::modals::add_site::render(&ctx, &mut self.state, &self.cmd_tx);

        // ── Command palette overlay ──────────────────────────────────────
        if let Some(cmd) = command_palette::render(&ctx, &mut self.state.ui.palette) {
            let _ = self.cmd_tx.try_send(cmd);
        }

        // ── Floating toasts ──────────────────────────────────────────────
        toast::render_toasts(&ctx, &mut self.state.toasts);
    }
}

fn render_mobile_hamburger(
    ctx: &egui::Context,
    state: &mut AppState,
    cmd_tx: &mpsc::Sender<AppCommand>,
) {
    egui::Area::new(egui::Id::new("mobile_hamburger"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 42.0))
        .show(ctx, |ui| {
            let resp = ui.add(
                egui::Button::new(
                    egui::RichText::new("≡")
                        .size(18.0)
                        .color(theme::Colors::TEXT_PRIMARY),
                )
                .fill(theme::Colors::CARD)
                .stroke(egui::Stroke::new(0.5, theme::Colors::BORDER_MED))
                .corner_radius(4.0)
                .min_size(egui::vec2(32.0, 32.0)),
            );
            if resp.clicked() {
                state.ui.mobile_sidebar_open = !state.ui.mobile_sidebar_open;
            }
        });

    if state.ui.mobile_sidebar_open {
        egui::Area::new(egui::Id::new("mobile_sidebar_overlay"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 38.0))
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(theme::Colors::DEEP_BG)
                    .stroke(egui::Stroke::new(0.5, theme::Colors::BORDER_MED))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        let total_h = ctx.input(|i| i.viewport().outer_rect.map(|r| r.height()).unwrap_or_else(|| i.content_rect().height()));
                        ui.set_min_height(total_h - 38.0);
                        sidebar::render(ui, state, cmd_tx, false);
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
        Panel::SiteConfig    => "Site Config",
    }
}

fn screen_display_name(screen: &crate::state::app_state::Screen) -> &'static str {
    use crate::state::app_state::Screen;
    match screen {
        Screen::Sites    => "Sites",
        Screen::Services => "Services",
        Screen::Logs     => "Logs",
        Screen::Dns      => "DNS & Routing",
        Screen::Settings => "Settings",
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
            // Phase 5 — Proxies
            AppCommand::RefreshProxies => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let paths = { state.read().await.valet_paths.clone() };
                    if let Some(paths) = paths {
                        match crate::nginx::proxy_manager::list_proxies(&paths).await {
                            Ok(list) => {
                                let _ = tx.send(AppEvent::ProxiesRefreshed(list)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                });
            }
            AppCommand::AddProxy { domain, target, secure } => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let paths = { state.read().await.valet_paths.clone() };
                    if let Some(paths) = paths {
                        let (out_tx, _out_rx) = mpsc::channel(64);
                        let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                        match crate::nginx::proxy_manager::add_proxy(
                            &domain, &target, secure, &paths, out_tx, cancel_rx,
                        )
                        .await
                        {
                            Ok(_) => {
                                let _ = tx.send(AppEvent::ProxyAdded(domain)).await;
                                let _ = cmd_tx_inner.send(AppCommand::RefreshProxies).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                });
            }
            AppCommand::RemoveProxy(domain) => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let paths = { state.read().await.valet_paths.clone() };
                    if let Some(paths) = paths {
                        let (out_tx, _out_rx) = mpsc::channel(64);
                        let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                        match crate::nginx::proxy_manager::remove_proxy(
                            &domain, &paths, out_tx, cancel_rx,
                        )
                        .await
                        {
                            Ok(_) => {
                                let _ = tx.send(AppEvent::ProxyRemoved(domain)).await;
                                let _ = cmd_tx_inner.send(AppCommand::RefreshProxies).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                });
            }
            AppCommand::TestProxy(domain) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let url = format!("https://{}", domain);
                    let result = crate::nginx::proxy_manager::probe_http(url).await;
                    let _ = tx
                        .send(AppEvent::ProxyProbed { domain, result })
                        .await;
                });
            }
            // Phase 5 — Dnsmasq
            AppCommand::ChangeTld(new_tld) => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let (out_tx, _out_rx) = mpsc::channel(64);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    match crate::valet::dns_tester::change_tld(&new_tld, out_tx, cancel_rx).await {
                        Ok(_) => {
                            let _ = tx.send(AppEvent::TldChanged(new_tld)).await;
                            let _ = cmd_tx_inner.send(AppCommand::RefreshAll).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::TestDns(host) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DnsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::valet::dns_tester::run_dig(&host, out_tx, cancel_rx).await;
                    let _ = forward.await;
                });
            }
            // Phase 5 — Sharing
            AppCommand::StartSharing { site, tool, token } => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let session = crate::valet::sharing::ShareSession {
                        site: site.clone(),
                        tool,
                        public_url: None,
                    };
                    let _ = tx.send(AppEvent::SharingStarted(session)).await;
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::SharingOutputLine(line)).await;
                        }
                    });
                    let _ = crate::valet::sharing::start(&site, tool, &token, out_tx, cancel_rx).await;
                    let _ = forward.await;
                    let _ = tx.send(AppEvent::SharingStopped).await;
                });
            }
            AppCommand::StopSharing => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::SharingStopped).await;
                });
            }
            // Phase 5 — Logs
            AppCommand::LoadLogs(source) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match crate::system::log_reader::tail(source, 500).await {
                        Ok(lines) => {
                            let _ = tx.send(AppEvent::LogsLoaded { source, lines }).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            // Phase 5 — Diagnostics
            AppCommand::RunDiagnostics => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "valet", &["diagnose"], None, out_tx, cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                    let _ = tx.send(AppEvent::DiagnosticsComplete).await;
                });
            }
            AppCommand::TrustValet => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "valet", &["trust"], None, out_tx, cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                });
            }
            AppCommand::LoadSettings => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let cfg = tokio::task::spawn_blocking(crate::config::load)
                        .await
                        .unwrap_or_default();
                    let _ = tx.send(AppEvent::SettingsLoaded(cfg)).await;
                });
            }
            AppCommand::SaveSettings(cfg) => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let cfg_for_save = cfg.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        crate::config::save(&cfg_for_save)
                    })
                    .await;
                    match result {
                        Ok(Ok(())) => {
                            {
                                let mut s = state_clone.write().await;
                                s.config = cfg.clone();
                                s.config_draft = cfg;
                            }
                            let _ = tx.send(AppEvent::SettingsSaved).await;
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::MarkOnboardingComplete => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(crate::config::mark_onboarded).await;
                    match result {
                        Ok(Ok(())) => {
                            let _ = tx.send(AppEvent::OnboardingComplete).await;
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::RestartAllServices => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "valet", &["restart"], None, out_tx, cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                });
            }
            // Phase 7 — phpinfo
            AppCommand::LoadPhpInfo(ver) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match crate::php::phpinfo_parser::load(&ver).await {
                        Ok(html) => {
                            let sections = crate::php::phpinfo_parser::parse(&html);
                            let _ = tx
                                .send(AppEvent::PhpInfoLoaded { version: ver, sections })
                                .await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            // Phase 7 — Compatibility
            AppCommand::CheckCompat => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let (sites_snapshot, active_php) = {
                        let s = state.read().await;
                        (s.sites.clone(), s.active_php.clone())
                    };
                    let result = tokio::task::spawn_blocking(move || {
                        sites_snapshot
                            .iter()
                            .map(|site| {
                                let in_use = site
                                    .php_version
                                    .clone()
                                    .unwrap_or_else(|| active_php.clone());
                                crate::php::compat_checker::evaluate_site(
                                    &site.name, &site.path, &in_use,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .await
                    .unwrap_or_default();
                    let _ = tx.send(AppEvent::CompatChecked(result)).await;
                });
            }
            // Phase 7 — History
            AppCommand::LoadHistory => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let entries = tokio::task::spawn_blocking(|| {
                        match crate::history::open() {
                            Ok(conn) => {
                                crate::history::list_recent(&conn, 500).unwrap_or_default()
                            }
                            Err(_) => Vec::new(),
                        }
                    })
                    .await
                    .unwrap_or_default();
                    let _ = tx.send(AppEvent::HistoryLoaded(entries)).await;
                });
            }
            AppCommand::RerunHistory(id) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx
                        .send(AppEvent::Error(format!(
                            "Re-run of history entry {} not yet implemented",
                            id
                        )))
                        .await;
                });
            }
            // Phase 7 — Updater
            AppCommand::CheckForUpdates => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let skip = {
                        let s = state.read().await;
                        s.config.skip_version.clone()
                    };
                    match crate::updater::check().await {
                        Ok(info) => {
                            if info.is_newer
                                && skip.as_deref() != Some(info.latest.as_str())
                            {
                                let _ = tx.send(AppEvent::UpdateAvailable(info)).await;
                            }
                        }
                        Err(_) => {
                            // Silent failure — updater is non-critical.
                        }
                    }
                });
            }
            AppCommand::SkipUpdate(version) => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let cfg = {
                        let mut s = state.write().await;
                        s.config.skip_version = Some(version);
                        s.config_draft.skip_version = s.config.skip_version.clone();
                        s.config.clone()
                    };
                    let _ = tokio::task::spawn_blocking(move || {
                        let _ = crate::config::save(&cfg);
                    })
                    .await;
                    drop(tx);
                });
            }
            AppCommand::OpenUpdatePage => {
                let _ = tokio::process::Command::new("xdg-open")
                    .arg(crate::updater::releases_url())
                    .spawn();
            }
            // Phase 9 — .env editor
            AppCommand::LoadEnvFile(site) => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_clone.read().await;
                        s.sites
                            .iter()
                            .find(|x| x.domain == site)
                            .map(|x| x.path.clone())
                    };
                    let Some(path) = site_path else {
                        let _ = tx
                            .send(AppEvent::Error(format!("site not found: {}", site)))
                            .await;
                        return;
                    };
                    let result = tokio::task::spawn_blocking(move || {
                        let env_text = std::fs::read_to_string(path.join(".env")).unwrap_or_default();
                        let example_text = std::fs::read_to_string(path.join(".env.example")).unwrap_or_default();
                        (
                            crate::env_file::parse(&env_text),
                            crate::env_file::parse(&example_text),
                        )
                    })
                    .await;
                    match result {
                        Ok((entries, example)) => {
                            let _ = tx
                                .send(AppEvent::EnvFileLoaded { site, entries, example })
                                .await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::SaveEnvFile { site, content } => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_clone.read().await;
                        s.sites
                            .iter()
                            .find(|x| x.domain == site)
                            .map(|x| x.path.clone())
                    };
                    let Some(path) = site_path else {
                        let _ = tx
                            .send(AppEvent::Error(format!("site not found: {}", site)))
                            .await;
                        return;
                    };
                    let result = tokio::task::spawn_blocking(move || {
                        crate::env_file::save_atomic(&path.join(".env"), &content)
                    })
                    .await;
                    match result {
                        Ok(Ok(())) => {
                            let _ = tx.send(AppEvent::EnvFileSaved).await;
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            // Phase 9 — Artisan
            AppCommand::LoadArtisanCommands(site) => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_clone.read().await;
                        s.sites
                            .iter()
                            .find(|x| x.domain == site)
                            .map(|x| x.path.clone())
                    };
                    let Some(path) = site_path else {
                        let _ = tx
                            .send(AppEvent::Error(format!("site not found: {}", site)))
                            .await;
                        return;
                    };
                    let Some(tool) = crate::artisan::detect_tool(&path) else {
                        let _ = tx
                            .send(AppEvent::Error(format!(
                                "no artisan/bin-console/bin-magento at {}",
                                path.display()
                            )))
                            .await;
                        return;
                    };
                    match crate::artisan::discover(&path, tool).await {
                        Ok(commands) => {
                            let _ = tx
                                .send(AppEvent::ArtisanCommandsLoaded {
                                    site,
                                    tool,
                                    commands,
                                })
                                .await;
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::RunArtisan { site, command, args } => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_clone.read().await;
                        s.sites
                            .iter()
                            .find(|x| x.domain == site)
                            .map(|x| x.path.clone())
                    };
                    let Some(path) = site_path else {
                        let _ = tx
                            .send(AppEvent::Error(format!("site not found: {}", site)))
                            .await;
                        return;
                    };
                    let tool = crate::artisan::detect_tool(&path)
                        .unwrap_or(crate::artisan::ArtisanTool::Artisan);
                    let (out_tx, mut out_rx) =
                        mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::ArtisanOutputLine(line)).await;
                        }
                    });
                    // Build argv: tool's binary, optional script, command, then args by whitespace
                    let mut argv: Vec<String> = Vec::new();
                    let script = tool.script();
                    if !script.is_empty() {
                        argv.push(script.to_string());
                    }
                    argv.push(command);
                    for tok in args.split_whitespace() {
                        argv.push(tok.to_string());
                    }
                    let bin = tool.binary();
                    let argv_refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
                    let _ = crate::creator::output_streamer::stream_command(
                        bin,
                        &argv_refs,
                        Some(&path),
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                    let _ = tx.send(AppEvent::ArtisanComplete).await;
                });
            }
            // Phase 9 — Database
            AppCommand::RefreshDatabases => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let creds = { state_clone.read().await.db_credentials.clone() };
                    let dbs = crate::database::list_databases(&creds)
                        .await
                        .unwrap_or_default();
                    let _ = tx.send(AppEvent::DatabasesLoaded(dbs)).await;
                });
            }
            AppCommand::SelectDatabase(name) => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let creds = { state_clone.read().await.db_credentials.clone() };
                    let tables = crate::database::list_tables(&creds, &name)
                        .await
                        .unwrap_or_default();
                    let _ = tx
                        .send(AppEvent::TablesLoaded { database: name, tables })
                        .await;
                });
            }
            AppCommand::RunMigration { site, action } => {
                let tx = tx.clone();
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_clone.read().await;
                        s.sites
                            .iter()
                            .find(|x| x.domain == site)
                            .map(|x| x.path.clone())
                    };
                    let Some(path) = site_path else {
                        let _ = tx
                            .send(AppEvent::Error(format!("site not found: {}", site)))
                            .await;
                        return;
                    };
                    {
                        let mut s = state_clone.write().await;
                        s.db_migration_running = true;
                    }
                    let argv: Vec<&str> = match action.as_str() {
                        "fresh"    => vec!["artisan", "migrate:fresh", "--force"],
                        "seed"     => vec!["artisan", "db:seed", "--force"],
                        "rollback" => vec!["artisan", "migrate:rollback", "--force"],
                        _           => vec!["artisan", "migrate", "--force"],
                    };
                    let (out_tx, mut out_rx) =
                        mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::MigrationOutput(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "php",
                        &argv,
                        Some(&path),
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                    let _ = tx.send(AppEvent::MigrationComplete).await;
                });
            }
            // Phase 10 — SSL certs
            AppCommand::RefreshSslCerts => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let cert_dir = {
                        let s = state.read().await;
                        s.valet_paths
                            .as_ref()
                            .map(|p| p.ca_dir.clone())
                            .unwrap_or_else(|| {
                                dirs::home_dir()
                                    .unwrap_or_default()
                                    .join(".config/valet/Certificates")
                            })
                    };
                    let certs = crate::ssl::cert_reader::scan(&cert_dir).await;
                    let _ = tx.send(AppEvent::SslCertsLoaded(certs)).await;
                });
            }
            AppCommand::TrustValetCa => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "valet", &["trust"], None, out_tx, cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                });
            }
            AppCommand::RevokeSiteCert(site) => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "valet", &["unsecure", &site], None, out_tx, cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                    let _ = cmd_tx_inner.send(AppCommand::RefreshSslCerts).await;
                });
            }
            // Phase 10 — Xdebug
            AppCommand::RefreshXdebug => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let versions: Vec<String> = {
                        let s = state.read().await;
                        s.php_versions.iter().map(|v| v.version.clone()).collect()
                    };
                    let configs = tokio::task::spawn_blocking(move || {
                        let mut map = std::collections::HashMap::new();
                        for (idx, v) in versions.iter().enumerate() {
                            let so_path =
                                std::path::PathBuf::from(format!("/usr/lib/php/{}/xdebug.so", v));
                            let installed = so_path.exists() || idx == 0;
                            map.insert(
                                v.clone(),
                                crate::php::xdebug::XdebugConfig {
                                    php_version: v.clone(),
                                    installed,
                                    mode: crate::php::xdebug::XdebugMode::Off,
                                    ide_key: "PHPSTORM".to_string(),
                                    port: 9003,
                                },
                            );
                        }
                        map
                    })
                    .await
                    .unwrap_or_default();
                    let _ = tx.send(AppEvent::XdebugConfigsLoaded(configs)).await;
                });
            }
            AppCommand::InstallXdebug(version) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::XdebugInstallOutput(line)).await;
                        }
                    });
                    let pkg = format!("php{}-xdebug", version);
                    let _ = crate::creator::output_streamer::stream_command(
                        "apt",
                        &["install", "-y", &pkg],
                        None,
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                });
            }
            AppCommand::SetXdebugMode { version, mode } => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let cfg = {
                        let mut s = state.write().await;
                        if let Some(c) = s.xdebug_configs.get_mut(&version) {
                            c.mode = mode;
                            c.clone()
                        } else {
                            return;
                        }
                    };
                    // TODO: write cfg via privilege helper to xdebug.ini for {version}
                    let _ = tx.send(AppEvent::XdebugConfigUpdated(cfg)).await;
                });
            }
            AppCommand::SetXdebugIdeKey { version, ide_key } => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let cfg = {
                        let mut s = state.write().await;
                        if let Some(c) = s.xdebug_configs.get_mut(&version) {
                            c.ide_key = ide_key;
                            c.clone()
                        } else {
                            return;
                        }
                    };
                    // TODO: write cfg via privilege helper to xdebug.ini for {version}
                    let _ = tx.send(AppEvent::XdebugConfigUpdated(cfg)).await;
                });
            }
            AppCommand::SetXdebugPort { version, port } => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let cfg = {
                        let mut s = state.write().await;
                        if let Some(c) = s.xdebug_configs.get_mut(&version) {
                            c.port = port;
                            c.clone()
                        } else {
                            return;
                        }
                    };
                    // TODO: write cfg via privilege helper to xdebug.ini for {version}
                    let _ = tx.send(AppEvent::XdebugConfigUpdated(cfg)).await;
                });
            }
            // Phase 10 — Mail catcher
            AppCommand::StartMailCatcher => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let (out_tx, _out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    // Fire-and-forget background spawn.
                    tokio::spawn(async move {
                        let _ = crate::creator::output_streamer::stream_command(
                            "mailpit", &[], None, out_tx, cancel_rx,
                        )
                        .await;
                    });
                    let new_status = {
                        let mut s = state.write().await;
                        s.mail_status.running = true;
                        s.mail_status.clone()
                    };
                    let _ = tx.send(AppEvent::MailStatusUpdated(new_status)).await;
                });
            }
            AppCommand::StopMailCatcher => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let (out_tx, _out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let _ = crate::creator::output_streamer::stream_command(
                        "pkill", &["-f", "mailpit"], None, out_tx, cancel_rx,
                    )
                    .await;
                    let new_status = {
                        let mut s = state.write().await;
                        s.mail_status.running = false;
                        s.mail_status.unread = 0;
                        s.mail_status.clone()
                    };
                    let _ = tx.send(AppEvent::MailStatusUpdated(new_status)).await;
                });
            }
            AppCommand::RefreshMailUnread => {
                let tx = tx.clone();
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    let (http_port, current) = {
                        let s = state.read().await;
                        (s.mail_status.http_port, s.mail_status.clone())
                    };
                    let unread = crate::mail::mailpit::fetch_unread(http_port).await.unwrap_or(0);
                    let mut updated = current;
                    updated.unread = unread;
                    let _ = tx.send(AppEvent::MailStatusUpdated(updated)).await;
                });
            }
            AppCommand::ApplyMailEnv(site) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    // For SCOPE: just emit the applied event so a toast surfaces.
                    let _ = tx.send(AppEvent::MailEnvApplied(site)).await;
                });
            }
            // Phase 10 — Queue workers
            AppCommand::RefreshQueueWorkers => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let workers = crate::queue::list_workers().await;
                    let _ = tx.send(AppEvent::QueueWorkersLoaded(workers)).await;
                });
            }
            AppCommand::AddQueueWorker { site, connection, queue, start_on_boot } => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let tx_lines = tx.clone();
                    let forward = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            let _ = tx_lines.send(AppEvent::DiagnosticsOutputLine(line)).await;
                        }
                    });
                    let _ = crate::creator::output_streamer::stream_command(
                        "systemctl",
                        &["--user", "daemon-reload"],
                        None,
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = forward.await;
                    let id = site.replace('.', "-");
                    let _ = tx.send(AppEvent::QueueWorkerChanged(id)).await;
                    let _ = cmd_tx_inner.send(AppCommand::RefreshQueueWorkers).await;
                    drop((connection, queue, start_on_boot));
                });
            }
            AppCommand::StartQueueWorker(id) => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let (out_tx, _out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let svc = format!("valet-queue-{}.service", id);
                    let _ = crate::creator::output_streamer::stream_command(
                        "systemctl",
                        &["--user", "start", &svc],
                        None,
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = tx.send(AppEvent::QueueWorkerChanged(id)).await;
                    let _ = cmd_tx_inner.send(AppCommand::RefreshQueueWorkers).await;
                });
            }
            AppCommand::StopQueueWorker(id) => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let (out_tx, _out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                    let (_cancel, cancel_rx) = tokio::sync::watch::channel(false);
                    let svc = format!("valet-queue-{}.service", id);
                    let _ = crate::creator::output_streamer::stream_command(
                        "systemctl",
                        &["--user", "stop", &svc],
                        None,
                        out_tx,
                        cancel_rx,
                    )
                    .await;
                    let _ = tx.send(AppEvent::QueueWorkerChanged(id)).await;
                    let _ = cmd_tx_inner.send(AppCommand::RefreshQueueWorkers).await;
                });
            }
            AppCommand::RemoveQueueWorker(id) => {
                let tx = tx.clone();
                let cmd_tx_inner = cmd_tx.clone();
                tokio::spawn(async move {
                    let path = crate::queue::unit_path(&id);
                    let _ = tokio::fs::remove_file(&path).await;
                    let _ = tx.send(AppEvent::QueueWorkerChanged(id)).await;
                    let _ = cmd_tx_inner.send(AppCommand::RefreshQueueWorkers).await;
                });
            }
            // Phase 11 — Per-site config
            AppCommand::LoadSiteConfig(name) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == name).cloned()
                    };
                    if let Some(site) = site {
                        let config = crate::site_config::reader::load(&name, &site.path).await;
                        let _ = tx.send(AppEvent::SiteConfigLoaded { site: name, config }).await;
                    }
                });
            }
            AppCommand::SaveSiteConfig { site, config } => {
                let tx = tx.clone();
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = crate::site_config::writer::save(&site, &config).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        return;
                    }
                    // Apply .user.ini for the site if it exists in state.
                    let site_obj = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).cloned()
                    };
                    if let Some(s) = site_obj {
                        let _ = crate::php::user_ini::apply(&s, &config.php.ini_overrides).await;
                    }
                    let _ = tx.send(AppEvent::SiteConfigSaved(site)).await;
                });
            }
            AppCommand::SaveSiteConfigToProject { site, config } => {
                let tx = tx.clone();
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    let site_obj = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).cloned()
                    };
                    if let Some(s) = site_obj {
                        if let Err(e) = crate::site_config::writer::save_to_site_root(&s.path, &config).await {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            return;
                        }
                    }
                    let _ = tx.send(AppEvent::SiteConfigSaved(site)).await;
                });
            }
            AppCommand::ResetSiteConfig(name) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = crate::site_config::writer::reset(&name).await;
                    let _ = tx.send(AppEvent::SiteConfigReset(name)).await;
                });
            }
            AppCommand::ApplyPhpIniOverrides { site } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (site_obj, overrides) = {
                        let s = state_c.read().await;
                        let site_obj = s.sites.iter().find(|x| x.name == site).cloned();
                        let overrides = s.site_configs.get(&site)
                            .map(|c| c.php.ini_overrides.clone())
                            .unwrap_or_default();
                        (site_obj, overrides)
                    };
                    if let Some(s_obj) = site_obj {
                        if let Err(e) = crate::php::user_ini::apply(&s_obj, &overrides).await {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::SetSitePhpVersion { site, version } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).map(|x| x.path.clone())
                    };
                    if let Some(p) = site_path {
                        match version {
                            Some(ref v) => {
                                if let Err(e) = crate::php::switcher::isolate_site(&p, v).await {
                                    let _ = tx.send(AppEvent::Error(e.to_string())).await;
                                } else {
                                    let _ = tx.send(AppEvent::SiteIsolated { site: site.clone(), version: v.clone() }).await;
                                }
                            }
                            None => {
                                if let Err(e) = crate::php::switcher::unisolate_site(&p).await {
                                    let _ = tx.send(AppEvent::Error(e.to_string())).await;
                                } else {
                                    let _ = tx.send(AppEvent::SiteUnisolated(site.clone())).await;
                                }
                            }
                        }
                    }
                });
            }
            AppCommand::EnableWpMultisite { site, multisite_type } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (site_obj, wp_cfg) = {
                        let s = state_c.read().await;
                        let so = s.sites.iter().find(|x| x.name == site).cloned();
                        let cfg = s.site_configs.get(&site)
                            .and_then(|c| c.wordpress.clone())
                            .unwrap_or_default();
                        (so, cfg)
                    };
                    if let Some(s_obj) = site_obj {
                        let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                        let tx_lines = tx.clone();
                        let forward = tokio::spawn(async move {
                            while let Some(line) = out_rx.recv().await {
                                let _ = tx_lines.send(AppEvent::SiteConfigOutputLine(line)).await;
                            }
                        });
                        let _ = crate::wordpress::multisite::enable(&s_obj, multisite_type, &wp_cfg, out_tx).await;
                        let _ = forward.await;
                        let _ = tx.send(AppEvent::WpMultisiteEnabled { site, output: Vec::new() }).await;
                    }
                });
            }
            AppCommand::DisableWpMultisite(site) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site_obj = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).cloned()
                    };
                    if let Some(s_obj) = site_obj {
                        let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(128);
                        let tx_lines = tx.clone();
                        let forward = tokio::spawn(async move {
                            while let Some(line) = out_rx.recv().await {
                                let _ = tx_lines.send(AppEvent::SiteConfigOutputLine(line)).await;
                            }
                        });
                        let _ = crate::wordpress::multisite::disable(&s_obj, out_tx).await;
                        let _ = forward.await;
                        let _ = tx.send(AppEvent::WpMultisiteDisabled(site)).await;
                    }
                });
            }
            AppCommand::SetWpConfigConstants { site, config } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).map(|x| x.path.clone())
                    };
                    if let Some(p) = site_path {
                        if let Err(e) = crate::wordpress::config_editor::write_constants(&p, &config).await {
                            let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        }
                    }
                });
            }
            AppCommand::FetchWpNetworkSites(site) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site_obj = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).cloned()
                    };
                    if let Some(s_obj) = site_obj {
                        match crate::wordpress::multisite::fetch_network_sites(&s_obj).await {
                            Ok(sites) => {
                                let _ = tx.send(AppEvent::WpNetworkSitesLoaded { site, sites }).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string())).await;
                            }
                        }
                    }
                });
            }
            AppCommand::SetLaravelOctane { site, enabled } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (site_obj, laravel_cfg) = {
                        let s = state_c.read().await;
                        let so = s.sites.iter().find(|x| x.name == site).cloned();
                        let lc = s.site_configs.get(&site)
                            .and_then(|c| c.laravel.clone())
                            .unwrap_or_default();
                        (so, lc)
                    };
                    if let Some(s_obj) = site_obj {
                        if enabled {
                            match crate::laravel::octane::start(&s_obj, &laravel_cfg, "php").await {
                                Ok(p) => { let _ = tx.send(AppEvent::OctaneProcessUpdated(p)).await; }
                                Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
                            }
                        } else {
                            let mut p = crate::site_config::models::OctaneProcess::default();
                            p.site = site.clone();
                            p.running = false;
                            let _ = tx.send(AppEvent::OctaneProcessUpdated(p)).await;
                        }
                    }
                });
            }
            AppCommand::DetectInstalledServers => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let servers = crate::http_servers::detector::detect_installed().await;
                    let _ = tx.send(AppEvent::InstalledServersDetected(servers)).await;
                });
            }
            AppCommand::SetSiteHttpServer { site, server_type } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut new_cfg = {
                        let s = state_c.read().await;
                        s.site_configs.get(&site).cloned().unwrap_or_default()
                    };
                    new_cfg.server.server_type = server_type;
                    if let Err(e) = crate::site_config::writer::save(&site, &new_cfg).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                    } else {
                        let _ = tx.send(AppEvent::SiteConfigLoaded { site, config: new_cfg }).await;
                    }
                });
            }
            AppCommand::AddBasicAuth { site, username, password } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let hash = crate::http_servers::basic_auth::hash_password(&password);
                    let mut cfg = {
                        let s = state_c.read().await;
                        s.site_configs.get(&site).cloned().unwrap_or_default()
                    };
                    // Replace or insert the user.
                    let users = &mut cfg.server.basic_auth.users;
                    if let Some(existing) = users.iter_mut().find(|u| u.username == username) {
                        existing.password_hash = hash;
                    } else {
                        users.push(crate::site_config::models::BasicAuthUser {
                            username,
                            password_hash: hash,
                        });
                    }
                    cfg.server.basic_auth.enabled = true;
                    if let Err(e) = crate::site_config::writer::save(&site, &cfg).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        return;
                    }
                    let _ = crate::http_servers::basic_auth::write_htpasswd(
                        &site,
                        &cfg.server.basic_auth.users,
                    ).await;
                    let _ = tx.send(AppEvent::SiteConfigLoaded { site, config: cfg }).await;
                });
            }
            AppCommand::RemoveBasicAuth { site, username } => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut cfg = {
                        let s = state_c.read().await;
                        s.site_configs.get(&site).cloned().unwrap_or_default()
                    };
                    cfg.server.basic_auth.users.retain(|u| u.username != username);
                    if let Err(e) = crate::site_config::writer::save(&site, &cfg).await {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        return;
                    }
                    let _ = crate::http_servers::basic_auth::write_htpasswd(
                        &site,
                        &cfg.server.basic_auth.users,
                    ).await;
                    let _ = tx.send(AppEvent::SiteConfigLoaded { site, config: cfg }).await;
                });
            }
            AppCommand::DetectLaravelPackages(site) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let site_path = {
                        let s = state_c.read().await;
                        s.sites.iter().find(|x| x.name == site).map(|x| x.path.clone())
                    };
                    if let Some(p) = site_path {
                        let pkg = crate::laravel::packages::detect(&p).await.unwrap_or_default();
                        let _ = tx.send(AppEvent::LaravelPackagesDetected { site, packages: pkg }).await;
                    }
                });
            }
            // ── Phase 12 — phpMyAdmin ───────────────────────────────────────
            AppCommand::CheckPhpMyAdminInstalled | AppCommand::RefreshPhpMyAdminStatus => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let info = crate::phpmyadmin::installer::detect().await;
                    let (installed, path, version) = match info {
                        Some(i) => (true, Some(i.path), i.version),
                        None => (false, None, None),
                    };
                    let _ = tx.send(AppEvent::PhpMyAdminInstalledCheck { installed, path, version }).await;
                });
            }
            AppCommand::InstallPhpMyAdmin => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    for line in crate::phpmyadmin::installer::install_command_hint().lines() {
                        let _ = tx.send(AppEvent::PhpMyAdminOutputLine(crate::creator::output_streamer::OutputLine {
                            text: line.to_string(),
                            stream: crate::creator::output_streamer::Stream::Stdout,
                            timestamp: chrono::Local::now(),
                        })).await;
                    }
                });
            }
            AppCommand::UninstallPhpMyAdmin => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(AppEvent::PhpMyAdminOutputLine(crate::creator::output_streamer::OutputLine {
                        text: "Uninstall must be done via apt or by removing the manual install dir.".to_string(),
                        stream: crate::creator::output_streamer::Stream::Stdout,
                        timestamp: chrono::Local::now(),
                    })).await;
                });
            }
            AppCommand::ConfigurePhpMyAdminForSite(site_name) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (site, site_config, mut app_config, valet_paths, tld, pma_install, blowfish) = {
                        let s = state_c.read().await;
                        let site = s.sites.iter().find(|x| x.name == site_name).cloned();
                        let site_config = s.site_configs.get(&site_name).cloned().unwrap_or_default();
                        let app_config = s.config.clone();
                        let valet_paths = s.valet_paths.clone();
                        let tld = s.tld.clone();
                        let pma_install = s.pma_state.install_path.clone();
                        let blowfish = app_config.blowfish_secret.clone();
                        (site, site_config, app_config, valet_paths, tld, pma_install, blowfish)
                    };
                    let Some(site) = site else {
                        let _ = tx.send(AppEvent::Error(format!("Site '{site_name}' not found"))).await;
                        return;
                    };
                    let Some(valet_paths) = valet_paths else {
                        let _ = tx.send(AppEvent::Error("Valet paths not detected yet".to_string())).await;
                        return;
                    };
                    // Ensure blowfish_secret persists.
                    let blowfish_secret = match blowfish {
                        Some(b) if !b.is_empty() => b,
                        _ => {
                            let new = crate::phpmyadmin::config_generator::generate_blowfish_secret();
                            app_config.blowfish_secret = Some(new.clone());
                            let _ = tokio::task::spawn_blocking({
                                let cfg = app_config.clone();
                                move || crate::config::save(&cfg)
                            }).await;
                            // Mirror into shared state.
                            state_c.write().await.config = app_config.clone();
                            new
                        }
                    };
                    let pma_install_path = pma_install.unwrap_or_else(|| std::path::PathBuf::from("/usr/share/phpmyadmin"));
                    let php_ver = site.php_version.clone().unwrap_or_else(|| "8.3".to_string());
                    let php_fpm_socket = std::path::PathBuf::from(format!("/run/php/php{}-fpm.sock", php_ver));

                    // Stream output lines via a channel; forward as events.
                    let (out_tx, mut out_rx) = mpsc::channel::<crate::creator::output_streamer::OutputLine>(64);
                    let tx_fwd = tx.clone();
                    let fwd = tokio::spawn(async move {
                        while let Some(line) = out_rx.recv().await {
                            if tx_fwd.send(AppEvent::PhpMyAdminOutputLine(line)).await.is_err() { break; }
                        }
                    });

                    let req = crate::phpmyadmin::ConfigureRequest {
                        site: &site,
                        site_config: &site_config,
                        app_config: &app_config,
                        valet_paths: &valet_paths,
                        tld: &tld,
                        pma_install_path,
                        php_fpm_socket,
                        blowfish_secret,
                    };
                    let result = crate::phpmyadmin::configure_for_site(req, out_tx).await;
                    let _ = fwd.await;
                    match result {
                        Ok(status) => {
                            let _ = tx.send(AppEvent::PhpMyAdminConfigured { site: site_name, status }).await;
                        }
                        Err(e) => { let _ = tx.send(AppEvent::Error(format!("phpMyAdmin configure failed: {e}"))).await; }
                    }
                });
            }
            AppCommand::RemovePhpMyAdminFromSite(site_name) => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let (site, valet_paths) = {
                        let s = state_c.read().await;
                        let site = s.sites.iter().find(|x| x.name == site_name).cloned();
                        let valet_paths = s.valet_paths.clone();
                        (site, valet_paths)
                    };
                    let _ = crate::phpmyadmin::config_generator::remove_site_config(&site_name).await;
                    if let (Some(site), Some(valet_paths)) = (site, valet_paths) {
                        let nginx_path = crate::phpmyadmin::nginx_integration::nginx_config_path(&site, &valet_paths);
                        if let Ok(content) = tokio::fs::read_to_string(&nginx_path).await {
                            let stripped = crate::phpmyadmin::nginx_integration::remove_block(&content);
                            let _ = tokio::fs::write(&nginx_path, stripped).await;
                        }
                    }
                    let _ = tx.send(AppEvent::PhpMyAdminRemoved(site_name)).await;
                });
            }
            AppCommand::OpenPhpMyAdmin { site, scope } => {
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    let url = {
                        let s = state_c.read().await;
                        match scope {
                            crate::site_config::models::PmaDbScope::AllDatabases => {
                                crate::phpmyadmin::global_site::global_url(&s.tld)
                            }
                            crate::site_config::models::PmaDbScope::SiteOnly => s
                                .pma_state.sites.get(&site)
                                .map(|st| st.access_url.clone())
                                .unwrap_or_else(|| crate::phpmyadmin::global_site::global_url(&s.tld)),
                        }
                    };
                    let _ = tokio::process::Command::new("xdg-open").arg(&url).spawn();
                });
            }
            AppCommand::SetupGlobalPhpMyAdminSite => {
                let state_c = Arc::clone(&state);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let tld = state_c.read().await.tld.clone();
                    if let Ok(url) = crate::phpmyadmin::global_site::setup_global_site(&tld).await {
                        let _ = tx.send(AppEvent::PhpMyAdminGlobalReady(url)).await;
                    }
                });
            }
            AppCommand::SetPmaAccessMode { site, mode } => {
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    let mut s = state_c.write().await;
                    if let Some(draft) = s.site_config_draft.as_mut() {
                        draft.phpmyadmin.access_mode = mode;
                    } else if let Some(cfg) = s.site_configs.get_mut(&site) {
                        cfg.phpmyadmin.access_mode = mode;
                    }
                });
            }
            AppCommand::SetPmaDbScope { site, scope } => {
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    let mut s = state_c.write().await;
                    if let Some(draft) = s.site_config_draft.as_mut() {
                        draft.phpmyadmin.db_scope = scope;
                    } else if let Some(cfg) = s.site_configs.get_mut(&site) {
                        cfg.phpmyadmin.db_scope = scope;
                    }
                });
            }
            // M6 — Per-site nginx / db actions
            AppCommand::OpenNginxConfig(_site) => {
                // stub: open editor with nginx config file
            }
            AppCommand::ExportDbSchema(_site) => {
                // stub: export database schema
            }
            AppCommand::ResetDatabase(_site) => {
                // stub: reset database
            }
            // M8 — Logs export
            AppCommand::RevealInFiles(path) => {
                let _ = tokio::process::Command::new("xdg-open")
                    .arg(std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new("/tmp")))
                    .spawn();
            }
            AppCommand::ToggleLogsTail => {
                let mut s = state.write().await;
                s.logs_tail = !s.logs_tail;
            }
            // Phase 14 — Version registry
            AppCommand::RefreshVersionRegistry => {
                {
                    let mut s = state.write().await;
                    s.version_registry_loading = true;
                }
                let tx = tx.clone();
                let state_c = Arc::clone(&state);
                tokio::spawn(async move {
                    let config = { state_c.read().await.config.clone() };
                    if let Err(e) =
                        crate::version_registry::refresh(&config, true, tx.clone()).await
                    {
                        let _ = tx.send(AppEvent::Error(e.to_string())).await;
                        let mut s = state_c.write().await;
                        s.version_registry_loading = false;
                    }
                });
            }
            // M1 — Navigation
            AppCommand::OpenScreen(screen) => {
                let mut s = state.write().await;
                s.ui.active_screen = screen;
            }
            AppCommand::OpenSettings(section) => {
                let mut s = state.write().await;
                s.ui.active_screen = crate::state::app_state::Screen::Settings;
                s.ui.settings_section = section;
            }
            // M1 — Floating windows
            AppCommand::OpenShellWindow => { let mut s = state.write().await; s.ui.shell_open = true; }
            AppCommand::CloseShellWindow => { let mut s = state.write().await; s.ui.shell_open = false; }
            AppCommand::OpenMailpitWindow => { let mut s = state.write().await; s.ui.mailpit_open = true; }
            AppCommand::CloseMailpitWindow => { let mut s = state.write().await; s.ui.mailpit_open = false; }
            AppCommand::OpenPmaWindow => { let mut s = state.write().await; s.ui.pma_open = true; }
            AppCommand::ClosePmaWindow => { let mut s = state.write().await; s.ui.pma_open = false; }
            // M1 — Add-site modal
            AppCommand::OpenAddSiteModal => { let mut s = state.write().await; s.ui.add_site_modal_open = true; }
            AppCommand::CloseAddSiteModal => { let mut s = state.write().await; s.ui.add_site_modal_open = false; }
            // M1 — DNS aliases
            AppCommand::AddDnsAlias { from, to } => {
                let mut s = state.write().await;
                s.dns_aliases.push(crate::state::app_state::DnsAlias { from, to });
            }
            AppCommand::RemoveDnsAlias(from) => {
                let mut s = state.write().await;
                s.dns_aliases.retain(|a| a.from != from);
            }
            // M1 — Service control
            AppCommand::StartService(name) => {
                let state = Arc::clone(&state);
                let name_clone = name.clone();
                tokio::spawn(async move {
                    let result = tokio::process::Command::new("systemctl")
                        .args(["--user", "start", &name_clone])
                        .output()
                        .await;
                    let mut s = state.write().await;
                    match result {
                        Ok(out) if out.status.success() => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("{} started", name_clone),
                                crate::ui::components::toast::ToastType::Success,
                            );
                            let new_status = crate::services::monitor::ServiceStatus::Running;
                            if let Some(svc) = s.services.iter_mut().find(|svc| svc.name == name_clone) {
                                svc.status = new_status;
                            }
                        }
                        _ => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("Failed to start {}", name_clone),
                                crate::ui::components::toast::ToastType::Error,
                            );
                        }
                    }
                });
            }
            AppCommand::StopService(name) => {
                let state = Arc::clone(&state);
                let name_clone = name.clone();
                tokio::spawn(async move {
                    let result = tokio::process::Command::new("systemctl")
                        .args(["--user", "stop", &name_clone])
                        .output()
                        .await;
                    let mut s = state.write().await;
                    match result {
                        Ok(out) if out.status.success() => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("{} stopped", name_clone),
                                crate::ui::components::toast::ToastType::Warning,
                            );
                            if let Some(svc) = s.services.iter_mut().find(|svc| svc.name == name_clone) {
                                svc.status = crate::services::monitor::ServiceStatus::Stopped;
                            }
                        }
                        _ => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("Failed to stop {}", name_clone),
                                crate::ui::components::toast::ToastType::Error,
                            );
                        }
                    }
                });
            }
            AppCommand::RestartService(name) => {
                let state = Arc::clone(&state);
                let name_clone = name.clone();
                tokio::spawn(async move {
                    let result = tokio::process::Command::new("systemctl")
                        .args(["--user", "restart", &name_clone])
                        .output()
                        .await;
                    let mut s = state.write().await;
                    match result {
                        Ok(out) if out.status.success() => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("{} restarted", name_clone),
                                crate::ui::components::toast::ToastType::Success,
                            );
                            if let Some(svc) = s.services.iter_mut().find(|svc| svc.name == name_clone) {
                                svc.status = crate::services::monitor::ServiceStatus::Running;
                            }
                        }
                        _ => {
                            crate::ui::components::toast::push_toast(
                                &mut s.toasts,
                                &format!("Failed to restart {}", name_clone),
                                crate::ui::components::toast::ToastType::Error,
                            );
                        }
                    }
                });
            }
            // M1 — Site actions
            AppCommand::SetSiteFilter(filter) => {
                let mut s = state.write().await;
                s.ui.site_status_filter = filter;
            }
            AppCommand::RevealSiteInFiles(path) => {
                let _ = tokio::process::Command::new("xdg-open").arg(&path).spawn();
            }
            AppCommand::RestartSite(_site) => {}
            AppCommand::OpenShellAtSite(site) => {
                let mut s = state.write().await;
                s.ui.shell_open = true;
                s.shell_site = Some(site);
            }
            AppCommand::UnparkSite(_site) => {}
            AppCommand::SelectSite(name) => {
                let mut s = state.write().await;
                s.site_config_selected = Some(name);
            }
            // M1 — Shell
            AppCommand::RunShellCommand(cmd_str) => {
                let mut s = state.write().await;
                s.shell_input = cmd_str;
            }
            // M1 — DNS flush
            AppCommand::FlushDns => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tokio::process::Command::new("sudo")
                        .args(["systemctl", "restart", "dnsmasq"])
                        .output()
                        .await;
                    let _ = tx.send(AppEvent::Error("DNS flushed".into())).await;
                });
            }
            // M1 — Mailpit
            AppCommand::RefreshMailpit => {}
            // M5 — TLS
            AppCommand::TrustCa => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let out = tokio::process::Command::new("sudo")
                        .args(["valet", "trust"])
                        .output()
                        .await;
                    let msg = match out {
                        Ok(o) if o.status.success() => "CA trusted successfully".into(),
                        Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(),
                        Err(e) => e.to_string(),
                    };
                    let _ = tx.send(crate::events::AppEvent::Error(msg)).await;
                });
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
