use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{accent_button, card_frame, divider, ghost_button, section_label, Colors};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("DNS / TLD")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshAll);
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // Seed tld_input default once
    if state.tld_input.is_empty() && !state.tld.is_empty() {
        state.tld_input = state.tld.clone();
    }

    // ── TLD section ──────────────────────────────────────────────────────
    section_label(ui, "Top-level domain");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        card_frame().show(ui, |ui| {
            ui.set_min_width(avail);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Current: .{}", state.tld))
                        .size(11.0)
                        .color(Colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                ui.add_sized(
                    egui::vec2(120.0, 28.0),
                    egui::TextEdit::singleline(&mut state.tld_input).hint_text("test"),
                );
                ui.add_space(8.0);
                if accent_button(ui, "Apply").clicked() {
                    let new_tld = state.tld_input.trim().to_string();
                    if !new_tld.is_empty() {
                        let _ = cmd_tx.try_send(AppCommand::ChangeTld(new_tld));
                    }
                }
            });
        });
    });

    ui.add_space(12.0);

    // ── DNS resolver test ────────────────────────────────────────────────
    section_label(ui, "DNS resolver test");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        card_frame().show(ui, |ui| {
            ui.set_min_width(avail);
            ui.horizontal(|ui| {
                ui.add_sized(
                    egui::vec2(220.0, 28.0),
                    egui::TextEdit::singleline(&mut state.dns_tester_host)
                        .hint_text("example.test"),
                );
                ui.add_space(8.0);
                if accent_button(ui, "Test →").clicked() {
                    let host = state.dns_tester_host.trim().to_string();
                    if !host.is_empty() {
                        let _ = cmd_tx.try_send(AppCommand::TestDns(host));
                    }
                }
            });
        });
    });

    ui.add_space(12.0);

    // ── Output box ───────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(w, 300.0), |ui| {
            terminal_output::render(ui, &state.dns_tester_output, true);
        });
    });
}
