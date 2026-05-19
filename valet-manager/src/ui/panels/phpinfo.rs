#![allow(dead_code)]
use egui::{Color32, CornerRadius, Frame, RichText, ScrollArea, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{card_frame, divider, ghost_button, with_alpha, Colors};

const ROW_HEIGHT: f32 = 28.0;
const HEADER_HEIGHT: f32 = 32.0;
const LEFT_PANE_WIDTH: f32 = 180.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Header ──────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("phpinfo()")
                    .size(15.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
            ui.label(
                RichText::new(format!("PHP {}", state.active_php))
                    .size(11.5)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadPhpInfo(state.active_php.clone()));
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // ── Empty state ─────────────────────────────────────────────────────
    if state.phpinfo_sections.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No phpinfo data loaded")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(12.0);
            if ghost_button(ui, "Load phpinfo").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadPhpInfo(state.active_php.clone()));
            }
        });
        return;
    }

    // ── Two-pane layout ─────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);

        // ── Left pane: section list ─────────────────────────────────
        let mut left_clicked: Option<usize> = None;
        Frame::NONE
            .fill(Colors::CARD)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(0.5, Colors::BORDER))
            .show(ui, |ui| {
                ui.set_min_width(LEFT_PANE_WIDTH);
                ui.set_max_width(LEFT_PANE_WIDTH);
                ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
                    ui.set_min_width(LEFT_PANE_WIDTH - 4.0);
                    for (idx, sec) in state.phpinfo_sections.iter().enumerate() {
                        let selected = idx == state.phpinfo_selected_section;
                        let bg = if selected {
                            Colors::ACCENT_DARK
                        } else {
                            Color32::TRANSPARENT
                        };
                        let fg = if selected {
                            Colors::TEXT_PRIMARY
                        } else {
                            Colors::TEXT_SECONDARY
                        };
                        let resp = ui.add(
                            egui::Button::new(
                                RichText::new(&sec.name).size(12.0).color(fg),
                            )
                            .fill(bg)
                            .stroke(Stroke::NONE)
                            .corner_radius(4.0)
                            .min_size(egui::vec2(LEFT_PANE_WIDTH - 12.0, 26.0)),
                        );
                        if resp.clicked() {
                            left_clicked = Some(idx);
                        }
                    }
                });
            });
        if let Some(idx) = left_clicked {
            state.phpinfo_selected_section = idx;
        }

        ui.add_space(10.0);

        // ── Right pane: search + entries ────────────────────────────
        ui.vertical(|ui| {
            // Search bar
            let avail = ui.available_width();
            ui.add_sized(
                egui::vec2(avail.min(540.0), 28.0),
                egui::TextEdit::singleline(&mut state.phpinfo_search)
                    .hint_text("Filter settings…")
                    .text_color(Colors::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            // Table card
            card_frame().show(ui, |ui| {
                ui.set_min_width(avail.min(540.0) - 8.0);

                // Resolve selected section
                let sel = state.phpinfo_selected_section.min(state.phpinfo_sections.len() - 1);
                let section = &state.phpinfo_sections[sel];

                // Header row
                let header_w = ui.available_width();
                let (hdr_rect, _) = ui.allocate_exact_size(
                    egui::vec2(header_w, HEADER_HEIGHT),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(
                    hdr_rect,
                    CornerRadius::ZERO,
                    with_alpha(Color32::BLACK, 30),
                );
                ui.painter().hline(
                    hdr_rect.x_range(),
                    hdr_rect.bottom(),
                    Stroke::new(0.5, Colors::BORDER),
                );
                let mut hdr_child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(hdr_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                let col_w = header_w / 3.0;
                let hdr_label = |t: &str| {
                    RichText::new(t)
                        .size(10.5)
                        .strong()
                        .color(Colors::TEXT_TERTIARY)
                };
                hdr_child.allocate_ui(egui::vec2(col_w, HEADER_HEIGHT), |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add_space(8.0);
                            ui.label(hdr_label("SETTING"));
                        },
                    );
                });
                hdr_child.allocate_ui(egui::vec2(col_w, HEADER_HEIGHT), |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(hdr_label("LOCAL VALUE"));
                        },
                    );
                });
                hdr_child.allocate_ui(egui::vec2(col_w, HEADER_HEIGHT), |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(hdr_label("MASTER VALUE"));
                        },
                    );
                });

                // Body rows
                let q = state.phpinfo_search.to_lowercase();
                let ctx_clone = ui.ctx().clone();
                ScrollArea::vertical()
                    .max_height(500.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for entry in section.entries.iter() {
                            let matched = q.is_empty()
                                || entry.key.to_lowercase().contains(&q)
                                || entry.local_value.to_lowercase().contains(&q)
                                || entry.master_value.to_lowercase().contains(&q);
                            if !matched {
                                continue;
                            }

                            let row_w = ui.available_width();
                            let (row_rect, row_resp) = ui.allocate_exact_size(
                                egui::vec2(row_w, ROW_HEIGHT),
                                egui::Sense::hover(),
                            );
                            if row_resp.hovered() {
                                ui.painter().rect_filled(
                                    row_rect,
                                    CornerRadius::ZERO,
                                    Colors::CARD_HOVER,
                                );
                            }
                            ui.painter().hline(
                                row_rect.x_range(),
                                row_rect.bottom(),
                                Stroke::new(0.5, Colors::BORDER),
                            );

                            let mut child = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(row_rect)
                                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                            );
                            let cell_w = row_w / 3.0;

                            // SETTING cell
                            child.allocate_ui(egui::vec2(cell_w, ROW_HEIGHT), |ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(8.0);
                                        let color = if !q.is_empty()
                                            && entry.key.to_lowercase().contains(&q)
                                        {
                                            Colors::WARNING
                                        } else {
                                            Colors::TEXT_PRIMARY
                                        };
                                        ui.label(
                                            RichText::new(&entry.key)
                                                .size(11.5)
                                                .monospace()
                                                .color(color),
                                        );
                                    },
                                );
                            });
                            // LOCAL VALUE
                            child.allocate_ui(egui::vec2(cell_w, ROW_HEIGHT), |ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&entry.local_value)
                                                .size(11.5)
                                                .monospace()
                                                .color(Colors::TEXT_SECONDARY),
                                        );
                                    },
                                );
                            });
                            // MASTER VALUE + copy button
                            child.allocate_ui(egui::vec2(cell_w, ROW_HEIGHT), |ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&entry.master_value)
                                                .size(11.5)
                                                .monospace()
                                                .color(Colors::TEXT_SECONDARY),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let copy = ui.add(
                                                    egui::Button::new(
                                                        RichText::new("⧉")
                                                            .size(11.0)
                                                            .color(Colors::TEXT_TERTIARY),
                                                    )
                                                    .fill(Color32::TRANSPARENT)
                                                    .stroke(Stroke::NONE)
                                                    .min_size(egui::vec2(18.0, 18.0)),
                                                );
                                                if copy.clicked() {
                                                    let text = format!(
                                                        "{}={}",
                                                        entry.key, entry.local_value
                                                    );
                                                    ctx_clone.copy_text(text);
                                                }
                                            },
                                        );
                                    },
                                );
                            });
                        }
                    });

            });
        });
    });
}
