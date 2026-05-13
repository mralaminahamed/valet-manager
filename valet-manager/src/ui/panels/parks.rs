use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, divider};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header (44px) ──────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Parks")
                    .size(16.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "Add Directory").clicked() {
                let tx = cmd_tx.clone();
                tokio::spawn(async move {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .set_title("Select directory to park")
                        .pick_folder()
                        .await
                    {
                        let _ = tx
                            .send(AppCommand::ParkDirectory(path.path().to_path_buf()))
                            .await;
                    }
                });
            }
        });
    });

    divider(ui);
    ui.add_space(4.0);

    // ── Scrollable content ───────────────────────────────────────────────
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if state.parks.is_empty() {
                // ── Empty state ──────────────────────────────────────────
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No parked directories")
                            .size(14.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(12.0);
                    if accent_button(ui, "Add a directory").clicked() {
                        let tx = cmd_tx.clone();
                        tokio::spawn(async move {
                            if let Some(path) = rfd::AsyncFileDialog::new()
                                .set_title("Select directory to park")
                                .pick_folder()
                                .await
                            {
                                let _ = tx
                                    .send(AppCommand::ParkDirectory(path.path().to_path_buf()))
                                    .await;
                            }
                        });
                    }
                });
            } else {
                // ── Park list ────────────────────────────────────────────
                for park in &state.parks {
                    let park = park.clone();
                    egui::Frame::NONE
                        .fill(Colors::CARD)
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin { left: 12, right: 8, top: 0, bottom: 0 })
                        .show(ui, |ui| {
                            ui.set_min_height(32.0);
                            ui.horizontal(|ui| {
                                ui.set_min_height(32.0);
                                // Folder icon
                                ui.label(
                                    RichText::new("📁")
                                        .color(Colors::TEXT_SECONDARY)
                                        .size(13.0),
                                );
                                ui.add_space(6.0);
                                // Path label — takes flex width
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center)
                                        .with_main_justify(true),
                                    |ui| {
                                        ui.label(
                                            RichText::new(park.to_string_lossy())
                                                .color(Colors::TEXT_PRIMARY)
                                                .size(13.0),
                                        );
                                    },
                                );
                                // Remove button
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(4.0);
                                        let remove_btn = ui.add(
                                            egui::Button::new(
                                                RichText::new("✕")
                                                    .color(Colors::DANGER)
                                                    .size(12.0),
                                            )
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::NONE),
                                        );
                                        if remove_btn.clicked() {
                                            let _ = cmd_tx
                                                .try_send(AppCommand::ForgetDirectory(park.clone()));
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(4.0);
                }
            }
        });
}
