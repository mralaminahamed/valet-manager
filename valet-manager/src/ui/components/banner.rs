use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use crate::ui::theme::{Colors, with_alpha};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BannerLevel {
    Warning,
    Danger,
    Info,
    Success,
}

#[allow(dead_code)]
pub fn level_color(level: BannerLevel) -> Color32 {
    match level {
        BannerLevel::Warning => Colors::WARNING,
        BannerLevel::Danger => Colors::DANGER,
        BannerLevel::Info => Colors::INFO,
        BannerLevel::Success => Colors::ACCENT,
    }
}

/// Render a full-width alert banner with a 3px left border, tinted bg, icon, and message.
#[allow(dead_code)]
pub fn alert_banner(ui: &mut egui::Ui, level: BannerLevel, icon: &str, message: &str) {
    let color = level_color(level);
    Frame::NONE
        .fill(with_alpha(color, 20))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin {
            left: 12,
            right: 12,
            top: 10,
            bottom: 10,
        })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ui.painter().line_segment(
                [rect.left_top(), rect.left_bottom()],
                Stroke::new(3.0, color),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(16.0).color(color));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(message)
                        .size(13.0)
                        .color(Colors::TEXT_PRIMARY),
                );
            });
        });
}

/// Render a banner with a right-aligned action area, given as a closure.
#[allow(dead_code)]
pub fn alert_banner_with_actions(
    ui: &mut egui::Ui,
    level: BannerLevel,
    icon: &str,
    message: &str,
    right: impl FnOnce(&mut egui::Ui),
) {
    let color = level_color(level);
    Frame::NONE
        .fill(with_alpha(color, 20))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin {
            left: 12,
            right: 12,
            top: 10,
            bottom: 10,
        })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ui.painter().line_segment(
                [rect.left_top(), rect.left_bottom()],
                Stroke::new(3.0, color),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(16.0).color(color));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(message)
                        .size(13.0)
                        .color(Colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn level_color_warning_is_warning() {
        assert_eq!(level_color(BannerLevel::Warning), Colors::WARNING);
    }
    #[test]
    fn level_color_danger_is_danger() {
        assert_eq!(level_color(BannerLevel::Danger), Colors::DANGER);
    }
    #[test]
    fn level_color_info_is_info() {
        assert_eq!(level_color(BannerLevel::Info), Colors::INFO);
    }
    #[test]
    fn level_color_success_is_accent() {
        assert_eq!(level_color(BannerLevel::Success), Colors::ACCENT);
    }
    #[test]
    fn banner_levels_are_distinct() {
        assert_ne!(BannerLevel::Warning, BannerLevel::Danger);
        assert_ne!(BannerLevel::Info, BannerLevel::Success);
    }
}
