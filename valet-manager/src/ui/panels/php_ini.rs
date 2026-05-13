use egui::{Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::php::ini_manager;
use crate::php::types::IniType;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, card_frame, divider, ghost_button};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Resolve active version ───────────────────────────────────────────
    let version = state
        .ui
        .selected_php_version
        .clone()
        .unwrap_or_else(|| state.active_php.clone());

    let ini_type = state.ui.php_ini_type.clone();
    let raw_edit_mode = state.ui.php_raw_edit_mode;

    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("PHP INI")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Save button
            if accent_button(ui, "Save").clicked() {
                let content = if state.ini_raw.is_empty() {
                    ini_manager::render(&state.ini_sections)
                } else {
                    state.ini_raw.clone()
                };
                let _ = cmd_tx.try_send(AppCommand::SaveIni {
                    version: version.clone(),
                    ini_type: ini_type.clone(),
                    content,
                });
            }

            // Raw / Parsed toggle
            if raw_edit_mode {
                if ghost_button(ui, "Parsed").clicked() {
                    // Display only — no command available to toggle mode
                }
            } else if ghost_button(ui, "Raw").clicked() {
                // Display only — no command available to toggle mode
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── INI type toggle (CLI / FPM) ──────────────────────────────────────
    ui.horizontal(|ui| {
        let cli_active = ini_type == IniType::Cli;
        let fpm_active = ini_type == IniType::Fpm;

        if cli_active {
            accent_button(ui, "CLI");
        } else if ghost_button(ui, "CLI").clicked() {
            let _ = cmd_tx.try_send(AppCommand::LoadIni {
                version: version.clone(),
                ini_type: IniType::Cli,
            });
        }

        if fpm_active {
            accent_button(ui, "FPM");
        } else if ghost_button(ui, "FPM").clicked() {
            let _ = cmd_tx.try_send(AppCommand::LoadIni {
                version: version.clone(),
                ini_type: IniType::Fpm,
            });
        }
    });

    ui.add_space(8.0);

    // ── Determine sections to display ────────────────────────────────────
    let sections: Vec<crate::php::types::IniSection> = if !state.ini_sections.is_empty() {
        state.ini_sections.clone()
    } else if !state.ini_raw.is_empty() {
        ini_manager::parse(&state.ini_raw)
    } else {
        Vec::new()
    };

    // ── Empty state ───────────────────────────────────────────────────────
    if sections.is_empty() && !raw_edit_mode {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Select a PHP version and load INI")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        return;
    }

    // ── Content area ──────────────────────────────────────────────────────
    let available_width = ui.available_width();

    if raw_edit_mode {
        // ── Mode 2: Raw view ──────────────────────────────────────────────
        let content = if !state.ini_raw.is_empty() {
            state.ini_raw.clone()
        } else {
            ini_manager::render(&sections)
        };

        if content.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select a PHP version and load INI")
                        .size(13.0)
                        .color(Colors::TEXT_SECONDARY),
                );
            });
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                card_frame().show(ui, |ui| {
                    ui.set_min_width(available_width - 28.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut content.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(30)
                            .font(egui::TextStyle::Monospace)
                            .text_color(Colors::TEXT_PRIMARY),
                    );
                });
            });
        }
    } else {
        // ── Mode 1: Parsed view ───────────────────────────────────────────
        let selected_idx = state.ui.selected_ini_section;
        let left_width = available_width * 0.30;
        let right_width = available_width * 0.70 - 8.0;

        ui.horizontal(|ui| {
            // ── Section list (left 30%) ───────────────────────────────────
            ui.allocate_ui(egui::vec2(left_width, ui.available_height()), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("ini_section_list")
                    .show(ui, |ui| {
                        ui.set_min_width(left_width);
                        for (idx, section) in sections.iter().enumerate() {
                            let is_selected = idx == selected_idx;
                            let bg = if is_selected {
                                Colors::ACCENT_DARK
                            } else {
                                Colors::CARD
                            };
                            let text_color = if is_selected {
                                Colors::TEXT_PRIMARY
                            } else {
                                Colors::TEXT_SECONDARY
                            };

                            Frame::NONE
                                .fill(bg)
                                .corner_radius(4.0)
                                .stroke(Stroke::new(0.5, Colors::BORDER))
                                .inner_margin(Margin { left: 8, right: 8, top: 6, bottom: 6 })
                                .show(ui, |ui| {
                                    ui.set_min_width(left_width - 8.0);
                                    ui.label(
                                        RichText::new(&section.name)
                                            .size(12.0)
                                            .color(text_color),
                                    );
                                });
                            ui.add_space(2.0);
                        }
                    });
            });

            ui.add_space(8.0);

            // ── Entry table (right 70%) ───────────────────────────────────
            ui.allocate_ui(egui::vec2(right_width, ui.available_height()), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("ini_entry_table")
                    .show(ui, |ui| {
                        ui.set_min_width(right_width);

                        for section in &sections {
                            if !section.entries.is_empty() {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(&section.name)
                                        .size(11.0)
                                        .color(Colors::TEXT_TERTIARY),
                                );
                                ui.add_space(4.0);

                                card_frame().show(ui, |ui| {
                                    ui.set_min_width(right_width - 28.0);
                                    for entry in &section.entries {
                                        let has_error = entry.comment.is_some();
                                        ui.horizontal(|ui| {
                                            let key_width = right_width * 0.40 - 14.0;
                                            let val_width = right_width * 0.60 - 14.0;

                                            ui.allocate_ui(
                                                egui::vec2(key_width, 20.0),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(&entry.key)
                                                            .size(12.0)
                                                            .color(if has_error {
                                                                Colors::DANGER
                                                            } else {
                                                                Colors::TEXT_SECONDARY
                                                            }),
                                                    );
                                                },
                                            );

                                            ui.allocate_ui(
                                                egui::vec2(val_width, 20.0),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(&entry.value)
                                                            .size(12.0)
                                                            .color(if has_error {
                                                                Colors::DANGER
                                                            } else {
                                                                Colors::TEXT_PRIMARY
                                                            }),
                                                    );
                                                },
                                            );
                                        });
                                        ui.add_space(2.0);
                                    }
                                });
                            }
                        }

                        if sections.iter().all(|s| s.entries.is_empty()) {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("No entries in this INI")
                                        .size(12.0)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                            });
                        }
                    });
            });
        });
    }
}
