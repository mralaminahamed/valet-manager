use egui::{Color32, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::{AppState, Panel};
use crate::ui::theme::{Colors, divider, section_label, status_dot, with_alpha};

/// Viewport ≥ 1000 px → full 220 px sidebar with labels.
#[allow(dead_code)]
pub fn should_show_full(viewport_width: f32) -> bool {
    viewport_width >= 1000.0
}

/// 800 px ≤ viewport < 1000 px → narrow icon-only rail.
pub fn should_show_icons(viewport_width: f32) -> bool {
    viewport_width >= 800.0 && viewport_width < 1000.0
}

/// Viewport < 800 px → hide sidebar; caller renders a hamburger overlay instead.
pub fn should_hide(viewport_width: f32) -> bool {
    viewport_width < 800.0
}

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>, icon_only: bool) {
    ui.spacing_mut().item_spacing.y = 2.0;

    // ── Brand area ────────────────────────────────────────────────────
    ui.add_space(14.0);
    if icon_only {
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            if ui.is_rect_visible(icon_rect) {
                let painter = ui.painter();
                painter.rect_filled(icon_rect, egui::CornerRadius::same(7), Colors::ACCENT_DEEP);
                painter.rect_stroke(
                    icon_rect,
                    egui::CornerRadius::same(7),
                    egui::Stroke::new(0.5, with_alpha(Colors::ACCENT, 38)),
                    egui::StrokeKind::Inside,
                );
                paint_v_mark(painter, icon_rect);
            }
        });
    } else {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            if ui.is_rect_visible(icon_rect) {
                let painter = ui.painter();
                painter.rect_filled(icon_rect, egui::CornerRadius::same(7), Colors::ACCENT_DEEP);
                painter.rect_stroke(
                    icon_rect,
                    egui::CornerRadius::same(7),
                    egui::Stroke::new(0.5, with_alpha(Colors::ACCENT, 38)),
                    egui::StrokeKind::Inside,
                );
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
                ui.label(
                    RichText::new("v0.1 · dev")
                        .size(10.5)
                        .color(Colors::TEXT_TERTIARY)
                        .text_style(egui::TextStyle::Monospace),
                );
            });
        });
    }
    ui.add_space(12.0);
    divider(ui);
    ui.add_space(4.0);

    // ── Nav sections ───────────────────────────────────────────────────
    if !icon_only { section_label(ui, "management"); }
    nav_item(ui, "⊞", "Dashboard",      Panel::Dashboard,     &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⌥", "PHP Versions",   Panel::PhpVersions,   &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "◻", "PHP Extensions", Panel::PhpExtensions, &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "≡", "PHP INI",        Panel::PhpIni,        &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "ⓘ", "phpinfo()",      Panel::PhpInfo,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "✓", "PHP Compat",     Panel::PhpCompat,     &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "◈", "Sites",          Panel::Sites,         &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⌂", "Parks",          Panel::Parks,         &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⚙", "Nginx",          Panel::Nginx,         &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "✚", "App Creator",   Panel::AppCreator,    &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⇆", "Proxies",        Panel::Proxies,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⬡", "dnsmasq",        Panel::Dnsmasq,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "🔒", "SSL Certs",     Panel::SslCerts,      &state.ui.active_panel, cmd_tx, icon_only);

    if !icon_only { section_label(ui, "development"); }
    nav_item(ui, "⊟", "Database",       Panel::Database,      &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "≣", ".env Editor",    Panel::EnvEditor,     &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "▶", "Artisan",        Panel::Artisan,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⏭", "Queue Workers",  Panel::QueueWorkers,  &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⊗", "Xdebug",         Panel::Xdebug,        &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "✉", "Mail Catcher",   Panel::MailCatcher,   &state.ui.active_panel, cmd_tx, icon_only);

    if !icon_only { section_label(ui, "tools"); }
    nav_item(ui, "↗", "Sharing",        Panel::Sharing,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "◇", "Drivers",        Panel::Drivers,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "≡", "Logs",           Panel::Logs,          &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⌛", "History",        Panel::History,       &state.ui.active_panel, cmd_tx, icon_only);
    nav_item(ui, "⊕", "Diagnostics",    Panel::Diagnostics,   &state.ui.active_panel, cmd_tx, icon_only);

    // ── Settings + service strip pinned at bottom ────────────────────
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        nav_item(ui, "⚙", "Settings", Panel::Settings, &state.ui.active_panel, cmd_tx, icon_only);

        divider(ui);
        if !icon_only {
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
        } else {
            // Icon-only: just status dots
            for svc in state.services.iter().rev() {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    status_dot(ui, &svc.status);
                });
            }
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
    icon_only: bool,
) {
    let is_active = active == &panel;
    let bg = if is_active { Colors::ACCENT_DARK } else { Color32::TRANSPARENT };
    let fg = if is_active { Color32::WHITE } else { Colors::TEXT_SECONDARY };
    let border = if is_active {
        Stroke::new(0.5, with_alpha(Colors::ACCENT, 51))
    } else {
        Stroke::NONE
    };

    let text = if icon_only { icon.to_string() } else { format!("{} {}", icon, label) };
    let (min_w, min_h) = if icon_only { (28.0, 28.0) } else { (204.0, 28.0) };

    let btn = egui::Button::new(
        RichText::new(text)
            .size(12.5)
            .color(fg),
    )
    .fill(bg)
    .stroke(border)
    .corner_radius(4.0)
    .min_size(egui::vec2(min_w, min_h));

    let resp = ui.add(btn);
    let resp = if icon_only { resp.on_hover_text(label) } else { resp };
    if resp.clicked() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_sidebar_at_or_above_1000_width() {
        assert!(should_show_full(1280.0));
        assert!(should_show_full(1000.0));
        assert!(!should_show_full(999.0));
    }

    #[test]
    fn icon_only_between_800_and_1000() {
        assert!(should_show_icons(800.0));
        assert!(should_show_icons(900.0));
        assert!(!should_show_icons(1000.0));
        assert!(!should_show_icons(799.0));
    }

    #[test]
    fn hide_below_800() {
        assert!(should_hide(700.0));
        assert!(should_hide(0.0));
        assert!(!should_hide(800.0));
        assert!(!should_hide(1200.0));
    }

    #[test]
    fn thresholds_are_mutually_exclusive() {
        for w in [400.0_f32, 799.9, 800.0, 999.9, 1000.0, 1280.0, 1920.0] {
            let states = [should_show_full(w), should_show_icons(w), should_hide(w)];
            // Exactly one must be true.
            assert_eq!(states.iter().filter(|b| **b).count(), 1, "width={}", w);
        }
    }
}
