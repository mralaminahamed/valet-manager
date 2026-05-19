use egui::{CornerRadius, FontFamily, Frame, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::system::log_reader::{classify, LineSeverity, LogSource};
use crate::ui::theme::{accent_button, divider, ghost_button, Colors};

const SOURCES: &[LogSource] = &[
    LogSource::NginxError,
    LogSource::PhpFpm,
    LogSource::ValetFpm,
    LogSource::Access,
];

fn severity_color(sev: LineSeverity) -> egui::Color32 {
    match sev {
        LineSeverity::Error => Colors::DANGER,
        LineSeverity::Warn => Colors::WARNING,
        LineSeverity::Notice => Colors::INFO,
        LineSeverity::Info => Colors::TEXT_PRIMARY,
    }
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Logs")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "↗ Export").clicked() {
                let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                let filename = format!("/tmp/valet-manager-logs-{}.txt", now);
                let content = state.logs_lines.join("\n");
                if std::fs::write(&filename, content).is_ok() {
                    let _ = cmd_tx.try_send(AppCommand::RevealInFiles(filename));
                }
            }
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadLogs(state.logs_source));
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // ── Source tabs ──────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.spacing_mut().item_spacing.x = 8.0;

        // Pulse dot
        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 5.0, Colors::ACCENT);
        ui.add_space(4.0);
        let tail_label = if state.logs_tail { "Tailing live" } else { "Paused" };
        ui.label(RichText::new(tail_label).size(11.5).color(Colors::TEXT_SECONDARY));
        if ui.small_button(if state.logs_tail { "⏸" } else { "▶" }).clicked() {
            state.logs_tail = !state.logs_tail;
        }
        ui.add_space(8.0);
        // Thin vertical separator
        let (sep, _) = ui.allocate_exact_size(egui::vec2(1.0, 16.0), egui::Sense::hover());
        ui.painter().rect_filled(sep, CornerRadius::ZERO, Colors::BORDER);
        ui.add_space(8.0);

        for src in SOURCES.iter() {
            let active = state.logs_source == *src;
            let resp = if active {
                accent_button(ui, src.label())
            } else {
                ghost_button(ui, src.label())
            };
            if resp.clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadLogs(*src));
            }
        }
    });
    ui.add_space(12.0);

    // ── Body ────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(w, ui.available_height().max(200.0)), |ui| {
            Frame::NONE
                .fill(Colors::DEEP_BG)
                .corner_radius(CornerRadius::same(6))
                .stroke(Stroke::new(0.5, Colors::BORDER))
                .inner_margin(egui::Margin { left: 10, right: 10, top: 8, bottom: 8 })
                .show(ui, |ui| {
                    ui.set_min_height(200.0);
                    let avail_w = ui.available_width();
                    ui.set_min_width(avail_w);

                    if state.logs_lines.is_empty() {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("Click Refresh to load logs")
                                    .size(12.0)
                                    .color(Colors::TEXT_TERTIARY),
                            );
                        });
                        ui.add_space(40.0);
                        return;
                    }

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .id_salt("logs_scroll")
                        .show(ui, |ui| {
                            for line in &state.logs_lines {
                                let sev = classify(line);
                                ui.label(
                                    RichText::new(line)
                                        .size(12.0)
                                        .family(FontFamily::Monospace)
                                        .color(severity_color(sev)),
                                );
                            }
                        });
                });
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_filename_format() {
        // Verify the format string produces no colons or 'T' separators
        let fake_now = chrono::DateTime::parse_from_rfc3339("2026-05-17T12:34:56+00:00")
            .unwrap()
            .with_timezone(&chrono::Local);
        let ts = fake_now.format("%Y-%m-%d_%H-%M-%S").to_string();
        let filename = format!("/tmp/valet-manager-logs-{}.txt", ts);
        assert!(filename.contains("2026-05-17"));
        assert!(!filename.contains(':'));
        assert!(!filename.contains('T'));
        assert!(filename.ends_with(".txt"));
    }
}
