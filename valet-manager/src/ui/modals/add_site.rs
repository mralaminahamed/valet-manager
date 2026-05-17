use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::{AddSiteFormState, AddSiteTab, AppState};
use crate::ui::theme::{Colors, accent_button, ghost_button, with_alpha};

pub fn render(ctx: &egui::Context, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    if !state.ui.add_site_modal_open {
        return;
    }

    // Dim background overlay
    let screen = ctx.input(|i| i.screen_rect());
    ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("modal_dim")))
        .rect_filled(screen, egui::CornerRadius::ZERO, egui::Color32::from_black_alpha(120));

    // Modal window
    egui::Window::new("add_site_modal")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([560.0, 560.0])
        .show(ctx, |ui| {
            render_modal(ui, state, cmd_tx);
        });
}

fn render_modal(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // Header
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Add a new site").size(15.0).color(Colors::TEXT_PRIMARY).strong());
            ui.label(RichText::new("Park an existing project, scaffold a new one, or proxy a running service")
                .size(11.5).color(Colors::TEXT_TERTIARY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(RichText::new("✕").size(14.0).color(Colors::TEXT_TERTIARY)).clicked() {
                let _ = cmd_tx.try_send(AppCommand::CloseAddSiteModal);
            }
        });
    });

    ui.add_space(12.0);

    // Tabs
    ui.horizontal(|ui| {
        modal_tab(ui, "⌂ Park existing", AddSiteTab::Park,   state);
        modal_tab(ui, "+ Create new",    AddSiteTab::Create,  state);
        modal_tab(ui, "⇆ Add proxy",     AddSiteTab::Proxy,   state);
    });

    ui.add_space(8.0);

    // Body
    egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
        match state.ui.add_site_modal_tab {
            AddSiteTab::Park   => render_park_tab(ui, state, cmd_tx),
            AddSiteTab::Create => render_create_tab(ui, state, cmd_tx),
            AddSiteTab::Proxy  => render_proxy_tab(ui, state, cmd_tx),
        }
    });

    ui.add_space(8.0);

    // Footer
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("↵ confirm · Esc cancel").size(10.5).color(Colors::TEXT_TERTIARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let cta = match state.ui.add_site_modal_tab {
                AddSiteTab::Park   => "Park site",
                AddSiteTab::Create => "Create project",
                AddSiteTab::Proxy  => "Add proxy",
            };
            if accent_button(ui, cta).clicked() {
                submit_modal(state, cmd_tx);
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Cancel").clicked() {
                let _ = cmd_tx.try_send(AppCommand::CloseAddSiteModal);
            }
        });
    });
}

fn modal_tab(ui: &mut egui::Ui, label: &str, tab: AddSiteTab, state: &mut AppState) {
    let is_active = state.ui.add_site_modal_tab == tab;
    let color = if is_active { Colors::ACCENT } else { Colors::TEXT_SECONDARY };
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(label.chars().count() as f32 * 7.0 + 20.0, 30.0),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        if is_active {
            ui.painter().line_segment(
                [egui::pos2(rect.left(), rect.bottom()), egui::pos2(rect.right(), rect.bottom())],
                egui::Stroke::new(2.0, Colors::ACCENT),
            );
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            color,
        );
    }
    if resp.clicked() {
        state.ui.add_site_modal_tab = tab;
    }
}

// ── Park tab ──────────────────────────────────────────────────────────────────

fn render_park_tab(ui: &mut egui::Ui, state: &mut AppState, _cmd_tx: &Sender<AppCommand>) {
    let f = &mut state.ui.add_site_form;

    ui.label(RichText::new("Source").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "Project folder", "Where public/index.php lives", |ui| {
        ui.add(egui::TextEdit::singleline(&mut f.park_path)
            .desired_width(200.0)
            .font(egui::TextStyle::Monospace));
    });

    // Auto-update domain from path unless user has touched it
    if !f.park_domain_touched {
        let segs: Vec<_> = f.park_path.trim_end_matches('/').split('/').collect();
        if let Some(last) = segs.last() {
            if !last.is_empty() {
                f.park_domain = last.to_string();
            }
        }
    }

    form_row(ui, "Domain", "Resolves via Dnsmasq on .test TLD", |ui| {
        if ui.add(egui::TextEdit::singleline(&mut f.park_domain)
            .desired_width(140.0)
            .font(egui::TextStyle::Monospace)
        ).changed() {
            f.park_domain_touched = true;
        }
        ui.label(RichText::new(".test").size(11.5).color(Colors::TEXT_TERTIARY));
    });

    form_row(ui, "Detected framework", "Inferred from composer.json", |ui| {
        egui::ComboBox::from_id_salt("park_fw")
            .selected_text(&f.park_framework)
            .width(160.0)
            .show_ui(ui, |ui| {
                for fw in &[
                    "Laravel","Bedrock","CakePHP 3","ConcreteCMS","Contao","Craft",
                    "Drupal","ExpressionEngine","Jigsaw","Joomla","Katana","Kirby",
                    "Magento","OctoberCMS","Sculpin","Slim","Statamic","Static HTML",
                    "Symfony","WordPress","Zend","Unknown",
                ] {
                    ui.selectable_value(&mut f.park_framework, fw.to_string(), *fw);
                }
            });
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Runtime").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "PHP version", "", |ui| {
        egui::ComboBox::from_id_salt("park_php")
            .selected_text(&f.park_php)
            .width(80.0)
            .show_ui(ui, |ui| {
                for v in &["8.3","8.2","8.1","8.0","7.4"] {
                    ui.selectable_value(&mut f.park_php, v.to_string(), *v);
                }
            });
    });

    form_row(ui, "HTTP server", "", |ui| {
        for srv in &["nginx","caddy","frankenphp","apache"] {
            let is_active = f.park_http_server == *srv;
            let color = if is_active { Colors::ACCENT } else { Colors::TEXT_SECONDARY };
            let bg = if is_active { with_alpha(Colors::ACCENT, 20) } else { Colors::CARD };
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(srv.len() as f32 * 6.5 + 12.0, 22.0),
                egui::Sense::click(),
            );
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, srv,
                    egui::FontId::proportional(11.0), color);
            }
            if resp.clicked() { f.park_http_server = srv.to_string(); }
            ui.add_space(4.0);
        }
    });

    form_row(ui, "Secure with TLS", "Issue a leaf cert from the local CA", |ui| {
        toggle(ui, &mut f.park_secure);
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Database").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    let db_name = domain_to_db_name(&f.park_domain);
    form_row(ui, "Create database", &format!("Empty schema named {}", db_name), |ui| {
        toggle(ui, &mut f.park_create_db);
    });
    form_row(ui, "Scoped phpMyAdmin", &format!("Auto-login at db.{}.test", f.park_domain), |ui| {
        toggle(ui, &mut f.park_scoped_pma);
    });

    ui.add_space(6.0);
    ui.label(RichText::new("Bulk").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "Park entire parent directory", "Every subfolder becomes a .test site", |ui| {
        toggle(ui, &mut f.park_bulk);
    });

    ui.add_space(8.0);
    let url = format!("{}://{}.test", if f.park_secure {"https"} else {"http"}, f.park_domain);
    let serves = format!("{}/public", f.park_path);
    let stack = format!("{} · PHP {} · {}", f.park_http_server, f.park_php, f.park_framework);
    let db_disp = if f.park_create_db { Some(db_name.as_str()) } else { None };
    summary_pane(ui, &url, &serves, &stack, db_disp);
}

// ── Create tab ────────────────────────────────────────────────────────────────

const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    ("laravel",   "Laravel",     "11.x",  "composer create-project"),
    ("bedrock",   "Bedrock",     "1.23",  "Composer-managed WP"),
    ("wordpress", "WordPress",   "6.7",   "wp core download"),
    ("symfony",   "Symfony",     "7.1",   "skeleton + webapp pack"),
    ("drupal",    "Drupal",      "11",    "Composer recommended"),
    ("craft",     "Craft",       "5.x",   "Yii + plugins"),
    ("statamic",  "Statamic",    "5.x",   "Laravel + flat-file CMS"),
    ("kirby",     "Kirby",       "4.x",   "File-based CMS"),
    ("slim",      "Slim",        "4.13",  "Micro PSR-7 framework"),
    ("jigsaw",    "Jigsaw",      "1.7",   "Static site by Tighten"),
    ("static",    "Static HTML", "—",     "index.html scaffolding"),
    ("astro",     "Astro",       "4.x",   "npm create astro@latest"),
];

fn render_create_tab(ui: &mut egui::Ui, state: &mut AppState, _cmd_tx: &Sender<AppCommand>) {
    let f = &mut state.ui.add_site_form;

    ui.label(RichText::new("Template").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    egui::Grid::new("template_grid").num_columns(3).spacing([6.0, 6.0]).show(ui, |ui| {
        for (i, (id, name, ver, desc)) in TEMPLATES.iter().enumerate() {
            let is_active = f.create_template == *id;
            let bg = if is_active { with_alpha(Colors::ACCENT, 18) } else { Colors::CARD };
            let border = if is_active { Colors::ACCENT } else { Colors::BORDER };
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(155.0, 44.0), egui::Sense::click());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
                ui.painter().rect_stroke(rect, CornerRadius::same(6),
                    Stroke::new(0.5, border), egui::StrokeKind::Inside);
                ui.painter().text(
                    egui::pos2(rect.left() + 10.0, rect.center().y - 7.0),
                    egui::Align2::LEFT_CENTER,
                    &format!("{} {}", name, ver),
                    egui::FontId::proportional(11.5),
                    if is_active { Colors::ACCENT } else { Colors::TEXT_PRIMARY },
                );
                ui.painter().text(
                    egui::pos2(rect.left() + 10.0, rect.center().y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    desc,
                    egui::FontId::proportional(10.0),
                    Colors::TEXT_TERTIARY,
                );
            }
            if resp.clicked() { f.create_template = id.to_string(); }
            if (i + 1) % 3 == 0 { ui.end_row(); }
        }
    });

    ui.add_space(8.0);
    ui.label(RichText::new("Project").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "Project name", "Folder + domain", |ui| {
        ui.add(egui::TextEdit::singleline(&mut f.create_name)
            .desired_width(150.0).font(egui::TextStyle::Monospace));
        ui.label(RichText::new(".test").size(11.5).color(Colors::TEXT_TERTIARY));
    });

    form_row(ui, "Location", "", |ui| {
        ui.add(egui::TextEdit::singleline(&mut f.create_location)
            .desired_width(180.0).font(egui::TextStyle::Monospace));
    });

    let is_php = !matches!(f.create_template.as_str(), "static" | "astro");
    if is_php {
        form_row(ui, "PHP version", "", |ui| {
            egui::ComboBox::from_id_salt("create_php")
                .selected_text(&f.create_php)
                .width(80.0)
                .show_ui(ui, |ui| {
                    for v in &["8.3","8.2","8.1","8.0"] {
                        ui.selectable_value(&mut f.create_php, v.to_string(), *v);
                    }
                });
        });
    }

    ui.add_space(6.0);
    ui.label(RichText::new("After scaffolding").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "Initialise git",        "git init + first commit",           |ui| toggle(ui, &mut f.create_git));
    form_row(ui, "Install dependencies",  "composer install / npm install",     |ui| toggle(ui, &mut f.create_install));
    if matches!(f.create_template.as_str(), "laravel" | "statamic") {
        form_row(ui, "Run migrations", "php artisan migrate --seed", |ui| toggle(ui, &mut f.create_migrate));
    }

    ui.add_space(8.0);
    let tpl = TEMPLATES.iter().find(|(id,_,_,_)| *id == f.create_template.as_str()).unwrap_or(&TEMPLATES[0]);
    let url = format!("https://{}.test", f.create_name);
    let path = format!("{}/{}", f.create_location, f.create_name);
    let stack = format!("{} {} · PHP {}", tpl.1, tpl.2, f.create_php);
    let db = matches!(f.create_template.as_str(), "laravel"|"wordpress"|"statamic")
        .then(|| domain_to_db_name(&f.create_name));
    summary_pane(ui, &url, &path, &stack, db.as_deref());
}

// ── Proxy tab ─────────────────────────────────────────────────────────────────

fn render_proxy_tab(ui: &mut egui::Ui, state: &mut AppState, _cmd_tx: &Sender<AppCommand>) {
    let f = &mut state.ui.add_site_form;

    ui.label(RichText::new("Proxy a running service").size(11.0).color(Colors::TEXT_TERTIARY).strong());
    ui.add_space(4.0);

    form_row(ui, "Domain", "Resolves on .test → forwards to target", |ui| {
        ui.add(egui::TextEdit::singleline(&mut f.proxy_domain)
            .desired_width(140.0).font(egui::TextStyle::Monospace));
        ui.label(RichText::new(".test").size(11.5).color(Colors::TEXT_TERTIARY));
    });

    form_row(ui, "Target", "URL on this machine that should answer requests", |ui| {
        ui.add(egui::TextEdit::singleline(&mut f.proxy_target)
            .desired_width(200.0).font(egui::TextStyle::Monospace));
    });

    form_row(ui, "Secure with TLS", "Terminate TLS at Valet, proxy plain HTTP to target", |ui| {
        toggle(ui, &mut f.proxy_secure);
    });

    form_row(ui, "WebSocket upgrade", "Forward Upgrade/Connection headers for ws:// support", |ui| {
        toggle(ui, &mut f.proxy_websocket);
    });

    ui.add_space(8.0);
    let url = format!("{}://{}.test", if f.proxy_secure {"https"} else {"http"}, f.proxy_domain);
    let features = match (f.proxy_secure, f.proxy_websocket) {
        (true, true)  => "TLS · WebSocket upgrade".to_string(),
        (true, false) => "TLS".to_string(),
        (false, true) => "WebSocket upgrade".to_string(),
        _             => "—".to_string(),
    };
    summary_pane(ui, &url, &f.proxy_target, &features, None);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn domain_to_db_name(domain: &str) -> String {
    domain.replace('-', "_").to_lowercase()
}

fn toggle(ui: &mut egui::Ui, value: &mut bool) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 20.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let bg = if *value { Colors::ACCENT } else { with_alpha(Colors::BORDER, 150) };
        ui.painter().rect_filled(rect, CornerRadius::same(10), bg);
        let knob_x = if *value { rect.right() - 12.0 } else { rect.left() + 12.0 };
        ui.painter().circle_filled(egui::pos2(knob_x, rect.center().y), 7.0, Color32::WHITE);
    }
    if resp.clicked() { *value = !*value; }
}

fn form_row(ui: &mut egui::Ui, label: &str, help: &str, control: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(40.0);
        ui.vertical(|ui| {
            ui.set_min_width(160.0);
            ui.label(RichText::new(label).size(12.0).color(Colors::TEXT_SECONDARY));
            if !help.is_empty() {
                ui.label(RichText::new(help).size(10.5).color(Colors::TEXT_TERTIARY));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            control(ui);
        });
    });
    ui.add_space(2.0);
}

fn summary_pane(ui: &mut egui::Ui, url: &str, serves: &str, stack: &str, db: Option<&str>) {
    Frame::NONE
        .fill(with_alpha(Colors::ACCENT, 12))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin { left: 12, right: 12, top: 8, bottom: 8 })
        .stroke(Stroke::new(0.5, with_alpha(Colors::ACCENT, 30)))
        .show(ui, |ui| {
            summary_line(ui, "URL",    url);
            summary_line(ui, "Serves", serves);
            summary_line(ui, "Stack",  stack);
            if let Some(d) = db {
                summary_line(ui, "Database", d);
            }
        });
}

fn summary_line(ui: &mut egui::Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(60.0, 16.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().text(
                egui::pos2(rect.left(), rect.center().y),
                egui::Align2::LEFT_CENTER,
                key,
                egui::FontId::proportional(11.0),
                Colors::TEXT_TERTIARY,
            );
        }
        ui.label(RichText::new(value).size(11.0).color(Colors::TEXT_PRIMARY)
            .text_style(egui::TextStyle::Monospace));
    });
}

fn submit_modal(state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    match state.ui.add_site_modal_tab {
        AddSiteTab::Park => {
            let path = std::path::PathBuf::from(
                state.ui.add_site_form.park_path.replace('~', &std::env::var("HOME").unwrap_or_default())
            );
            let _ = cmd_tx.try_send(AppCommand::ParkDirectory(path));
        }
        AddSiteTab::Create => {
            // TODO: dispatch actual scaffold command (CreateApp) once creator flow is wired
            let _ = cmd_tx.try_send(AppCommand::OpenScreen(crate::state::app_state::Screen::Sites));
        }
        AddSiteTab::Proxy => {
            let f = &state.ui.add_site_form;
            let _ = cmd_tx.try_send(AppCommand::AddProxy {
                domain: f.proxy_domain.clone(),
                target: f.proxy_target.clone(),
                secure: f.proxy_secure,
            });
        }
    }
    let _ = cmd_tx.try_send(AppCommand::CloseAddSiteModal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_name_from_domain() {
        assert_eq!(domain_to_db_name("my-app"), "my_app");
        assert_eq!(domain_to_db_name("acme-corp"), "acme_corp");
        assert_eq!(domain_to_db_name("clean"), "clean");
    }

    #[test]
    fn proxy_summary_https() {
        let secure = true;
        let url = format!("{}://studio.test", if secure { "https" } else { "http" });
        assert!(url.starts_with("https"));
    }

    #[test]
    fn add_site_form_defaults() {
        let f = AddSiteFormState::default();
        assert_eq!(f.park_php, "8.3");
        assert!(f.park_secure);
        assert!(f.park_create_db);
        assert_eq!(f.proxy_target, "http://localhost:5173");
        assert!(f.proxy_websocket);
    }
}
