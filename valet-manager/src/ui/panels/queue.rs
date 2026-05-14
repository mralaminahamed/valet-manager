use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::queue::QueueWorker;
use crate::state::app_state::AppState;
use crate::ui::theme::{
    accent_button, divider, ghost_button, with_alpha, Colors,
};

const COL_QUEUE:      f32 = 90.0;
const COL_CONNECTION: f32 = 90.0;
const COL_STATUS:     f32 = 70.0;
const COL_JOBS:       f32 = 60.0;
const COL_FAILED:     f32 = 60.0;
const COL_ACTION:     f32 = 60.0;
const ROW_HEIGHT:     f32 = 36.0;
const HEADER_H:       f32 = 32.0;

fn status_color(status: &str) -> Color32 {
    match status {
        "active" => Colors::ACCENT,
        "failed" => Colors::DANGER,
        _ => Colors::TEXT_TERTIARY,
    }
}

fn table_header(ui: &mut egui::Ui, flex_width: f32) {
    let header_bg = with_alpha(Color32::BLACK, 38);
    let border = Stroke::new(0.5, Colors::BORDER);
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, HEADER_H), egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, header_bg);
    ui.painter().hline(rect.x_range(), rect.bottom(), border);

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let hdr = |text: &str| {
        RichText::new(text).size(10.5).strong().color(Colors::TEXT_TERTIARY)
    };

    child.add_space(10.0);
    child.allocate_ui(egui::vec2(flex_width, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("SITE"));
        });
    });
    child.allocate_ui(egui::vec2(COL_QUEUE, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("QUEUE"));
        });
    });
    child.allocate_ui(egui::vec2(COL_CONNECTION, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("CONNECTION"));
        });
    });
    child.allocate_ui(egui::vec2(COL_STATUS, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("STATUS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_JOBS, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.label(hdr("JOBS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_FAILED, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.label(hdr("FAILED"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ACTION, HEADER_H), |_| {});
}

fn render_row(
    ui: &mut egui::Ui,
    worker: &QueueWorker,
    flex_width: f32,
    cmd_tx: &Sender<AppCommand>,
) {
    let avail = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(avail, ROW_HEIGHT), egui::Sense::hover());
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, Colors::CARD_HOVER);
    }
    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(0.5, Colors::BORDER),
    );

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // Site (flex)
    child.add_space(10.0);
    child.allocate_ui(egui::vec2(flex_width, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&worker.site)
                    .size(12.5)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });

    // Queue
    child.allocate_ui(egui::vec2(COL_QUEUE, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&worker.queue)
                    .size(11.0)
                    .monospace()
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });

    // Connection
    child.allocate_ui(egui::vec2(COL_CONNECTION, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&worker.connection)
                    .size(11.0)
                    .monospace()
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });

    // Status pill
    let sc = status_color(&worker.status);
    child.allocate_ui(egui::vec2(COL_STATUS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            Frame::NONE
                .fill(with_alpha(sc, 30))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin { left: 7, right: 7, top: 2, bottom: 2 })
                .show(ui, |ui| {
                    ui.label(RichText::new(&worker.status).size(10.5).color(sc));
                });
        });
    });

    // Jobs (right-aligned)
    child.allocate_ui(egui::vec2(COL_JOBS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.label(
                RichText::new(worker.jobs_processed.to_string())
                    .size(11.0)
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });

    // Failed (right-aligned)
    child.allocate_ui(egui::vec2(COL_FAILED, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            let color = if worker.jobs_failed > 0 {
                Colors::DANGER
            } else {
                Colors::TEXT_TERTIARY
            };
            ui.label(
                RichText::new(worker.jobs_failed.to_string())
                    .size(11.0)
                    .color(color),
            );
        });
    });

    // Actions
    child.allocate_ui(egui::vec2(COL_ACTION, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let btn = ui.add(
                egui::Button::new(
                    RichText::new("⋮").size(16.0).color(Colors::TEXT_SECONDARY),
                )
                .min_size(egui::vec2(24.0, 24.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE),
            );
            let id = worker.id.clone();
            egui::Popup::menu(&btn).show(|ui| {
                ui.set_min_width(140.0);
                if ui.button("Start").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::StartQueueWorker(id.clone()));
                }
                if ui.button("Stop").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::StopQueueWorker(id.clone()));
                }
                ui.separator();
                if ui
                    .add(
                        egui::Button::new(RichText::new("Remove").color(Colors::DANGER))
                            .fill(Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    let _ = cmd_tx.try_send(AppCommand::RemoveQueueWorker(id.clone()));
                }
            });
        });
    });
}

fn render_modal(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    let ctx = ui.ctx().clone();

    // Dim
    egui::Area::new(egui::Id::new("queue_modal_dim"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(&ctx, |ui| {
            let r = ctx.input(|i| i.viewport().outer_rect.unwrap_or(i.content_rect()));
            ui.painter().rect_filled(
                r,
                CornerRadius::ZERO,
                with_alpha(Color32::BLACK, 140),
            );
        });

    let mut close = false;
    let mut commit = false;
    egui::Area::new(egui::Id::new("queue_modal_card"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(&ctx, |ui| {
            Frame::NONE
                .fill(Colors::CARD)
                .corner_radius(CornerRadius::same(10))
                .stroke(Stroke::new(0.5, Colors::BORDER_MED))
                .inner_margin(Margin { left: 18, right: 18, top: 16, bottom: 16 })
                .show(ui, |ui| {
                    ui.set_min_width(400.0);
                    ui.set_min_height(260.0);

                    ui.label(
                        RichText::new("Add queue worker")
                            .size(15.0)
                            .strong()
                            .color(Colors::TEXT_PRIMARY),
                    );
                    ui.add_space(10.0);

                    // Site picker
                    ui.label(
                        RichText::new("Site").size(11.0).color(Colors::TEXT_SECONDARY),
                    );
                    let mut new_pick: Option<String> = None;
                    let current = state
                        .queue_add_site
                        .clone()
                        .unwrap_or_else(|| "(select)".to_string());
                    egui::ComboBox::from_id_salt("queue_add_site_combo")
                        .selected_text(current)
                        .width(360.0)
                        .show_ui(ui, |ui| {
                            for site in &state.sites {
                                if ui
                                    .selectable_label(
                                        state.queue_add_site.as_deref() == Some(site.domain.as_str()),
                                        &site.domain,
                                    )
                                    .clicked()
                                {
                                    new_pick = Some(site.domain.clone());
                                }
                            }
                        });
                    if let Some(p) = new_pick {
                        state.queue_add_site = Some(p);
                    }
                    ui.add_space(8.0);

                    // Connection
                    ui.label(
                        RichText::new("Connection")
                            .size(11.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                    ui.add_sized(
                        egui::vec2(360.0, 26.0),
                        egui::TextEdit::singleline(&mut state.queue_add_connection),
                    );
                    ui.add_space(8.0);

                    // Queue
                    ui.label(
                        RichText::new("Queue").size(11.0).color(Colors::TEXT_SECONDARY),
                    );
                    ui.add_sized(
                        egui::vec2(360.0, 26.0),
                        egui::TextEdit::singleline(&mut state.queue_add_queue),
                    );
                    ui.add_space(8.0);

                    ui.checkbox(&mut state.queue_add_start_on_boot, "Start on boot");
                    ui.add_space(14.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_create = state.queue_add_site.is_some();
                        let resp = if can_create {
                            accent_button(ui, "Create")
                        } else {
                            ghost_button(ui, "Create")
                        };
                        if resp.clicked() && can_create {
                            commit = true;
                        }
                        ui.add_space(8.0);
                        if ghost_button(ui, "Cancel").clicked() {
                            close = true;
                        }
                    });
                });
        });

    if commit {
        if let Some(site) = state.queue_add_site.clone() {
            let _ = cmd_tx.try_send(AppCommand::AddQueueWorker {
                site,
                connection: state.queue_add_connection.clone(),
                queue: state.queue_add_queue.clone(),
                start_on_boot: state.queue_add_start_on_boot,
            });
        }
        state.queue_add_modal = false;
    }
    if close {
        state.queue_add_modal = false;
    }
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // Header
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Queue workers")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "Add worker").clicked() {
                state.queue_add_modal = true;
            }
            ui.add_space(6.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshQueueWorkers);
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // Table
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let table_w = ui.available_width() - 22.0;
        let fixed = COL_QUEUE + COL_CONNECTION + COL_STATUS + COL_JOBS + COL_FAILED + COL_ACTION + 10.0;
        let flex_w = (table_w - fixed).max(120.0);

        Frame::NONE
            .fill(Colors::CARD)
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(0.5, Colors::BORDER))
            .show(ui, |ui| {
                ui.set_clip_rect(ui.max_rect());
                ui.set_width(table_w);
                table_header(ui, flex_w);

                egui::ScrollArea::vertical()
                    .id_salt("queue_workers_scroll")
                    .show(ui, |ui| {
                        if state.queue_workers.is_empty() {
                            ui.add_space(28.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No queue workers configured")
                                        .size(13.0)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                            });
                            ui.add_space(28.0);
                        } else {
                            for worker in &state.queue_workers {
                                render_row(ui, worker, flex_w, cmd_tx);
                            }
                        }
                    });
            });
    });

    if state.queue_add_modal {
        render_modal(ui, state, cmd_tx);
    }
}
