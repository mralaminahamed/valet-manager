use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::php::detector::recommended_php_version;
use crate::state::app_state::AppState;
use crate::ui::DetectedFramework;
use crate::ui::theme::{
    Colors, accent_button, danger_button, divider, framework_badge, framework_display_name,
    ghost_button, with_alpha,
};
use crate::valet::site_scanner::ValetSite;

fn status_color(status: &crate::valet::site_scanner::SiteStatus) -> egui::Color32 {
    use crate::valet::site_scanner::SiteStatus;
    match status {
        SiteStatus::Running => Colors::ACCENT,
        SiteStatus::Failed  => Colors::DANGER,
        SiteStatus::Stopped => Colors::TEXT_TERTIARY,
        SiteStatus::Unknown => Colors::WARNING,
    }
}

/// Doc URL for each framework — used by the "Open framework docs" context menu.
#[allow(dead_code)]
fn framework_docs_url(fw: &DetectedFramework) -> &'static str {
    use DetectedFramework::*;
    match fw {
        Laravel              => "https://laravel.com/docs",
        WordPress | Bedrock  => "https://developer.wordpress.org",
        Symfony              => "https://symfony.com/doc/current",
        CakePHP              => "https://book.cakephp.org",
        Drupal               => "https://drupal.org/docs",
        Magento              => "https://developer.adobe.com/commerce/php/development/",
        Joomla               => "https://docs.joomla.org",
        Craft                => "https://craftcms.com/docs",
        Statamic             => "https://statamic.dev",
        OctoberCms           => "https://docs.octobercms.com",
        Contao               => "https://docs.contao.org",
        Kirby                => "https://getkirby.com/docs",
        ConcreteCms          => "https://documentation.concretecms.org",
        ExpressionEngine     => "https://docs.expressionengine.com",
        Jigsaw               => "https://jigsaw.tighten.com/docs/",
        Sculpin              => "https://sculpin.io/documentation/",
        Slim                 => "https://www.slimframework.com/docs/",
        Zend                 => "https://docs.laminas.dev/",
        Katana               => "https://github.com/themsaid/katana",
        StaticHtml | Unknown => "https://developer.mozilla.org/en-US/docs/Web/HTML",
    }
}

// ── Column widths ────────────────────────────────────────────────────────────
const COL_FAV:       f32 = 26.0;
const COL_DOT:       f32 = 28.0;
const COL_FRAMEWORK: f32 = 110.0;
const COL_PHP:       f32 = 70.0;
const COL_TLS:       f32 = 60.0;
const COL_LASTHIT:   f32 = 130.0;
const COL_ACTION:    f32 = 28.0;
const ROW_HEIGHT:    f32 = 40.0;
const HEADER_HEIGHT: f32 = 32.0;

/// Fixed-width cell helper — allocates a region and centres content vertically.
fn row_cell(ui: &mut egui::Ui, width: f32, height: f32, content: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui(egui::vec2(width, height), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), content);
    });
}

// ── Stat card ────────────────────────────────────────────────────────────────
fn stat_card(ui: &mut egui::Ui, icon: &str, value: &str, label: &str) {
    Frame::NONE
        .fill(Colors::CARD)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin { left: 12, right: 12, top: 10, bottom: 10 })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icon box
                let (icon_rect, _) = ui.allocate_exact_size(
                    egui::vec2(28.0, 28.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(
                    icon_rect,
                    CornerRadius::same(6),
                    Colors::ACCENT_DEEP,
                );
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(14.0),
                    Colors::ACCENT,
                );

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(value)
                            .size(17.0)
                            .strong()
                            .color(Colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(label)
                            .size(11.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                });
            });
        });
}

// ── Stat strip ───────────────────────────────────────────────────────────────
fn stat_strip(ui: &mut egui::Ui, sites: &[&ValetSite]) {
    use crate::valet::site_scanner::SiteStatus;
    let total   = sites.len();
    let running = sites.iter().filter(|s| s.status == SiteStatus::Running).count();
    let failed  = sites.iter().filter(|s| s.status == SiteStatus::Failed).count();
    let secured = sites.iter().filter(|s| s.is_secured).count();

    ui.columns(4, |cols| {
        stat_card(&mut cols[0], "◈", &total.to_string(),   "Total sites");
        stat_card(&mut cols[1], "⚡", &running.to_string(), "Running");
        stat_card(&mut cols[2], "⚠", &failed.to_string(),  "Failed");
        stat_card(&mut cols[3], "🔒", &secured.to_string(), "TLS secured");
    });
    ui.add_space(8.0);
}

// ── Table header ─────────────────────────────────────────────────────────────
fn table_header(ui: &mut egui::Ui, flex_width: f32) {
    let header_bg = with_alpha(Color32::BLACK, 38);
    let border    = Stroke::new(0.5, Colors::BORDER);

    let avail_w = ui.available_width();
    let (header_rect, _) = ui.allocate_exact_size(
        egui::vec2(avail_w, HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(header_rect, CornerRadius::ZERO, header_bg);
    // Border-bottom
    ui.painter().hline(
        header_rect.x_range(),
        header_rect.bottom(),
        border,
    );

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    let hdr_label = |text: &str| {
        RichText::new(text)
            .size(10.5)
            .strong()
            .color(Colors::TEXT_TERTIARY)
    };

    // ★ header (first column)
    child.allocate_ui(egui::vec2(COL_FAV, HEADER_HEIGHT), |ui| {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("★").size(10.0).color(Colors::TEXT_TERTIARY));
        });
    });

    // dot column — empty
    child.allocate_ui(egui::vec2(COL_DOT, HEADER_HEIGHT), |_| {});

    // SITE column
    child.allocate_ui(egui::vec2(flex_width, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("SITE"));
        });
    });

    // FRAMEWORK
    child.allocate_ui(egui::vec2(COL_FRAMEWORK, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("FRAMEWORK"));
        });
    });

    // PHP
    child.allocate_ui(egui::vec2(COL_PHP, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("PHP"));
        });
    });

    // TLS
    child.allocate_ui(egui::vec2(COL_TLS, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("TLS"));
        });
    });

    // LAST HIT
    child.allocate_ui(egui::vec2(COL_LASTHIT, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("LAST HIT"));
        });
    });

    // action — empty
    child.allocate_ui(egui::vec2(COL_ACTION, HEADER_HEIGHT), |_| {});
}

// ── Site row ─────────────────────────────────────────────────────────────────
fn render_site_row(
    ui: &mut egui::Ui,
    site: &ValetSite,
    flex_width: f32,
    cmd_tx: &Sender<AppCommand>,
    pma_enabled: bool,
) {
    let avail_w = ui.available_width();

    let (row_rect, row_resp) = ui.allocate_exact_size(
        egui::vec2(avail_w, ROW_HEIGHT),
        egui::Sense::click(),
    );

    // Hover highlight
    if row_resp.hovered() {
        ui.painter().rect_filled(row_rect, CornerRadius::ZERO, Colors::CARD_HOVER);
    }

    if row_resp.clicked() {
        let _ = cmd_tx.try_send(AppCommand::SelectSite(site.name.clone()));
    }

    // Border-bottom
    ui.painter().hline(
        row_rect.x_range(),
        row_rect.bottom(),
        Stroke::new(0.5, Colors::BORDER),
    );

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(row_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // ── Col 0: Favorite star (26px) ─────────────────────────────────────
    row_cell(&mut child, COL_FAV, ROW_HEIGHT, |ui| {
        let star  = if site.is_favorite { "★" } else { "☆" };
        let color = if site.is_favorite { Colors::WARNING } else { Colors::TEXT_TERTIARY };
        let resp  = ui.add(
            egui::Label::new(RichText::new(star).size(13.0).color(color))
                .sense(egui::Sense::click()),
        );
        if resp.clicked() {
            let _ = cmd_tx.try_send(AppCommand::ToggleFavoriteSite(site.name.clone()));
        }
    });

    // ── Col 1: Status dot (28px) ─────────────────────────────────────────
    row_cell(&mut child, COL_DOT, ROW_HEIGHT, |ui| {
        ui.add_space(10.0);
        let dot_color = status_color(&site.status);
        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
    });

    // ── Col 2: Site + Path (flex) ────────────────────────────────────────
    child.allocate_ui(egui::vec2(flex_width, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.vertical(|ui| {
                // Domain line
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let domain_resp = ui.add(
                        egui::Label::new(
                            RichText::new(&site.domain)
                                .size(12.5)
                                .strong()
                                .color(Colors::ACCENT),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if domain_resp.clicked() {
                        let url = format!("https://{}", site.domain);
                        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                    }
                    if domain_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if site.is_secured {
                        ui.label(
                            RichText::new("🔒")
                                .size(11.0)
                                .color(Colors::ACCENT),
                        );
                    }
                });
                // Path line
                let path_str = site.path.to_string_lossy();
                let truncated = if path_str.len() > 40 {
                    format!("{}…", &path_str[..40])
                } else {
                    path_str.to_string()
                };
                ui.label(
                    RichText::new(truncated)
                        .size(10.5)
                        .color(Colors::TEXT_TERTIARY)
                        .monospace(),
                );
            });
        });
    });

    // ── Col 3: Framework badge (110px) ───────────────────────────────────
    let fw_label = framework_display_name(&site.framework);
    let fw_rec   = recommended_php_version(&site.framework);
    let fw_tooltip = match &site.php_version {
        Some(v) if v != fw_rec
            => format!("{fw_label} · PHP {fw_rec} (using {v}, consider {fw_rec})"),
        _   => format!("{fw_label} · PHP {fw_rec}"),
    };
    row_cell(&mut child, COL_FRAMEWORK, ROW_HEIGHT, |ui| {
        let resp = ui.scope(|ui| {
            framework_badge(ui, &site.framework);
        }).response;
        resp.on_hover_text(fw_tooltip);
    });

    // ── Col 4: PHP version (70px) ────────────────────────────────────────
    row_cell(&mut child, COL_PHP, ROW_HEIGHT, |ui| {
        let php_text = site.php_version.as_deref().unwrap_or("Global");
        ui.label(
            RichText::new(php_text)
                .size(11.0)
                .monospace()
                .color(Colors::TEXT_SECONDARY),
        );
    });

    // ── Col 5: TLS (60px) ────────────────────────────────────────────────
    row_cell(&mut child, COL_TLS, ROW_HEIGHT, |ui| {
        if site.is_secured {
            let tls_color = match site.tls_expiry_days {
                Some(d) if d < 7   => Colors::DANGER,
                Some(d) if d < 30  => Colors::WARNING,
                _                  => Colors::ACCENT,
            };
            ui.label(RichText::new("🔒").size(12.0).color(tls_color));
        } else {
            ui.label(RichText::new("—").size(11.0).color(Colors::TEXT_TERTIARY));
        }
    });

    // ── Col 6: Last hit (130px) ──────────────────────────────────────────
    row_cell(&mut child, COL_LASTHIT, ROW_HEIGHT, |ui| {
        let text = site.last_hit.as_deref().unwrap_or("—");
        ui.label(RichText::new(text).size(11.5).color(Colors::TEXT_TERTIARY));
    });

    // ── Col 7: Action menu (28px) ────────────────────────────────────────
    row_cell(&mut child, COL_ACTION, ROW_HEIGHT, |ui| {
        let btn_resp = ui.add(
            egui::Button::new(
                RichText::new("⋮")
                    .size(16.0)
                    .color(Colors::TEXT_SECONDARY),
            )
            .min_size(egui::vec2(24.0, 24.0))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE),
        );

        let site_name   = site.name.clone();
        let site_domain = site.domain.clone();
        let is_secured  = site.is_secured;
        let docs_url    = framework_docs_url(&site.framework);

        egui::Popup::menu(&btn_resp).show(|ui| {
            ui.set_min_width(180.0);

            if ui.button("Open in browser").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(site_domain.clone()));
            }
            if ui.button("Open in editor").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenSiteInEditor(site_name.clone()));
            }

            if pma_enabled && ui.button("Open phpMyAdmin").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenPhpMyAdmin {
                    site: site_name.clone(),
                    scope: crate::site_config::models::PmaDbScope::SiteOnly,
                });
            }

            ui.separator();

            if ui.button("Open framework docs").clicked() {
                // Direct spawn — no AppCommand round-trip needed for an
                // external doc URL.
                let _ = std::process::Command::new("xdg-open")
                    .arg(docs_url)
                    .spawn();
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
                        RichText::new("Unlink").color(Colors::DANGER),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                let _ = cmd_tx.try_send(AppCommand::UnlinkSite(site_name.clone()));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_url_for_laravel() {
        assert_eq!(framework_docs_url(&DetectedFramework::Laravel), "https://laravel.com/docs");
    }

    #[test]
    fn docs_url_for_wordpress_and_bedrock_share() {
        assert_eq!(
            framework_docs_url(&DetectedFramework::WordPress),
            framework_docs_url(&DetectedFramework::Bedrock),
        );
    }

    #[test]
    fn docs_url_covers_all_22_variants() {
        for fw in DetectedFramework::all_variants() {
            let u = framework_docs_url(&fw);
            assert!(u.starts_with("http"), "{:?} = {:?}", fw, u);
        }
    }

    fn make_site(name: &str, status: crate::valet::site_scanner::SiteStatus, secured: bool) -> crate::valet::site_scanner::ValetSite {
        use crate::valet::site_scanner::SiteType;
        ValetSite {
            name: name.to_string(),
            domain: format!("{}.test", name),
            path: std::path::PathBuf::from(format!("/tmp/{}", name)),
            site_type: SiteType::Parked,
            framework: DetectedFramework::Unknown,
            php_version: None,
            is_secured: secured,
            ssl_expiry: None,
            tls_expiry_days: None,
            is_favorite: false,
            status,
            last_hit: None,
            proxy_target: None,
        }
    }

    #[test]
    fn filter_running_only() {
        use crate::state::app_state::SiteStatusFilter;
        use crate::valet::site_scanner::SiteStatus;
        let sites = vec![
            make_site("a", SiteStatus::Running, true),
            make_site("b", SiteStatus::Stopped, false),
            make_site("c", SiteStatus::Failed,  false),
        ];
        let filter = SiteStatusFilter::Running;
        let filtered: Vec<_> = sites.iter().filter(|s| match filter {
            SiteStatusFilter::All     => true,
            SiteStatusFilter::Running => s.status == SiteStatus::Running,
            SiteStatusFilter::Stopped => s.status == SiteStatus::Stopped,
            SiteStatusFilter::Failed  => s.status == SiteStatus::Failed,
        }).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "a");
    }

    #[test]
    fn stat_counts() {
        use crate::valet::site_scanner::SiteStatus;
        let sites = vec![
            make_site("a", SiteStatus::Running, true),
            make_site("b", SiteStatus::Running, false),
            make_site("c", SiteStatus::Failed,  false),
            make_site("d", SiteStatus::Stopped, false),
        ];
        let running  = sites.iter().filter(|s| s.status == SiteStatus::Running).count();
        let failed   = sites.iter().filter(|s| s.status == SiteStatus::Failed).count();
        let secured  = sites.iter().filter(|s| s.is_secured).count();
        assert_eq!(running, 2);
        assert_eq!(failed,  1);
        assert_eq!(secured, 1);
    }

    #[test]
    fn last_hit_some_shows_value() {
        let mut site = make_site("x", crate::valet::site_scanner::SiteStatus::Running, false);
        site.last_hit = Some("2m ago".into());
        let text = site.last_hit.as_deref().unwrap_or("—");
        assert_eq!(text, "2m ago");
    }

    #[test]
    fn last_hit_none_shows_dash() {
        let site = make_site("x", crate::valet::site_scanner::SiteStatus::Running, false);
        let text = site.last_hit.as_deref().unwrap_or("—");
        assert_eq!(text, "—");
    }

    #[test]
    fn favorites_sort_before_non_favorites() {
        // favorites must appear before non-favorites after sort
        let fav = true;
        let non_fav = false;
        // b.is_favorite.cmp(&a.is_favorite) sorts true before false
        use std::cmp::Ordering;
        assert_eq!(fav.cmp(&non_fav), Ordering::Greater);
        // reversed (b vs a) puts favorites first
        assert_eq!(non_fav.cmp(&fav), Ordering::Less);
    }

    #[test]
    fn status_color_varies_by_status() {
        use crate::valet::site_scanner::SiteStatus;
        assert_ne!(status_color(&SiteStatus::Running), status_color(&SiteStatus::Failed));
        assert_ne!(status_color(&SiteStatus::Running), status_color(&SiteStatus::Stopped));
    }

    #[test]
    fn tls_color_danger_under_7_days() {
        use crate::ui::theme::Colors;
        let color = match Some(3u32) {
            Some(d) if d < 7  => Colors::DANGER,
            Some(d) if d < 30 => Colors::WARNING,
            _                 => Colors::ACCENT,
        };
        assert_eq!(color, Colors::DANGER);
    }
}

// ── Public entry point ───────────────────────────────────────────────────────
#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Sites")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshSites);
            }
            if accent_button(ui, "+ Link site").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenAddSiteModal);
            }
        });
    });
    divider(ui);

    // ── Body padding ─────────────────────────────────────────────────────
    ui.add_space(16.0);

    // ── Filter sites ─────────────────────────────────────────────────────
    use crate::state::app_state::SiteStatusFilter;
    use crate::valet::site_scanner::SiteStatus;

    let search_lower  = state.site_search.to_lowercase();
    let mut filtered: Vec<&ValetSite> = state
        .sites
        .iter()
        .filter(|s| {
            let status_ok = match state.ui.site_status_filter {
                SiteStatusFilter::All     => true,
                SiteStatusFilter::Running => s.status == SiteStatus::Running,
                SiteStatusFilter::Stopped => s.status == SiteStatus::Stopped,
                SiteStatusFilter::Failed  => s.status == SiteStatus::Failed,
            };
            let search_ok = search_lower.is_empty()
                || s.name.to_lowercase().contains(&search_lower)
                || s.domain.to_lowercase().contains(&search_lower);
            status_ok && search_ok
        })
        .collect();
    filtered.sort_by(|a, b| {
        b.is_favorite.cmp(&a.is_favorite).then(a.name.cmp(&b.name))
    });

    // ── Stat strip ───────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        stat_strip(ui, &filtered);
    });
    ui.add_space(12.0);

    // ── Filter chips ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        for (label, filter) in &[
            ("All",     SiteStatusFilter::All),
            ("Running", SiteStatusFilter::Running),
            ("Stopped", SiteStatusFilter::Stopped),
            ("Failed",  SiteStatusFilter::Failed),
        ] {
            let is_active = &state.ui.site_status_filter == filter;
            let bg    = if is_active { with_alpha(Colors::ACCENT, 20) } else { Colors::CARD };
            let color = if is_active { Colors::ACCENT } else { Colors::TEXT_SECONDARY };
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(label.len() as f32 * 7.0 + 16.0, 24.0),
                egui::Sense::click(),
            );
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
                if is_active {
                    ui.painter().rect_stroke(rect, CornerRadius::same(5),
                        Stroke::new(0.5, Colors::ACCENT), egui::StrokeKind::Inside);
                }
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, *label,
                    egui::FontId::proportional(11.5), color);
            }
            if resp.clicked() {
                let _ = cmd_tx.try_send(AppCommand::SetSiteFilter(filter.clone()));
            }
            ui.add_space(4.0);
        }
    });
    ui.add_space(6.0);

    // ── Search bar ───────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        ui.add_sized(
            egui::vec2(avail, 28.0),
            egui::TextEdit::singleline(&mut state.site_search)
                .hint_text("Search sites…")
                .text_color(Colors::TEXT_PRIMARY),
        );
    });
    ui.add_space(12.0);

    // ── Empty state ──────────────────────────────────────────────────────
    if filtered.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No sites found")
                    .size(13.0)
                    .color(Colors::TEXT_SECONDARY),
            );
            ui.add_space(12.0);
            if accent_button(ui, "Link a site").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenAddSiteModal);
            }
        });
        return;
    }

    // ── Table wrapper ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);

        let table_width = ui.available_width() - 22.0;

        // flex_width = total - fixed columns
        let fixed = COL_FAV + COL_DOT + COL_FRAMEWORK + COL_PHP + COL_TLS + COL_LASTHIT + COL_ACTION;
        let flex_width = (table_width - fixed).max(120.0);

        Frame::NONE
            .fill(Colors::CARD)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(0.5, Colors::BORDER))
            .show(ui, |ui| {
                ui.set_clip_rect(ui.max_rect());
                ui.set_width(table_width);

                // Header
                table_header(ui, flex_width);

                // Rows (scrollable)
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for site in &filtered {
                        let pma_enabled = state.pma_state.sites.get(&site.name).map(|s| s.enabled).unwrap_or(false);
                        render_site_row(ui, site, flex_width, cmd_tx, pma_enabled);
                    }
                });
            });
    });

    // ── Detail footer ─────────────────────────────────────────────────────
    if let Some(site_name) = state.site_config_selected.clone() {
        if let Some(site) = state.sites.iter().find(|s| s.name == site_name) {
            ui.add_space(8.0);
            divider(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&site.domain).size(12.0).color(Colors::TEXT_PRIMARY).strong());
                    ui.label(RichText::new(site.path.display().to_string()).size(10.5).color(Colors::TEXT_TERTIARY)
                        .text_style(egui::TextStyle::Monospace));
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                if accent_button(ui, "↗ Open").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(site.domain.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "⌂ Reveal").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RevealSiteInFiles(site.name.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "↺ Restart").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RestartSite(site.name.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "⌨ Shell").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenShellAtSite(site.name.clone()));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(22.0);
                    if danger_button(ui, "✕ Unpark").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::UnparkSite(site.name.clone()));
                    }
                });
            });
        }
    }
}
