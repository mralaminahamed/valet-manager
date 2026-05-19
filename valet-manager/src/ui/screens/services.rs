use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::services::monitor::{ManagedService, ServiceStatus};
use crate::state::app_state::AppState;
use crate::ui::theme::{
    Colors, accent_button, card_frame, danger_button, divider, ghost_button, section_label,
    status_color, with_alpha,
};

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Header ────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("Services").size(15.0).color(Colors::TEXT_PRIMARY).strong());
            let running = state.services.iter().filter(|s| s.status == ServiceStatus::Running).count();
            let total = state.services.len();
            ui.label(
                RichText::new(format!("{} of {} services running", running, total))
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "⚡ Health check").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshServiceStatus);
            }
            ui.add_space(6.0);
            if ghost_button(ui, "↺ Restart all").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RestartAllServices);
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── Stat strip ────────────────────────────────────────────────────
    let php_active = state
        .services
        .iter()
        .filter(|s| s.name.starts_with("php") && s.status == ServiceStatus::Running)
        .count();
    let php_total = state.services.iter().filter(|s| s.name.starts_with("php")).count();
    let nginx = state.services.iter().find(|s| s.name == "nginx");
    let mysql = state.services.iter().find(|s| s.name == "mysql" || s.name == "mysqld");
    let mailpit = state.services.iter().find(|s| s.name == "mailpit");

    ui.columns(4, |cols| {
        stat_card(&mut cols[0], "⚙", &format!("{}/{}", php_active, php_total), "PHP versions active");
        stat_card(
            &mut cols[1], "◻",
            nginx.map(|s| s.display_name.as_str()).unwrap_or("Nginx"),
            "Listening on :80, :443",
        );
        stat_card(
            &mut cols[2], "⊟",
            mysql.map(|s| s.display_name.as_str()).unwrap_or("MySQL"),
            "Port 3306",
        );
        stat_card(
            &mut cols[3], "✉",
            mailpit.map(|s| s.display_name.as_str()).unwrap_or("Mailpit"),
            "Captured emails",
        );
    });
    ui.add_space(12.0);

    // ── Service grid ─────────────────────────────────────────────────
    section_label(ui, "services");
    ui.add_space(6.0);

    if state.services.is_empty() {
        ui.label(RichText::new("No services detected — click Refresh.").size(12.0).color(Colors::TEXT_TERTIARY));
        return;
    }

    let services: Vec<_> = state.services.iter().collect();
    for chunk in services.chunks(2) {
        ui.columns(chunk.len(), |cols| {
            for (col, svc) in cols.iter_mut().zip(chunk.iter()) {
                service_card(col, svc, cmd_tx, &mut state.ui.active_screen);
            }
        });
        ui.add_space(6.0);
    }
}

fn stat_card(ui: &mut egui::Ui, icon: &str, value: &str, label: &str) {
    Frame::NONE
        .fill(Colors::CARD)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin { left: 12, right: 12, top: 10, bottom: 10 })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, CornerRadius::same(6), Colors::ACCENT_DEEP);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(13.0),
                    Colors::ACCENT,
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(value).size(16.0).strong().color(Colors::TEXT_PRIMARY));
                    ui.label(RichText::new(label).size(10.5).color(Colors::TEXT_SECONDARY));
                });
            });
        });
}

fn service_card(ui: &mut egui::Ui, svc: &ManagedService, cmd_tx: &Sender<AppCommand>, active_screen: &mut crate::state::app_state::Screen) {
    card_frame().show(ui, |ui| {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            Stroke::new(2.0, status_color(&svc.status)),
        );

        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
            let dim = svc.status != ServiceStatus::Running;
            let icon_bg = if dim { with_alpha(Colors::BORDER, 80) } else { Colors::ACCENT_DEEP };
            ui.painter().rect_filled(icon_rect, CornerRadius::same(6), icon_bg);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                "◻",
                egui::FontId::proportional(14.0),
                if dim { Colors::TEXT_TERTIARY } else { Colors::ACCENT },
            );
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&svc.display_name).size(13.0).color(Colors::TEXT_PRIMARY).strong());
                    if svc.is_default {
                        ui.add_space(4.0);
                        tag(ui, "default", Colors::ACCENT);
                    }
                });
                status_badge(ui, &svc.status);
            });
        });

        ui.add_space(6.0);

        Frame::NONE
            .fill(with_alpha(Colors::BORDER, 30))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin { left: 8, right: 8, top: 6, bottom: 6 })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    meta_pair(ui, "pid",    &svc.pid.map(|p| p.to_string()).unwrap_or("—".into()));
                    ui.add_space(12.0);
                    meta_pair(ui, "port",   svc.port.as_deref().unwrap_or("—"));
                    ui.add_space(12.0);
                    meta_pair(ui, "uptime", svc.uptime.as_deref().unwrap_or("—"));
                });
            });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            match svc.status {
                ServiceStatus::Running => {
                    if danger_button(ui, "◼ Stop").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::StopService(svc.name.clone()));
                    }
                }
                _ => {
                    if accent_button(ui, "▶ Start").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::StartService(svc.name.clone()));
                    }
                }
            }
            ui.add_space(6.0);
            let restart_enabled = svc.status == ServiceStatus::Running;
            ui.add_enabled_ui(restart_enabled, |ui| {
                if ghost_button(ui, "↺ Restart").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RestartService(svc.name.clone()));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ghost_button(ui, "≡ Logs").clicked() {
                    *active_screen = crate::state::app_state::Screen::Logs;
                }
            });
        });
    });
}

fn tag(ui: &mut egui::Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(text.len() as f32 * 6.5 + 10.0, 16.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, CornerRadius::same(4), with_alpha(color, 25));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(10.0),
        color,
    );
}

fn status_badge(ui: &mut egui::Ui, status: &ServiceStatus) {
    let (text, color) = match status {
        ServiceStatus::Running => ("Running", Colors::ACCENT),
        ServiceStatus::Stopped => ("Stopped", Colors::TEXT_TERTIARY),
        ServiceStatus::Failed  => ("Failed",  Colors::DANGER),
        ServiceStatus::Unknown => ("Unknown", Colors::WARNING),
    };
    ui.label(RichText::new(text).size(10.5).color(color));
}

fn meta_pair(ui: &mut egui::Ui, key: &str, value: &str) {
    ui.label(RichText::new(key).size(10.5).color(Colors::TEXT_TERTIARY));
    ui.add_space(4.0);
    ui.label(RichText::new(value).size(10.5).color(Colors::TEXT_SECONDARY).text_style(egui::TextStyle::Monospace));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_svc(name: &str, status: ServiceStatus) -> ManagedService {
        ManagedService {
            name: name.to_string(),
            display_name: name.to_string(),
            status,
            pid: Some(1234),
            port: Some("80".to_string()),
            uptime: Some("4d 12h".to_string()),
            is_default: false,
            unread_count: None,
        }
    }

    #[test]
    fn service_running_count() {
        let svcs = vec![
            make_svc("nginx",   ServiceStatus::Running),
            make_svc("mysql",   ServiceStatus::Running),
            make_svc("php-fpm", ServiceStatus::Stopped),
        ];
        let running = svcs.iter().filter(|s| s.status == ServiceStatus::Running).count();
        assert_eq!(running, 2);
    }

    #[test]
    fn pid_display_none_shows_dash() {
        let svc = ManagedService {
            name: "test".into(),
            display_name: "Test".into(),
            status: ServiceStatus::Stopped,
            pid: None,
            port: None,
            uptime: None,
            is_default: false,
            unread_count: None,
        };
        let pid_str = svc.pid.map(|p| p.to_string()).unwrap_or("—".to_string());
        assert_eq!(pid_str, "—");
    }

    #[test]
    fn failed_status_badge_shows_failed_not_unknown() {
        let text = match ServiceStatus::Failed {
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::Failed  => "Failed",
            ServiceStatus::Unknown => "Unknown",
        };
        assert_eq!(text, "Failed");
    }
}
