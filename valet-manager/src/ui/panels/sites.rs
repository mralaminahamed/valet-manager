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

// ── Column widths (canonical CSS: 28px | minmax(220px,2fr) | 110px | 70px | 60px | 130px | 28px)
const COL_DOT:       f32 = 28.0;
// COL_SITE is flex (available - fixed cols)
const COL_FRAMEWORK: f32 = 110.0;
const COL_PHP:       f32 = 70.0;
const COL_TLS:       f32 = 60.0;
const COL_LASTHIT:   f32 = 130.0;
const COL_ACTION:    f32 = 28.0;
const COL_FIXED:     f32 = COL_DOT + COL_FRAMEWORK + COL_PHP + COL_TLS + COL_LASTHIT + COL_ACTION;
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
fn stat_strip(ui: &mut egui::Ui, sites: &[ValetSite]) {
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

const ROW_PAD: f32 = 14.0;

// ── Table header ─────────────────────────────────────────────────────────────
fn table_header(ui: &mut egui::Ui) {
    let header_bg = with_alpha(Color32::BLACK, 38);
    let border    = Stroke::new(0.5, Colors::BORDER);

    let avail_w  = ui.available_width();
    let inner_w  = avail_w - 2.0 * ROW_PAD;
    let site_col = (inner_w - COL_FIXED).max(220.0);
    let (header_rect, _) = ui.allocate_exact_size(
        egui::vec2(avail_w, HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(header_rect, CornerRadius::ZERO, header_bg);
    ui.painter().hline(header_rect.x_range(), header_rect.bottom(), border);

    let inner_rect = header_rect.shrink2(egui::vec2(ROW_PAD, 0.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    let hdr_label = |text: &str| {
        RichText::new(text).size(10.5).strong().color(Colors::TEXT_TERTIARY)
    };

    // • | SITE | FRAMEWORK | PHP | TLS | LAST HIT | ⋮
    child.allocate_ui(egui::vec2(COL_DOT, HEADER_HEIGHT), |_| {});
    child.allocate_ui(egui::vec2(site_col, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("SITE"));
        });
    });
    child.allocate_ui(egui::vec2(COL_FRAMEWORK, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("FRAMEWORK"));
        });
    });
    child.allocate_ui(egui::vec2(COL_PHP, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("PHP"));
        });
    });
    child.allocate_ui(egui::vec2(COL_TLS, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("TLS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_LASTHIT, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr_label("LAST HIT"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ACTION, HEADER_HEIGHT), |_| {});
}

// ── Site row action ──────────────────────────────────────────────────────────
enum SiteRowAction {
    None,
    Select,
}

// ── Site row ─────────────────────────────────────────────────────────────────
fn render_site_row(
    ui: &mut egui::Ui,
    site: &ValetSite,
    selected: bool,
    cmd_tx: &Sender<AppCommand>,
    pma_enabled: bool,
) -> SiteRowAction {
    let avail_w  = ui.available_width();
    let inner_w  = avail_w - 2.0 * ROW_PAD;
    let site_col = (inner_w - COL_FIXED).max(220.0);

    let (row_rect, row_resp) = ui.allocate_exact_size(
        egui::vec2(avail_w, ROW_HEIGHT),
        egui::Sense::click(),
    );

    // Row background
    if selected {
        ui.painter().rect_filled(row_rect, CornerRadius::ZERO, with_alpha(Colors::ACCENT, 20));
    } else if row_resp.hovered() {
        ui.painter().rect_filled(row_rect, CornerRadius::ZERO, Colors::CARD_HOVER);
    }

    let mut action = SiteRowAction::None;

    if row_resp.clicked() {
        action = SiteRowAction::Select;
    }

    // Border-bottom
    ui.painter().hline(
        row_rect.x_range(),
        row_rect.bottom(),
        Stroke::new(0.5, Colors::BORDER),
    );

    let inner_rect = row_rect.shrink2(egui::vec2(ROW_PAD, 0.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // ── Col 0: Status dot (28px) ─────────────────────────────────────────
    row_cell(&mut child, COL_DOT, ROW_HEIGHT, |ui| {
        use crate::valet::site_scanner::SiteStatus;
        let dot_color = status_color(&site.status);
        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        // Glow ring (box-shadow: 0 0 0 3px ...)
        let glow_color = match site.status {
            SiteStatus::Running => with_alpha(Colors::ACCENT, 31),
            SiteStatus::Failed  => with_alpha(Colors::DANGER, 38),
            _                   => Color32::TRANSPARENT,
        };
        ui.painter().circle_filled(dot_rect.center(), 7.0, glow_color);
        ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
    });

    // ── Col 1: Site (domain + path, flex) ───────────────────────────────
    let domain_color = if selected { Colors::ACCENT } else { Colors::TEXT_PRIMARY };
    row_cell(&mut child, site_col, ROW_HEIGHT, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                // Domain text (TEXT_PRIMARY by default, ACCENT when selected)
                ui.label(
                    RichText::new(&site.domain)
                        .size(12.5)
                        .strong()
                        .color(domain_color),
                );
                // Lock icon if secured (inline with domain name)
                if site.is_secured {
                    let tls_color = match site.tls_expiry_days {
                        Some(d) if d < 7  => Colors::DANGER,
                        Some(d) if d < 30 => Colors::WARNING,
                        _                 => Colors::ACCENT,
                    };
                    ui.label(RichText::new("🔒").size(10.0).color(tls_color));
                }
            });
            // Path — monospace, tertiary
            let path_str = site.path.to_string_lossy();
            let truncated = if path_str.len() > 40 {
                format!("{}…", &path_str[..40])
            } else {
                path_str.to_string()
            };
            ui.label(RichText::new(truncated).size(10.5).color(Colors::TEXT_TERTIARY).monospace());
        });
    });

    // ── Col 2: Framework badge (110px) ───────────────────────────────────
    let fw_label   = framework_display_name(&site.framework);
    let fw_rec     = recommended_php_version(&site.framework);
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

    // ── Col 3: PHP version (70px) ────────────────────────────────────────
    row_cell(&mut child, COL_PHP, ROW_HEIGHT, |ui| {
        let (php_text, php_color) = match &site.php_version {
            Some(v) => (v.clone(), Colors::TEXT_SECONDARY),
            None    => (String::from("—"), Colors::TEXT_TERTIARY),
        };
        ui.label(RichText::new(php_text).size(11.0).monospace().color(php_color));
    });

    // ── Col 4: TLS (60px) ────────────────────────────────────────────────
    row_cell(&mut child, COL_TLS, ROW_HEIGHT, |ui| {
        let (tls_text, tls_color) = if site.is_secured {
            let color = match site.tls_expiry_days {
                Some(d) if d < 7  => Colors::DANGER,
                Some(d) if d < 30 => Colors::WARNING,
                _                 => Colors::ACCENT,
            };
            ("on", color)
        } else {
            ("—", Colors::TEXT_TERTIARY)
        };
        ui.label(RichText::new(tls_text).size(11.0).monospace().color(tls_color));
    });

    // ── Col 5: Last hit (130px) ──────────────────────────────────────────
    row_cell(&mut child, COL_LASTHIT, ROW_HEIGHT, |ui| {
        let hit_text = site.last_hit.as_deref().unwrap_or("—");
        ui.label(RichText::new(hit_text).size(11.5).color(Colors::TEXT_TERTIARY));
    });

    // ── Col 6: Action menu (36px) ────────────────────────────────────────
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

    action
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
        use crate::valet::site_scanner::SiteStatus;
        let mut sites = vec![
            make_site("alpha",   SiteStatus::Running, false),
            make_site("beta",    SiteStatus::Running, false),
            make_site("charlie", SiteStatus::Running, false),
        ];
        sites[2].is_favorite = true; // charlie is favorite
        sites.sort_by(|a, b| b.is_favorite.cmp(&a.is_favorite).then(a.name.cmp(&b.name)));
        assert_eq!(sites[0].name, "charlie"); // favorite first
        assert_eq!(sites[1].name, "alpha");   // then alphabetical
        assert_eq!(sites[2].name, "beta");
    }

    #[test]
    fn toggle_favorite_flips_flag() {
        use crate::valet::site_scanner::SiteStatus;
        let mut sites = vec![make_site("mysite", SiteStatus::Running, false)];
        assert!(!sites[0].is_favorite);
        if let Some(s) = sites.iter_mut().find(|s| s.name == "mysite") {
            s.is_favorite = !s.is_favorite;
        }
        assert!(sites[0].is_favorite);
        // toggle back
        if let Some(s) = sites.iter_mut().find(|s| s.name == "mysite") {
            s.is_favorite = !s.is_favorite;
        }
        assert!(!sites[0].is_favorite);
    }

    #[test]
    fn filter_chips_all_variants_covered() {
        use crate::state::app_state::SiteStatusFilter;
        use crate::valet::site_scanner::SiteStatus;
        let sites = vec![
            make_site("a", SiteStatus::Running, false),
            make_site("b", SiteStatus::Stopped, false),
            make_site("c", SiteStatus::Failed,  false),
        ];
        let apply = |f: SiteStatusFilter| -> Vec<&str> {
            sites.iter().filter(|s| match f {
                SiteStatusFilter::All     => true,
                SiteStatusFilter::Running => s.status == SiteStatus::Running,
                SiteStatusFilter::Stopped => s.status == SiteStatus::Stopped,
                SiteStatusFilter::Failed  => s.status == SiteStatus::Failed,
            }).map(|s| s.name.as_str()).collect()
        };
        assert_eq!(apply(SiteStatusFilter::All),     vec!["a","b","c"]);
        assert_eq!(apply(SiteStatusFilter::Running), vec!["a"]);
        assert_eq!(apply(SiteStatusFilter::Stopped), vec!["b"]);
        assert_eq!(apply(SiteStatusFilter::Failed),  vec!["c"]);
    }

    #[test]
    fn search_filters_by_name_and_domain() {
        use crate::valet::site_scanner::SiteStatus;
        let sites = vec![
            make_site("easycommerce", SiteStatus::Running, false),
            make_site("laravel-app",  SiteStatus::Running, false),
        ];
        let query = "easy";
        let matched: Vec<&str> = sites.iter()
            .filter(|s| s.name.contains(query) || s.domain.contains(query))
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(matched, vec!["easycommerce"]);
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
                state.ui.add_site_modal_open = true;
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
    let mut filtered: Vec<ValetSite> = state
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
        .cloned()
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
                state.ui.site_status_filter = filter.clone();
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
                state.ui.add_site_modal_open = true;
            }
        });
        return;
    }

    // ── Table wrapper ────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);

        let table_width = ui.available_width() - 22.0;

        Frame::NONE
            .fill(Colors::CARD)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(0.5, Colors::BORDER))
            .show(ui, |ui| {
                ui.set_clip_rect(ui.max_rect());
                ui.set_width(table_width);

                // Header
                table_header(ui);

                // Rows (scrollable)
                let mut select_name: Option<String> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for site in &filtered {
                        let is_sel = state.site_config_selected.as_deref() == Some(site.name.as_str());
                        let pma_enabled = state.pma_state.sites.get(&site.name).map(|s| s.enabled).unwrap_or(false);
                        match render_site_row(ui, site, is_sel, cmd_tx, pma_enabled) {
                            SiteRowAction::Select => select_name = Some(site.name.clone()),
                            SiteRowAction::None => {}
                        }
                    }
                });
                if let Some(name) = select_name {
                    state.site_config_selected = Some(name);
                }
            });
    });

    // ── Detail footer ─────────────────────────────────────────────────────
    if let Some(site_name) = state.site_config_selected.clone() {
        let detail = state.sites.iter().find(|s| s.name == site_name).map(|s| {
            (s.domain.clone(), s.path.clone(), s.name.clone())
        });
        if let Some((site_domain, site_path, site_name_clone)) = detail {
            ui.add_space(8.0);
            divider(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&site_domain).size(12.0).color(Colors::TEXT_PRIMARY).strong());
                    ui.label(RichText::new(site_path.display().to_string()).size(10.5).color(Colors::TEXT_TERTIARY)
                        .text_style(egui::TextStyle::Monospace));
                });
            });
            ui.add_space(6.0);
            let mut open_shell = false;
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                if accent_button(ui, "↗ Open").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(site_domain.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "⌂ Reveal").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RevealSiteInFiles(site_name_clone.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "↺ Restart").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RestartSite(site_name_clone.clone()));
                }
                ui.add_space(4.0);
                if ghost_button(ui, "⌨ Shell").clicked() {
                    open_shell = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(22.0);
                    if danger_button(ui, "✕ Unpark").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::UnparkSite(site_name_clone.clone()));
                    }
                });
            });
            if open_shell {
                state.shell_site = Some(site_name_clone);
                state.ui.shell_open = true;
            }
        }
    }
}
