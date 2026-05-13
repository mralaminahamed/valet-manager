use egui::{Color32, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::{AppState, Panel};
use crate::ui::theme::{Colors, divider, section_label, status_dot, with_alpha};

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    ui.spacing_mut().item_spacing.y = 2.0;

    // ── Logo area ──────────────────────────────────────────────────────
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let (icon_rect, _) =
            ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
        if ui.is_rect_visible(icon_rect) {
            let painter = ui.painter();
            painter.rect_filled(icon_rect, egui::CornerRadius::same(7), Colors::ACCENT_DEEP);
            paint_v_mark(painter, icon_rect);
        }
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Valet Manager")
                    .size(13.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
            let sub = match &state.valet_variant {
                Some(v) => format!("{} · .{}", v.display_name(), state.tld),
                None    => format!("Not detected · .{}", state.tld),
            };
            ui.label(RichText::new(sub).size(10.0).color(Colors::TEXT_TERTIARY));
        });
    });
    ui.add_space(8.0);
    divider(ui);
    ui.add_space(4.0);

    // ── Nav sections ───────────────────────────────────────────────────
    section_label(ui, "management");
    nav_item(ui, "⊞", "Dashboard",      Panel::Dashboard,     &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⌥", "PHP Versions",   Panel::PhpVersions,   &state.ui.active_panel, cmd_tx);
    nav_item(ui, "◻", "PHP Extensions", Panel::PhpExtensions, &state.ui.active_panel, cmd_tx);
    nav_item(ui, "≡", "PHP INI",        Panel::PhpIni,        &state.ui.active_panel, cmd_tx);
    nav_item(ui, "ⓘ", "phpinfo()",      Panel::PhpInfo,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "✓", "PHP Compat",     Panel::PhpCompat,     &state.ui.active_panel, cmd_tx);
    nav_item(ui, "◈", "Sites",          Panel::Sites,         &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⌂", "Parks",          Panel::Parks,         &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⚙", "Nginx",          Panel::Nginx,         &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⇆", "Proxies",        Panel::Proxies,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⬡", "dnsmasq",        Panel::Dnsmasq,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "🔒", "SSL Certs",     Panel::SslCerts,      &state.ui.active_panel, cmd_tx);

    section_label(ui, "development");
    nav_item(ui, "⊟", "Database",       Panel::Database,      &state.ui.active_panel, cmd_tx);
    nav_item(ui, "≣", ".env Editor",    Panel::EnvEditor,     &state.ui.active_panel, cmd_tx);
    nav_item(ui, "▶", "Artisan",        Panel::Artisan,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⏭", "Queue Workers",  Panel::QueueWorkers,  &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⊗", "Xdebug",         Panel::Xdebug,        &state.ui.active_panel, cmd_tx);
    nav_item(ui, "✉", "Mail Catcher",   Panel::MailCatcher,   &state.ui.active_panel, cmd_tx);

    section_label(ui, "tools");
    nav_item(ui, "↗", "Sharing",        Panel::Sharing,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "◇", "Drivers",        Panel::Drivers,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "≡", "Logs",           Panel::Logs,          &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⌛", "History",        Panel::History,       &state.ui.active_panel, cmd_tx);
    nav_item(ui, "⊕", "Diagnostics",    Panel::Diagnostics,   &state.ui.active_panel, cmd_tx);

    // ── Settings + service strip pinned at bottom ────────────────────
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        nav_item(ui, "⚙", "Settings", Panel::Settings, &state.ui.active_panel, cmd_tx);

        divider(ui);
        section_label(ui, "services");

        for svc in state.services.iter().rev() {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                status_dot(ui, &svc.status);
                ui.add_space(4.0);
                let mut label = svc.display_name.clone();
                if svc.name == "mailpit" {
                    if let Some(n) = svc.unread_count.filter(|&n| n > 0) {
                        label.push_str(&format!(" · {}", n));
                    }
                }
                ui.label(RichText::new(&label).size(12.0).color(Colors::TEXT_SECONDARY));
            });
        }
        ui.add_space(4.0);
    });
}

fn nav_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    panel: Panel,
    active: &Panel,
    cmd_tx: &Sender<AppCommand>,
) {
    let is_active = active == &panel;
    let bg = if is_active { Colors::ACCENT_DARK } else { Color32::TRANSPARENT };
    let fg = if is_active { Color32::WHITE } else { Colors::TEXT_SECONDARY };

    let btn = egui::Button::new(
        RichText::new(format!("{} {}", icon, label))
            .size(12.0)
            .color(fg),
    )
    .fill(bg)
    .stroke(Stroke::NONE)
    .corner_radius(4.0)
    .min_size(egui::vec2(188.0, 28.0));

    if ui.add(btn).clicked() {
        let _ = cmd_tx.try_send(AppCommand::OpenPanel(panel));
    }
}

fn paint_v_mark(painter: &egui::Painter, rect: egui::Rect) {
    use egui::epaint::PathShape;

    let ox = rect.left();
    let oy = rect.top();

    // Outer V (white fill)
    let outer = vec![
        egui::pos2(ox + 4.0,  oy + 6.4),
        egui::pos2(ox + 22.4, oy + 6.4),
        egui::pos2(ox + 13.2, oy + 17.1),
    ];
    painter.add(egui::Shape::Path(PathShape::convex_polygon(
        outer,
        Colors::TEXT_PRIMARY,
        Stroke::NONE,
    )));

    // Inner cutout (ACCENT_DEEP fill to punch hole)
    let inner = vec![
        egui::pos2(ox + 6.5,  oy + 7.5),
        egui::pos2(ox + 19.9, oy + 7.5),
        egui::pos2(ox + 13.2, oy + 15.6),
    ];
    painter.add(egui::Shape::Path(PathShape::convex_polygon(
        inner,
        Colors::ACCENT_DEEP,
        Stroke::NONE,
    )));

    // Three accent lines below tip
    let lines: [(f32, u8); 3] = [(4.6, 255), (3.6, 191), (2.6, 128)];
    for (i, (width, alpha)) in lines.iter().enumerate() {
        let y = oy + 18.5 + i as f32 * 1.8;
        let cx = ox + 13.2;
        let color = with_alpha(Colors::ACCENT, *alpha);
        painter.rect_filled(
            egui::Rect::from_center_size(egui::pos2(cx, y), egui::vec2(*width, 0.64)),
            egui::CornerRadius::ZERO,
            color,
        );
    }
}
