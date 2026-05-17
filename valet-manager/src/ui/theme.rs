use egui::{Color32, CornerRadius, Frame, Margin, Response, RichText, Stroke, Visuals};
use crate::services::monitor::ServiceStatus;
use crate::ui::DetectedFramework;

pub struct Colors;

impl Colors {
    pub const DEEP_BG:        Color32 = Color32::from_rgb(0x0E, 0x16, 0x13);
    pub const SURFACE:        Color32 = Color32::from_rgb(0x16, 0x20, 0x19);
    pub const CARD:           Color32 = Color32::from_rgb(0x1C, 0x2A, 0x26);
    pub const CARD_HOVER:     Color32 = Color32::from_rgb(0x20, 0x30, 0x2B);
    // White at ~7% alpha, stored as premultiplied (r*a/255, g*a/255, b*a/255, a)
    pub const BORDER:         Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
    pub const BORDER_MED:     Color32 = Color32::from_rgba_premultiplied(28, 28, 28, 28);
    pub const TEXT_PRIMARY:   Color32 = Color32::from_rgb(0xD8, 0xED, 0xE6);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x6E, 0x94, 0x88);
    pub const TEXT_TERTIARY:  Color32 = Color32::from_rgb(0x3A, 0x55, 0x50);
    pub const ACCENT:         Color32 = Color32::from_rgb(0x5D, 0xCA, 0xA5);
    pub const ACCENT_DARK:    Color32 = Color32::from_rgb(0x0F, 0x6E, 0x56);
    pub const ACCENT_DEEP:    Color32 = Color32::from_rgb(0x04, 0x34, 0x2C);
    pub const DANGER:         Color32 = Color32::from_rgb(0xE2, 0x4B, 0x4A);
    pub const WARNING:        Color32 = Color32::from_rgb(0xEF, 0x9F, 0x27);
    #[allow(dead_code)]
    pub const INFO:           Color32 = Color32::from_rgb(0x37, 0x8A, 0xDD);
    #[allow(dead_code)]
    pub const SUCCESS:        Color32 = Color32::from_rgb(0x5D, 0xCA, 0xA5); // same as ACCENT
    #[allow(dead_code)]
    pub const PURPLE:         Color32 = Color32::from_rgb(0x9B, 0x5F, 0xF5);
    // Close button danger tint: rgba(226,75,74,0.18) = premultiplied ~(41,14,13,46)
    pub const CLOSE_BTN_BG:   Color32 = Color32::from_rgba_premultiplied(41, 14, 13, 46);
}

pub fn apply_dark(ctx: &egui::Context) {
    let mut v = Visuals::dark();

    v.panel_fill           = Colors::SURFACE;
    v.window_fill          = Colors::SURFACE;
    v.extreme_bg_color     = Colors::DEEP_BG;
    v.faint_bg_color       = Colors::CARD;
    v.code_bg_color        = Colors::CARD;
    v.override_text_color  = Some(Colors::TEXT_PRIMARY);
    v.window_stroke        = Stroke::new(0.5, Colors::BORDER);
    v.hyperlink_color      = Colors::ACCENT;
    v.warn_fg_color        = Colors::WARNING;
    v.error_fg_color       = Colors::DANGER;

    v.widgets.noninteractive.bg_fill   = Colors::CARD;
    v.widgets.noninteractive.fg_stroke = Stroke::new(0.5, Colors::TEXT_TERTIARY);
    v.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Colors::BORDER);

    v.widgets.inactive.bg_fill   = Colors::CARD;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Colors::TEXT_SECONDARY);
    v.widgets.inactive.bg_stroke = Stroke::new(0.5, Colors::BORDER_MED);

    v.widgets.hovered.bg_fill   = Colors::CARD_HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Colors::TEXT_PRIMARY);
    v.widgets.hovered.bg_stroke = Stroke::new(0.5, Colors::BORDER_MED);

    v.widgets.active.bg_fill   = Colors::ACCENT_DARK;
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.bg_stroke = Stroke::new(0.5, Color32::WHITE);

    v.widgets.open.bg_fill = Colors::CARD_HOVER;

    v.selection.bg_fill = Color32::from_rgba_unmultiplied(
        Colors::ACCENT.r(), Colors::ACCENT.g(), Colors::ACCENT.b(), 40,
    );
    v.selection.stroke = Stroke::new(1.0, Colors::ACCENT);

    ctx.set_visuals(v);
}

pub fn status_color(status: &ServiceStatus) -> Color32 {
    match status {
        ServiceStatus::Running => Colors::ACCENT,
        ServiceStatus::Stopped => Colors::DANGER,
        ServiceStatus::Failed  => Colors::DANGER,
        ServiceStatus::Unknown => Colors::TEXT_TERTIARY,
    }
}

#[allow(dead_code)]
pub fn framework_badge_colors(fw: &DetectedFramework) -> (Color32, Color32) {
    match fw {
        DetectedFramework::Laravel    => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,30), Color32::from_rgb(0xBA,0x75,0x17)),
        DetectedFramework::Magento    => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,25), Color32::from_rgb(0x85,0x4F,0x0B)),
        DetectedFramework::Joomla     => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,22), Color32::from_rgb(0x85,0x4F,0x0B)),
        DetectedFramework::ExpressionEngine => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,20), Color32::from_rgb(0x63,0x38,0x06)),
        DetectedFramework::WordPress  => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,30), Color32::from_rgb(0x18,0x5F,0xA5)),
        DetectedFramework::Bedrock    => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,22), Color32::from_rgb(0x0C,0x44,0x7C)),
        DetectedFramework::Drupal     => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,35), Color32::from_rgb(0x18,0x5F,0xA5)),
        DetectedFramework::Symfony    => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,20), Color32::from_rgb(0x0C,0x44,0x7C)),
        DetectedFramework::Contao     => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,18), Color32::from_rgb(0x0C,0x44,0x7C)),
        DetectedFramework::Craft      => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,25), Color32::from_rgb(0x08,0x50,0x41)),
        DetectedFramework::Statamic   => (Color32::from_rgba_unmultiplied(0x9B,0x5F,0xF5,33), Color32::from_rgb(0xB5,0x94,0xF5)),
        DetectedFramework::Slim       => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,20), Color32::from_rgb(0x08,0x50,0x41)),
        DetectedFramework::Jigsaw     => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,22), Color32::from_rgb(0x0F,0x6E,0x56)),
        DetectedFramework::Sculpin    => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,18), Color32::from_rgb(0x08,0x50,0x41)),
        DetectedFramework::CakePHP    => (Color32::from_rgba_unmultiplied(0xE2,0x4B,0x4A,25), Color32::from_rgb(0xA3,0x2D,0x2D)),
        DetectedFramework::Kirby      => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,33), Color32::from_rgb(0xCB,0x7F,0x10)),
        DetectedFramework::OctoberCms => (Color32::from_rgba_unmultiplied(0x7F,0x77,0xDD,25), Color32::from_rgb(0x53,0x4A,0xB7)),
        DetectedFramework::Katana     => (Color32::from_rgba_unmultiplied(0x7F,0x77,0xDD,20), Color32::from_rgb(0x3C,0x34,0x89)),
        DetectedFramework::ConcreteCms=> (Color32::from_rgba_unmultiplied(0xD4,0x53,0x7E,22), Color32::from_rgb(0x99,0x35,0x56)),
        DetectedFramework::Zend       => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,25), Color32::from_rgb(0x5F,0x5E,0x5A)),
        DetectedFramework::StaticHtml => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,15), Color32::from_rgb(0x5F,0x5E,0x5A)),
        DetectedFramework::Unknown    => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,10), Color32::from_rgb(0x44,0x44,0x41)),
    }
}

pub fn accent_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Color32::WHITE).size(12.0),
        )
        .fill(Colors::ACCENT_DARK)
        .stroke(Stroke::new(0.5, with_alpha(Colors::ACCENT, 77)))
        .corner_radius(4.0)
        .min_size(egui::vec2(0.0, 28.0)),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Colors::TEXT_SECONDARY).size(12.0),
        )
        .fill(Colors::CARD)
        .stroke(Stroke::new(0.5, Colors::BORDER_MED))
        .corner_radius(4.0)
        .min_size(egui::vec2(0.0, 28.0)),
    )
}

#[allow(dead_code)]
pub fn danger_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Colors::DANGER).size(12.0),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(0.5, Colors::DANGER))
        .corner_radius(4.0)
        .min_size(egui::vec2(0.0, 28.0)),
    )
}

pub fn status_dot(ui: &mut egui::Ui, status: &ServiceStatus) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().circle_filled(rect.center(), 3.5, status_color(status));
    }
}

#[allow(dead_code)]
pub fn framework_badge(ui: &mut egui::Ui, fw: &DetectedFramework) {
    let (bg, fg) = framework_badge_colors(fw);
    let text = framework_display_name(fw);
    Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).size(10.5));
        });
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new(text.to_uppercase())
                .size(10.0)
                .color(Colors::TEXT_TERTIARY),
        );
    });
    ui.add_space(2.0);
}

pub fn divider(ui: &mut egui::Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().hline(
            rect.x_range(),
            rect.center().y,
            Stroke::new(0.5, Colors::BORDER),
        );
    }
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn card_frame() -> Frame {
    Frame::NONE
        .fill(Colors::CARD)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin { left: 14, right: 14, top: 12, bottom: 12 })
}

#[allow(dead_code)]
pub fn framework_display_name(fw: &DetectedFramework) -> &'static str {
    match fw {
        DetectedFramework::Laravel       => "Laravel",
        DetectedFramework::WordPress     => "WordPress",
        DetectedFramework::Bedrock       => "Bedrock",
        DetectedFramework::CakePHP       => "CakePHP",
        DetectedFramework::ConcreteCms   => "Concrete5",
        DetectedFramework::Contao        => "Contao",
        DetectedFramework::Craft         => "Craft CMS",
        DetectedFramework::Drupal        => "Drupal",
        DetectedFramework::ExpressionEngine => "ExpressionEngine",
        DetectedFramework::Jigsaw        => "Jigsaw",
        DetectedFramework::Joomla        => "Joomla",
        DetectedFramework::Katana        => "Katana",
        DetectedFramework::Kirby         => "Kirby",
        DetectedFramework::Magento       => "Magento",
        DetectedFramework::OctoberCms    => "OctoberCMS",
        DetectedFramework::Sculpin       => "Sculpin",
        DetectedFramework::Slim          => "Slim",
        DetectedFramework::Statamic      => "Statamic",
        DetectedFramework::StaticHtml    => "Static HTML",
        DetectedFramework::Symfony       => "Symfony",
        DetectedFramework::Zend          => "Zend/Laminas",
        DetectedFramework::Unknown       => "PHP",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purple_is_distinct_from_accent() {
        assert_ne!(Colors::PURPLE, Colors::ACCENT);
    }

    #[test]
    fn purple_rgb_matches_design_token() {
        assert_eq!(Colors::PURPLE, Color32::from_rgb(0x9B, 0x5F, 0xF5));
    }

    #[test]
    fn with_alpha_sets_alpha_byte() {
        let c = with_alpha(Colors::ACCENT, 100);
        assert_eq!(c.a(), 100);
        // Premultiplied opaque returns same color
        let opaque = with_alpha(Colors::ACCENT, 255);
        assert_eq!(opaque.r(), Colors::ACCENT.r());
        assert_eq!(opaque.g(), Colors::ACCENT.g());
        assert_eq!(opaque.b(), Colors::ACCENT.b());
    }

    #[test]
    fn all_22_variants_have_badge_color() {
        for fw in DetectedFramework::all_variants() {
            let (bg, fg) = framework_badge_colors(&fw);
            assert!(bg.a() > 0, "{:?} bg alpha = 0", fw);
            assert!(fg.a() == 255, "{:?} fg not opaque", fw);
        }
    }

    #[test]
    fn all_22_variants_have_display_name() {
        for fw in DetectedFramework::all_variants() {
            let name = framework_display_name(&fw);
            assert!(!name.is_empty(), "{:?} display name is empty", fw);
        }
    }

    #[test]
    fn all_variants_count_is_22() {
        assert_eq!(DetectedFramework::all_variants().len(), 22);
    }
}
