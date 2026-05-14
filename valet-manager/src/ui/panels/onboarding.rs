use egui::{CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{Colors, accent_button, card_frame, ghost_button, section_label};

const TOTAL_STEPS: u8 = 5;

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    ui.add_space(28.0);
    ui.vertical_centered(|ui| {
        ui.set_max_width(640.0);

        // Header
        ui.label(
            RichText::new("Welcome to Valet Manager")
                .size(22.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new("A short setup to make sure your environment is ready.")
                .size(12.5)
                .color(Colors::TEXT_SECONDARY),
        );

        ui.add_space(14.0);
        step_indicator(ui, state.onboarding_step);
        ui.add_space(14.0);

        match state.onboarding_step {
            0 => step_welcome(ui, state),
            1 => step_php(ui, state),
            2 => step_valet(ui, state),
            3 => step_verify(ui, state, cmd_tx),
            _ => step_ready(ui, state, cmd_tx),
        }
    });
}

fn step_welcome(ui: &mut egui::Ui, state: &mut AppState) {
    card_frame().show(ui, |ui| {
        ui.set_min_width(420.0);
        section_label(ui, "step 1 · welcome");
        ui.add_space(4.0);
        ui.label(
            RichText::new("Valet Manager is a native Linux GUI for Laravel Valet+. \
                We'll check that PHP and Valet are installed, then run \
                `valet diagnose` to validate your setup.")
                .size(12.5)
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "Get started →").clicked() {
                state.onboarding_step = 1;
            }
        });
    });
}

fn step_php(ui: &mut egui::Ui, state: &mut AppState) {
    let has_php = !state.php_versions.is_empty();
    card_frame().show(ui, |ui| {
        ui.set_min_width(420.0);
        section_label(ui, "step 2 · php");
        ui.add_space(4.0);
        check_row(ui, has_php, if has_php { "PHP is installed" } else { "PHP not found on PATH" });
        if !has_php {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Install PHP via your distro package manager, e.g.:")
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(2.0);
            code_hint(ui, "sudo apt install php php-cli");
        }
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "Continue →").clicked() {
                state.onboarding_step = 2;
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Skip").clicked() {
                state.onboarding_step = 2;
            }
        });
    });
}

fn step_valet(ui: &mut egui::Ui, state: &mut AppState) {
    let has_valet = state.valet_variant.is_some();
    card_frame().show(ui, |ui| {
        ui.set_min_width(420.0);
        section_label(ui, "step 3 · valet");
        ui.add_space(4.0);
        check_row(ui, has_valet, if has_valet { "Valet detected" } else { "Valet not detected" });
        if !has_valet {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Install Valet via Composer:")
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(2.0);
            code_hint(ui, "composer global require cpriego/valet-linux");
        }
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "Continue →").clicked() {
                state.onboarding_step = 3;
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Skip").clicked() {
                state.onboarding_step = 3;
            }
        });
    });
}

fn step_verify(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    card_frame().show(ui, |ui| {
        ui.set_min_width(420.0);
        section_label(ui, "step 4 · verify");
        ui.add_space(4.0);
        ui.label(
            RichText::new("Run `valet diagnose` to capture environment info.")
                .size(12.5)
                .color(Colors::TEXT_PRIMARY),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if accent_button(ui, "Run valet diagnose").clicked() {
                state.diagnostics_running = true;
                let _ = cmd_tx.try_send(AppCommand::RunDiagnostics);
            }
            ui.add_space(6.0);
            if state.diagnostics_running {
                ui.label(
                    RichText::new("running…")
                        .size(11.5)
                        .color(Colors::TEXT_TERTIARY),
                );
            }
        });
        ui.add_space(8.0);
        if !state.diagnostics_output.is_empty() {
            terminal_output::render(ui, &state.diagnostics_output, true);
        }
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "Continue →").clicked() {
                state.onboarding_step = 4;
            }
        });
    });
}

fn step_ready(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    card_frame().show(ui, |ui| {
        ui.set_min_width(420.0);
        section_label(ui, "step 5 · ready");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            check_dot(ui, true);
            ui.add_space(6.0);
            ui.label(
                RichText::new("All set! You're ready to use Valet Manager.")
                    .size(13.0)
                    .color(Colors::TEXT_PRIMARY),
            );
        });
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "Open Valet Manager").clicked() {
                state.onboarding_complete = true;
                state.onboarding_step = 0;
                let _ = cmd_tx.try_send(AppCommand::MarkOnboardingComplete);
            }
        });
    });
}

fn step_indicator(ui: &mut egui::Ui, current: u8) {
    ui.horizontal(|ui| {
        for i in 0..TOTAL_STEPS {
            let is_done = i < current;
            let is_active = i == current;
            let color = if is_active {
                Colors::ACCENT
            } else if is_done {
                Colors::ACCENT_DARK
            } else {
                Colors::TEXT_TERTIARY
            };
            let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 4.0), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, CornerRadius::same(2), color);
            }
            ui.add_space(6.0);
        }
    });
}

fn check_row(ui: &mut egui::Ui, ok: bool, msg: &str) {
    ui.horizontal(|ui| {
        check_dot(ui, ok);
        ui.add_space(6.0);
        let color = if ok { Colors::TEXT_PRIMARY } else { Colors::WARNING };
        ui.label(RichText::new(msg).size(12.5).color(color));
    });
}

fn check_dot(ui: &mut egui::Ui, ok: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let color = if ok { Colors::ACCENT } else { Colors::WARNING };
        let painter = ui.painter();
        painter.circle_filled(rect.center(), 7.0, color);
        let sym = if ok { "✓" } else { "!" };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            sym,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    }
}

fn code_hint(ui: &mut egui::Ui, code: &str) {
    Frame::NONE
        .fill(Colors::DEEP_BG)
        .corner_radius(CornerRadius::same(4))
        .stroke(Stroke::new(0.5, Colors::BORDER_MED))
        .inner_margin(Margin { left: 8, right: 8, top: 4, bottom: 4 })
        .show(ui, |ui| {
            ui.label(
                RichText::new(code)
                    .size(11.5)
                    .color(Colors::TEXT_PRIMARY)
                    .family(egui::FontFamily::Monospace),
            );
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn total_steps_is_five() {
        assert_eq!(super::TOTAL_STEPS, 5);
    }
}
