use egui::{Color32, CornerRadius, FontFamily, Frame, Margin, RichText, ScrollArea, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::database::{self, format_thousands, DbEngine};
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::components::toast::{push_toast, ToastType};
use crate::ui::theme::{
    accent_button, danger_button, divider, ghost_button, section_label, with_alpha, Colors,
};
use crate::ui::DetectedFramework;

const ENGINES: &[(DbEngine, &str)] = &[
    (DbEngine::MySql, "MySQL"),
    (DbEngine::PostgreSql, "PostgreSQL"),
    (DbEngine::Sqlite, "SQLite"),
];

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Database manager")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });

        ui.add_space(16.0);
        // Engine picker
        egui::ComboBox::from_id_salt("db_engine_picker")
            .selected_text(state.db_engine.display_name())
            .width(140.0)
            .show_ui(ui, |ui| {
                for (eng, name) in ENGINES {
                    if ui
                        .selectable_label(state.db_engine == *eng, *name)
                        .clicked()
                    {
                        state.db_engine = *eng;
                    }
                }
            });

        // Connection badge dot
        ui.add_space(8.0);
        let dot_color = if !state.databases.is_empty() {
            Colors::ACCENT
        } else {
            Colors::DANGER
        };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().circle_filled(rect.center(), 4.0, dot_color);
        }

        ui.add_space(12.0);
        // Site picker
        let current = state
            .db_selected_site
            .clone()
            .unwrap_or_else(|| "(select a site)".to_string());
        let mut new_site: Option<String> = None;
        egui::ComboBox::from_id_salt("db_site_picker")
            .selected_text(current)
            .width(220.0)
            .show_ui(ui, |ui| {
                for site in &state.sites {
                    if ui
                        .selectable_label(
                            state.db_selected_site.as_deref() == Some(site.domain.as_str()),
                            &site.domain,
                        )
                        .clicked()
                    {
                        new_site = Some(site.domain.clone());
                    }
                }
            });
        if let Some(s) = new_site {
            state.db_selected_site = Some(s);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "+").clicked() {
                push_toast(
                    &mut state.toasts,
                    "Create database — coming soon",
                    ToastType::Info,
                );
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshDatabases);
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    if state.databases.is_empty() {
        empty_state(ui, "No engine connected. Press Refresh to scan.");
    }

    // ── Split layout: left databases / right tables ─────────────────────
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        // LEFT column: databases list (168px)
        ui.allocate_ui(egui::vec2(168.0, 0.0), |ui| {
            section_label(ui, "DATABASES");
            ui.add_space(2.0);
            ScrollArea::vertical()
                .id_salt("db_list_scroll")
                .max_height(360.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut clicked: Option<String> = None;
                    for db in &state.databases {
                        let selected =
                            state.db_selected_database.as_deref() == Some(db.name.as_str());
                        let (rect, resp) = ui
                            .allocate_exact_size(egui::vec2(168.0, 28.0), egui::Sense::click());
                        if ui.is_rect_visible(rect) {
                            let p = ui.painter();
                            let bg = if selected {
                                with_alpha(Colors::ACCENT, 30)
                            } else if resp.hovered() {
                                Colors::CARD_HOVER
                            } else {
                                Color32::TRANSPARENT
                            };
                            p.rect_filled(rect, CornerRadius::same(4), bg);
                            if selected {
                                let left = egui::Rect::from_min_size(
                                    rect.min,
                                    egui::vec2(2.0, rect.height()),
                                );
                                p.rect_filled(left, CornerRadius::ZERO, Colors::ACCENT);
                            }
                            p.text(
                                egui::pos2(rect.min.x + 10.0, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                &db.name,
                                egui::FontId::proportional(12.5),
                                Colors::TEXT_PRIMARY,
                            );
                            p.text(
                                egui::pos2(rect.max.x - 8.0, rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                format_thousands(db.table_count as u64),
                                egui::FontId::proportional(10.0),
                                Colors::TEXT_TERTIARY,
                            );
                        }
                        if resp.clicked() {
                            clicked = Some(db.name.clone());
                        }
                    }
                    if let Some(name) = clicked {
                        let _ = cmd_tx.try_send(AppCommand::SelectDatabase(name));
                    }
                });
        });

        ui.add_space(12.0);

        // RIGHT column: tables
        ui.vertical(|ui| {
            let title = match &state.db_selected_database {
                Some(d) => format!("TABLES · {}", d),
                None => "TABLES".to_string(),
            };
            section_label(ui, &title);
            ui.add_space(2.0);

            if state.db_selected_database.is_none() {
                empty_state(ui, "Select a database");
            } else if state.db_tables.is_empty() {
                empty_state(ui, "Empty database");
            } else {
                let avail = ui.available_width();
                Frame::NONE
                    .fill(Colors::CARD)
                    .corner_radius(CornerRadius::same(8))
                    .stroke(Stroke::new(0.5, Colors::BORDER))
                    .inner_margin(Margin { left: 0, right: 0, top: 0, bottom: 0 })
                    .show(ui, |ui| {
                        ui.set_min_width(avail);
                        ScrollArea::vertical()
                            .id_salt("db_tables_scroll")
                            .max_height(360.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                // Header row
                                table_header_row(ui);
                                for t in &state.db_tables {
                                    table_row(ui, &t.name, t.rows, &t.engine);
                                }
                            });
                    });
            }
        });
    });

    // ── Laravel actions bar ─────────────────────────────────────────────
    let is_laravel = state
        .sites
        .iter()
        .find(|s| Some(&s.domain) == state.db_selected_site.as_ref())
        .map(|s| s.framework == DetectedFramework::Laravel)
        .unwrap_or(false);

    if is_laravel {
        ui.add_space(10.0);
        // Top border
        let w = ui.available_width();
        let (line_rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
        ui.painter()
            .hline(line_rect.x_range(), line_rect.center().y, Stroke::new(0.5, Colors::BORDER));
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(22.0);
            let site_opt = state.db_selected_site.clone();
            let running = state.db_migration_running;
            if accent_button(ui, "Migrate").clicked() && !running {
                if let Some(site) = site_opt.clone() {
                    state.db_migration_output.clear();
                    let _ = cmd_tx.try_send(AppCommand::RunMigration {
                        site,
                        action: "migrate".to_string(),
                    });
                }
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Migrate fresh").clicked() && !running {
                if let Some(site) = site_opt.clone() {
                    state.db_migration_output.clear();
                    let _ = cmd_tx.try_send(AppCommand::RunMigration {
                        site,
                        action: "fresh".to_string(),
                    });
                }
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Seed").clicked() && !running {
                if let Some(site) = site_opt.clone() {
                    state.db_migration_output.clear();
                    let _ = cmd_tx.try_send(AppCommand::RunMigration {
                        site,
                        action: "seed".to_string(),
                    });
                }
            }
            ui.add_space(6.0);
            if danger_button(ui, "Rollback").clicked() && !running {
                if let Some(site) = site_opt.clone() {
                    state.db_migration_output.clear();
                    let _ = cmd_tx.try_send(AppCommand::RunMigration {
                        site,
                        action: "rollback".to_string(),
                    });
                }
            }
        });

        if state.db_migration_running || !state.db_migration_output.is_empty() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                let avail = ui.available_width() - 22.0;
                ui.allocate_ui(egui::vec2(avail, 180.0), |ui| {
                    Frame::NONE
                        .fill(Colors::DEEP_BG)
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin { left: 0, right: 0, top: 0, bottom: 0 })
                        .show(ui, |ui| {
                            terminal_output::render(ui, &state.db_migration_output, true);
                        });
                });
            });
        }
    }
    let _ = database::DbEngine::MySql; // keep import used
}

fn table_header_row(ui: &mut egui::Ui) {
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, 28.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::ZERO, with_alpha(Color32::BLACK, 38));
    p.hline(rect.x_range(), rect.bottom(), Stroke::new(0.5, Colors::BORDER));
    let hdr = |text: &str, pos: egui::Pos2, align: egui::Align2| {
        ui.painter().text(
            pos,
            align,
            text,
            egui::FontId::proportional(10.5),
            Colors::TEXT_TERTIARY,
        );
    };
    hdr("NAME", egui::pos2(rect.min.x + 12.0, rect.center().y), egui::Align2::LEFT_CENTER);
    hdr(
        "ROWS",
        egui::pos2(rect.max.x - 80.0 - 70.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
    );
    hdr(
        "ENGINE",
        egui::pos2(rect.max.x - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
    );
}

fn table_row(ui: &mut egui::Ui, name: &str, rows: u64, engine: &str) {
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, 28.0), egui::Sense::hover());
    let p = ui.painter();
    p.hline(rect.x_range(), rect.bottom(), Stroke::new(0.5, Colors::BORDER));
    p.text(
        egui::pos2(rect.min.x + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::new(12.0, FontFamily::Monospace),
        Colors::TEXT_PRIMARY,
    );
    p.text(
        egui::pos2(rect.max.x - 80.0 - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format_thousands(rows),
        egui::FontId::new(12.0, FontFamily::Monospace),
        Colors::TEXT_PRIMARY,
    );
    p.text(
        egui::pos2(rect.max.x - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        engine,
        egui::FontId::proportional(11.5),
        Colors::TEXT_TERTIARY,
    );
}

fn empty_state(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(msg)
                .size(12.5)
                .color(Colors::TEXT_SECONDARY),
        );
    });
    ui.add_space(20.0);
}
