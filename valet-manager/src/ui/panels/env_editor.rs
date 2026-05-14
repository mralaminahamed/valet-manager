use egui::{Color32, CornerRadius, FontFamily, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::env_file::{self, EnvEntry};
use crate::state::app_state::AppState;
use crate::ui::components::banner::{alert_banner, BannerLevel};
use crate::ui::theme::{accent_button, divider, ghost_button, section_label, Colors};

const GROUPS: &[&str] = &["APP", "DB", "MAIL", "REDIS", "Other"];

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new(".env editor")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.add_space(16.0);
        // Site picker
        let current = state
            .env_selected_site
            .clone()
            .unwrap_or_else(|| "(select a site)".to_string());
        let mut new_pick: Option<String> = None;
        egui::ComboBox::from_id_salt("env_site_picker")
            .selected_text(current)
            .width(280.0)
            .show_ui(ui, |ui| {
                for site in &state.sites {
                    if ui
                        .selectable_label(
                            state.env_selected_site.as_deref() == Some(site.domain.as_str()),
                            &site.domain,
                        )
                        .clicked()
                    {
                        new_pick = Some(site.domain.clone());
                    }
                }
            });
        if let Some(pick) = new_pick {
            if state.env_selected_site.as_deref() != Some(pick.as_str()) {
                state.env_selected_site = Some(pick.clone());
                let _ = cmd_tx.try_send(AppCommand::LoadEnvFile(pick));
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.checkbox(&mut state.env_show_secrets, "Show secrets");
        });
    });
    divider(ui);
    ui.add_space(8.0);

    if state.env_selected_site.is_none() {
        empty_state(ui, "Select a site to edit its .env file");
        return;
    }

    // ── Banner: diff vs .env.example ────────────────────────────────────
    let (missing, extra) = env_file::diff_keys(&state.env_entries, &state.env_example_entries);
    if !missing.is_empty() || !extra.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            let avail = ui.available_width() - 18.0;
            ui.allocate_ui(egui::vec2(avail, 0.0), |ui| {
                alert_banner(
                    ui,
                    BannerLevel::Warning,
                    "!",
                    &format!(
                        "{} missing, {} extra keys vs .env.example",
                        missing.len(),
                        extra.len()
                    ),
                );
            });
        });
        ui.add_space(8.0);
    }

    // ── Group tabs row ──────────────────────────────────────────────────
    section_label(ui, "Sections");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        for group in GROUPS {
            let selected = state.env_active_group == *group;
            let resp = ui.add(
                egui::Button::new(
                    RichText::new(*group)
                        .size(11.5)
                        .color(if selected { Color32::WHITE } else { Colors::TEXT_SECONDARY }),
                )
                .fill(if selected { Colors::ACCENT_DARK } else { Colors::CARD })
                .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                .corner_radius(4.0)
                .min_size(egui::vec2(0.0, 26.0)),
            );
            if resp.clicked() {
                state.env_active_group = group.to_string();
            }
            ui.add_space(6.0);
        }
    });
    ui.add_space(10.0);

    // ── Entries table ───────────────────────────────────────────────────
    let active_group = state.env_active_group.clone();
    let show_secrets = state.env_show_secrets;

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        let avail = ui.available_width() - 18.0;
        ui.allocate_ui(egui::vec2(avail, 0.0), |ui| {
            Frame::NONE
                .fill(Colors::CARD)
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(0.5, Colors::BORDER))
                .inner_margin(Margin { left: 12, right: 12, top: 10, bottom: 10 })
                .show(ui, |ui| {
                    let any_matching = state.env_entries.iter().any(|e| matches!(e, EnvEntry::Kv { key, .. } if env_file::group_key(key) == active_group));

                    if !any_matching {
                        empty_state(ui, "No entries in this section");
                        return;
                    }

                    egui::Grid::new("env_grid")
                        .num_columns(2)
                        .min_col_width(160.0)
                        .spacing(egui::vec2(12.0, 6.0))
                        .show(ui, |ui| {
                            for entry in state.env_entries.iter_mut() {
                                if let EnvEntry::Kv { key, value, is_secret, .. } = entry {
                                    if env_file::group_key(key) != active_group {
                                        continue;
                                    }
                                    ui.add_sized(
                                        egui::vec2(180.0, 24.0),
                                        egui::Label::new(
                                            RichText::new(key.as_str())
                                                .family(FontFamily::Monospace)
                                                .size(12.0)
                                                .color(Colors::TEXT_PRIMARY),
                                        ),
                                    );
                                    let avail = ui.available_width();
                                    let mut edit = egui::TextEdit::singleline(value)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(avail);
                                    if *is_secret {
                                        edit = edit.password(!show_secrets);
                                    }
                                    ui.add(edit);
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
    });

    ui.add_space(12.0);

    // ── Action buttons ──────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if accent_button(ui, "Save").clicked() {
            if let Some(site) = state.env_selected_site.clone() {
                let content = env_file::render(&state.env_entries);
                let _ = cmd_tx.try_send(AppCommand::SaveEnvFile { site, content });
            }
        }
        ui.add_space(6.0);
        if ghost_button(ui, "Revert").clicked() {
            if let Some(site) = state.env_selected_site.clone() {
                let _ = cmd_tx.try_send(AppCommand::LoadEnvFile(site));
            }
        }
    });
}

fn empty_state(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(msg)
                .size(13.0)
                .color(Colors::TEXT_SECONDARY),
        );
    });
}
