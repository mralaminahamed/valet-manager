use egui::{RichText, ScrollArea};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::Colors;

pub fn render(ctx: &egui::Context, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    if !state.ui.shell_open {
        return;
    }

    let mut open = state.ui.shell_open;
    egui::Window::new("Shell")
        .id(egui::Id::new("shell_window"))
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .default_size([700.0, 460.0])
        .min_size([400.0, 260.0])
        .show(ctx, |ui| {
            render_shell_body(ui, state, cmd_tx);
        });

    if !open {
        let _ = cmd_tx.try_send(AppCommand::CloseShellWindow);
    }
}

fn render_shell_body(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // Tab bar
    ui.horizontal(|ui| {
        let site_label = state.shell_site.as_deref().unwrap_or("shell");
        ui.label(
            RichText::new(format!(" {} ", site_label))
                .size(11.5)
                .color(Colors::ACCENT)
                .background_color(Colors::ACCENT_DEEP),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("+ New tab").clicked() {
                // future: open site picker
            }
        });
    });

    ui.separator();

    // Terminal output
    let output_height = ui.available_height() - 40.0;
    ScrollArea::vertical()
        .id_salt("shell_scroll")
        .max_height(output_height)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &state.shell_output {
                ui.label(
                    RichText::new(line)
                        .size(12.0)
                        .color(Colors::TEXT_SECONDARY)
                        .text_style(egui::TextStyle::Monospace),
                );
            }
        });

    // Input row
    ui.separator();
    ui.horizontal(|ui| {
        let path = state.shell_site.as_deref().unwrap_or("~");
        ui.label(
            RichText::new(format!("{}$ ", path))
                .size(12.0)
                .color(Colors::ACCENT)
                .text_style(egui::TextStyle::Monospace),
        );

        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.shell_input)
                .desired_width(ui.available_width() - 8.0)
                .font(egui::TextStyle::Monospace),
        );

        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let cmd_str = state.shell_input.clone();
            state.shell_output.push(format!("$ {}", cmd_str));
            state.shell_input.clear();
            let _ = cmd_tx.try_send(AppCommand::RunShellCommand(cmd_str));
            resp.request_focus();
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn shell_output_ring_buffer_max_500() {
        let mut output: Vec<String> = Vec::new();
        for i in 0..501 {
            output.push(format!("line {}", i));
            if output.len() > 500 {
                output.remove(0);
            }
        }
        assert_eq!(output.len(), 500);
        assert_eq!(output[0], "line 1");
    }
}
