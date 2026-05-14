use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::ssl::cert_reader::{CertInfo, CertStatus};
use crate::state::app_state::AppState;
use crate::ui::components::banner::{alert_banner_with_actions, BannerLevel};
use crate::ui::theme::{
    accent_button, divider, ghost_button, with_alpha, Colors,
};

const COL_ISSUER:  f32 = 160.0;
const COL_EXPIRES: f32 = 110.0;
const COL_DAYS:    f32 = 60.0;
const COL_STATUS:  f32 = 90.0;
const COL_ACTION:  f32 = 60.0;
const ROW_HEIGHT:  f32 = 36.0;
const HEADER_H:    f32 = 32.0;

fn status_color_for(status: CertStatus) -> Color32 {
    match status {
        CertStatus::Critical | CertStatus::Expired => Colors::DANGER,
        CertStatus::Warning => Colors::WARNING,
        CertStatus::Ok => Colors::ACCENT,
    }
}

fn status_label(status: CertStatus) -> &'static str {
    match status {
        CertStatus::Ok => "OK",
        CertStatus::Warning => "Warning",
        CertStatus::Critical => "Critical",
        CertStatus::Expired => "Expired",
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
            ui.label(hdr("DOMAIN"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ISSUER, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("ISSUER"));
        });
    });
    child.allocate_ui(egui::vec2(COL_EXPIRES, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(hdr("EXPIRES"));
        });
    });
    child.allocate_ui(egui::vec2(COL_DAYS, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(hdr("DAYS"));
            ui.add_space(8.0);
        });
    });
    child.allocate_ui(egui::vec2(COL_STATUS, HEADER_H), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.label(hdr("STATUS"));
        });
    });
    child.allocate_ui(egui::vec2(COL_ACTION, HEADER_H), |_| {});
}

fn render_row(
    ui: &mut egui::Ui,
    cert: &CertInfo,
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

    let sc = status_color_for(cert.status);

    // Domain
    child.add_space(10.0);
    child.allocate_ui(egui::vec2(flex_width, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if cert.status != CertStatus::Expired {
                ui.label(RichText::new("🔒").size(11.0).color(sc));
            }
            ui.label(
                RichText::new(&cert.domain)
                    .size(12.5)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });

    // Issuer
    child.allocate_ui(egui::vec2(COL_ISSUER, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(&cert.issuer)
                    .size(11.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
    });

    // Expires
    child.allocate_ui(egui::vec2(COL_EXPIRES, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(cert.expires_at.format("%b %d %Y").to_string())
                    .size(11.0)
                    .color(Colors::TEXT_PRIMARY),
            );
        });
    });

    // Days remaining (right-aligned)
    child.allocate_ui(egui::vec2(COL_DAYS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new(cert.days_remaining.to_string())
                    .size(11.0)
                    .strong()
                    .color(sc),
            );
        });
    });

    // Status pill
    child.allocate_ui(egui::vec2(COL_STATUS, ROW_HEIGHT), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            Frame::NONE
                .fill(with_alpha(sc, 30))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin { left: 8, right: 8, top: 2, bottom: 2 })
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(status_label(cert.status))
                            .size(10.5)
                            .color(sc),
                    );
                });
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
            let domain = cert.domain.clone();
            egui::Popup::menu(&btn).show(|ui| {
                ui.set_min_width(140.0);
                if ui.button("Refresh").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::RefreshSslCerts);
                }
                ui.separator();
                if ui
                    .add(
                        egui::Button::new(RichText::new("Revoke").color(Colors::DANGER))
                            .fill(Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    let _ = cmd_tx.try_send(AppCommand::RevokeSiteCert(domain.clone()));
                }
            });
        });
    });
}

#[allow(dead_code)]
pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    // Header
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("SSL Certificates")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ghost_button(ui, "Refresh").clicked() {
                let _ = cmd_tx.try_send(AppCommand::RefreshSslCerts);
            }
        });
    });
    divider(ui);
    ui.add_space(12.0);

    // CA trust banner
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(avail, 0.0), |ui| {
            if state.ssl_ca_trusted {
                alert_banner_with_actions(
                    ui,
                    BannerLevel::Success,
                    "✓",
                    "Valet CA trusted",
                    |_ui| {},
                );
            } else {
                let cmd_tx_inner = cmd_tx.clone();
                alert_banner_with_actions(
                    ui,
                    BannerLevel::Warning,
                    "⚠",
                    "Valet CA not trusted",
                    |ui| {
                        if accent_button(ui, "Trust CA").clicked() {
                            let _ = cmd_tx_inner.try_send(AppCommand::TrustValetCa);
                        }
                    },
                );
            }
        });
    });

    ui.add_space(12.0);

    // Table
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let table_w = ui.available_width() - 22.0;
        let fixed = COL_ISSUER + COL_EXPIRES + COL_DAYS + COL_STATUS + COL_ACTION + 10.0;
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
                    .id_salt("ssl_certs_scroll")
                    .show(ui, |ui| {
                        if state.ssl_certs.is_empty() {
                            ui.add_space(28.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No certificates issued yet")
                                        .size(13.0)
                                        .color(Colors::TEXT_SECONDARY),
                                );
                            });
                            ui.add_space(28.0);
                        } else {
                            for cert in &state.ssl_certs {
                                render_row(ui, cert, flex_w, cmd_tx);
                            }
                        }
                    });
            });
    });
}
