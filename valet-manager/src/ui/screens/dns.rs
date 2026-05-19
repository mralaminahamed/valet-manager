use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::{Colors, accent_button, divider, ghost_button};

pub fn render(ui: &mut egui::Ui, state: &mut AppState, cmd_tx: &Sender<AppCommand>) {
    // ── Header ────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("DNS & Routing").size(15.0).color(Colors::TEXT_PRIMARY).strong());
            ui.label(RichText::new("Wildcard DNS, parked directories, and proxy targets")
                .size(12.0).color(Colors::TEXT_SECONDARY));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if accent_button(ui, "+ Add proxy").clicked() {
                state.ui.add_site_modal_open = true;
            }
            ui.add_space(6.0);
            if ghost_button(ui, "↺ Flush DNS").clicked() {
                let _ = cmd_tx.try_send(AppCommand::FlushDns);
            }
        });
    });
    divider(ui);
    ui.add_space(8.0);

    // ── Stat strip ────────────────────────────────────────────────────
    ui.columns(4, |cols| {
        stat_card(&mut cols[0], "TLD",        &format!(".{}", state.tld),             Colors::ACCENT);
        stat_card(&mut cols[1], "Parked dirs", &state.parks.len().to_string(),          Colors::TEXT_PRIMARY);
        stat_card(&mut cols[2], "Proxy rules", &state.proxies.len().to_string(),        Colors::INFO);
        stat_card(&mut cols[3], "Aliases",     &state.dns_aliases.len().to_string(),    Colors::WARNING);
    });
    ui.add_space(12.0);

    // ── Resolver section ─────────────────────────────────────────────
    section_card(ui, "◈", "Resolver", "Dnsmasq · listening on 127.0.0.1:53", |ui| {
        kv_row(ui, "TLD", |ui| {
            let mut tld = state.tld.clone();
            egui::ComboBox::from_id_salt("dns_tld_combo")
                .selected_text(format!(".{}", tld))
                .width(110.0)
                .show_ui(ui, |ui| {
                    for opt in &["test", "local", "dev", "localhost"] {
                        if ui.selectable_value(&mut tld, opt.to_string(), format!(".{}", opt)).changed() {
                            let _ = cmd_tx.try_send(AppCommand::ChangeTld(tld.clone()));
                        }
                    }
                });
        });
        kv_row_text(ui, "Upstream",  "1.1.1.1, 8.8.8.8");
        kv_row_text(ui, "Catch-all", &format!("*.{} → 127.0.0.1", state.tld));
    });
    ui.add_space(8.0);

    // ── Parked directories section ───────────────────────────────────
    let park_meta = format!("{} sites resolved", state.sites.len());
    section_card(ui, "⌂", "Parked directories", &park_meta, |ui| {
        if state.parks.is_empty() {
            ui.label(RichText::new("No directories parked yet.").size(11.5).color(Colors::TEXT_TERTIARY));
        }
        for park in &state.parks {
            let label = park.display().to_string();
            kv_row(ui, &label, |ui| {
                if ghost_button(ui, "✕").clicked() {
                    let _ = cmd_tx.try_send(AppCommand::ForgetDirectory(park.clone()));
                }
            });
        }
        ui.add_space(4.0);
        if ghost_button(ui, "+ Park new directory").clicked() {
            state.ui.add_site_modal_open = true;
        }
    });
    ui.add_space(8.0);

    // ── Proxy rules section ──────────────────────────────────────────
    section_card(ui, "⇆", "Proxy rules", "Reverse proxy to local processes", |ui| {
        if state.proxies.is_empty() {
            ui.label(RichText::new("No proxy rules configured.").size(11.5).color(Colors::TEXT_TERTIARY));
        }
        for proxy in &state.proxies {
            let label = if proxy.secured {
                format!("🔒 {}", proxy.domain)
            } else {
                proxy.domain.clone()
            };
            kv_row(ui, &label, |ui| {
                ui.label(RichText::new("→").size(11.5).color(Colors::TEXT_TERTIARY));
                ui.add_space(4.0);
                ui.label(RichText::new(&proxy.target).size(11.5).color(Colors::TEXT_SECONDARY)
                    .text_style(egui::TextStyle::Monospace));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ghost_button(ui, "✕").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::RemoveProxy(proxy.domain.clone()));
                    }
                });
            });
        }
        ui.add_space(4.0);
        if ghost_button(ui, "+ Add proxy").clicked() {
            state.ui.add_site_modal_open = true;
        }
    });
    ui.add_space(8.0);

    // ── Aliases section ──────────────────────────────────────────────
    section_card(ui, "≡", "Aliases", "Map one domain to another", |ui| {
        if state.dns_aliases.is_empty() {
            ui.label(RichText::new("No aliases configured.").size(11.5).color(Colors::TEXT_TERTIARY));
        }
        for alias in &state.dns_aliases {
            kv_row(ui, &alias.from, |ui| {
                ui.label(RichText::new("→").size(11.5).color(Colors::TEXT_TERTIARY));
                ui.add_space(4.0);
                ui.label(RichText::new(&alias.to).size(11.5).color(Colors::TEXT_SECONDARY)
                    .text_style(egui::TextStyle::Monospace));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ghost_button(ui, "✕").clicked() {
                        let _ = cmd_tx.try_send(AppCommand::RemoveDnsAlias(alias.from.clone()));
                    }
                });
            });
        }
        ui.add_space(4.0);
        if ghost_button(ui, "+ Add alias").clicked() {
            state.ui.add_site_modal_open = true;
        }
    });
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32) {
    Frame::NONE
        .fill(Colors::CARD)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin { left: 12, right: 12, top: 10, bottom: 10 })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(10.0).color(Colors::TEXT_TERTIARY));
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(18.0).color(value_color).strong());
        });
}

fn section_card(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    meta: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    Frame::NONE
        .fill(Colors::CARD)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin { left: 14, right: 14, top: 12, bottom: 12 })
        .stroke(Stroke::new(0.5, Colors::BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(14.0).color(Colors::ACCENT));
                ui.add_space(6.0);
                ui.label(RichText::new(title).size(13.0).color(Colors::TEXT_PRIMARY).strong());
                ui.add_space(8.0);
                ui.label(RichText::new(meta).size(11.0).color(Colors::TEXT_TERTIARY));
            });
            ui.add_space(8.0);
            body(ui);
        });
}

fn kv_row(ui: &mut egui::Ui, key: &str, control: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(28.0);
        ui.label(RichText::new(key).size(12.0).color(Colors::TEXT_SECONDARY)
            .text_style(egui::TextStyle::Monospace));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            control(ui);
        });
    });
    ui.add_space(1.0);
}

fn kv_row_text(ui: &mut egui::Ui, key: &str, value: &str) {
    kv_row(ui, key, |ui| {
        ui.label(RichText::new(value).size(12.0).color(Colors::TEXT_TERTIARY));
    });
}

#[cfg(test)]
mod tests {
    use crate::state::app_state::DnsAlias;

    #[test]
    fn remove_alias_by_from() {
        let mut aliases = vec![
            DnsAlias { from: "shop.test".into(), to: "old-shop.test".into() },
            DnsAlias { from: "v2.api.test".into(), to: "api.test".into() },
        ];
        aliases.retain(|a| a.from != "shop.test");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].from, "v2.api.test");
    }

    #[test]
    fn tld_options_contains_test() {
        let opts = ["test", "local", "dev", "localhost"];
        assert!(opts.contains(&"test"));
    }
}
