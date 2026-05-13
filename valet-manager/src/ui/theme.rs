use egui::{Color32, Frame, Margin, Response, RichText, Rounding, Stroke, Visuals};
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
    pub const INFO:           Color32 = Color32::from_rgb(0x37, 0x8A, 0xDD);
}

pub fn apply_dark(ctx: &egui::Context) {
    let mut v = Visuals::dark();

    v.panel_fill        = Colors::SURFACE;
    v.window_fill       = Colors::SURFACE;
    v.extreme_bg_color  = Colors::DEEP_BG;
    v.faint_bg_color    = Colors::CARD;
    v.hyperlink_color   = Colors::ACCENT;
    v.warn_fg_color     = Colors::WARNING;
    v.error_fg_color    = Colors::DANGER;

    v.widgets.noninteractive.bg_fill   = Colors::CARD;
    v.widgets.noninteractive.fg_stroke = Stroke::new(0.5, Colors::TEXT_TERTIARY);
    v.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Colors::BORDER);

    v.widgets.inactive.bg_fill   = Colors::CARD;
    v.widgets.inactive.fg_stroke = Stroke::new(0.5, Colors::TEXT_SECONDARY);
    v.widgets.inactive.bg_stroke = Stroke::new(0.5, Colors::BORDER_MED);

    v.widgets.hovered.bg_fill   = Colors::CARD_HOVER;
    v.widgets.hovered.fg_stroke = Stroke::new(0.5, Colors::TEXT_PRIMARY);
    v.widgets.hovered.bg_stroke = Stroke::new(0.5, Colors::BORDER_MED);

    v.widgets.active.bg_fill   = Colors::ACCENT_DARK;
    v.widgets.active.fg_stroke = Stroke::new(0.5, Color32::WHITE);
    v.widgets.active.bg_stroke = Stroke::new(0.5, Color32::WHITE);

    v.selection.bg_fill = Color32::from_rgba_unmultiplied(
        Colors::ACCENT.r(), Colors::ACCENT.g(), Colors::ACCENT.b(), 40,
    );

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

pub fn framework_badge_colors(fw: &DetectedFramework) -> (Color32, Color32) {
    match fw {
        DetectedFramework::Laravel => (
            Color32::from_rgba_unmultiplied(0xEF, 0x9F, 0x27, 28),
            Color32::from_rgb(0xBA, 0x75, 0x17),
        ),
        DetectedFramework::WordPress | DetectedFramework::Bedrock => (
            Color32::from_rgba_unmultiplied(0x37, 0x8A, 0xDD, 30),
            Color32::from_rgb(0x18, 0x5F, 0xA5),
        ),
        DetectedFramework::Symfony => (
            Color32::from_rgba_unmultiplied(0x5D, 0xCA, 0xA5, 25),
            Color32::from_rgb(0x08, 0x50, 0x41),
        ),
        DetectedFramework::Proxy | DetectedFramework::None => {
            (Colors::CARD, Colors::TEXT_TERTIARY)
        }
        _ => (Colors::CARD, Colors::TEXT_SECONDARY),
    }
}

pub fn accent_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Colors::TEXT_PRIMARY).size(12.0),
        )
        .fill(Colors::ACCENT_DARK)
        .stroke(Stroke::NONE)
        .rounding(4.0),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Colors::TEXT_SECONDARY).size(12.0),
        )
        .fill(Colors::CARD)
        .stroke(Stroke::new(0.5, Colors::BORDER_MED))
        .rounding(4.0),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label.into()).color(Colors::DANGER).size(12.0),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(0.5, Colors::DANGER))
        .rounding(4.0),
    )
}

pub fn status_dot(ui: &mut egui::Ui, status: &ServiceStatus) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().circle_filled(rect.center(), 3.5, status_color(status));
    }
}

pub fn framework_badge(ui: &mut egui::Ui, fw: &DetectedFramework) {
    let (bg, fg) = framework_badge_colors(fw);
    let text = framework_name(fw);
    Frame::none()
        .fill(bg)
        .rounding(Rounding::same(20))
        .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(fg).size(10.0));
        });
}

pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.0)
            .color(Colors::TEXT_TERTIARY),
    );
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

pub fn card_frame() -> Frame {
    Frame::none()
        .fill(Colors::CARD)
        .rounding(Rounding::same(8))
        .inner_margin(Margin { left: 14, right: 14, top: 12, bottom: 12 })
}

fn framework_name(fw: &DetectedFramework) -> &'static str {
    match fw {
        DetectedFramework::Laravel   => "Laravel",
        DetectedFramework::WordPress => "WordPress",
        DetectedFramework::Symfony   => "Symfony",
        DetectedFramework::Bedrock   => "Bedrock",
        DetectedFramework::Proxy     => "Proxy",
        DetectedFramework::None      => "—",
        DetectedFramework::Other(_)  => "Other",
    }
}
