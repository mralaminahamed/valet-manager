#![allow(dead_code)]
use egui::{Color32, CornerRadius, FontFamily, Frame, Margin, RichText, ScrollArea, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::history::HistoryEntry;
use crate::state::app_state::AppState;
use crate::ui::theme::{divider, ghost_button, with_alpha, Colors};

const ROW_HEIGHT: f32 = 36.0;
const HEADER_HEIGHT: f32 = 32.0;
const COL_TIME: f32 = 100.0;
const COL_DURATION: f32 = 90.0;
const COL_EXIT: f32 = 80.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Header ──────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("History")
                    .size(15.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadHistory);
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // ── Empty state ─────────────────────────────────────────────────────
    if state.history.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No history yet — commands run through panels will appear here")
                    .size(12.5)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(12.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadHistory);
            }
        });
        return;
    }

    // ── Table ───────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let table_w = ui.available_width() - 22.0;
        Frame::NONE
            .fill(Colors::CARD)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(0.5, Colors::BORDER))
            .show(ui, |ui| {
                ui.set_width(table_w);
                let fixed = COL_TIME + COL_DURATION + COL_EXIT;
                let flex_w = (table_w - fixed).max(200.0);
                render_header(ui, flex_w);
                let entries: Vec<HistoryEntry> = state.history.clone();
                let mut toggled: Option<i64> = None;
                let mut rerun: Option<i64> = None;
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for entry in entries.iter() {
                            let expanded = state.history_expanded == Some(entry.id);
                            if let Some(action) = render_row(ui, entry, flex_w, expanded) {
                                match action {
                                    RowAction::Toggle => toggled = Some(entry.id),
                                    RowAction::Rerun => rerun = Some(entry.id),
                                }
                            }
                        }
                    });
                if let Some(id) = toggled {
                    state.history_expanded = if state.history_expanded == Some(id) {
                        None
                    } else {
                        Some(id)
                    };
                }
                if let Some(id) = rerun {
                    let _ = cmd_tx.try_send(AppCommand::RerunHistory(id));
                }
            });
    });
}

#[derive(Debug, Clone, Copy)]
enum RowAction {
    Toggle,
    Rerun,
}

fn render_header(ui: &mut egui::Ui, flex_w: f32) {
    let avail_w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail_w, HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::ZERO,
        with_alpha(Color32::BLACK, 38),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(0.5, Colors::BORDER),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let hdr = |t: &str| {
        RichText::new(t)
            .size(10.5)
            .strong()
            .color(Colors::TEXT_TERTIARY)
    };
    child.allocate_ui(egui::vec2(COL_TIME, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(hdr("TIME"));
        });
    });
    child.allocate_ui(egui::vec2(flex_w, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("COMMAND"));
        });
    });
    child.allocate_ui(egui::vec2(COL_DURATION, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("DURATION"));
        });
    });
    child.allocate_ui(egui::vec2(COL_EXIT, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("EXIT"));
        });
    });
}

fn render_row(
    ui: &mut egui::Ui,
    entry: &HistoryEntry,
    flex_w: f32,
    expanded: bool,
) -> Option<RowAction> {
    let avail_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(avail_w, ROW_HEIGHT),
        egui::Sense::click(),
    );
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, Colors::CARD_HOVER);
    }
    ui.painter()
        .hline(rect.x_range(), rect.bottom(), Stroke::new(0.5, Colors::BORDER));

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // TIME
    child.allocate_ui(egui::vec2(COL_TIME, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(crate::history::ago(entry.started_at))
                    .size(11.5)
                    .color(Colors::TEXT_TERTIARY),
            );
        });
    });
    // COMMAND
    child.allocate_ui(egui::vec2(flex_w, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let mut s = format!("{} {}", entry.command, entry.args);
            s = s.trim().to_string();
            if s.len() > 60 {
                s = format!("{}…", &s[..60]);
            }
            ui.label(
                RichText::new(s)
                    .size(12.0)
                    .family(FontFamily::Monospace)
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });
    // DURATION
    child.allocate_ui(egui::vec2(COL_DURATION, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}ms", entry.duration_ms))
                    .size(11.5)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });
    // EXIT
    child.allocate_ui(egui::vec2(COL_EXIT, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            exit_pill(ui, entry.exit_code);
        });
    });

    let mut action = if resp.clicked() {
        Some(RowAction::Toggle)
    } else {
        None
    };

    if expanded {
        Frame::NONE
            .fill(Colors::DEEP_BG)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin {
                left: 14,
                right: 14,
                top: 10,
                bottom: 10,
            })
            .show(ui, |ui| {
                ui.set_width(avail_w - 4.0);
                ui.label(
                    RichText::new("Output preview")
                        .size(10.5)
                        .color(Colors::TEXT_TERTIARY),
                );
                ui.add_space(4.0);
                let preview = if entry.output_preview.is_empty() {
                    "(no output captured)".to_string()
                } else {
                    entry.output_preview.clone()
                };
                ui.label(
                    RichText::new(preview)
                        .size(11.0)
                        .family(FontFamily::Monospace)
                        .color(Colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                if ghost_button(ui, "Re-run").clicked() {
                    action = Some(RowAction::Rerun);
                }
            });
    }

    action
}

fn exit_pill(ui: &mut egui::Ui, exit_code: Option<i32>) {
    let (bg, fg, text) = match exit_code {
        Some(0) => (with_alpha(Colors::ACCENT, 30), Colors::ACCENT, "0".to_string()),
        Some(c) => (
            with_alpha(Colors::DANGER, 30),
            Colors::DANGER,
            c.to_string(),
        ),
        None => (
            with_alpha(Colors::DANGER, 30),
            Colors::DANGER,
            "—".to_string(),
        ),
    };
    Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin {
            left: 8,
            right: 8,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(fg).monospace());
        });
}
