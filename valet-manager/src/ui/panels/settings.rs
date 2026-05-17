use egui::{RichText, CornerRadius};
use tokio::sync::mpsc::Sender;
use crate::commands::AppCommand;
use crate::state::app_state::{AppState, SettingsSection};
use crate::ui::theme::{Colors, accent_button, ghost_button, divider, with_alpha};

const PALETTES: [([&str; 3], &str); 5] = [
    (["#5DCAA5", "#0F6E56", "#04342C"], "Mint"),
    (["#378ADD", "#1F4E8C", "#0B243F"], "Cobalt"),
    (["#EF9F27", "#9B6314", "#3B2706"], "Amber"),
    (["#A56BFA", "#5A2EA8", "#21103F"], "Violet"),
    (["#E24B4A", "#962525", "#3B0E0E"], "Ember"),
];

fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    egui::Color32::from_rgb(r, g, b)
}

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // Header
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("Settings").size(15.0).color(Colors::TEXT_PRIMARY).strong());
            ui.label(RichText::new("Preferences — saved automatically").size(12.0).color(Colors::TEXT_SECONDARY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ghost_button(ui, "↗ Export config");
        });
    });
    divider(ui);
    ui.add_space(8.0);

    let avail = ui.available_width();
    let nav_width = 160.0_f32.min(avail * 0.22);

    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(nav_width, ui.available_height()), |ui| {
            settings_nav_item(ui, "Appearance",         SettingsSection::Appearance, state);
            settings_nav_item(ui, "Behavior",           SettingsSection::Behavior,   state);
            settings_nav_item(ui, "PHP",                SettingsSection::Php,        state);
            settings_nav_item(ui, "TLS & Certificates", SettingsSection::Tls,        state);
            settings_nav_item(ui, "Paths",              SettingsSection::Paths,      state);
            settings_nav_item(ui, "Updates",            SettingsSection::Updates,    state);
            settings_nav_item(ui, "About",              SettingsSection::About,      state);
        });

        ui.add_space(8.0);
        let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, ui.available_height()), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, egui::CornerRadius::ZERO, Colors::BORDER);
        ui.add_space(16.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            match state.ui.settings_section {
                SettingsSection::Appearance => render_appearance(ui, state, cmd_tx),
                SettingsSection::Behavior   => render_behavior(ui, state),
                SettingsSection::Php        => render_php(ui, state),
                SettingsSection::Tls        => render_tls(ui, state, cmd_tx),
                SettingsSection::Paths      => render_paths(ui, state),
                SettingsSection::Updates    => render_updates(ui, state, cmd_tx),
                SettingsSection::About      => render_about(ui, cmd_tx),
            }
        });
    });

    let dirty = state.config != state.config_draft;
    if dirty {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if accent_button(ui, "Save").clicked() {
                let _ = cmd_tx.try_send(AppCommand::SaveSettings(state.config_draft.clone()));
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Discard").clicked() {
                state.config_draft = state.config.clone();
            }
        });
    }
}

fn settings_nav_item(
    ui: &mut egui::Ui,
    label: &str,
    section: SettingsSection,
    state: &mut AppState,
) {
    let is_active = state.ui.settings_section == section;
    let desired = egui::vec2(ui.available_width(), 30.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        if is_active {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(5), with_alpha(Colors::ACCENT, 18));
        } else if resp.hovered() {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(5), with_alpha(Colors::BORDER, 50));
        }
        let color = if is_active { Colors::ACCENT } else { Colors::TEXT_SECONDARY };
        ui.painter().text(
            egui::pos2(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.5),
            color,
        );
    }

    if resp.clicked() {
        state.ui.settings_section = section;
    }
}

fn render_appearance(ui: &mut egui::Ui, state: &mut AppState, _cmd_tx: &Sender<AppCommand>) {
    setting_group(ui, "Theme", |ui| {
        setting_row(ui, "Mode", "Dark is default; light mode planned.", |ui| {
            ui.label(RichText::new("Dark").size(11.5).color(Colors::TEXT_TERTIARY));
        });

        setting_row(ui, "Accent palette", "Drives active states, links, and primary buttons.", |ui| {
            ui.horizontal(|ui| {
                for (i, (colors, label)) in PALETTES.iter().enumerate() {
                    let is_active = state.config_draft.appearance.accent_index == i;
                    let swatch_size = egui::vec2(42.0, 22.0);
                    let (rect, resp) = ui.allocate_exact_size(swatch_size, egui::Sense::click());
                    if ui.is_rect_visible(rect) {
                        let seg_w = swatch_size.x / 3.0;
                        for (j, hex) in colors.iter().enumerate() {
                            let seg = egui::Rect::from_min_size(
                                egui::pos2(rect.left() + j as f32 * seg_w, rect.top()),
                                egui::vec2(seg_w, swatch_size.y),
                            );
                            let cr = if j == 0 {
                                egui::CornerRadius { nw: 4, sw: 4, ne: 0, se: 0 }
                            } else if j == 2 {
                                egui::CornerRadius { nw: 0, sw: 0, ne: 4, se: 4 }
                            } else {
                                egui::CornerRadius::ZERO
                            };
                            ui.painter().rect_filled(seg, cr, hex_to_color32(hex));
                        }
                        if is_active {
                            ui.painter().rect_stroke(rect, egui::CornerRadius::same(4),
                                egui::Stroke::new(2.0, Colors::ACCENT), egui::StrokeKind::Outside);
                        }
                    }
                    if resp.clicked() {
                        state.config_draft.appearance.accent_index = i;
                    }
                    resp.on_hover_text(*label);
                    ui.add_space(4.0);
                }
            });
        });
    });

    setting_group(ui, "Layout", |ui| {
        setting_row(ui, "Density", "Spacing of rows, cards, and tables.", |ui| {
            let idx = ["compact","comfy","roomy"].iter().position(|&d| d == state.config_draft.appearance.density).unwrap_or(0);
            chip_row(ui, &["compact", "comfy", "roomy"], idx, |i| {
                state.config_draft.appearance.density = ["compact","comfy","roomy"][i].to_string();
            });
        });
        setting_row(ui, "Sidebar width", "Width of the navigation panel.", |ui| {
            ui.add(egui::Slider::new(&mut state.config_draft.appearance.sidebar_width, 170.0..=300.0)
                .suffix("px").step_by(2.0));
        });
        setting_row(ui, "Window radius", "Corner rounding of the app window.", |ui| {
            ui.add(egui::Slider::new(&mut state.config_draft.appearance.window_radius, 0.0..=22.0)
                .suffix("px"));
        });
    });

    setting_group(ui, "Display", |ui| {
        setting_toggle_row(ui, "Stat strip", "Show summary cards at the top of each page.", &mut state.config_draft.appearance.show_stats);
        setting_toggle_row(ui, "Framework badges", "Coloured pills next to each site's name.", &mut state.config_draft.appearance.show_badges);
        setting_toggle_row(ui, "Status dot glow", "Soft halo around running & failed indicators.", &mut state.config_draft.appearance.dot_glow);
        setting_toggle_row(ui, "Tabular numerals", "Monospaced figures in paths, timestamps, and counts.", &mut state.config_draft.appearance.mono_numerals);
    });

    setting_group(ui, "Reset", |ui| {
        setting_row(ui, "Restore defaults", "Reset all appearance preferences to defaults.", |ui| {
            if ghost_button(ui, "↺ Reset").clicked() {
                state.config_draft.appearance = crate::config::AppearanceConfig::default();
            }
        });
    });
}

fn render_behavior(ui: &mut egui::Ui, state: &mut AppState) {
    setting_group(ui, "Behavior", |ui| {
        setting_toggle_row(ui, "Launch at login",              "Start Valet Manager when you log in.",           &mut state.config_draft.launch_at_login);
        setting_toggle_row(ui, "Show in system tray",          "Keep a tray icon for quick service controls.",   &mut state.config_draft.show_tray_icon);
        setting_toggle_row(ui, "Confirm destructive actions",  "Ask before unparking or deleting sites.",        &mut state.config_draft.confirm_destructive);
        setting_toggle_row(ui, "Auto-restart failed services", "Retry up to 3 times before marking failed.",     &mut state.config_draft.auto_restart_services);
    });
}

fn render_php(ui: &mut egui::Ui, state: &mut AppState) {
    setting_group(ui, "PHP versions", |ui| {
        setting_row(ui, "Default version", "Used for newly parked sites.", |ui| {
            egui::ComboBox::from_id_salt("php_default_ver")
                .selected_text(&state.config_draft.default_php_version)
                .show_ui(ui, |ui| {
                    for v in &["8.3", "8.2", "8.1", "8.0", "7.4"] {
                        ui.selectable_value(&mut state.config_draft.default_php_version, v.to_string(), *v);
                    }
                });
        });
        setting_row(ui, "memory_limit", "Per request, e.g. 512M.", |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.config_draft.default_memory_limit).desired_width(100.0));
        });
        setting_toggle_row(ui, "Xdebug", "Attach automatically when DBGp client is listening.", &mut state.config_draft.xdebug_enabled);
    });
}

fn render_tls(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    setting_group(ui, "Local certificate authority", |ui| {
        setting_row(ui, "Root CA", "Trusted by the system & browsers.", |ui| {
            ui.label(RichText::new("valet-manager-CA").size(11.5).color(Colors::TEXT_TERTIARY)
                .text_style(egui::TextStyle::Monospace));
        });
        setting_toggle_row(ui, "Auto-renew leaf certs", "Renew certificates 14 days before expiry.", &mut state.config_draft.auto_renew_certs);
        setting_row(ui, "Trust on this machine", "Re-install the root CA into the system trust store.", |ui| {
            if ghost_button(ui, "↺ Reinstall").clicked() {
                let _ = cmd_tx.try_send(AppCommand::TrustCa);
            }
        });
    });
}

fn render_paths(ui: &mut egui::Ui, state: &mut AppState) {
    setting_group(ui, "Filesystem paths", |ui| {
        setting_row(ui, "Config directory", "Where parked dirs, certs and proxies live.", |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.config_draft.config_directory).desired_width(240.0));
        });
        setting_row(ui, "Log directory", "Combined and per-site logs.", |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.config_draft.log_directory).desired_width(240.0));
        });
        setting_row(ui, "Composer binary", "Used by site templates & framework detection.", |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.config_draft.composer_binary).desired_width(240.0));
        });
    });
}

fn render_updates(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    setting_group(ui, "Updates", |ui| {
        setting_row(ui, "Current version", "Stable channel.", |ui| {
            ui.label(RichText::new("v0.1 · dev").size(11.5).color(Colors::TEXT_TERTIARY)
                .text_style(egui::TextStyle::Monospace));
        });
        setting_toggle_row(ui, "Automatic updates", "Download and install in the background.", &mut state.config_draft.auto_update);
        setting_row(ui, "Check for updates", "Manually trigger an update check.", |ui| {
            if accent_button(ui, "↺ Check now").clicked() {
                let _ = cmd_tx.try_send(AppCommand::CheckForUpdates);
            }
        });
        if let Some(info) = &state.update_info {
            if info.is_newer {
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Version {} available", info.latest))
                    .size(12.0).color(Colors::ACCENT));
            }
        }
    });
}

fn render_about(ui: &mut egui::Ui, cmd_tx: &Sender<AppCommand>) {
    setting_group(ui, "About Valet Manager", |ui| {
        setting_row(ui, "Version", "Built on trunk.", |ui| {
            ui.label(RichText::new("v0.1 · egui 0.34").size(11.5).color(Colors::TEXT_TERTIARY)
                .text_style(egui::TextStyle::Monospace));
        });
        setting_row(ui, "Diagnostics", "Bundle logs and config for a bug report.", |ui| {
            if ghost_button(ui, "Generate report").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RunDiagnostics);
            }
        });
    });
}

fn setting_group(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).size(13.0).color(Colors::TEXT_PRIMARY).strong());
    ui.add_space(6.0);
    body(ui);
    ui.add_space(16.0);
}

fn setting_row(ui: &mut egui::Ui, label: &str, help: &str, control: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(36.0);
        ui.vertical(|ui| {
            ui.set_min_width(200.0);
            ui.label(RichText::new(label).size(12.5).color(Colors::TEXT_SECONDARY));
            ui.label(RichText::new(help).size(10.5).color(Colors::TEXT_TERTIARY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            control(ui);
        });
    });
    ui.add_space(4.0);
}

fn setting_toggle_row(ui: &mut egui::Ui, label: &str, help: &str, value: &mut bool) {
    setting_row(ui, label, help, |ui| {
        let text = if *value { "ON" } else { "OFF" };
        let color = if *value { Colors::ACCENT } else { Colors::TEXT_TERTIARY };
        if ui.button(RichText::new(text).size(11.0).color(color)).clicked() {
            *value = !*value;
        }
    });
}

fn chip_row(ui: &mut egui::Ui, options: &[&str], active: usize, mut on_change: impl FnMut(usize)) {
    ui.horizontal(|ui| {
        for (i, opt) in options.iter().enumerate() {
            let is_active = i == active;
            let color = if is_active { Colors::ACCENT } else { Colors::TEXT_TERTIARY };
            let bg = if is_active { with_alpha(Colors::ACCENT, 20) } else { Colors::CARD };
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(opt.len() as f32 * 7.0 + 16.0, 24.0),
                egui::Sense::click(),
            );
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
                if is_active {
                    ui.painter().rect_stroke(rect, CornerRadius::same(5),
                        egui::Stroke::new(0.5, Colors::ACCENT), egui::StrokeKind::Inside);
                }
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, opt,
                    egui::FontId::proportional(11.5), color);
            }
            if resp.clicked() { on_change(i); }
            ui.add_space(4.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::state::app_state::SettingsSection;

    #[test]
    fn settings_section_default_appearance() {
        assert_eq!(SettingsSection::default(), SettingsSection::Appearance);
    }

    #[test]
    fn hex_to_color32_parses_mint() {
        let c = super::hex_to_color32("#5DCAA5");
        assert_eq!(c.r(), 0x5D);
        assert_eq!(c.g(), 0xCA);
        assert_eq!(c.b(), 0xA5);
    }

    #[test]
    fn palette_labels_are_nonempty() {
        for (_, label) in &super::PALETTES {
            assert!(!label.is_empty());
        }
    }
}
