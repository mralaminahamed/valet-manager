use egui::RichText;
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, ghost_button, with_alpha};

pub fn render(ctx: &egui::Context, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    if !state.ui.mailpit_open {
        return;
    }

    let mut open = state.ui.mailpit_open;
    egui::Window::new("Mailpit Inbox")
        .id(egui::Id::new("mailpit_window"))
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .default_size([680.0, 480.0])
        .min_size([400.0, 280.0])
        .show(ctx, |ui| {
            render_mailpit_body(ui, state, cmd_tx);
        });

    state.ui.mailpit_open = open;
}

fn render_mailpit_body(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Mailpit Inbox")
                .size(13.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if accent_button(ui, "↗ Open in browser").clicked() {
                let _ = std::process::Command::new("xdg-open")
                    .arg("http://localhost:8025")
                    .spawn();
            }
            if ghost_button(ui, "↺ Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshMailpit);
            }
        });
    });

    ui.separator();

    if state.mailpit_messages.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No messages captured yet.")
                    .size(12.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            ui.label(
                RichText::new("All outgoing mail is redirected here.")
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for msg in &state.mailpit_messages {
            let bg = if msg.read {
                Colors::CARD
            } else {
                with_alpha(Colors::ACCENT, 10)
            };
            egui::Frame::NONE
                .fill(bg)
                .inner_margin(egui::Margin {
                    left: 10,
                    right: 10,
                    top: 8,
                    bottom: 8,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Unread dot or spacer
                        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 16.0), egui::Sense::hover());
                        if !msg.read {
                            ui.painter().circle_filled(
                                egui::pos2(dot_rect.center().x, dot_rect.center().y),
                                3.5,
                                Colors::ACCENT,
                            );
                        }
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&msg.subject)
                                    .size(12.5)
                                    .color(Colors::TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&msg.from)
                                        .size(11.0)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                                ui.label(
                                    RichText::new("·")
                                        .size(11.0)
                                        .color(Colors::TEXT_TERTIARY),
                                );
                                ui.label(
                                    RichText::new(&msg.received)
                                        .size(11.0)
                                        .color(Colors::TEXT_TERTIARY),
                                );
                            });
                        });
                    });
                });
            ui.separator();
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_mailpit_messages_empty() {
        let json = r#"{"messages":[],"total":0}"#;
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgs = v["messages"].as_array().map(|a| a.len()).unwrap_or(0);
        assert_eq!(msgs, 0);
    }
}
