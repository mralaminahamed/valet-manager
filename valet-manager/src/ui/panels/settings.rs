use egui::{RichText, ScrollArea};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::config::AppConfig;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, card_frame, danger_button, ghost_button, section_label};

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new("Settings")
                .size(20.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );

        let dirty = state.config != state.config_draft;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(18.0);
            let resp = ui.add_enabled_ui(dirty, |ui| accent_button(ui, "Save")).inner;
            if resp.clicked() && dirty {
                let _ = cmd_tx.try_send(AppCommand::SaveSettings(state.config_draft.clone()));
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Reload").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadSettings);
            }
        });
    });
    section_label(ui, "preferences");

    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.vertical(|ui| {
                let inner_w = (ui.available_width() - 36.0).max(420.0);
                ui.set_max_width(inner_w);

                settings_section(ui, "Appearance", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Theme").size(12.5).color(Colors::TEXT_SECONDARY));
                        ui.add_space(8.0);
                        egui::ComboBox::from_id_salt("settings_theme")
                            .selected_text(state.config_draft.theme.clone())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.config_draft.theme, "dark".to_string(), "dark");
                                ui.selectable_value(&mut state.config_draft.theme, "light".to_string(), "light");
                            });
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Font size").size(12.5).color(Colors::TEXT_SECONDARY));
                        ui.add_space(8.0);
                        ui.add(
                            egui::DragValue::new(&mut state.config_draft.font_size)
                                .range(11.0..=20.0)
                                .speed(0.1),
                        );
                    });
                });

                ui.add_space(10.0);
                settings_section(ui, "Editor / Tools", |ui| {
                    labeled_text(ui, "Editor command", &mut state.config_draft.editor_command);
                    labeled_text(ui, "Terminal", &mut state.config_draft.terminal);
                    labeled_text(ui, "File manager", &mut state.config_draft.file_manager);
                });

                ui.add_space(10.0);
                settings_section(ui, "Notifications", |ui| {
                    ui.checkbox(&mut state.config_draft.notifications.php_switched,    "PHP switched");
                    ui.checkbox(&mut state.config_draft.notifications.service_failed,  "Service failed");
                    ui.checkbox(&mut state.config_draft.notifications.creation_complete,"Creation complete");
                    ui.checkbox(&mut state.config_draft.notifications.ssl_expiring,    "SSL expiring");
                    ui.checkbox(&mut state.config_draft.notifications.update_available,"Update available");
                });

                ui.add_space(10.0);
                settings_section(ui, "App Creator", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Default parent directory")
                                .size(12.5)
                                .color(Colors::TEXT_SECONDARY),
                        );
                        ui.add_space(8.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.config_draft.default_parent_directory)
                                .desired_width(260.0),
                        );
                        ui.add_space(6.0);
                        if ghost_button(ui, "…").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                state.config_draft.default_parent_directory =
                                    p.to_string_lossy().to_string();
                            }
                        }
                    });
                });

                ui.add_space(10.0);
                settings_section(ui, "Version Registry", |ui| {
                    // Last refreshed.
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Last refreshed")
                                .size(12.5)
                                .color(Colors::TEXT_SECONDARY),
                        );
                        ui.add_space(8.0);
                        let stamp = match state.version_registry.last_refreshed {
                            Some(ts) => ts.format("%Y-%m-%d %H:%M UTC").to_string(),
                            None => "Never".to_string(),
                        };
                        ui.label(RichText::new(stamp).size(12.5).color(Colors::TEXT_PRIMARY));
                    });
                    ui.add_space(6.0);
                    // Auto-refresh checkbox.
                    ui.checkbox(
                        &mut state.config_draft.version_registry_auto_refresh,
                        "Refresh automatically",
                    );
                    ui.add_space(4.0);
                    // TTL hours.
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("TTL (hours)")
                                .size(12.5)
                                .color(Colors::TEXT_SECONDARY),
                        );
                        ui.add_space(8.0);
                        ui.add(
                            egui::DragValue::new(
                                &mut state.config_draft.version_registry_ttl_hours,
                            )
                            .range(1u64..=168u64)
                            .speed(1.0),
                        );
                    });
                    ui.add_space(6.0);
                    // Refresh-now button (disabled while loading).
                    ui.horizontal(|ui| {
                        let enabled = !state.version_registry_loading;
                        let resp = ui
                            .add_enabled_ui(enabled, |ui| accent_button(ui, "Refresh now"))
                            .inner;
                        if resp.clicked() && enabled {
                            let _ = cmd_tx.try_send(AppCommand::RefreshVersionRegistry);
                        }
                        ui.add_space(8.0);
                        if state.version_registry_loading {
                            ui.label(
                                RichText::new("Loading…")
                                    .size(11.5)
                                    .color(Colors::WARNING),
                            );
                        }
                    });
                    ui.add_space(8.0);
                    // Tracked sources — simple 2-column list.
                    ui.label(
                        RichText::new("Tracked sources")
                            .size(11.5)
                            .color(Colors::TEXT_TERTIARY),
                    );
                    ui.add_space(4.0);
                    let mut rows: Vec<(String, String, bool)> = Vec::new();
                    // PHP minors.
                    for p in &state.version_registry.php {
                        rows.push((format!("PHP {}", p.minor), p.latest_patch.clone(), true));
                    }
                    // Frameworks.
                    let mut fws: Vec<&String> =
                        state.version_registry.frameworks.keys().collect();
                    fws.sort();
                    for key in fws {
                        if let Some(v) = state.version_registry.frameworks.get(key) {
                            rows.push((
                                v.name.clone(),
                                v.latest_version.clone(),
                                !v.latest_version.is_empty(),
                            ));
                        }
                    }
                    // Servers.
                    let mut svs: Vec<&String> =
                        state.version_registry.servers.keys().collect();
                    svs.sort();
                    for key in svs {
                        if let Some(v) = state.version_registry.servers.get(key) {
                            rows.push((
                                v.name.clone(),
                                v.latest_version.clone(),
                                !v.latest_version.is_empty(),
                            ));
                        }
                    }
                    // Tools.
                    let mut tls: Vec<&String> =
                        state.version_registry.tools.keys().collect();
                    tls.sort();
                    for key in tls {
                        if let Some(v) = state.version_registry.tools.get(key) {
                            rows.push((
                                v.name.clone(),
                                v.latest_version.clone(),
                                !v.latest_version.is_empty(),
                            ));
                        }
                    }
                    for row in rows.chunks(2) {
                        ui.columns(row.len(), |cols| {
                            for (col, (name, ver, present)) in cols.iter_mut().zip(row.iter()) {
                                col.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(7.0, 7.0),
                                        egui::Sense::hover(),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let c = if *present {
                                            Colors::ACCENT
                                        } else {
                                            Colors::TEXT_TERTIARY
                                        };
                                        ui.painter().circle_filled(rect.center(), 3.5, c);
                                    }
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(name)
                                            .size(11.5)
                                            .color(Colors::TEXT_SECONDARY),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(ver)
                                            .size(11.5)
                                            .color(Colors::TEXT_PRIMARY),
                                    );
                                });
                            }
                        });
                    }
                });

                ui.add_space(10.0);
                settings_section(ui, "Danger zone", |ui| {
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Reset to defaults").clicked() {
                            state.config_draft = AppConfig::default();
                        }
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Restores all settings to factory defaults (unsaved).")
                                .size(11.5)
                                .color(Colors::TEXT_TERTIARY),
                        );
                    });
                });

                ui.add_space(24.0);
            });
        });
    });
}

fn settings_section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    section_label(ui, title);
    card_frame().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        body(ui);
    });
}

fn labeled_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.5).color(Colors::TEXT_SECONDARY));
        ui.add_space(8.0);
        ui.add(egui::TextEdit::singleline(value).desired_width(260.0));
    });
    ui.add_space(4.0);
}
