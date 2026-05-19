use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use crate::ui::theme::{Colors, with_alpha, accent_button, ghost_button, danger_button};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub code_block: Option<String>,
    pub confirm_label: String,
    pub cancel_label: String,
    pub is_danger: bool,
    pub open: bool,
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        Self {
            title: String::new(),
            message: String::new(),
            code_block: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            is_danger: false,
            open: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DialogResult { Pending, Confirmed, Cancelled }

/// Renders a modal-style dialog centered in the current UI region.
/// Returns DialogResult on button click.
#[allow(dead_code)]
pub fn render(ctx: &egui::Context, dialog: &mut ConfirmDialog) -> DialogResult {
    if !dialog.open { return DialogResult::Pending; }

    let mut result = DialogResult::Pending;

    // Dim background
    egui::Area::new(egui::Id::new("confirm_dialog_dim"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.input(|i| i.viewport().outer_rect.unwrap_or(i.content_rect()));
            ui.painter().rect_filled(screen, CornerRadius::ZERO, with_alpha(Color32::BLACK, 140));
        });

    // Card
    let card_width = 360.0;
    let card_height = if dialog.code_block.is_some() { 220.0 } else { 160.0 };

    egui::Area::new(egui::Id::new("confirm_dialog"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Colors::CARD)
                .corner_radius(CornerRadius::same(10))
                .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                .inner_margin(Margin { left: 18, right: 18, top: 16, bottom: 16 })
                .show(ui, |ui| {
                    ui.set_min_width(card_width);
                    ui.set_min_height(card_height);

                    ui.label(RichText::new(&dialog.title).size(16.0).color(Colors::TEXT_PRIMARY).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new(&dialog.message).size(13.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(10.0);

                    if let Some(code) = &dialog.code_block {
                        Frame::NONE
                            .fill(Colors::DEEP_BG)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin { left: 10, right: 10, top: 8, bottom: 8 })
                            .show(ui, |ui| {
                                ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                                    ui.label(RichText::new(code).size(12.0).color(Colors::TEXT_PRIMARY).monospace());
                                });
                            });
                        ui.add_space(12.0);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Right-most: confirm button
                        let clicked_confirm = if dialog.is_danger {
                            danger_button(ui, &dialog.confirm_label).clicked()
                        } else {
                            accent_button(ui, &dialog.confirm_label).clicked()
                        };
                        if clicked_confirm { result = DialogResult::Confirmed; }
                        ui.add_space(8.0);
                        if ghost_button(ui, &dialog.cancel_label).clicked() {
                            result = DialogResult::Cancelled;
                        }
                    });
                });
        });

    if result != DialogResult::Pending { dialog.open = false; }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_dialog_is_closed() {
        let d = ConfirmDialog::default();
        assert!(!d.open);
    }
    #[test]
    fn dialog_result_variants_distinct() {
        assert_ne!(DialogResult::Pending, DialogResult::Confirmed);
        assert_ne!(DialogResult::Confirmed, DialogResult::Cancelled);
    }
}
