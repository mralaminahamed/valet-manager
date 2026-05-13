use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, card_frame, divider, ghost_button};

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("PHP Versions")
                    .size(16.0)
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

    // ── Empty state ──────────────────────────────────────────────────────
    if state.php_versions.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("No PHP versions detected")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        return;
    }

    // ── 2-column version card grid ───────────────────────────────────────
    let available_width = ui.available_width();
    let col_width = (available_width - 16.0) / 2.0;

    ui.horizontal_wrapped(|ui| {
        for version in &state.php_versions {
            ui.allocate_ui(egui::vec2(col_width, 0.0), |ui| {
                let response = card_frame().show(ui, |ui| {
                    // Row 1: PHP version label
                    ui.label(
                        RichText::new(format!("PHP {}", version.version))
                            .size(14.0)
                            .color(Colors::TEXT_PRIMARY),
                    );

                    ui.add_space(4.0);

                    // Row 2: FPM status with dot
                    ui.horizontal(|ui| {
                        let dot_rect = ui.allocate_exact_size(
                            egui::vec2(7.0, 7.0),
                            egui::Sense::hover(),
                        );
                        if ui.is_rect_visible(dot_rect.0) {
                            let dot_color = if version.fpm_running {
                                Colors::ACCENT
                            } else {
                                Colors::DANGER
                            };
                            ui.painter().circle_filled(dot_rect.0.center(), 3.5, dot_color);
                        }
                        ui.add_space(4.0);
                        let fpm_label = format!("FPM: {}", version.fpm_status.label());
                        ui.label(
                            RichText::new(fpm_label)
                                .size(12.0)
                                .color(Colors::TEXT_SECONDARY),
                        );
                    });

                    ui.add_space(2.0);

                    // Row 3: binary path, truncated to 30 chars
                    let path_str = version.binary_path.to_string_lossy();
                    let truncated = if path_str.len() > 30 {
                        format!("{}…", &path_str[..30])
                    } else {
                        path_str.to_string()
                    };
                    ui.label(
                        RichText::new(truncated)
                            .size(10.0)
                            .color(Colors::TEXT_TERTIARY),
                    );

                    ui.add_space(6.0);

                    // Row 4: action button
                    ui.horizontal(|ui| {
                        if version.is_active {
                            ui.add_enabled(
                                false,
                                egui::Button::new(
                                    RichText::new("Active")
                                        .size(12.0)
                                        .color(Colors::TEXT_TERTIARY),
                                )
                                .fill(Colors::CARD)
                                .stroke(egui::Stroke::new(0.5, Colors::BORDER_MED))
                                .corner_radius(4.0),
                            );
                        } else if accent_button(ui, "Switch").clicked() {
                            let _ = cmd_tx
                                .try_send(AppCommand::SwitchGlobalPhp(version.version.clone()));
                        }
                    });
                });

                // Draw 3px left ACCENT border for active version
                if version.is_active {
                    let rect = response.response.rect;
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
                        egui::CornerRadius::ZERO,
                        Colors::ACCENT,
                    );
                }
            });
        }
    });
}
