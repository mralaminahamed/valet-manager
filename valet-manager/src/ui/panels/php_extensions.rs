use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, divider, ghost_button};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ────────────────────────────────────────────────────
    let selected_version = state
        .ui
        .selected_php_version
        .clone()
        .unwrap_or_else(|| state.active_php.clone());

    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("PHP Extensions")
                    .size(16.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::LoadExtensions(selected_version.clone()));
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── PHP version picker ───────────────────────────────────────────────
    if !state.php_versions.is_empty() {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Version:")
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(4.0);
            for version in &state.php_versions {
                let is_active = state
                    .ui
                    .selected_php_version
                    .as_deref()
                    .map(|v| v == version.version)
                    .unwrap_or(false);

                let clicked = if is_active {
                    accent_button(ui, &version.version).clicked()
                } else {
                    ghost_button(ui, &version.version).clicked()
                };

                if clicked {
                    let _ = cmd_tx
                        .try_send(AppCommand::LoadExtensions(version.version.clone()));
                }
            }
        });
        ui.add_space(8.0);
    }

    // ── Search bar ───────────────────────────────────────────────────────
    let mut search_query = state.ui.php_search_query.clone();
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut search_query)
                .hint_text("Search extensions…")
                .desired_width(ui.available_width()),
        );
    });
    ui.add_space(8.0);

    // ── Extension list ───────────────────────────────────────────────────
    if state.extensions.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Select a PHP version to view extensions")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        return;
    }

    let filtered: Vec<_> = state
        .extensions
        .iter()
        .filter(|ext| {
            search_query.is_empty()
                || ext.name.to_lowercase().contains(&search_query.to_lowercase())
        })
        .collect();

    if filtered.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("No extensions match your search")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for ext in &filtered {
            ui.horizontal(|ui| {
                if ext.enabled {
                    if accent_button(ui, "✓ Enabled").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::DisableExtension {
                            version: selected_version.clone(),
                            extension: ext.name.clone(),
                        });
                    }
                } else if ghost_button(ui, "○ Disabled").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::EnableExtension {
                        version: selected_version.clone(),
                        extension: ext.name.clone(),
                    });
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new(&ext.name)
                        .size(12.0)
                        .color(Colors::TEXT_PRIMARY),
                );
            });
            ui.add_space(4.0);
        }
    });
}
