use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{accent_button, divider, ghost_button, Colors};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Diagnostics")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // ── Run button row ──────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if state.diagnostics_running {
            // disabled-looking ghost button (no-op when running)
            let _ = ghost_button(ui, "Running…");
        } else if accent_button(ui, "▶ Run diagnostics").clicked() {
            let _ = cmd_tx.try_send(AppCommand::RunDiagnostics);
        }
    });
    ui.add_space(12.0);

    // ── Terminal output ─────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(w, 320.0), |ui| {
            terminal_output::render(ui, &state.diagnostics_output, true);
        });
    });

    ui.add_space(12.0);

    // ── Bottom row ──────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if ghost_button(ui, "Trust Valet").clicked() {
            let _ = cmd_tx.try_send(AppCommand::TrustValet);
        }
        ui.add_space(8.0);
        if ghost_button(ui, "Restart all services").clicked() {
            let _ = cmd_tx.try_send(AppCommand::RestartAllServices);
        }
        // Reference to colors to satisfy linter when not used elsewhere
        let _ = Colors::TEXT_TERTIARY;
    });
}
