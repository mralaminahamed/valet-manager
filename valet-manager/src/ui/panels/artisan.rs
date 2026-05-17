use egui::{CornerRadius, FontFamily, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::artisan::{self, ArtisanCommand};
use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{
    accent_button, card_frame, divider, ghost_button, section_label, with_alpha, Colors,
};

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Artisan")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });

        ui.add_space(16.0);
        let current = state
            .artisan_site
            .clone()
            .unwrap_or_else(|| "(select a site)".to_string());
        let mut new_pick: Option<String> = None;
        egui::ComboBox::from_id_salt("artisan_site_picker")
            .selected_text(current)
            .width(280.0)
            .show_ui(ui, |ui| {
                for site in &state.sites {
                    if artisan::framework_cli::detect_cli(site).is_some() {
                        if ui
                            .selectable_label(
                                state.artisan_site.as_deref() == Some(site.domain.as_str()),
                                &site.domain,
                            )
                            .clicked()
                        {
                            new_pick = Some(site.domain.clone());
                        }
                    }
                }
            });
        if let Some(pick) = new_pick {
            if state.artisan_site.as_deref() != Some(pick.as_str()) {
                state.artisan_site = Some(pick.clone());
                state.artisan_output.clear();
                let _ = cmd_tx.try_send(AppCommand::LoadArtisanCommands(pick));
            }
        }

        // CLI kind badge — switches based on detect_cli result of the
        // currently-selected site.
        let cli_label_opt: Option<&'static str> = state
            .artisan_site
            .as_ref()
            .and_then(|dom| {
                state
                    .sites
                    .iter()
                    .find(|s| s.domain == *dom)
                    .and_then(|s| artisan::framework_cli::detect_cli(s).map(|c| c.label()))
            })
            .or_else(|| state.artisan_tool.map(|tool| match tool {
                artisan::ArtisanTool::Artisan    => "Artisan",
                artisan::ArtisanTool::BinConsole => "bin/console",
                artisan::ArtisanTool::BinMagento => "bin/magento",
                artisan::ArtisanTool::Drush      => "drush",
            }));
        if let Some(label) = cli_label_opt {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                Frame::NONE
                    .fill(with_alpha(Colors::PURPLE, 30))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin { left: 8, right: 8, top: 2, bottom: 2 })
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(label)
                                .size(10.5)
                                .color(Colors::PURPLE),
                        );
                    });
            });
        }
    });
    divider(ui);
    ui.add_space(8.0);

    if state.artisan_site.is_none() {
        empty_state(ui, "Select a site with artisan/bin-console/bin-magento detected");
        return;
    }

    // ── Command + args + run row ────────────────────────────────────────
    section_label(ui, "Command");
    ui.horizontal(|ui| {
        ui.add_space(22.0);

        let cmd_edit_resp = ui.add_sized(
            egui::vec2(ui.available_width() - 320.0, 28.0),
            egui::TextEdit::singleline(&mut state.artisan_input)
                .font(egui::TextStyle::Monospace)
                .hint_text("e.g. migrate"),
        );

        ui.add_space(6.0);
        ui.add_sized(
            egui::vec2(200.0, 28.0),
            egui::TextEdit::singleline(&mut state.artisan_args)
                .font(egui::TextStyle::Monospace)
                .hint_text("--force --seed"),
        );
        ui.add_space(6.0);

        let run_clicked = accent_button(ui, "▶ Run").clicked();
        if run_clicked {
            if let Some(site) = state.artisan_site.clone() {
                if !state.artisan_input.trim().is_empty() {
                    state.artisan_output.clear();
                    let _ = cmd_tx.try_send(AppCommand::RunArtisan {
                        site,
                        command: state.artisan_input.trim().to_string(),
                        args: state.artisan_args.clone(),
                    });
                }
            }
        }

        // Arrow navigation in autocomplete when input has focus
        if cmd_edit_resp.has_focus() {
            let filtered = artisan::fuzzy_filter(&state.artisan_commands, &state.artisan_input);
            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowDown) && !filtered.is_empty() {
                    state.artisan_autocomplete_selected =
                        (state.artisan_autocomplete_selected + 1) % filtered.len();
                }
                if i.key_pressed(egui::Key::ArrowUp) && !filtered.is_empty() {
                    state.artisan_autocomplete_selected =
                        (state.artisan_autocomplete_selected + filtered.len() - 1)
                            % filtered.len();
                }
                if i.key_pressed(egui::Key::Tab) {
                    if let Some(c) = filtered.get(state.artisan_autocomplete_selected) {
                        state.artisan_input = c.name.clone();
                    }
                }
            });
        }
    });

    // Autocomplete dropdown (below input)
    if !state.artisan_input.trim().is_empty() && !state.artisan_commands.is_empty() {
        let filtered = artisan::fuzzy_filter(&state.artisan_commands, &state.artisan_input);
        if !filtered.is_empty() && !exact_match(&filtered, &state.artisan_input) {
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                let avail = ui.available_width() - 22.0 - 320.0;
                ui.allocate_ui(egui::vec2(avail, 0.0), |ui| {
                    Frame::NONE
                        .fill(Colors::DEEP_BG)
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                        .inner_margin(Margin { left: 0, right: 0, top: 4, bottom: 4 })
                        .show(ui, |ui| {
                            for (i, c) in filtered.iter().enumerate() {
                                let selected = i == state.artisan_autocomplete_selected;
                                let bg = if selected { Colors::CARD_HOVER } else { egui::Color32::TRANSPARENT };
                                Frame::NONE
                                    .fill(bg)
                                    .inner_margin(Margin { left: 12, right: 12, top: 4, bottom: 4 })
                                    .show(ui, |ui| {
                                        let resp = ui
                                            .allocate_response(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
                                        if resp.clicked() {
                                            state.artisan_input = c.name.clone();
                                        }
                                        let mut child = ui.new_child(
                                            egui::UiBuilder::new()
                                                .max_rect(resp.rect)
                                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                                        );
                                        child.label(
                                            RichText::new(&c.name)
                                                .family(FontFamily::Monospace)
                                                .size(12.0)
                                                .color(Colors::TEXT_PRIMARY),
                                        );
                                        if !c.description.is_empty() {
                                            child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(
                                                    RichText::new(&c.description)
                                                        .size(10.5)
                                                        .color(Colors::TEXT_TERTIARY),
                                                );
                                            });
                                        }
                                    });
                            }
                        });
                });
            });
        }
    }

    ui.add_space(10.0);

    // ── Quick commands row ──────────────────────────────────────────────
    if let Some(tool) = state.artisan_tool {
        section_label(ui, "Quick commands");
        ui.horizontal_wrapped(|ui| {
            ui.add_space(22.0);
            for q in artisan::quick_commands(tool) {
                if ghost_button(ui, q).clicked() {
                    state.artisan_input = q.to_string();
                }
            }
        });
        ui.add_space(10.0);
    }

    // ── Output terminal ─────────────────────────────────────────────────
    section_label(ui, "Output");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(avail, 0.0), |ui| {
            card_frame().show(ui, |ui| {
                terminal_output::render(ui, &state.artisan_output, true);
            });
        });
    });
}

fn exact_match(items: &[ArtisanCommand], q: &str) -> bool {
    items.iter().any(|c| c.name == q)
}

fn empty_state(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(msg)
                .size(13.0)
                .color(Colors::TEXT_SECONDARY),
        );
    });
}
