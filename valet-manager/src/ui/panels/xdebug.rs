use egui::{Color32, CornerRadius, Frame, Margin, RichText};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::php::xdebug::XdebugMode;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{
    accent_button, card_frame, divider, ghost_button, section_label, with_alpha, Colors,
};

fn small_badge(ui: &mut egui::Ui, text: &str, fg: Color32) {
    Frame::NONE
        .fill(with_alpha(fg, 30))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.5).color(fg));
        });
}

fn mode_pill(ui: &mut egui::Ui, mode: XdebugMode, active: bool) -> egui::Response {
    let label = mode.label();
    if active {
        accent_button(ui, label)
    } else {
        ghost_button(ui, label)
    }
}

fn render_php_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    version: &str,
    is_active: bool,
    cmd_tx: &Sender<AppCommand>,
) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        let cfg = state.xdebug_configs.get(version).cloned();
        let installed = cfg.as_ref().map(|c| c.installed).unwrap_or(false);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("PHP {}", version))
                    .size(14.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
            ui.add_space(8.0);
            if is_active {
                small_badge(ui, "active", Colors::ACCENT);
                ui.add_space(4.0);
            }
            if installed {
                small_badge(ui, "installed", Colors::INFO);
            } else {
                small_badge(ui, "not installed", Colors::TEXT_TERTIARY);
            }
        });
        ui.add_space(8.0);

        if let Some(cfg) = cfg.clone() {
            if cfg.installed {
                ui.label(
                    RichText::new("Mode")
                        .size(10.0)
                        .color(Colors::TEXT_TERTIARY),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for mode in XdebugMode::all() {
                        let resp = mode_pill(ui, mode, cfg.mode == mode);
                        if resp.clicked() {
                            let _ = cmd_tx.try_send(AppCommand::SetXdebugMode {
                                version: version.to_string(),
                                mode,
                            });
                        }
                    }
                });
                ui.add_space(10.0);

                // IDE key + port row
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("IDE key")
                            .size(11.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(6.0);
                    let mut current_ide = cfg.ide_key.clone();
                    let mut new_ide: Option<String> = None;
                    egui::ComboBox::from_id_salt(format!("xdebug_ide_{}", version))
                        .selected_text(&current_ide)
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for opt in ["PHPSTORM", "VSCODE", "NETBEANS", "XDEBUG_SESSION"] {
                                if ui
                                    .selectable_value(&mut current_ide, opt.to_string(), opt)
                                    .clicked()
                                {
                                    new_ide = Some(opt.to_string());
                                }
                            }
                        });
                    if let Some(ide) = new_ide {
                        let _ = cmd_tx.try_send(AppCommand::SetXdebugIdeKey {
                            version: version.to_string(),
                            ide_key: ide,
                        });
                    }

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("Port")
                            .size(11.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                    ui.add_space(6.0);
                    let mut port_str = cfg.port.to_string();
                    let resp = ui.add_sized(
                        egui::vec2(70.0, 24.0),
                        egui::TextEdit::singleline(&mut port_str)
                            .font(egui::TextStyle::Monospace),
                    );
                    if resp.lost_focus() {
                        if let Ok(p) = port_str.parse::<u16>() {
                            if p != cfg.port {
                                let _ = cmd_tx.try_send(AppCommand::SetXdebugPort {
                                    version: version.to_string(),
                                    port: p,
                                });
                            }
                        }
                    }
                });
            } else {
                ui.label(
                    RichText::new("Xdebug not installed")
                        .size(12.0)
                        .color(Colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                if accent_button(ui, "Install Xdebug").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::InstallXdebug(version.to_string()));
                }
                if !state.xdebug_install_output.is_empty() {
                    ui.add_space(8.0);
                    terminal_output::render(ui, &state.xdebug_install_output, true);
                }
            }
        } else {
            // No config snapshot yet — show install button
            ui.label(
                RichText::new("Xdebug not installed")
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(8.0);
            if accent_button(ui, "Install Xdebug").clicked() {
                let _ = cmd_tx.try_send(AppCommand::InstallXdebug(version.to_string()));
            }
        }
    });
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Xdebug")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshXdebug);
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    if state.php_versions.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No PHP versions detected")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        return;
    }

    section_label(ui, "Per-version configuration");

    // Snapshot list so we can mutably borrow state in render_php_card
    let versions: Vec<(String, bool)> = state
        .php_versions
        .iter()
        .map(|v| (v.version.clone(), v.is_active))
        .collect();

    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width() - 22.0);
            for (version, is_active) in &versions {
                render_php_card(ui, state, version, *is_active, cmd_tx);
                ui.add_space(10.0);
            }
        });
    });
}

