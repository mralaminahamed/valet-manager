use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, divider, framework_badge, ghost_button};
use crate::valet::site_scanner::ValetSite;

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Sites")
                    .size(16.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshSites);
            }
        });
    });
    divider(ui);
    ui.add_space(4.0);

    // ── Search bar ───────────────────────────────────────────────────────
    let search_query = state.site_search.clone();
    let available_width = ui.available_width();
    ui.add_sized(
        egui::vec2(available_width, 24.0),
        egui::TextEdit::singleline(&mut search_query.clone())
            .hint_text("Search sites...")
            .text_color(Colors::TEXT_PRIMARY),
    );
    ui.add_space(8.0);

    // ── Filter and sort sites ────────────────────────────────────────────
    let search_lower = search_query.to_lowercase();
    let mut filtered: Vec<&ValetSite> = state
        .sites
        .iter()
        .filter(|s| {
            search_lower.is_empty()
                || s.name.to_lowercase().contains(&search_lower)
                || s.domain.to_lowercase().contains(&search_lower)
        })
        .collect();

    if state.site_sort_favorites_top {
        filtered.sort_by(|a, b| {
            b.is_favorite
                .cmp(&a.is_favorite)
                .then(a.name.cmp(&b.name))
        });
    } else {
        filtered.sort_by(|a, b| a.name.cmp(&b.name));
    }

    // ── Empty state ──────────────────────────────────────────────────────
    if filtered.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("📁")
                    .size(32.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("No sites found")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(12.0);
            if accent_button(ui, "Link a site").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshSites);
            }
        });
        return;
    }

    // ── Column header row ────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(26.0, 16.0), |ui| {
            ui.label(RichText::new("★").size(11.0).color(Colors::TEXT_TERTIARY));
        });
        ui.allocate_ui(egui::vec2(170.0, 16.0), |ui| {
            ui.label(RichText::new("Domain").size(11.0).color(Colors::TEXT_TERTIARY));
        });
        ui.allocate_ui(egui::vec2(180.0, 16.0), |ui| {
            ui.label(RichText::new("Path").size(11.0).color(Colors::TEXT_TERTIARY));
        });
        ui.allocate_ui(egui::vec2(80.0, 16.0), |ui| {
            ui.label(RichText::new("PHP").size(11.0).color(Colors::TEXT_TERTIARY));
        });
        ui.allocate_ui(egui::vec2(96.0, 16.0), |ui| {
            ui.label(
                RichText::new("Framework")
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
        });
        ui.allocate_ui(egui::vec2(36.0, 16.0), |ui| {
            ui.label(RichText::new("TLS").size(11.0).color(Colors::TEXT_TERTIARY));
        });
        ui.allocate_ui(egui::vec2(36.0, 16.0), |ui| {
            ui.label(RichText::new("⋮").size(11.0).color(Colors::TEXT_TERTIARY));
        });
    });
    divider(ui);

    // ── Site rows ────────────────────────────────────────────────────────
    egui::ScrollArea::vertical().show(ui, |ui| {
        for site in &filtered {
            render_site_row(ui, site, state, cmd_tx);
        }
    });
}

fn render_site_row(
    ui: &mut egui::Ui,
    site: &ValetSite,
    state: &AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    let row_height = 32.0;

    let (row_rect, row_response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::hover(),
    );

    // Hover background
    if row_response.hovered() {
        ui.painter().rect_filled(row_rect, egui::CornerRadius::ZERO, Colors::CARD_HOVER);
    }

    // Render row contents inside the allocated rect
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(row_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // ★ Favorite column (26px)
    child_ui.allocate_ui(egui::vec2(26.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            let (star_char, star_color) = if site.is_favorite {
                ("★", Colors::WARNING)
            } else {
                ("☆", Colors::TEXT_TERTIARY)
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(star_char).color(star_color).size(13.0))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE),
                )
                .clicked()
            {
                let _ = cmd_tx.try_send(AppCommand::ToggleFavoriteSite(site.name.clone()));
            }
        });
    });

    // Domain column (170px)
    child_ui.allocate_ui(egui::vec2(170.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(&site.domain)
                            .size(13.0)
                            .color(Colors::ACCENT),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
                )
                .clicked()
            {
                let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(site.domain.clone()));
            }
        });
    });

    // Path column (~180px)
    child_ui.allocate_ui(egui::vec2(180.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            let path_str = site.path.to_string_lossy();
            let truncated = if path_str.len() > 28 {
                format!("{}…", &path_str[..28])
            } else {
                path_str.to_string()
            };
            ui.label(
                RichText::new(truncated)
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
        });
    });

    // PHP column (80px) — ComboBox
    child_ui.allocate_ui(egui::vec2(80.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            let mut selected = site
                .php_version
                .clone()
                .unwrap_or_else(|| "Global".to_string());
            egui::ComboBox::from_id_salt(format!("php_{}", site.name))
                .selected_text(&selected)
                .width(72.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut selected, "Global".to_string(), "Global")
                        .clicked()
                    {
                        let _ = cmd_tx.try_send(AppCommand::UnisolateSite(site.name.clone()));
                    }
                    for ver in &state.php_versions {
                        if ui
                            .selectable_value(
                                &mut selected,
                                ver.version.clone(),
                                &ver.version,
                            )
                            .clicked()
                        {
                            let _ = cmd_tx.try_send(AppCommand::IsolateSite {
                                site: site.name.clone(),
                                version: ver.version.clone(),
                            });
                        }
                    }
                });
        });
    });

    // Framework column (96px)
    child_ui.allocate_ui(egui::vec2(96.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            framework_badge(ui, &site.framework);
        });
    });

    // TLS column (36px)
    child_ui.allocate_ui(egui::vec2(36.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            let (lock_char, lock_color) = if site.is_secured {
                ("🔒", Colors::ACCENT)
            } else {
                ("🔓", Colors::TEXT_TERTIARY)
            };
            ui.label(RichText::new(lock_char).size(13.0).color(lock_color));
        });
    });

    // ⋮ Actions column (36px) — popup menu via egui::Popup::menu
    child_ui.allocate_ui(egui::vec2(36.0, row_height), |ui| {
        ui.centered_and_justified(|ui| {
            let menu_resp = ui.add(
                egui::Button::new(RichText::new("⋮").size(14.0).color(Colors::TEXT_SECONDARY))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
            );

            let site_name = site.name.clone();
            let site_domain = site.domain.clone();
            let is_secured = site.is_secured;

            egui::Popup::menu(&menu_resp).show(|ui| {
                ui.set_min_width(160.0);

                if ui.button("Open in browser").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(site_domain.clone()));
                }

                if ui.button("Open in editor").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenSiteInEditor(site_name.clone()));
                }

                ui.separator();

                if is_secured {
                    if ui.button("Unsecure").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::UnsecureSite(site_name.clone()));
                    }
                } else if ui.button("Secure").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::SecureSite(site_name.clone()));
                }

                ui.separator();

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Unlink site").color(Colors::DANGER),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    let _ = cmd_tx.try_send(AppCommand::UnlinkSite(site_name.clone()));
                }
            });
        });
    });
}
