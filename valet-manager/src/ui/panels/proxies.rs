use egui::{Color32, CornerRadius, Frame, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::nginx::proxy_manager::{HttpProbe, ValetProxy};
use crate::state::app_state::AppState;
use crate::ui::theme::{
    accent_button, divider, ghost_button, with_alpha, Colors,
};

const COL_DOT: f32 = 28.0;
const COL_TARGET: f32 = 160.0;
const COL_TLS: f32 = 60.0;
const COL_HTTP: f32 = 80.0;
const COL_ACTION: f32 = 28.0;
const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 32.0;

// ── Persistent UI state (lives on AppState extension via locals; we use simple
// in-frame storage in egui memory because the spec keeps state minimal).
fn input_state(ui: &mut egui::Ui) -> (String, String, bool) {
    let id_domain = ui.id().with("proxy_add_domain");
    let id_target = ui.id().with("proxy_add_target");
    let id_secure = ui.id().with("proxy_add_secure");
    let domain = ui.data(|d| d.get_temp::<String>(id_domain)).unwrap_or_default();
    let target = ui.data(|d| d.get_temp::<String>(id_target)).unwrap_or_default();
    let secure = ui.data(|d| d.get_temp::<bool>(id_secure)).unwrap_or(false);
    (domain, target, secure)
}

fn save_input_state(ui: &mut egui::Ui, domain: String, target: String, secure: bool) {
    let id_domain = ui.id().with("proxy_add_domain");
    let id_target = ui.id().with("proxy_add_target");
    let id_secure = ui.id().with("proxy_add_secure");
    ui.data_mut(|d| d.insert_temp(id_domain, domain));
    ui.data_mut(|d| d.insert_temp(id_target, target));
    ui.data_mut(|d| d.insert_temp(id_secure, secure));
}

fn status_dot_color(probe: Option<&HttpProbe>) -> Color32 {
    match probe {
        Some(HttpProbe::Ok(_)) => Colors::ACCENT,
        Some(HttpProbe::ClientError(_)) => Colors::WARNING,
        Some(HttpProbe::ServerError(_)) | Some(HttpProbe::Failed) => Colors::DANGER,
        Some(HttpProbe::Pending) | None => Colors::TEXT_TERTIARY,
    }
}

fn http_text(probe: Option<&HttpProbe>) -> (String, Color32) {
    match probe {
        Some(HttpProbe::Ok(s)) => (s.to_string(), Colors::ACCENT),
        Some(HttpProbe::ClientError(s)) => (s.to_string(), Colors::WARNING),
        Some(HttpProbe::ServerError(s)) => (s.to_string(), Colors::DANGER),
        Some(HttpProbe::Failed) => ("ERR".to_string(), Colors::DANGER),
        Some(HttpProbe::Pending) => ("…".to_string(), Colors::TEXT_TERTIARY),
        None => ("—".to_string(), Colors::TEXT_TERTIARY),
    }
}

fn table_header(ui: &mut egui::Ui, flex_width: f32) {
    let header_bg = with_alpha(Color32::BLACK, 38);
    let border = Stroke::new(0.5, Colors::BORDER);
    let avail_w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, HEADER_HEIGHT), egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, header_bg);
    ui.painter().hline(rect.x_range(), rect.bottom(), border);

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let hdr = |text: &str| {
        RichText::new(text)
            .size(10.5)
            .strong()
            .color(Colors::TEXT_TERTIARY)
    };

    child.allocate_ui(egui::vec2(COL_DOT, HEADER_HEIGHT), |_| {});
    child.allocate_ui(egui::vec2(flex_width, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("DOMAIN"));
        });
    });
    child.allocate_ui(egui::vec2(COL_TARGET, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("TARGET"));
        });
    });
    child.allocate_ui(egui::vec2(COL_TLS, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("TLS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_HTTP, HEADER_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("HTTP"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ACTION, HEADER_HEIGHT), |_| {});
}

fn render_row(
    ui: &mut egui::Ui,
    proxy: &ValetProxy,
    status: Option<&HttpProbe>,
    flex_width: f32,
    cmd_tx: &Sender<AppCommand>,
) {
    let avail_w = ui.available_width();
    let (row_rect, resp) = ui.allocate_exact_size(egui::vec2(avail_w, ROW_HEIGHT), egui::Sense::hover());
    if resp.hovered() {
        ui.painter().rect_filled(row_rect, CornerRadius::ZERO, Colors::CARD_HOVER);
    }
    ui.painter().hline(
        row_rect.x_range(),
        row_rect.bottom(),
        Stroke::new(0.5, Colors::BORDER),
    );

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(row_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // Status dot
    child.allocate_ui(egui::vec2(COL_DOT, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(10.0);
            let color = status_dot_color(status);
            let (dot_rect, _) =
                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, color);
        });
    });

    // Domain
    child.allocate_ui(egui::vec2(flex_width, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&proxy.domain)
                    .size(12.5)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
    });

    // Target (monospace 11px)
    child.allocate_ui(egui::vec2(COL_TARGET, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&proxy.target)
                    .size(11.0)
                    .monospace()
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });

    // TLS
    child.allocate_ui(egui::vec2(COL_TLS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if proxy.secured {
                ui.label(
                    RichText::new("TLS")
                        .size(11.0)
                        .strong()
                        .color(Colors::ACCENT),
                );
            } else {
                ui.label(
                    RichText::new("—")
                        .size(11.0)
                        .color(Colors::TEXT_TERTIARY),
                );
            }
        });
    });

    // HTTP code
    child.allocate_ui(egui::vec2(COL_HTTP, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let (text, color) = http_text(status);
            ui.label(
                RichText::new(text)
                    .size(11.0)
                    .monospace()
                    .color(color),
            );
        });
    });

    // Action
    child.allocate_ui(egui::vec2(COL_ACTION, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let btn = ui.add(
                egui::Button::new(
                    RichText::new("⋮")
                        .size(16.0)
                        .color(Colors::TEXT_SECONDARY),
                )
                .min_size(egui::vec2(24.0, 24.0))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE),
            );
            let domain = proxy.domain.clone();
            egui::Popup::menu(&btn).show(|ui| {
                ui.set_min_width(140.0);
                if ui.button("Test").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::TestProxy(domain.clone()));
                }
                ui.separator();
                if ui
                    .add(
                        egui::Button::new(RichText::new("Remove").color(Colors::DANGER))
                            .fill(Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    let _ = cmd_tx.try_send(AppCommand::RemoveProxy(domain.clone()));
                }
            });
        });
    });
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Proxies")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshProxies);
            }
        });
    });
    divider(ui);
    ui.add_space(16.0);

    // ── Add row ──────────────────────────────────────────────────────────
    let (mut domain, mut target, mut secure) = input_state(ui);

    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        // Reserve fixed widths: target 200, secure 80, button 80 + spacing
        let fixed = 200.0 + 80.0 + 80.0 + 16.0 * 3.0;
        let flex_w = (avail - fixed).max(120.0);

        ui.add_sized(
            egui::vec2(flex_w, 28.0),
            egui::TextEdit::singleline(&mut domain).hint_text("domain.test"),
        );
        ui.add_space(8.0);
        ui.add_sized(
            egui::vec2(200.0, 28.0),
            egui::TextEdit::singleline(&mut target).hint_text("http://localhost:3000"),
        );
        ui.add_space(8.0);
        ui.checkbox(&mut secure, "Secure");
        ui.add_space(8.0);
        if accent_button(ui, "+ Add").clicked()
            && !domain.trim().is_empty()
            && !target.trim().is_empty()
        {
            let _ = cmd_tx.try_send(AppCommand::AddProxy {
                domain: domain.trim().to_string(),
                target: target.trim().to_string(),
                secure,
            });
            domain.clear();
            target.clear();
            secure = false;
        }
    });

    save_input_state(ui, domain, target, secure);

    ui.add_space(12.0);

    // ── Table ────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let table_w = ui.available_width() - 22.0;
        let fixed = COL_DOT + COL_TARGET + COL_TLS + COL_HTTP + COL_ACTION;
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
                    .id_salt("proxies_scroll")
                    .show(ui, |ui| {
                        if state.proxies.is_empty() {
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No proxies configured")
                                        .size(12.0)
                                        .color(Colors::TEXT_TERTIARY),
                                );
                            });
                            ui.add_space(20.0);
                        } else {
                            for proxy in &state.proxies {
                                let probe = state.proxy_status.get(&proxy.domain);
                                render_row(ui, proxy, probe, flex_w, cmd_tx);
                            }
                        }
                    });
            });
    });
}
