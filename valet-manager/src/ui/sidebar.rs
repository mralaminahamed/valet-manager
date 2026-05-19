use egui::{RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::{AppState, Screen};
use crate::ui::theme::{Colors, divider, section_label, with_alpha};

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

    // ── Brand ──────────────────────────────────────────────────────────
    ui.add_space(14.0);
    if !icon_only {
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
    } else {
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            if ui.is_rect_visible(icon_rect) {
                let painter = ui.painter();
                painter.rect_filled(icon_rect, egui::CornerRadius::same(7), Colors::ACCENT_DEEP);
                paint_v_mark(painter, icon_rect);
            }
        });
    }
    ui.add_space(12.0);
    divider(ui);
    ui.add_space(6.0);

    // ── Workspace nav ──────────────────────────────────────────────────
    if !icon_only {
        section_label(ui, "workspace");
        ui.add_space(2.0);
    }

    let site_count = state.sites.len();
    let svc_count  = state.services.len();

    screen_nav_item(ui, "◈", "Sites",    Screen::Sites,    &state.ui.active_screen, Some(site_count), cmd_tx, icon_only);
    screen_nav_item(ui, "◻", "Services", Screen::Services, &state.ui.active_screen, Some(svc_count),  cmd_tx, icon_only);
    screen_nav_item(ui, "≡", "Logs",     Screen::Logs,     &state.ui.active_screen, None,             cmd_tx, icon_only);
    screen_nav_item(ui, "⬡", "DNS",      Screen::Dns,      &state.ui.active_screen, None,             cmd_tx, icon_only);
    screen_nav_item(ui, "⚙", "Settings", Screen::Settings, &state.ui.active_screen, None,             cmd_tx, icon_only);

    ui.add_space(8.0);

    // ── Quick actions ──────────────────────────────────────────────────
    if !icon_only {
        section_label(ui, "quick actions");
        ui.add_space(2.0);

        quick_action(ui, "✚", "Park directory", cmd_tx, AppCommand::OpenAddSiteModal);
        quick_action(ui, "⌨", "Open shell",     cmd_tx, AppCommand::OpenShellWindow);
        quick_action_label(ui, "⊟", "phpMyAdmin", ":8082", cmd_tx, AppCommand::OpenPmaWindow);
        quick_action(ui, "✉", "Mailpit inbox",  cmd_tx, AppCommand::OpenMailpitWindow);
    }

    ui.add_space(8.0);

    // ── Footer ─────────────────────────────────────────────────────────
    if !icon_only {
        let avail = ui.available_height();
        ui.add_space((avail - 52.0).max(0.0));

        divider(ui);
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add_space(14.0);
            let all_running = state.services.iter().all(|s| s.status == crate::services::monitor::ServiceStatus::Running);
            let dot_color = if all_running { Colors::ACCENT } else { Colors::WARNING };
            let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                let health = if all_running { "All services healthy" } else { "Some services stopped" };
                ui.label(RichText::new(health).size(11.5).color(Colors::TEXT_PRIMARY).strong());
                ui.label(
                    RichText::new(match &state.valet_uptime {
                        Some(up) => format!("valet · uptime {}", up),
                        None     => format!("valet · {} sites", state.sites.len()),
                    })
                        .size(10.5)
                        .color(Colors::TEXT_TERTIARY),
                );
            });
        });
        ui.add_space(8.0);
    }
}

fn screen_nav_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    screen: Screen,
    active: &Screen,
    count: Option<usize>,
    cmd_tx: &Sender<AppCommand>,
    icon_only: bool,
) {
    let is_active = active == &screen;
    let text_color = if is_active { egui::Color32::WHITE } else { Colors::TEXT_SECONDARY };
    let bg = if is_active { Colors::ACCENT_DARK } else { egui::Color32::TRANSPARENT };

    let desired_size = egui::vec2(ui.available_width(), 28.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        if resp.hovered() || is_active {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(6), bg);
        }
        let inner_rect = rect.shrink2(egui::vec2(10.0, 0.0));
        if icon_only {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                text_color,
            );
        } else {
            ui.painter().text(
                egui::pos2(inner_rect.left(), rect.center().y),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                text_color,
            );
            ui.painter().text(
                egui::pos2(inner_rect.left() + 22.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(12.5),
                text_color,
            );
            if let Some(n) = count {
                if n > 0 {
                    let badge_str = n.to_string();
                    let badge_rect = egui::Rect::from_min_size(
                        egui::pos2(inner_rect.right() - 26.0, rect.center().y - 9.0),
                        egui::vec2(26.0, 18.0),
                    );
                    ui.painter().rect_filled(badge_rect, egui::CornerRadius::same(9), with_alpha(Colors::ACCENT, 22));
                    ui.painter().text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &badge_str,
                        egui::FontId::proportional(10.5),
                        Colors::ACCENT,
                    );
                }
            }
        }
    }

    if resp.clicked() {
        let _ = cmd_tx.try_send(AppCommand::OpenScreen(screen));
    }
}

fn quick_action(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    cmd_tx: &Sender<AppCommand>,
    cmd: AppCommand,
) {
    quick_action_label(ui, icon, label, "", cmd_tx, cmd);
}

fn quick_action_label(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    badge: &str,
    cmd_tx: &Sender<AppCommand>,
    cmd: AppCommand,
) {
    let desired_size = egui::vec2(ui.available_width(), 30.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        if resp.hovered() {
            ui.painter().rect_filled(rect, egui::CornerRadius::same(5), with_alpha(Colors::ACCENT, 12));
        }
        let inner_rect = rect.shrink2(egui::vec2(14.0, 0.0));
        ui.painter().text(
            egui::pos2(inner_rect.left(), rect.center().y),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(12.0),
            Colors::TEXT_TERTIARY,
        );
        ui.painter().text(
            egui::pos2(inner_rect.left() + 20.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            Colors::TEXT_SECONDARY,
        );
        if !badge.is_empty() {
            ui.painter().text(
                egui::pos2(inner_rect.right(), rect.center().y),
                egui::Align2::RIGHT_CENTER,
                badge,
                egui::FontId::monospace(10.0),
                Colors::TEXT_TERTIARY,
            );
        }
    }

    if resp.clicked() {
        let _ = cmd_tx.try_send(cmd);
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
