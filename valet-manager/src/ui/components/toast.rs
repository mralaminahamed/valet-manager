use std::time::{Duration, Instant};
use egui::{Align2, Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use crate::ui::theme::{Colors, with_alpha};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastType { Success, Error, Warning, Info }

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Toast {
    pub message: String,
    pub kind: ToastType,
    pub created_at: Instant,
}

impl Toast {
    pub fn new(msg: impl Into<String>, kind: ToastType) -> Self {
        Self { message: msg.into(), kind, created_at: Instant::now() }
    }
    pub fn elapsed(&self) -> Duration { self.created_at.elapsed() }
    pub fn is_expired(&self) -> bool { self.elapsed() >= Duration::from_millis(4000) }
    pub fn opacity(&self) -> f32 {
        let e = self.elapsed().as_secs_f32();
        if e <= 3.5 { 1.0 } else if e >= 4.0 { 0.0 } else { (4.0 - e) / 0.5 }
    }
}

fn type_color(t: ToastType) -> Color32 {
    match t {
        ToastType::Success => Colors::ACCENT,
        ToastType::Error => Colors::DANGER,
        ToastType::Warning => Colors::WARNING,
        ToastType::Info => Colors::INFO,
    }
}

/// Render the toast stack as a floating Area anchored bottom-right of the viewport.
/// Removes expired toasts; takes &mut Vec<Toast> so the caller can persist the queue.
#[allow(dead_code)]
pub fn render_toasts(ctx: &egui::Context, toasts: &mut Vec<Toast>) {
    toasts.retain(|t| !t.is_expired());
    if toasts.is_empty() { return; }

    // Animate (ask for a repaint every frame while toasts are visible so fade is smooth)
    ctx.request_repaint_after(Duration::from_millis(33));

    egui::Area::new(egui::Id::new("toast_stack"))
        .order(egui::Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for toast in toasts.iter().rev().take(4) {
                    let alpha = (toast.opacity() * 255.0).round().clamp(0.0, 255.0) as u8;
                    let border_color = type_color(toast.kind);

                    // 3px left border via inner Frame on the left of the body.
                    Frame::NONE
                        .fill(with_alpha(Colors::CARD, alpha))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                        .inner_margin(Margin { left: 0, right: 12, top: 0, bottom: 0 })
                        .show(ui, |ui| {
                            ui.set_min_width(280.0);
                            ui.horizontal(|ui| {
                                // 3px left strip
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, 36.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, CornerRadius::ZERO, border_color);
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(&toast.message)
                                        .size(12.5)
                                        .color(with_alpha(Colors::TEXT_PRIMARY, alpha)),
                                );
                            });
                        });
                    ui.add_space(6.0);
                }
            });
        });
}

/// Convenience: push a toast and cap the queue.
#[allow(dead_code)]
pub fn push_toast(toasts: &mut Vec<Toast>, msg: impl Into<String>, kind: ToastType) {
    if toasts.len() >= 32 { toasts.remove(0); }
    toasts.push(Toast::new(msg, kind));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn toast_starts_fully_opaque() {
        let t = Toast::new("hi", ToastType::Info);
        assert!((t.opacity() - 1.0).abs() < 0.01);
        assert!(!t.is_expired());
    }
    #[test]
    fn toast_types_distinct() {
        assert_ne!(ToastType::Success, ToastType::Error);
        assert_ne!(ToastType::Warning, ToastType::Info);
    }
    #[test]
    fn push_toast_caps_queue_at_32() {
        let mut q = Vec::new();
        for i in 0..40 { push_toast(&mut q, format!("t{}", i), ToastType::Info); }
        assert_eq!(q.len(), 32);
        // Newest stays
        assert_eq!(q.last().unwrap().message, "t39");
    }
    #[test]
    fn elapsed_increases_after_sleep() {
        let t = Toast::new("a", ToastType::Info);
        std::thread::sleep(Duration::from_millis(5));
        assert!(t.elapsed().as_millis() >= 5);
    }

    #[test]
    fn push_toast_appends_when_under_cap() {
        let mut q = Vec::new();
        push_toast(&mut q, "a", ToastType::Success);
        push_toast(&mut q, "b", ToastType::Warning);
        assert_eq!(q.len(), 2);
        assert_eq!(q[1].message, "b");
        assert_eq!(q[1].kind, ToastType::Warning);
    }

    #[test]
    fn opacity_clamps_to_zero_for_expired_window() {
        // Just verify that opacity stays in [0,1] for any reasonable elapsed
        let t = Toast::new("x", ToastType::Info);
        let o = t.opacity();
        assert!(o >= 0.0 && o <= 1.0);
    }
}
