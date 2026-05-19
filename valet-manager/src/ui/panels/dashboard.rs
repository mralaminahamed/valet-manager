#![allow(dead_code)]
use egui::{Color32, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::services::monitor::ServiceStatus;
use crate::state::app_state::AppState;
use crate::ui::components::banner::{alert_banner, alert_banner_with_actions, BannerLevel};
use crate::ui::theme::{
    Colors, accent_button, card_frame, divider, ghost_button, section_label,
    status_color, status_dot, with_alpha,
};

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Dashboard")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
            let sub = format!(
                "PHP {} · {} · {} sites",
                state.active_php,
                state.valet_variant.as_ref().map(|v| v.display_name()).unwrap_or("Valet"),
                state.sites.len(),
            );
            ui.label(RichText::new(sub).size(12.0).color(Colors::TEXT_SECONDARY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "⌘K").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenCommandPalette);
            }
            ui.add_space(6.0);
            if ghost_button(ui, "↺ Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshAll);
            }
            // phpMyAdmin quick action when any site has it enabled
            let any_pma = state.pma_state.sites.values().any(|s| s.enabled);
            if any_pma {
                ui.add_space(6.0);
                if accent_button(ui, "phpMyAdmin").clicked() {
                    if let Some((site, _)) = state.pma_state.sites.iter().find(|(_, s)| s.enabled) {
                        let _ = cmd_tx.try_send(AppCommand::OpenPhpMyAdmin {
                            site: site.clone(),
                            scope: crate::site_config::models::PmaDbScope::SiteOnly,
                        });
                    }
                }
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── Alert banner ────────────────────────────────────────────────────
    if let Some(err) = &state.ui.last_error {
        legacy_alert_banner(ui, Colors::DANGER, "⚠", err, None, cmd_tx);
        ui.add_space(6.0);
    }

    // ── Update available banner ────────────────────────────────────────
    if let Some(info) = &state.update_info {
        if info.is_newer {
            let latest = info.latest.clone();
            let cmd_tx_inner = cmd_tx.clone();
            alert_banner_with_actions(
                ui,
                BannerLevel::Info,
                "↑",
                &format!("Version {} available", info.latest),
                |ui| {
                    if ghost_button(ui, "Skip").clicked() {
                        let _ = cmd_tx_inner.try_send(AppCommand::SkipUpdate(latest.clone()));
                    }
                    ui.add_space(6.0);
                    if accent_button(ui, "Update").clicked() {
                        let _ = cmd_tx_inner.try_send(AppCommand::OpenUpdatePage);
                    }
                },
            );
            ui.add_space(6.0);
        }
    }

    // ── PHP compat banner ──────────────────────────────────────────────
    let incompat = state
        .site_compat
        .iter()
        .filter(|c| c.status == crate::php::compat_checker::CompatStatus::Incompatible)
        .count();
    if incompat > 0 {
        alert_banner(
            ui,
            BannerLevel::Warning,
            "⚠",
            &format!("{} sites have PHP compat issues", incompat),
        );
        ui.add_space(6.0);
    }

    // ── Phase 14 — PHP EOL banners for installed minors ─────────────────
    for php_info in &state.version_registry.php {
        let installed = state
            .php_versions
            .iter()
            .any(|v| v.version == php_info.minor);
        if !installed {
            continue;
        }
        use crate::version_registry::models::EolStatus;
        match php_info.eol_status() {
            EolStatus::Eol => {
                alert_banner(
                    ui,
                    BannerLevel::Danger,
                    "⛔",
                    &format!(
                        "PHP {} is EOL since {}.",
                        php_info.minor, php_info.security_support_until
                    ),
                );
                ui.add_space(6.0);
            }
            EolStatus::Critical => {
                alert_banner(
                    ui,
                    BannerLevel::Warning,
                    "⚠",
                    &format!(
                        "PHP {} EOL in {} days ({}).",
                        php_info.minor,
                        php_info.days_until_eol(),
                        php_info.security_support_until
                    ),
                );
                ui.add_space(6.0);
            }
            EolStatus::Warning => {
                alert_banner(
                    ui,
                    BannerLevel::Info,
                    "ℹ",
                    &format!(
                        "PHP {} security support ends {}.",
                        php_info.minor, php_info.security_support_until
                    ),
                );
                ui.add_space(6.0);
            }
            EolStatus::Active => {}
        }
    }

    // ── SSL cert status banner ──────────────────────────────────────────
    let ssl_critical = state
        .ssl_certs
        .iter()
        .filter(|c| {
            matches!(
                c.status,
                crate::ssl::cert_reader::CertStatus::Critical
                    | crate::ssl::cert_reader::CertStatus::Expired
            )
        })
        .count();
    let ssl_warning = state
        .ssl_certs
        .iter()
        .filter(|c| c.status == crate::ssl::cert_reader::CertStatus::Warning)
        .count();
    if ssl_critical > 0 {
        alert_banner(
            ui,
            BannerLevel::Danger,
            "⚠",
            &format!("{} SSL certs expired or critical", ssl_critical),
        );
        ui.add_space(6.0);
    } else if ssl_warning > 0 {
        alert_banner(
            ui,
            BannerLevel::Warning,
            "⚠",
            &format!("{} SSL certs expiring soon", ssl_warning),
        );
        ui.add_space(6.0);
    }

    // ── Stats grid (4 equal columns) ────────────────────────────────────
    let running = state.services.iter().filter(|s| s.status == ServiceStatus::Running).count();
    let total   = state.services.len();
    let svc_color = if running == total && total > 0 { Colors::ACCENT } else { Colors::WARNING };

    ui.columns(4, |cols| {
        stat_card(&mut cols[0], "Active PHP",  &state.active_php, Colors::ACCENT);
        stat_card(&mut cols[1], "Sites",        &state.sites.len().to_string(), Colors::TEXT_PRIMARY);
        stat_card(&mut cols[2], "Services",     &format!("{}/{}", running, total), svc_color);
        stat_card(&mut cols[3], "TLD",          &format!(".{}", state.tld), Colors::TEXT_PRIMARY);
    });
    ui.add_space(12.0);

    // ── Services ────────────────────────────────────────────────────────
    section_label(ui, "services");
    ui.add_space(4.0);

    if state.services.is_empty() {
        ui.label(
            RichText::new("No services detected yet — run a Refresh.")
                .size(12.0)
                .color(Colors::TEXT_TERTIARY),
        );
        return;
    }

    let services: Vec<_> = state.services.iter().collect();
    for chunk in services.chunks(2) {
        ui.columns(chunk.len(), |cols| {
            for (col, svc) in cols.iter_mut().zip(chunk.iter()) {
                card_frame().show(col, |ui| {
                    // 2px left accent border
                    let rect = ui.max_rect();
                    ui.painter().line_segment(
                        [rect.left_top(), rect.left_bottom()],
                        Stroke::new(2.0, status_color(&svc.status)),
                    );
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                status_dot(ui, &svc.status);
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(&svc.display_name)
                                        .size(13.0)
                                        .color(Colors::TEXT_PRIMARY),
                                );
                            });
                            ui.label(
                                RichText::new(svc.status.label())
                                    .size(10.0)
                                    .color(Colors::TEXT_TERTIARY),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let icon = if svc.name == "mailpit" { "↗" } else { "↺" };
                            if ghost_button(ui, icon).clicked() {
                                let _ = cmd_tx.try_send(AppCommand::RefreshServiceStatus);
                            }
                        });
                    });
                });
            }
        });
        ui.add_space(6.0);
    }
}

fn stat_card(ui: &mut egui::Ui, title: &str, value: &str, value_color: Color32) {
    card_frame().show(ui, |ui| {
        ui.label(RichText::new(title).size(10.0).color(Colors::TEXT_TERTIARY));
        ui.add_space(4.0);
        ui.label(RichText::new(value).size(18.0).color(value_color).strong());
    });
}

fn legacy_alert_banner(
    ui: &mut egui::Ui,
    color: Color32,
    icon: &str,
    msg: &str,
    _btn: Option<(&str, AppCommand)>,
    _cmd_tx: &Sender<AppCommand>,
) {
    let bg = with_alpha(color, 18);
    Frame::NONE
        .fill(bg)
        .inner_margin(Margin { left: 12, right: 8, top: 8, bottom: 8 })
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ui.painter().line_segment(
                [rect.left_top(), rect.left_bottom()],
                Stroke::new(3.0, color),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(15.0).color(color));
                ui.add_space(6.0);
                ui.label(RichText::new(msg).size(12.0).color(Colors::TEXT_PRIMARY));
            });
        });
}
