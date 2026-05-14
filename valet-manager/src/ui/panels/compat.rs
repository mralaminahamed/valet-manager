use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::php::compat_checker::{CompatStatus, SiteCompat};
use crate::state::app_state::AppState;
use crate::ui::theme::{accent_button, divider, ghost_button, with_alpha, Colors};

const ROW_HEIGHT: f32 = 40.0;
const HEADER_HEIGHT: f32 = 32.0;
const COL_REQUIRES: f32 = 130.0;
const COL_IN_USE: f32 = 90.0;
const COL_STATUS: f32 = 130.0;
const COL_ACTIONS: f32 = 150.0;

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Header ──────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("PHP Compatibility")
                    .size(15.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Re-check").clicked() {
                let _ = cmd_tx.try_send(AppCommand::CheckCompat);
            }
        });
    });
    divider(ui);
    ui.add_space(16.0);

    // ── Summary ─────────────────────────────────────────────────────────
    let total = state.site_compat.len();
    let compatible = state
        .site_compat
        .iter()
        .filter(|c| c.status == CompatStatus::Compatible)
        .count();
    let incompatible = state
        .site_compat
        .iter()
        .filter(|c| c.status == CompatStatus::Incompatible)
        .count();

    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.label(
            RichText::new(format!("{}/{} sites compatible", compatible, total))
                .size(12.0)
                .color(Colors::TEXT_SECONDARY),
        );
        if incompatible > 0 {
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!("{} need attention", incompatible))
                    .size(12.0)
                    .color(Colors::DANGER),
            );
        }
    });
    ui.add_space(12.0);

    if state.site_compat.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No compatibility data yet")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(12.0);
            if accent_button(ui, "Run check").clicked() {
                let _ = cmd_tx.try_send(AppCommand::CheckCompat);
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
                let fixed = COL_REQUIRES + COL_IN_USE + COL_STATUS + COL_ACTIONS;
                let flex_w = (table_w - fixed).max(140.0);

                // Header
                render_header(ui, flex_w);

                // Rows
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for c in state.site_compat.iter() {
                            render_row(ui, c, flex_w, cmd_tx);
                        }
                    });
            });
    });
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
    child.allocate_ui(egui::vec2(flex_w, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(hdr("SITE"));
        });
    });
    child.allocate_ui(egui::vec2(COL_REQUIRES, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("REQUIRES"));
        });
    });
    child.allocate_ui(egui::vec2(COL_IN_USE, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("IN USE"));
        });
    });
    child.allocate_ui(egui::vec2(COL_STATUS, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("STATUS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ACTIONS, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("ACTIONS"));
        });
    });
}

fn render_row(ui: &mut egui::Ui, c: &SiteCompat, flex_w: f32, cmd_tx: &Sender<AppCommand>) {
    let avail_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(avail_w, ROW_HEIGHT),
        egui::Sense::hover(),
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

    // SITE
    child.allocate_ui(egui::vec2(flex_w, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(&c.site_name)
                    .size(12.5)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });
    // REQUIRES
    child.allocate_ui(egui::vec2(COL_REQUIRES, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let txt = c
                .php_required
                .clone()
                .unwrap_or_else(|| "—".to_string());
            ui.label(
                RichText::new(txt)
                    .size(11.5)
                    .monospace()
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });
    // IN USE
    child.allocate_ui(egui::vec2(COL_IN_USE, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&c.php_in_use)
                    .size(11.5)
                    .monospace()
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });
    // STATUS
    child.allocate_ui(egui::vec2(COL_STATUS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            status_pill(ui, &c.status);
        });
    });
    // ACTIONS
    child.allocate_ui(egui::vec2(COL_ACTIONS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if c.status == CompatStatus::Incompatible {
                let suggestion = c
                    .php_required
                    .as_deref()
                    .and_then(crate::php::compat_checker::suggest_version);
                if let Some(ver) = suggestion {
                    if accent_button(ui, format!("Isolate {}", ver)).clicked() {
                        let _ = cmd_tx.try_send(AppCommand::IsolateSite {
                            site: c.site_name.clone(),
                            version: ver,
                        });
                    }
                }
            }
        });
    });
}

fn status_pill(ui: &mut egui::Ui, status: &CompatStatus) {
    let (bg, fg, label) = match status {
        CompatStatus::Compatible => (
            with_alpha(Colors::ACCENT, 30),
            Colors::ACCENT,
            "Compatible",
        ),
        CompatStatus::Incompatible => (
            with_alpha(Colors::DANGER, 30),
            Colors::DANGER,
            "Incompatible",
        ),
        CompatStatus::NoRequirement => (Colors::CARD, Colors::TEXT_TERTIARY, "No requirement"),
    };
    Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin {
            left: 8,
            right: 8,
            top: 3,
            bottom: 3,
        })
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(11.0).color(fg));
        });
}
