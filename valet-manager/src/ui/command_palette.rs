use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::commands::AppCommand;
use crate::state::app_state::{AppState, Panel};
use crate::ui::theme::{Colors, with_alpha};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PaletteCategory {
    PhpVersion,
    Site,
    Service,
    Artisan,
    Panel,
    Action,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PaletteResult {
    pub label: String,
    pub subtitle: Option<String>,
    pub category: PaletteCategory,
    pub action: AppCommand,
    pub shortcut: Option<String>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct PaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub results: Vec<PaletteResult>,
    pub index: Vec<PaletteResult>,  // unfiltered
}

#[allow(dead_code)]
pub fn build_index(state: &AppState) -> Vec<PaletteResult> {
    let mut idx = Vec::new();

    // PHP versions
    for v in &state.php_versions {
        let status = if v.is_active { "active" } else { "available" };
        idx.push(PaletteResult {
            label: format!("Switch to PHP {}", v.version),
            subtitle: Some(format!("{} · {}", v.full_version, status)),
            category: PaletteCategory::PhpVersion,
            action: AppCommand::SwitchGlobalPhp(v.version.clone()),
            shortcut: None,
        });
    }

    // Sites (max 20)
    for site in state.sites.iter().take(20) {
        idx.push(PaletteResult {
            label: format!("Open {}", site.domain),
            subtitle: Some(site.path.to_string_lossy().to_string()),
            category: PaletteCategory::Site,
            action: AppCommand::OpenSiteInBrowser(site.domain.clone()),
            shortcut: None,
        });
        idx.push(PaletteResult {
            label: format!("Open {} in editor", site.domain),
            subtitle: None,
            category: PaletteCategory::Site,
            action: AppCommand::OpenSiteInEditor(site.name.clone()),
            shortcut: None,
        });
    }

    // Panels (every variant we want to be palette-reachable)
    for (label, panel) in [
        ("Go to Dashboard", Panel::Dashboard),
        ("Go to PHP Versions", Panel::PhpVersions),
        ("Go to PHP Extensions", Panel::PhpExtensions),
        ("Go to PHP INI", Panel::PhpIni),
        ("Go to phpinfo()", Panel::PhpInfo),
        ("Go to PHP Compatibility", Panel::PhpCompat),
        ("Go to Sites", Panel::Sites),
        ("Go to Parks", Panel::Parks),
        ("Go to Nginx", Panel::Nginx),
        ("Go to App Creator", Panel::AppCreator),
        ("Go to Proxies", Panel::Proxies),
        ("Go to dnsmasq", Panel::Dnsmasq),
        ("Go to SSL Certs", Panel::SslCerts),
        ("Go to Database", Panel::Database),
        ("Go to .env Editor", Panel::EnvEditor),
        ("Go to Artisan", Panel::Artisan),
        ("Go to Queue Workers", Panel::QueueWorkers),
        ("Go to Xdebug", Panel::Xdebug),
        ("Go to Mailpit", Panel::MailCatcher),
        ("Go to Sharing", Panel::Sharing),
        ("Go to Logs", Panel::Logs),
        ("Go to History", Panel::History),
        ("Go to Diagnostics", Panel::Diagnostics),
        ("Go to Settings", Panel::Settings),
    ] {
        idx.push(PaletteResult {
            label: label.to_string(),
            subtitle: None,
            category: PaletteCategory::Panel,
            action: AppCommand::OpenPanel(panel),
            shortcut: None,
        });
    }

    // Common actions
    idx.push(PaletteResult {
        label: "Refresh everything".to_string(),
        subtitle: Some("Re-scan PHP / sites / services".to_string()),
        category: PaletteCategory::Action,
        action: AppCommand::RefreshAll,
        shortcut: Some("Ctrl+R".to_string()),
    });
    idx.push(PaletteResult {
        label: "Reload nginx".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::ReloadNginx,
        shortcut: None,
    });
    idx.push(PaletteResult {
        label: "Check for updates".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::CheckForUpdates,
        shortcut: None,
    });
    idx.push(PaletteResult {
        label: "Run diagnostics".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::RunDiagnostics,
        shortcut: None,
    });

    // Phase 10 — Install Xdebug entries for PHP versions where it's not installed
    for v in &state.php_versions {
        let needs_install = state
            .xdebug_configs
            .get(&v.version)
            .map(|c| !c.installed)
            .unwrap_or(false);
        if needs_install {
            idx.push(PaletteResult {
                label: format!("Install Xdebug for PHP {}", v.version),
                subtitle: Some(format!("php{}-xdebug", v.version)),
                category: PaletteCategory::Action,
                action: AppCommand::InstallXdebug(v.version.clone()),
                shortcut: None,
            });
        }
    }

    // Phase 10 — Mail catcher actions
    idx.push(PaletteResult {
        label: "Mail catcher: Start".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::StartMailCatcher,
        shortcut: None,
    });
    idx.push(PaletteResult {
        label: "Mail catcher: Stop".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::StopMailCatcher,
        shortcut: None,
    });

    // Phase 10 — SSL refresh
    idx.push(PaletteResult {
        label: "Refresh SSL certs".to_string(),
        subtitle: None,
        category: PaletteCategory::Action,
        action: AppCommand::RefreshSslCerts,
        shortcut: None,
    });

    // Phase 13 — framework-specific quick actions per site.
    //
    // NOTE: RunArtisan today dispatches `php artisan ...`. Magento/Drupal
    // calls below will need a Framework-CLI-aware dispatcher in a later
    // phase to invoke bin/magento / drush correctly. Until then, the
    // palette entry shows up but execution will only succeed for sites
    // whose runner actually understands the command.
    // TODO Phase 13+: switch on FrameworkCli in the dispatcher.
    use crate::ui::DetectedFramework;
    for site in state.sites.iter().take(20) {
        match site.framework {
            DetectedFramework::Magento => {
                idx.push(PaletteResult {
                    label: format!("Flush Magento cache @ {}", site.domain),
                    subtitle: Some("bin/magento cache:flush".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "cache:flush".into(),
                        args: String::new(),
                    },
                    shortcut: None,
                });
                idx.push(PaletteResult {
                    label: format!("Reindex Magento @ {}", site.domain),
                    subtitle: Some("bin/magento indexer:reindex".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "indexer:reindex".into(),
                        args: String::new(),
                    },
                    shortcut: None,
                });
            }
            DetectedFramework::Drupal => {
                idx.push(PaletteResult {
                    label: format!("Drupal cache rebuild @ {}", site.domain),
                    subtitle: Some("drush cr".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "cr".into(),
                        args: String::new(),
                    },
                    shortcut: None,
                });
                idx.push(PaletteResult {
                    label: format!("Drupal update DB @ {}", site.domain),
                    subtitle: Some("drush updb -y".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "updb".into(),
                        args: "-y".into(),
                    },
                    shortcut: None,
                });
            }
            DetectedFramework::Symfony => {
                idx.push(PaletteResult {
                    label: format!("Clear Symfony cache @ {}", site.domain),
                    subtitle: Some("bin/console cache:clear".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "cache:clear".into(),
                        args: String::new(),
                    },
                    shortcut: None,
                });
            }
            DetectedFramework::OctoberCms => {
                idx.push(PaletteResult {
                    label: format!("OctoberCMS migrate @ {}", site.domain),
                    subtitle: Some("artisan october:migrate".into()),
                    category: PaletteCategory::Artisan,
                    action: AppCommand::RunArtisan {
                        site: site.name.clone(),
                        command: "october:migrate".into(),
                        args: String::new(),
                    },
                    shortcut: None,
                });
            }
            _ => {}
        }
    }

    idx
}

#[allow(dead_code)]
pub fn filter(index: &[PaletteResult], query: &str) -> Vec<PaletteResult> {
    if query.is_empty() {
        return index.iter().take(8).cloned().collect();
    }
    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, &PaletteResult)> = index.iter()
        .filter_map(|r| {
            matcher.fuzzy_match(&r.label, query).map(|s| {
                let category_bonus = if r.category == PaletteCategory::PhpVersion { 100 } else { 0 };
                (s + category_bonus, r)
            })
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(8).map(|(_, r)| r.clone()).collect()
}

fn category_color(cat: PaletteCategory) -> Color32 {
    match cat {
        PaletteCategory::PhpVersion => Colors::WARNING,
        PaletteCategory::Site       => Colors::ACCENT,
        PaletteCategory::Service    => Colors::INFO,
        PaletteCategory::Artisan    => Colors::PURPLE,
        PaletteCategory::Panel      => Colors::TEXT_SECONDARY,
        PaletteCategory::Action     => Colors::TEXT_SECONDARY,
    }
}

fn category_label(cat: PaletteCategory) -> &'static str {
    match cat {
        PaletteCategory::PhpVersion => "PHP",
        PaletteCategory::Site       => "site",
        PaletteCategory::Service    => "svc",
        PaletteCategory::Artisan    => "art",
        PaletteCategory::Panel      => "go",
        PaletteCategory::Action     => "act",
    }
}

/// Render the palette overlay if open. Returns Some(AppCommand) when the user
/// selected a result; closes the palette in that case.
#[allow(dead_code)]
pub fn render(ctx: &egui::Context, state: &mut PaletteState) -> Option<AppCommand> {
    if !state.open { return None; }

    // Re-filter on every frame when query changes; simpler to do it always.
    state.results = filter(&state.index, &state.query);
    if state.selected >= state.results.len() { state.selected = 0; }

    // Keyboard handling
    let mut commit: Option<AppCommand> = None;
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            state.open = false;
        }
        if i.key_pressed(egui::Key::ArrowDown) && !state.results.is_empty() {
            state.selected = (state.selected + 1) % state.results.len();
        }
        if i.key_pressed(egui::Key::ArrowUp) && !state.results.is_empty() {
            state.selected = (state.selected + state.results.len() - 1) % state.results.len();
        }
        if i.key_pressed(egui::Key::Enter) {
            if let Some(r) = state.results.get(state.selected).cloned() {
                commit = Some(r.action);
                state.open = false;
            }
        }
    });

    if commit.is_some() { return commit; }
    if !state.open { return None; }

    // Dim background
    egui::Area::new(egui::Id::new("palette_dim"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let r = ctx.input(|i| i.viewport().outer_rect.unwrap_or(i.content_rect()));
            ui.painter().rect_filled(r, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(0, 0, 0, 100));
        });

    // Card
    egui::Area::new(egui::Id::new("palette_card"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 120.0))
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Colors::DEEP_BG)
                .corner_radius(CornerRadius::same(10))
                .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                .inner_margin(Margin { left: 0, right: 0, top: 0, bottom: 0 })
                .show(ui, |ui| {
                    ui.set_min_width(520.0);
                    ui.set_max_width(520.0);

                    // Search input
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        let resp = ui.add_sized(
                            egui::vec2(520.0 - 28.0, 32.0),
                            egui::TextEdit::singleline(&mut state.query)
                                .font(egui::TextStyle::Heading)
                                .hint_text("Search commands, sites, PHP versions…")
                                .text_color(Colors::TEXT_PRIMARY),
                        );
                        resp.request_focus();
                    });
                    ui.add_space(6.0);
                    // Divider
                    let avail = ui.available_width();
                    let (line_rect, _) = ui.allocate_exact_size(egui::vec2(avail, 1.0), egui::Sense::hover());
                    ui.painter().hline(line_rect.x_range(), line_rect.center().y, Stroke::new(0.5, Colors::BORDER));

                    // Results
                    ScrollArea::vertical().max_height(360.0).auto_shrink([false, false]).show(ui, |ui| {
                        for (i, r) in state.results.clone().iter().enumerate() {
                            let bg = if i == state.selected { Colors::CARD_HOVER } else { Color32::TRANSPARENT };
                            Frame::NONE
                                .fill(bg)
                                .inner_margin(Margin { left: 14, right: 14, top: 0, bottom: 0 })
                                .show(ui, |ui| {
                                    let (row_rect, row_resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), egui::Sense::click());
                                    if row_resp.clicked() {
                                        commit = Some(r.action.clone());
                                        state.open = false;
                                    }
                                    let mut child = ui.new_child(
                                        egui::UiBuilder::new()
                                            .max_rect(row_rect)
                                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                                    );
                                    // Category badge
                                    let cat_color = category_color(r.category);
                                    Frame::NONE
                                        .fill(with_alpha(cat_color, 30))
                                        .corner_radius(CornerRadius::same(10))
                                        .inner_margin(Margin { left: 6, right: 6, top: 1, bottom: 1 })
                                        .show(&mut child, |ui| {
                                            ui.label(RichText::new(category_label(r.category)).size(10.0).color(cat_color));
                                        });
                                    child.add_space(10.0);
                                    child.vertical(|ui| {
                                        ui.label(RichText::new(&r.label).size(13.0).color(Colors::TEXT_PRIMARY));
                                        if let Some(sub) = &r.subtitle {
                                            ui.label(RichText::new(sub).size(11.0).color(Colors::TEXT_SECONDARY));
                                        }
                                    });
                                    if let Some(sc) = &r.shortcut {
                                        child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(sc).size(11.0).color(Colors::TEXT_TERTIARY));
                                        });
                                    }
                                });
                        }
                    });
                    ui.add_space(6.0);
                    // Footer
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.label(RichText::new("↑↓ navigate · ↵ execute · esc dismiss").size(11.0).color(Colors::TEXT_TERTIARY));
                        ui.add_space(14.0);
                    });
                    ui.add_space(6.0);
                });
        });

    commit
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_query_returns_first_8() {
        let idx: Vec<PaletteResult> = (0..20).map(|i| PaletteResult {
            label: format!("Item {}", i),
            subtitle: None,
            category: PaletteCategory::Action,
            action: AppCommand::RefreshAll,
            shortcut: None,
        }).collect();
        let r = filter(&idx, "");
        assert_eq!(r.len(), 8);
    }
    #[test]
    fn fuzzy_filter_matches() {
        let idx = vec![
            PaletteResult { label: "Open Dashboard".into(), subtitle: None, category: PaletteCategory::Panel, action: AppCommand::OpenPanel(Panel::Dashboard), shortcut: None },
            PaletteResult { label: "Refresh".into(),       subtitle: None, category: PaletteCategory::Action, action: AppCommand::RefreshAll, shortcut: None },
        ];
        let r = filter(&idx, "dash");
        assert!(r.iter().any(|x| x.label.contains("Dashboard")));
    }
    #[test]
    fn category_colors_distinct() {
        assert_ne!(category_color(PaletteCategory::PhpVersion), category_color(PaletteCategory::Site));
        assert_ne!(category_color(PaletteCategory::Panel), category_color(PaletteCategory::PhpVersion));
    }
    #[test]
    fn category_labels_short() {
        assert_eq!(category_label(PaletteCategory::PhpVersion), "PHP");
        assert_eq!(category_label(PaletteCategory::Site), "site");
        assert_eq!(category_label(PaletteCategory::Artisan), "art");
    }
    #[test]
    fn build_index_includes_panels() {
        let state = AppState::default();
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label == "Go to Dashboard"));
        assert!(idx.iter().any(|r| r.label == "Go to Database"));
        assert!(idx.iter().any(|r| r.label == "Go to Artisan"));
        assert!(idx.iter().any(|r| r.label == "Go to .env Editor"));
    }
    #[test]
    fn build_index_includes_actions() {
        let state = AppState::default();
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label == "Refresh everything"));
        assert!(idx.iter().any(|r| r.label == "Reload nginx"));
    }
    #[test]
    fn palette_state_defaults_closed() {
        let s = PaletteState::default();
        assert!(!s.open);
        assert_eq!(s.selected, 0);
        assert!(s.query.is_empty());
    }

    #[test]
    fn build_index_includes_mail_catcher_actions() {
        let state = AppState::default();
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label == "Mail catcher: Start"));
        assert!(idx.iter().any(|r| r.label == "Mail catcher: Stop"));
    }

    #[test]
    fn build_index_includes_ssl_refresh() {
        let state = AppState::default();
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label == "Refresh SSL certs"));
    }

    // ── Phase 13 — framework quick actions ────────────────────────────

    fn make_site(name: &str, fw: crate::ui::DetectedFramework) -> crate::valet::site_scanner::ValetSite {
        crate::valet::site_scanner::ValetSite {
            name: name.to_string(),
            domain: format!("{}.test", name),
            path: std::path::PathBuf::from(format!("/srv/{}", name)),
            site_type: crate::valet::site_scanner::SiteType::Linked,
            framework: fw,
            php_version: None,
            is_secured: false,
            ssl_expiry: None,
            tls_expiry_days: None,
            is_favorite: false,
            status: crate::valet::site_scanner::SiteStatus::Unknown,
            last_hit: None,
            proxy_target: None,
        }
    }

    #[test]
    fn build_index_emits_magento_quick_actions() {
        let mut state = AppState::default();
        state.sites.push(make_site("shop", crate::ui::DetectedFramework::Magento));
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label.starts_with("Flush Magento cache @ shop.test")));
        assert!(idx.iter().any(|r| r.label.starts_with("Reindex Magento @ shop.test")));
    }

    #[test]
    fn build_index_emits_drupal_quick_actions() {
        let mut state = AppState::default();
        state.sites.push(make_site("intra", crate::ui::DetectedFramework::Drupal));
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label.starts_with("Drupal cache rebuild @ intra.test")));
        assert!(idx.iter().any(|r| r.label.starts_with("Drupal update DB @ intra.test")));
    }

    #[test]
    fn build_index_emits_symfony_cache_clear() {
        let mut state = AppState::default();
        state.sites.push(make_site("api", crate::ui::DetectedFramework::Symfony));
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label.starts_with("Clear Symfony cache @ api.test")));
    }

    #[test]
    fn build_index_emits_octobercms_migrate() {
        let mut state = AppState::default();
        state.sites.push(make_site("blog", crate::ui::DetectedFramework::OctoberCms));
        let idx = build_index(&state);
        assert!(idx.iter().any(|r| r.label.starts_with("OctoberCMS migrate @ blog.test")));
    }

    #[test]
    fn build_index_does_not_emit_for_wordpress_in_phase_13() {
        // WordPress is not in the Phase 13 quick-action list.
        let mut state = AppState::default();
        state.sites.push(make_site("blog", crate::ui::DetectedFramework::WordPress));
        let idx = build_index(&state);
        assert!(!idx.iter().any(|r| r.label.contains("Magento")));
        assert!(!idx.iter().any(|r| r.label.contains("Symfony cache")));
    }
}
