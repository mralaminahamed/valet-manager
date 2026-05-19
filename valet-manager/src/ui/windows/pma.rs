use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, ghost_button};

pub fn render(ctx: &egui::Context, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    if !state.ui.pma_open {
        return;
    }

    let mut open = state.ui.pma_open;
    egui::Window::new("phpMyAdmin")
        .id(egui::Id::new("pma_window"))
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .default_size([640.0, 460.0])
        .min_size([360.0, 240.0])
        .show(ctx, |ui| {
            render_pma_body(ui, state, cmd_tx);
        });

    state.ui.pma_open = open;
}

fn render_pma_body(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // Header row
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("phpMyAdmin")
                .size(13.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.label(
            RichText::new("· port :8082")
                .size(11.0)
                .color(Colors::TEXT_TERTIARY),
        );
        if let Some(version) = &state.pma_state.version {
            ui.label(
                RichText::new(format!("v{}", version))
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(12.0);

    if !state.pma_state.installed {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.label(
                RichText::new("phpMyAdmin is not installed.")
                    .size(12.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Go to Site Config › Database to set it up.")
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
        });
        return;
    }

    // Global access section
    ui.label(
        RichText::new("Global access")
            .size(11.5)
            .color(Colors::TEXT_SECONDARY),
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        let global_domain = state.pma_state.global_site_domain.clone();
        if accent_button(ui, format!("↗ Open {}", global_domain)).clicked() {
            let url = format!("https://{}", global_domain);
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        }
    });

    // Per-site scoped entries
    let sites: Vec<(String, crate::state::app_state::PmaSiteStatus)> = state
        .pma_state
        .sites
        .iter()
        .filter(|(_, s)| s.enabled)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if !sites.is_empty() {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(
            RichText::new("Per-site scoped access")
                .size(11.5)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (site_name, site_status) in &sites {
                egui::Frame::NONE
                    .fill(Colors::CARD)
                    .inner_margin(egui::Margin {
                        left: 10,
                        right: 10,
                        top: 8,
                        bottom: 8,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(site_name)
                                        .size(12.5)
                                        .color(Colors::TEXT_PRIMARY)
                                        .strong(),
                                );
                                if !site_status.db_name.is_empty() {
                                    ui.label(
                                        RichText::new(format!("db: {}", site_status.db_name))
                                            .size(11.0)
                                            .color(Colors::TEXT_SECONDARY),
                                    );
                                }
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ghost_button(ui, "↗ Open scoped").clicked() {
                                        let _ = cmd_tx.try_send(AppCommand::OpenPhpMyAdmin {
                                            site: site_name.clone(),
                                            scope: crate::site_config::models::PmaDbScope::SiteOnly,
                                        });
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::state::app_state::{AppState, PhpMyAdminState, PmaSiteStatus};

    #[test]
    fn pma_state_defaults_not_installed() {
        let state = AppState::default();
        assert!(!state.pma_state.installed);
    }

    #[test]
    fn pma_state_default_global_domain() {
        let state = AppState::default();
        assert_eq!(state.pma_state.global_site_domain, "phpmyadmin.test");
    }

    #[test]
    fn pma_state_default_sites_empty() {
        let state = AppState::default();
        assert!(state.pma_state.sites.is_empty());
    }

    #[test]
    fn pma_window_defaults_closed() {
        let state = AppState::default();
        assert!(!state.ui.pma_open);
    }

    #[test]
    fn pma_site_status_default_disabled() {
        let s = PmaSiteStatus::default();
        assert!(!s.enabled);
        assert!(s.access_url.is_empty());
        assert!(s.db_name.is_empty());
    }

    #[test]
    fn php_my_admin_state_default() {
        let s = PhpMyAdminState::default();
        assert!(!s.installed);
        assert!(s.sites.is_empty());
        assert!(s.version.is_none());
        assert!(s.install_path.is_none());
    }
}
