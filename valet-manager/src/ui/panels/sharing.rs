use egui::{Color32, CornerRadius, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::components::terminal_output;
use crate::ui::theme::{
    accent_button, card_frame, danger_button, divider, ghost_button, section_label, Colors,
};
use crate::valet::sharing::ShareTool;

const TOOLS: &[(ShareTool, &str, &str)] = &[
    (ShareTool::Ngrok, "ngrok", "Tunnel via ngrok.io"),
    (ShareTool::Expose, "expose", "Self-hosted via beyondco/expose"),
    (ShareTool::Cloudflared, "cloudflared", "Cloudflare quick tunnels"),
];

fn render_tool_card(
    ui: &mut egui::Ui,
    tool: ShareTool,
    name: &str,
    desc: &str,
    selected: bool,
) -> bool {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(200.0, 80.0), egui::Sense::click());
    let (fill, stroke) = if selected {
        (Colors::ACCENT_DARK, Stroke::new(1.5, Colors::ACCENT))
    } else if resp.hovered() {
        (Colors::CARD_HOVER, Stroke::new(0.5, Colors::BORDER_MED))
    } else {
        (Colors::CARD, Stroke::new(0.5, Colors::BORDER_MED))
    };

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(8),
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        let inner = rect.shrink2(egui::vec2(12.0, 10.0));
        painter.text(
            inner.min,
            egui::Align2::LEFT_TOP,
            name,
            egui::FontId::proportional(13.0),
            if selected { Color32::WHITE } else { Colors::TEXT_PRIMARY },
        );
        painter.text(
            egui::pos2(inner.min.x, inner.min.y + 22.0),
            egui::Align2::LEFT_TOP,
            desc,
            egui::FontId::proportional(11.0),
            Colors::TEXT_SECONDARY,
        );
    }
    let _ = tool; // captured by caller via name
    resp.clicked()
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Panel header ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Sharing")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshAll);
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── Tunnel tool ─────────────────────────────────────────────────────
    section_label(ui, "Tunnel tool");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.spacing_mut().item_spacing.x = 12.0;
        for (tool, name, desc) in TOOLS.iter() {
            let selected = state.share_tool == *tool;
            if render_tool_card(ui, *tool, name, desc, selected) {
                state.share_tool = *tool;
            }
        }
    });

    ui.add_space(12.0);

    // ── Auth token ──────────────────────────────────────────────────────
    section_label(ui, "Auth token (optional)");
    let tool_label = state.share_tool.label().to_string();
    let token_entry = state
        .share_tokens
        .entry(tool_label.clone())
        .or_insert_with(String::new);
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        card_frame().show(ui, |ui| {
            ui.set_min_width(avail);
            ui.add_sized(
                egui::vec2(ui.available_width(), 28.0),
                egui::TextEdit::singleline(token_entry)
                    .hint_text(format!("{} auth token", tool_label))
                    .password(true),
            );
        });
    });

    ui.add_space(12.0);

    // ── Site to share ──────────────────────────────────────────────────
    section_label(ui, "Site to share");
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let current = state
            .share_site
            .clone()
            .unwrap_or_else(|| "(select a site)".to_string());
        egui::ComboBox::from_id_salt("share_site_picker")
            .selected_text(current)
            .width(300.0)
            .show_ui(ui, |ui| {
                for site in &state.sites {
                    let label = site.domain.clone();
                    let mut selected = state.share_site.as_deref() == Some(label.as_str());
                    if ui.selectable_value(&mut selected, true, &label).clicked() {
                        state.share_site = Some(label);
                    }
                }
            });
    });

    ui.add_space(12.0);

    // ── Share / Stop ───────────────────────────────────────────────────
    let is_active = state.share_active.is_some();
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if is_active {
            if danger_button(ui, "■ Stop").clicked() {
                let _ = cmd_tx.try_send(AppCommand::StopSharing);
            }
        } else if accent_button(ui, "▶ Share").clicked() {
            if let Some(site) = state.share_site.clone() {
                let token = state
                    .share_tokens
                    .get(&tool_label)
                    .cloned()
                    .unwrap_or_default();
                let _ = cmd_tx.try_send(AppCommand::StartSharing {
                    site,
                    tool: state.share_tool,
                    token,
                });
            }
        }
    });

    // ── Public URL banner ───────────────────────────────────────────────
    if let Some(session) = &state.share_active {
        if let Some(url) = &session.public_url {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                let url_clone = url.clone();
                let resp = ui.add(
                    egui::Button::new(
                        RichText::new(url)
                            .size(14.0)
                            .color(Colors::ACCENT)
                            .strong(),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE),
                );
                if resp.clicked() {
                    let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(url_clone));
                }
            });
        }
    }

    ui.add_space(12.0);

    // ── Output box ──────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(w, 280.0), |ui| {
            terminal_output::render(ui, &state.share_output, true);
        });
    });
}
