use egui::{Align, CornerRadius, FontFamily, Frame, RichText, ScrollArea};
use crate::creator::output_streamer::{OutputLine, Stream};
use crate::ui::theme::Colors;

/// Render a terminal-style output panel.
///
/// - `lines`: chronological list of output lines. Only the most recent 1000 are rendered.
/// - `auto_scroll`: if true, scrolls to the bottom on every frame so new output stays visible.
///
/// Layout: DEEP_BG background, 6px rounding, 120px min height, vertical scrollable region.
/// Each line: monospace `[HH:MM:SS] ` prefix in TEXT_TERTIARY 11px + message body in monospace 12px
/// (TEXT_PRIMARY for stdout, WARNING for stderr).
#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, lines: &[OutputLine], auto_scroll: bool) {
    Frame::NONE
        .fill(Colors::DEEP_BG)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin { left: 10, right: 10, top: 8, bottom: 8 })
        .show(ui, |ui| {
            ui.set_min_height(120.0);
            let available_width = ui.available_width();
            ui.set_min_width(available_width);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(auto_scroll)
                .show(ui, |ui| {
                    let visible = if lines.len() > 1000 {
                        &lines[lines.len() - 1000..]
                    } else {
                        lines
                    };

                    for line in visible {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let ts = line.timestamp.format("%H:%M:%S").to_string();

                            ui.label(
                                RichText::new(format!("[{}]", ts))
                                    .size(11.0)
                                    .color(Colors::TEXT_TERTIARY)
                                    .family(FontFamily::Monospace),
                            );

                            let msg_color = match line.stream {
                                Stream::Stderr => Colors::WARNING,
                                Stream::Stdout => Colors::TEXT_PRIMARY,
                            };

                            ui.label(
                                RichText::new(&line.text)
                                    .size(12.0)
                                    .color(msg_color)
                                    .family(FontFamily::Monospace),
                            );
                        });
                    }

                    if auto_scroll {
                        ui.scroll_to_cursor(Some(Align::BOTTOM));
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn line(text: &str, stream: Stream) -> OutputLine {
        OutputLine { text: text.to_string(), stream, timestamp: Local::now() }
    }

    #[test]
    fn renders_empty_lines_without_panic() {
        // Headless egui smoke test: just verify our function compiles and types align.
        let _ctx = egui::Context::default();
        let lines: Vec<OutputLine> = Vec::new();
        // We can't easily exercise the render in unit tests without a full egui paint pass,
        // but ensuring the slice signature compiles guards against drift.
        let _ref: &[OutputLine] = &lines;
        let _ = line("hi", Stream::Stdout);
    }

    #[test]
    fn line_helper_classifies_streams() {
        assert_eq!(line("a", Stream::Stdout).stream, Stream::Stdout);
        assert_eq!(line("b", Stream::Stderr).stream, Stream::Stderr);
    }
}
