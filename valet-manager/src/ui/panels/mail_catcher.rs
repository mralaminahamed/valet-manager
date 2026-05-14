use egui::{CornerRadius, FontFamily, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::mail::mailpit::{env_lines, MailStatus};
use crate::services::monitor::ServiceStatus;
use crate::state::app_state::AppState;
use crate::ui::theme::{
    accent_button, card_frame, danger_button, divider, ghost_button, section_label,
    status_dot, with_alpha, Colors,
};

fn small_badge(ui: &mut egui::Ui, text: &str, fg: egui::Color32) {
    Frame::NONE
        .fill(with_alpha(fg, 30))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.5).color(fg));
        });
}

fn status_card(ui: &mut egui::Ui, status: &MailStatus, cmd_tx: &Sender<AppCommand>) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());

        // Row 1: tool name + ports + dot + status text
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(status.tool.label())
                    .size(14.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
            ui.add_space(10.0);
            small_badge(ui, &format!("SMTP {}", status.smtp_port), Colors::INFO);
            ui.add_space(6.0);
            small_badge(ui, &format!("HTTP {}", status.http_port), Colors::INFO);
            ui.add_space(12.0);
            let svc_status = if status.running {
                ServiceStatus::Running
            } else {
                ServiceStatus::Stopped
            };
            status_dot(ui, &svc_status);
            ui.add_space(6.0);
            let (label, color) = if status.running {
                ("Running", Colors::ACCENT)
            } else {
                ("Stopped", Colors::DANGER)
            };
            ui.label(RichText::new(label).size(12.0).color(color));
        });
        ui.add_space(6.0);

        // Row 2: unread count
        if status.running {
            ui.label(
                RichText::new(format!("Unread messages: {}", status.unread))
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(8.0);
        }

        // Row 3: start/stop button
        ui.horizontal(|ui| {
            if status.running {
                if danger_button(ui, "Stop").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::StopMailCatcher);
                }
                ui.add_space(8.0);
                if ghost_button(ui, "Refresh count").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RefreshMailUnread);
                }
            } else if accent_button(ui, "Start").clicked() {
                let _ = cmd_tx.try_send(AppCommand::StartMailCatcher);
            }
        });
    });
}

fn env_preview(ui: &mut egui::Ui, body: &str) {
    Frame::NONE
        .fill(Colors::DEEP_BG)
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .inner_margin(Margin { left: 10, right: 10, top: 8, bottom: 8 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for line in body.lines() {
                ui.label(
                    RichText::new(line)
                        .size(12.0)
                        .color(Colors::TEXT_PRIMARY)
                        .family(FontFamily::Monospace),
                );
            }
        });
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // Header
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Mail catcher")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        let http_port = state.mail_status.http_port;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshMailUnread);
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Open Web UI").clicked() {
                let url = format!("http://127.0.0.1:{}", http_port);
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // Status card
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width() - 22.0);
            status_card(ui, &state.mail_status, cmd_tx);
        });
    });

    ui.add_space(12.0);
    section_label(ui, "SMTP config");

    // Site selector + apply
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let current = state
            .mail_apply_site
            .clone()
            .unwrap_or_else(|| "(select a site)".to_string());
        let mut new_pick: Option<String> = None;
        egui::ComboBox::from_id_salt("mail_site_picker")
            .selected_text(current)
            .width(260.0)
            .show_ui(ui, |ui| {
                for site in &state.sites {
                    if ui
                        .selectable_label(
                            state.mail_apply_site.as_deref() == Some(site.domain.as_str()),
                            &site.domain,
                        )
                        .clicked()
                    {
                        new_pick = Some(site.domain.clone());
                    }
                }
            });
        if let Some(pick) = new_pick {
            state.mail_apply_site = Some(pick);
        }

        ui.add_space(12.0);
        let can_apply = state.mail_apply_site.is_some();
        let resp = if can_apply {
            accent_button(ui, "Apply to .env")
        } else {
            ghost_button(ui, "Apply to .env")
        };
        if resp.clicked() && can_apply {
            if let Some(site) = state.mail_apply_site.clone() {
                let _ = cmd_tx.try_send(AppCommand::ApplyMailEnv(site));
            }
        }
    });

    ui.add_space(10.0);

    // Preview
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width() - 22.0);
            let body = env_lines(state.mail_status.tool, state.mail_status.smtp_port);
            env_preview(ui, &body);
        });
    });
}
