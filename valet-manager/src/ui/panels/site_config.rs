#![allow(dead_code)]

use egui::{RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::php::user_ini::validate_ini_value;
use crate::site_config::models::{
    BasicAuthUser, HttpServerType, LaravelConfig, MultisiteType, NodePackageManager, OctaneServer,
    SiteConfig, WordPressConfig, XdebugMode,
};
use crate::state::app_state::AppState;
use crate::ui::theme::{
    accent_button, card_frame, divider, framework_badge, ghost_button, section_label, with_alpha,
    Colors,
};
use crate::ui::DetectedFramework;

const TABS: &[&str] = &["PHP", "WordPress", "Laravel", "Server", "Database", "Development"];

fn site_is_wp(state: &AppState, name: &str) -> bool {
    state.sites.iter().any(|s| {
        s.name == name
            && matches!(s.framework, DetectedFramework::WordPress | DetectedFramework::Bedrock)
    })
}

fn site_is_laravel(state: &AppState, name: &str) -> bool {
    state.sites.iter().any(|s| {
        s.name == name
            && matches!(s.framework, DetectedFramework::Laravel | DetectedFramework::Statamic)
    })
}

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    ui.add_space(8.0);

    // ── Header row ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new("Site config")
                .size(15.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        if let Some(selected) = state.site_config_selected.clone() {
            let saved = state.site_configs.get(&selected).cloned().unwrap_or_default();
            let draft = state.site_config_draft.clone().unwrap_or_default();
            if saved != draft {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("● Unsaved changes")
                        .size(11.0)
                        .color(Colors::WARNING),
                );
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(18.0);
            // Site selector
            let mut sel = state.site_config_selected.clone().unwrap_or_default();
            let prev = sel.clone();
            egui::ComboBox::from_id_salt("site_config_site_picker")
                .selected_text(if sel.is_empty() { "Select site…".to_string() } else { sel.clone() })
                .show_ui(ui, |ui| {
                    for s in &state.sites {
                        ui.selectable_value(&mut sel, s.name.clone(), s.domain.clone());
                    }
                });
            if sel != prev && !sel.is_empty() {
                state.site_config_selected = Some(sel.clone());
                state.site_config_draft = None;
                state.site_config_output.clear();
                let _ = cmd_tx.try_send(AppCommand::LoadSiteConfig(sel.clone()));
                if site_is_laravel(state, &sel) {
                    let _ = cmd_tx.try_send(AppCommand::DetectLaravelPackages(sel));
                }
            }
        });
    });
    ui.add_space(8.0);
    divider(ui);
    ui.add_space(8.0);

    let Some(selected) = state.site_config_selected.clone() else {
        empty_state(ui);
        return;
    };

    // ── Tab strip ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        for (idx, label) in TABS.iter().enumerate() {
            let active = state.ui.site_config_tab == idx as u8;
            let resp = if active {
                accent_button(ui, *label)
            } else {
                ghost_button(ui, *label)
            };
            if resp.clicked() {
                state.ui.site_config_tab = idx as u8;
            }
        }
    });
    ui.add_space(10.0);

    // Ensure draft exists.
    if state.site_config_draft.is_none() {
        state.site_config_draft = Some(state.site_configs.get(&selected).cloned().unwrap_or_default());
    }

    let mut draft = state.site_config_draft.clone().unwrap_or_default();
    let mut to_send: Vec<AppCommand> = Vec::new();

    egui::ScrollArea::vertical()
        .id_salt("site_config_body")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width() - 18.0);
                    match state.ui.site_config_tab {
                        0 => render_php_tab(ui, state, &selected, &mut draft, &mut to_send),
                        1 => render_wp_tab(ui, state, &selected, &mut draft, &mut to_send),
                        2 => render_laravel_tab(ui, state, &selected, &mut draft, &mut to_send),
                        3 => render_server_tab(ui, state, &selected, &mut draft, &mut to_send),
                        4 => render_db_tab(ui, &mut draft),
                        5 => render_dev_tab(ui, &mut draft),
                        _ => {}
                    }
                });
            });
            ui.add_space(120.0);
        });

    // Save the draft back to state.
    state.site_config_draft = Some(draft.clone());

    // Bottom action bar.
    ui.add_space(8.0);
    divider(ui);
    egui::Frame::NONE
        .fill(Colors::DEEP_BG)
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_height(44.0);
            ui.horizontal_centered(|ui| {
                ui.add_space(18.0);
                if ghost_button(ui, "Reset to defaults").clicked() {
                    to_send.push(AppCommand::ResetSiteConfig(selected.clone()));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(18.0);
                    let saved = state.site_configs.get(&selected).cloned().unwrap_or_default();
                    let dirty = saved != draft;
                    let save_label = if dirty { "Save" } else { "Saved" };
                    let save_resp = accent_button(ui, save_label);
                    if save_resp.clicked() && dirty {
                        to_send.push(AppCommand::SaveSiteConfig {
                            site: selected.clone(),
                            config: draft.clone(),
                        });
                    }
                    ui.add_space(8.0);
                    if ghost_button(ui, "Save to project (.valet-manager.toml)").clicked() {
                        to_send.push(AppCommand::SaveSiteConfigToProject {
                            site: selected.clone(),
                            config: draft.clone(),
                        });
                    }
                });
            });
        });

    // Send commands at end (avoid borrow issues).
    for c in to_send {
        let _ = cmd_tx.try_send(c);
    }
}

fn empty_state(ui: &mut egui::Ui) {
    ui.add_space(80.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Select a site to configure")
                .size(13.0)
                .color(Colors::TEXT_SECONDARY),
        );
    });
}

// ── PHP TAB ───────────────────────────────────────────────────────────
fn render_php_tab(
    ui: &mut egui::Ui,
    state: &AppState,
    _selected: &str,
    draft: &mut SiteConfig,
    _to_send: &mut Vec<AppCommand>,
) {
    section_label(ui, "PHP version");
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Version").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut current_value = draft.php.version.clone().unwrap_or_else(|| "inherit".to_string());
            let prev = current_value.clone();
            egui::ComboBox::from_id_salt("site_php_version")
                .selected_text(&current_value)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut current_value, "inherit".to_string(), "Inherit global");
                    for v in &state.php_versions {
                        ui.selectable_value(&mut current_value, v.version.clone(), v.version.clone());
                    }
                });
            if current_value != prev {
                draft.php.version = if current_value == "inherit" { None } else { Some(current_value) };
            }
        });
    });
    ui.add_space(8.0);

    section_label(ui, "INI overrides");
    card_frame().show(ui, |ui| {
        ini_text(ui, "memory_limit", &mut draft.php.ini_overrides.memory_limit, "512M");
        ini_text(ui, "upload_max_filesize", &mut draft.php.ini_overrides.upload_max_filesize, "64M");
        ini_text(ui, "post_max_size", &mut draft.php.ini_overrides.post_max_size, "64M");
        ini_uint(ui, "max_execution_time", &mut draft.php.ini_overrides.max_execution_time);
        ini_uint(ui, "max_input_vars", &mut draft.php.ini_overrides.max_input_vars);
        ini_uint(ui, "max_file_uploads", &mut draft.php.ini_overrides.max_file_uploads);
        ini_uint(ui, "session_gc_maxlifetime", &mut draft.php.ini_overrides.session_gc_maxlifetime);
        ini_text(ui, "date_timezone", &mut draft.php.ini_overrides.date_timezone, "UTC");
    });
    ui.add_space(8.0);

    section_label(ui, "Xdebug");
    card_frame().show(ui, |ui| {
        ui.checkbox(&mut draft.php.xdebug_enabled, "Enable Xdebug for this site");
        ui.horizontal(|ui| {
            ui.label(RichText::new("Mode").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut cur = draft.php.xdebug_mode;
            egui::ComboBox::from_id_salt("xdebug_mode_site")
                .selected_text(format!("{:?}", cur))
                .show_ui(ui, |ui| {
                    for m in [XdebugMode::Off, XdebugMode::Develop, XdebugMode::Debug, XdebugMode::Profile, XdebugMode::Trace, XdebugMode::Coverage] {
                        ui.selectable_value(&mut cur, m, format!("{:?}", m));
                    }
                });
            draft.php.xdebug_mode = cur;
        });
    });
}

fn ini_text(ui: &mut egui::Ui, key: &str, value: &mut Option<String>, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).size(12.0).color(Colors::TEXT_SECONDARY));
        let mut text = value.clone().unwrap_or_default();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(180.0)
                .hint_text(hint),
        );
        if resp.changed() {
            *value = if text.is_empty() { None } else { Some(text.clone()) };
        }
        if let Some(v) = value.as_ref() {
            if let Some(err) = validate_ini_value(key, v) {
                ui.label(RichText::new(err).size(11.0).color(Colors::DANGER));
            }
        }
    });
}

fn ini_uint(ui: &mut egui::Ui, key: &str, value: &mut Option<u32>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).size(12.0).color(Colors::TEXT_SECONDARY));
        let mut text = value.map(|v| v.to_string()).unwrap_or_default();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(120.0)
                .hint_text("integer"),
        );
        if resp.changed() {
            *value = text.parse::<u32>().ok();
            if text.is_empty() {
                *value = None;
            }
        }
        if !text.is_empty() && value.is_none() {
            ui.label(RichText::new("expected integer").size(11.0).color(Colors::DANGER));
        }
    });
}

// ── WORDPRESS TAB ─────────────────────────────────────────────────────
fn render_wp_tab(
    ui: &mut egui::Ui,
    state: &AppState,
    selected: &str,
    draft: &mut SiteConfig,
    to_send: &mut Vec<AppCommand>,
) {
    if !site_is_wp(state, selected) {
        ui.label(
            RichText::new("This tab is only available for WordPress sites.")
                .size(12.0)
                .color(Colors::TEXT_TERTIARY),
        );
        return;
    }
    if draft.wordpress.is_none() {
        draft.wordpress = Some(WordPressConfig::default());
    }
    let wp = draft.wordpress.as_mut().unwrap();

    section_label(ui, "Multisite");
    card_frame().show(ui, |ui| {
        ui.checkbox(&mut wp.multisite_enabled, "Multisite enabled");
        if wp.multisite_enabled {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Type").size(12.0).color(Colors::TEXT_SECONDARY));
                ui.radio_value(&mut wp.multisite_type, MultisiteType::Subdomain, "Subdomain");
                ui.radio_value(&mut wp.multisite_type, MultisiteType::Subdirectory, "Subdirectory");
            });
            ui.horizontal(|ui| {
                if accent_button(ui, "Enable multisite").clicked() {
                    to_send.push(AppCommand::EnableWpMultisite {
                        site: selected.to_string(),
                        multisite_type: wp.multisite_type,
                    });
                }
                if ghost_button(ui, "Fetch network sites").clicked() {
                    to_send.push(AppCommand::FetchWpNetworkSites(selected.to_string()));
                }
            });
        }
    });
    ui.add_space(8.0);

    // Network sites table if present
    if let Some(sites) = state.wp_network_sites.get(selected) {
        if !sites.is_empty() {
            section_label(ui, "Network sites");
            card_frame().show(ui, |ui| {
                for ns in sites {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("#{}", ns.blog_id))
                                .size(11.0)
                                .color(Colors::TEXT_TERTIARY),
                        );
                        ui.label(RichText::new(&ns.title).size(12.0).color(Colors::TEXT_PRIMARY));
                        ui.label(RichText::new(&ns.url).size(11.0).color(Colors::TEXT_SECONDARY));
                    });
                }
            });
            ui.add_space(8.0);
        }
    }

    // Streaming output box
    if !state.site_config_output.is_empty() {
        section_label(ui, "Output");
        crate::ui::components::terminal_output::render(ui, &state.site_config_output, true);
        ui.add_space(8.0);
    }

    section_label(ui, "Debug flags");
    card_frame().show(ui, |ui| {
        ui.checkbox(&mut wp.wp_debug, "WP_DEBUG");
        if wp.wp_debug {
            ui.indent("wp_debug_subs", |ui| {
                ui.checkbox(&mut wp.wp_debug_log, "WP_DEBUG_LOG");
                ui.checkbox(&mut wp.wp_debug_display, "WP_DEBUG_DISPLAY");
                ui.checkbox(&mut wp.script_debug, "SCRIPT_DEBUG");
                ui.checkbox(&mut wp.savequeries, "SAVEQUERIES");
            });
        }
    });
    ui.add_space(8.0);

    section_label(ui, "URLs");
    card_frame().show(ui, |ui| {
        labeled_opt_text(ui, "WP_HOME", &mut wp.wp_home);
        labeled_opt_text(ui, "WP_SITEURL", &mut wp.wp_siteurl);
    });
}

fn labeled_opt_text(ui: &mut egui::Ui, label: &str, value: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.0).color(Colors::TEXT_SECONDARY));
        let mut text = value.clone().unwrap_or_default();
        let resp = ui.add(egui::TextEdit::singleline(&mut text).desired_width(220.0));
        if resp.changed() {
            *value = if text.is_empty() { None } else { Some(text) };
        }
    });
}

// ── LARAVEL TAB ───────────────────────────────────────────────────────
fn render_laravel_tab(
    ui: &mut egui::Ui,
    state: &AppState,
    selected: &str,
    draft: &mut SiteConfig,
    to_send: &mut Vec<AppCommand>,
) {
    if !site_is_laravel(state, selected) {
        ui.label(
            RichText::new("This tab is only available for Laravel sites.")
                .size(12.0)
                .color(Colors::TEXT_TERTIARY),
        );
        return;
    }
    if draft.laravel.is_none() {
        draft.laravel = Some(LaravelConfig::default());
    }
    let lc = draft.laravel.as_mut().unwrap();

    section_label(ui, "Packages");
    card_frame().show(ui, |ui| {
        if let Some(pkg) = state.laravel_packages.get(selected) {
            ui.horizontal_wrapped(|ui| {
                if pkg.octane { pkg_badge(ui, "Octane", Colors::ACCENT); }
                if pkg.horizon { pkg_badge(ui, "Horizon", Colors::INFO); }
                if pkg.telescope { pkg_badge(ui, "Telescope", Colors::PURPLE); }
                if pkg.pulse { pkg_badge(ui, "Pulse", Colors::WARNING); }
                if pkg.reverb { pkg_badge(ui, "Reverb", Colors::DANGER); }
                if pkg.filament { pkg_badge(ui, "Filament", Colors::ACCENT); }
                if pkg.livewire { pkg_badge(ui, "Livewire", Colors::PURPLE); }
                if pkg.inertia { pkg_badge(ui, "Inertia", Colors::INFO); }
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(RichText::new("No packages detected yet.").size(11.0).color(Colors::TEXT_TERTIARY));
                if ghost_button(ui, "Detect packages").clicked() {
                    to_send.push(AppCommand::DetectLaravelPackages(selected.to_string()));
                }
            });
        }
    });
    ui.add_space(8.0);

    section_label(ui, "Octane");
    card_frame().show(ui, |ui| {
        let was_enabled = lc.octane_enabled;
        ui.checkbox(&mut lc.octane_enabled, "Use Laravel Octane");
        ui.horizontal(|ui| {
            ui.label(RichText::new("Server").size(12.0).color(Colors::TEXT_SECONDARY));
            for srv in [OctaneServer::Swoole, OctaneServer::RoadRunner, OctaneServer::FrankenPhp] {
                ui.radio_value(&mut lc.octane_server, srv, srv.display_name());
            }
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Port").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut port = lc.octane_port.unwrap_or(8000);
            let resp = ui.add(egui::DragValue::new(&mut port).range(1024..=65535));
            if resp.changed() {
                lc.octane_port = Some(port);
            }
        });
        if lc.octane_enabled != was_enabled {
            to_send.push(AppCommand::SetLaravelOctane {
                site: selected.to_string(),
                enabled: lc.octane_enabled,
            });
        }
    });
    ui.add_space(8.0);

    section_label(ui, "Optional packages");
    card_frame().show(ui, |ui| {
        ui.checkbox(&mut lc.horizon_enabled, "Horizon enabled");
        ui.checkbox(&mut lc.telescope_enabled, "Telescope enabled");
        ui.checkbox(&mut lc.pulse_enabled, "Pulse enabled");
        ui.checkbox(&mut lc.reverb_enabled, "Reverb enabled");
    });
}

fn pkg_badge(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    egui::Frame::NONE
        .fill(with_alpha(color, 30))
        .stroke(Stroke::new(0.5, with_alpha(color, 60)))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin { left: 7, right: 7, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(10.5).color(color));
        });
}

// ── SERVER TAB ────────────────────────────────────────────────────────
fn render_server_tab(
    ui: &mut egui::Ui,
    state: &mut AppState,
    selected: &str,
    draft: &mut SiteConfig,
    to_send: &mut Vec<AppCommand>,
) {
    section_label(ui, "Server type");
    card_frame().show(ui, |ui| {
        let available: Vec<HttpServerType> = if state.installed_http_servers.is_empty() {
            vec![HttpServerType::Nginx]
        } else {
            state.installed_http_servers.clone()
        };
        ui.horizontal_wrapped(|ui| {
            for t in &available {
                let mut cur = draft.server.server_type;
                if ui
                    .radio_value(&mut cur, *t, t.display_name())
                    .clicked()
                {
                    if draft.server.server_type != cur {
                        draft.server.server_type = cur;
                        to_send.push(AppCommand::SetSiteHttpServer {
                            site: selected.to_string(),
                            server_type: cur,
                        });
                    }
                }
            }
        });
        if state.installed_http_servers.is_empty() {
            ui.add_space(4.0);
            if ghost_button(ui, "Detect installed servers").clicked() {
                to_send.push(AppCommand::DetectInstalledServers);
            }
        }
    });
    ui.add_space(8.0);

    section_label(ui, "Custom directives");
    card_frame().show(ui, |ui| {
        ui.label(
            RichText::new("Injected verbatim into the server { } block")
                .size(11.0)
                .color(Colors::TEXT_TERTIARY),
        );
        ui.add(
            egui::TextEdit::multiline(&mut draft.server.custom_directives)
                .desired_rows(5)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace),
        );
    });
    ui.add_space(8.0);

    section_label(ui, "Limits");
    card_frame().show(ui, |ui| {
        labeled_opt_text(ui, "client_max_body_size", &mut draft.server.client_max_body_size);
        ui.horizontal(|ui| {
            ui.label(RichText::new("read_timeout (s)").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut v = draft.server.read_timeout.unwrap_or(60);
            let r = ui.add(egui::DragValue::new(&mut v).range(1..=600));
            if r.changed() {
                draft.server.read_timeout = Some(v);
            }
        });
    });
    ui.add_space(8.0);

    section_label(ui, "Basic auth");
    card_frame().show(ui, |ui| {
        ui.checkbox(&mut draft.server.basic_auth.enabled, "Require login");
        if draft.server.basic_auth.enabled {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Realm").size(12.0).color(Colors::TEXT_SECONDARY));
                ui.add(
                    egui::TextEdit::singleline(&mut draft.server.basic_auth.realm)
                        .desired_width(220.0)
                        .hint_text("Restricted area"),
                );
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new("Users")
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            let mut remove_username: Option<String> = None;
            for u in &draft.server.basic_auth.users {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&u.username).size(12.0).color(Colors::TEXT_PRIMARY));
                    ui.label(RichText::new("•••••").size(12.0).color(Colors::TEXT_TERTIARY));
                    if ghost_button(ui, "Remove").clicked() {
                        remove_username = Some(u.username.clone());
                    }
                });
            }
            if let Some(uname) = remove_username {
                to_send.push(AppCommand::RemoveBasicAuth {
                    site: selected.to_string(),
                    username: uname,
                });
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.site_config_new_auth_user)
                        .desired_width(140.0)
                        .hint_text("username"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.ui.site_config_new_auth_pass)
                        .password(true)
                        .desired_width(140.0)
                        .hint_text("password"),
                );
                if accent_button(ui, "Add user").clicked()
                    && !state.ui.site_config_new_auth_user.is_empty()
                    && !state.ui.site_config_new_auth_pass.is_empty()
                {
                    to_send.push(AppCommand::AddBasicAuth {
                        site: selected.to_string(),
                        username: state.ui.site_config_new_auth_user.clone(),
                        password: state.ui.site_config_new_auth_pass.clone(),
                    });
                    state.ui.site_config_new_auth_user.clear();
                    state.ui.site_config_new_auth_pass.clear();
                }
            });
            // Suppress unused warnings on BasicAuthUser fields.
            let _ = std::mem::size_of::<BasicAuthUser>();
        }
    });
}

// ── DATABASE TAB ──────────────────────────────────────────────────────
fn render_db_tab(ui: &mut egui::Ui, draft: &mut SiteConfig) {
    section_label(ui, "Database");
    card_frame().show(ui, |ui| {
        labeled_opt_text(ui, "Engine override", &mut draft.database.engine);
        ui.checkbox(&mut draft.database.backup_before_destroy, "Take a backup before destructive migrations");
        labeled_opt_text(ui, "Backup path", &mut draft.database.backup_path);
    });
}

// ── DEVELOPMENT TAB ───────────────────────────────────────────────────
fn render_dev_tab(ui: &mut egui::Ui, draft: &mut SiteConfig) {
    section_label(ui, "Development");
    card_frame().show(ui, |ui| {
        labeled_opt_text(ui, "Node version", &mut draft.development.node_version);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Package manager").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut cur = draft.development.package_manager;
            egui::ComboBox::from_id_salt("pkg_manager_combo")
                .selected_text(cur.display_name())
                .show_ui(ui, |ui| {
                    for pm in [NodePackageManager::Npm, NodePackageManager::Pnpm, NodePackageManager::Yarn, NodePackageManager::Bun] {
                        ui.selectable_value(&mut cur, pm, pm.display_name());
                    }
                });
            draft.development.package_manager = cur;
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Dev server port").size(12.0).color(Colors::TEXT_SECONDARY));
            let mut v = draft.development.dev_server_port.unwrap_or(0);
            let r = ui.add(egui::DragValue::new(&mut v).range(0..=65535));
            if r.changed() {
                draft.development.dev_server_port = if v == 0 { None } else { Some(v) };
            }
        });
        labeled_opt_text(ui, "Start command", &mut draft.development.start_command);
    });
}

// Suppress unused warnings on imports until consumed by future iterations.
#[allow(dead_code)]
fn _unused_imports_keeper(_: &DetectedFramework) {
    let _ = framework_badge;
}
