use egui::RichText;
use crate::ui::theme::{Colors, accent_button, divider, ghost_button};

#[allow(dead_code)]
pub fn render(
    ui: &mut egui::Ui,
    state: &crate::state::app_state::AppState,
    cmd_tx: &tokio::sync::mpsc::Sender<crate::commands::AppCommand>,
) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
        // Left pane: site list (200px)
        ui.allocate_ui(egui::vec2(200.0, ui.available_height()), |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Nginx")
                        .size(14.0)
                        .color(Colors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                divider(ui);
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("nginx_site_list_scroll")
                    .show(ui, |ui| {
                        let mut sites: Vec<&String> =
                            state.nginx_configs.keys().collect();
                        sites.sort();

                        for site_name in sites {
                            let is_selected =
                                state.nginx_selected.as_deref() == Some(site_name.as_str());

                            let response = ui.add(
                                egui::Button::selectable(is_selected, site_name.as_str()),
                            );

                            if response.clicked() {
                                // Selection is wired in Task #22; no-op here
                            }
                        }
                    });
            });
        });

        // Right pane: editor (flex)
        ui.vertical(|ui| {
            match &state.nginx_selected {
                None => {
                    // Centered placeholder
                    let available = ui.available_size();
                    ui.allocate_ui(available, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new("Select a site to view its config")
                                    .color(Colors::TEXT_SECONDARY),
                            );
                        });
                    });
                }
                Some(selected) => {
                    let selected = selected.clone();
                    let config = state
                        .nginx_configs
                        .get(&selected)
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let mut config_text = config.to_string();

                    // Header
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(selected.as_str())
                                .size(14.0)
                                .color(Colors::TEXT_PRIMARY),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ghost_button(ui, "Open in editor").clicked() {
                                    let _ = cmd_tx.try_send(
                                        crate::commands::AppCommand::OpenSiteInEditor(
                                            selected.clone(),
                                        ),
                                    );
                                }
                            },
                        );
                    });
                    ui.add_space(4.0);
                    divider(ui);

                    // Bottom bar pinned to bottom, then the editor fills the rest
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if accent_button(ui, "Save & Reload").clicked() {
                                let _ = cmd_tx.try_send(
                                    crate::commands::AppCommand::SaveNginxConfig {
                                        site: selected.clone(),
                                        content: config_text.clone(),
                                    },
                                );
                                let _ = cmd_tx
                                    .try_send(crate::commands::AppCommand::ReloadNginx);
                            }
                            ui.add_space(8.0);
                            if ghost_button(ui, "Revert").clicked() {
                                // No-op: config_text is local; full revert requires re-fetch
                            }
                        });
                        ui.add_space(4.0);
                        divider(ui);

                        // Config editor fills remaining space
                        egui::ScrollArea::both()
                            .id_salt("nginx_config_editor_scroll")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut config_text)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .lock_focus(true),
                                );
                            });
                    });
                }
            }
        });
    });
}
