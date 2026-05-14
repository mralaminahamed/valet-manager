use egui::{Color32, CornerRadius, Margin, RichText, Stroke};
use tokio::sync::mpsc::Sender;

use crate::commands::{AppCommand, AppCreationRequest};
use crate::creator::project_types::{all_project_types, OptionType, ProjectGroup, ProjectOption, ProjectType};
use crate::state::app_state::{AppState, Panel};
use crate::state::creator_state::CreatorStep;
use crate::ui::theme::{
    accent_button, card_frame, danger_button, divider, ghost_button, Colors,
};

// ── Public entry point ──────────────────────────────────────────────────────
#[allow(dead_code)]
pub fn render(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    // ── Panel header (always) ───────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("App Creator")
                    .size(15.0)
                    .color(Colors::TEXT_PRIMARY)
                    .strong(),
            );
        });
    });
    divider(ui);
    ui.add_space(8.0);

    let step = state.creator.step.clone();
    match step {
        CreatorStep::SelectType  => render_step_select_type(ui, state, cmd_tx),
        CreatorStep::Configure   => render_step_configure(ui, state, cmd_tx),
        CreatorStep::PostInstall => render_step_post_install(ui, state, cmd_tx),
        CreatorStep::Progress    => render_step_progress(ui, state, cmd_tx),
        CreatorStep::Complete    => render_step_complete(ui, state, cmd_tx),
        CreatorStep::Error       => render_step_error(ui, state, cmd_tx),
    }
}

// ── Step indicator (steps 1, 2, 3) ──────────────────────────────────────────
#[allow(dead_code)]
fn step_indicator(ui: &mut egui::Ui, current: u8) {
    ui.add_space(12.0);
    let avail_w = ui.available_width();
    let row_h = 36.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail_w, row_h),
        egui::Sense::hover(),
    );

    if !ui.is_rect_visible(rect) {
        ui.add_space(20.0);
        return;
    }

    let painter = ui.painter();
    let circle_r = 10.0;
    let left_pad = 22.0;
    let right_pad = 22.0;
    let n = 5_u8;

    let track_left = rect.left() + left_pad + circle_r;
    let track_right = rect.right() - right_pad - circle_r;
    let track_span = (track_right - track_left).max(1.0);
    let step_gap = track_span / (n as f32 - 1.0);
    let y = rect.center().y;

    // Draw connecting line segments first (under circles)
    for i in 0..(n - 1) {
        let x0 = track_left + step_gap * i as f32 + circle_r;
        let x1 = track_left + step_gap * (i + 1) as f32 - circle_r;
        painter.line_segment(
            [egui::pos2(x0, y), egui::pos2(x1, y)],
            Stroke::new(0.5, Colors::BORDER),
        );
    }

    // Draw circles + numbers
    for i in 0..n {
        let n_idx = i + 1; // 1..=5
        let cx = track_left + step_gap * i as f32;
        let center = egui::pos2(cx, y);
        let (fill, text_color, label) = if n_idx < current {
            (Colors::ACCENT_DEEP, Colors::ACCENT, "✓".to_string())
        } else if n_idx == current {
            (Colors::ACCENT, Colors::ACCENT_DEEP, n_idx.to_string())
        } else {
            (Colors::CARD, Colors::TEXT_TERTIARY, n_idx.to_string())
        };

        painter.circle_filled(center, circle_r, fill);
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.0),
            text_color,
        );
    }

    ui.add_space(20.0);
}

// ── Step 1: SelectType ──────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_select_type(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    step_indicator(ui, 1);

    // Section heading
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.label(
            RichText::new("Choose project type")
                .size(14.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
    });
    ui.add_space(8.0);

    // Group tab row
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.spacing_mut().item_spacing.x = 8.0;

        let tabs: &[(&str, ProjectGroup)] = &[
            ("Laravel",   ProjectGroup::Laravel),
            ("WordPress", ProjectGroup::WordPress),
            ("PHP",       ProjectGroup::Php),
            ("Node",      ProjectGroup::Node),
            ("Static",    ProjectGroup::Static),
        ];

        for (label, group) in tabs {
            let active = state.creator.active_group == *group;
            let resp = if active {
                accent_button(ui, *label)
            } else {
                ghost_button(ui, *label)
            };
            if resp.clicked() {
                state.creator.active_group = *group;
            }
        }
    });
    ui.add_space(16.0);

    // Card grid
    let filtered: Vec<ProjectType> = all_project_types()
        .into_iter()
        .filter(|t| t.group == state.creator.active_group)
        .collect();

    let selected_id = state
        .creator
        .selected_type
        .as_ref()
        .map(|t| t.id.clone());

    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let avail = ui.available_width() - 22.0;
        let gap = 8.0;
        let cols = 3.0;
        let card_w = ((avail - gap * (cols - 1.0)) / cols).max(160.0);
        let card_h = 110.0;

        ui.vertical(|ui| {
            for row_chunk in filtered.chunks(3) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for pt in row_chunk {
                        render_type_card(ui, pt, &selected_id, card_w, card_h, state);
                    }
                });
                ui.add_space(gap);
            }
        });
    });
    ui.add_space(16.0);

    // Tools status row
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.spacing_mut().item_spacing.x = 8.0;
        if let Some(pt) = state.creator.selected_type.clone() {
            for tool in &pt.required_tools {
                ui.label(
                    RichText::new(format!("✓ {:?}", tool))
                        .size(11.0)
                        .color(Colors::TEXT_SECONDARY),
                );
            }
        }
    });

    // Manual-download hint (ExpressionEngine et al.)
    let needs_manual_download = state
        .creator
        .selected_type
        .as_ref()
        .map(|t| t.show_manual_download)
        .unwrap_or(false);
    if needs_manual_download {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(22.0);
            ui.label(
                RichText::new("Manual installation required — visit expressionengine.com/download")
                    .size(11.0)
                    .color(Colors::WARNING),
            );
        });
    }

    // Footer
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(22.0);
            let has_selection = state.creator.selected_type.is_some();
            let can_proceed = has_selection && !needs_manual_download;
            if can_proceed {
                if accent_button(ui, "Next →").clicked() {
                    if let Some(pt) = state.creator.selected_type.clone() {
                        let _ = cmd_tx.try_send(AppCommand::CreatorSelectType {
                            type_id: pt.id.clone(),
                        });
                        // Pre-fill form values with defaults
                        for opt in &pt.options {
                            state.creator
                                .form_values
                                .entry(opt.key.clone())
                                .or_insert_with(|| opt.default_value.clone());
                        }
                        state.creator.step = CreatorStep::Configure;
                        let _ = cmd_tx.try_send(AppCommand::CreatorNextStep);
                    }
                }
            } else {
                let _ = ghost_button(ui, "Next →");
            }
        });
    });
}

#[allow(dead_code)]
fn render_type_card(
    ui: &mut egui::Ui,
    pt: &ProjectType,
    selected_id: &Option<String>,
    card_w: f32,
    card_h: f32,
    state: &mut AppState,
) {
    let is_selected = selected_id.as_ref() == Some(&pt.id);
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(card_w, card_h),
        egui::Sense::click(),
    );

    let (fill, stroke) = if is_selected {
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

        // Node group: 3px-wide left-border strip in INFO color along inside-left.
        // Render BEFORE inner content so content overlays cleanly.
        if pt.group == ProjectGroup::Node {
            let strip_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.min.y),
                egui::pos2(rect.min.x + 3.0, rect.max.y),
            );
            painter.rect_filled(strip_rect, CornerRadius::same(0), Colors::INFO);
        }

        // Inner content
        let inner = rect.shrink2(egui::vec2(12.0, 12.0));

        // Icon box 28x28 in top-left
        let icon_rect = egui::Rect::from_min_size(
            inner.min,
            egui::vec2(28.0, 28.0),
        );
        painter.rect_filled(
            icon_rect,
            CornerRadius::same(6),
            Colors::ACCENT_DEEP,
        );
        let monogram = pt
            .display_name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default();
        painter.text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            monogram,
            egui::FontId::proportional(14.0),
            Colors::ACCENT,
        );

        // Name
        let name_pos = egui::pos2(inner.min.x, icon_rect.max.y + 8.0);
        painter.text(
            name_pos,
            egui::Align2::LEFT_TOP,
            &pt.display_name,
            egui::FontId::proportional(13.0),
            Colors::TEXT_PRIMARY,
        );

        // Description (truncated)
        let desc = if pt.description.len() > 60 {
            format!("{}…", &pt.description[..60])
        } else {
            pt.description.clone()
        };
        let desc_pos = egui::pos2(inner.min.x, name_pos.y + 18.0);
        painter.text(
            desc_pos,
            egui::Align2::LEFT_TOP,
            desc,
            egui::FontId::proportional(11.0),
            Colors::TEXT_SECONDARY,
        );

        // Below-description extensions
        let extra_y = desc_pos.y + 16.0;

        // Node port note
        if pt.group == ProjectGroup::Node {
            if let Some(port) = pt.default_port {
                painter.text(
                    egui::pos2(inner.min.x, extra_y),
                    egui::Align2::LEFT_TOP,
                    format!("Served via proxy → :{}", port),
                    egui::FontId::proportional(10.5),
                    Colors::INFO,
                );
            }
        }

        // Magento duration warning
        if let Some(warning) = &pt.duration_warning {
            painter.text(
                egui::pos2(inner.min.x, extra_y),
                egui::Align2::LEFT_TOP,
                warning,
                egui::FontId::proportional(10.5),
                Colors::WARNING,
            );
        }

        // ExpressionEngine "Download required" pill badge
        if pt.show_manual_download {
            let badge_text = "Download required";
            let font = egui::FontId::proportional(10.0);
            // Measure text width — approximate via Galley measurement.
            let galley = painter.layout_no_wrap(
                badge_text.to_string(),
                font.clone(),
                Colors::WARNING,
            );
            let pad_h = 6.0;
            let pad_v = 2.0;
            let badge_w = galley.size().x + pad_h * 2.0;
            let badge_h = galley.size().y + pad_v * 2.0;
            let badge_rect = egui::Rect::from_min_size(
                egui::pos2(inner.min.x, extra_y),
                egui::vec2(badge_w, badge_h),
            );
            painter.rect_filled(
                badge_rect,
                CornerRadius::same(12),
                crate::ui::theme::with_alpha(Colors::WARNING, 30),
            );
            painter.text(
                badge_rect.left_top() + egui::vec2(pad_h, pad_v),
                egui::Align2::LEFT_TOP,
                badge_text,
                font,
                Colors::WARNING,
            );
        }
    }

    if resp.clicked() {
        state.creator.selected_type = Some(pt.clone());
    }
}

// ── Step 2: Configure ───────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_configure(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    step_indicator(ui, 2);

    let pt = match state.creator.selected_type.clone() {
        Some(p) => p,
        None => {
            ui.label(
                RichText::new("No project type selected")
                    .size(12.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            return;
        }
    };

    // Header row
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.label(
            RichText::new(&pt.display_name)
                .size(14.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(8.0);
        // Inline group label
        ui.label(
            RichText::new(format!("{:?}", pt.group))
                .size(11.0)
                .color(Colors::TEXT_SECONDARY),
        );
    });
    ui.add_space(12.0);

    // Duration warning banner (Magento etc.)
    if let Some(warning) = &pt.duration_warning {
        ui.horizontal(|ui| {
            ui.add_space(22.0);
            let avail = ui.available_width() - 44.0;
            egui::Frame::NONE
                .fill(crate::ui::theme::with_alpha(Colors::WARNING, 30))
                .corner_radius(egui::CornerRadius::same(6))
                .stroke(egui::Stroke::new(0.5, Colors::WARNING))
                .inner_margin(egui::Margin { left: 12, right: 12, top: 8, bottom: 8 })
                .show(ui, |ui| {
                    ui.set_min_width(avail);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⏱")
                                .size(13.0)
                                .color(Colors::WARNING),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(warning)
                                .size(12.0)
                                .color(Colors::WARNING),
                        );
                    });
                });
        });
        ui.add_space(10.0);
    }

    // Form — 2-col grid
    let options = pt.options.clone();
    let tld = state.tld.clone();
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let form_w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(form_w, 0.0), |ui| {
            egui::Grid::new("creator_form")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    let total = options.len();
                    for (i, opt) in options.iter().enumerate() {
                        let is_name = opt.key == "name";
                        form_field(ui, opt, &mut state.creator.form_values, &tld, is_name);
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                    if total % 2 == 1 {
                        ui.label("");
                        ui.end_row();
                    }
                });
        });
    });

    // Footer
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if ghost_button(ui, "← Back").clicked() {
            state.creator.step = CreatorStep::SelectType;
            let _ = cmd_tx.try_send(AppCommand::CreatorStepBack);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(22.0);
            // Validate required fields
            let all_required_filled = options.iter().all(|opt| {
                if !opt.required {
                    return true;
                }
                state
                    .creator
                    .form_values
                    .get(&opt.key)
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false)
            });
            if all_required_filled {
                if accent_button(ui, "Next: Post-install →").clicked() {
                    state.creator.step = CreatorStep::PostInstall;
                    let _ = cmd_tx.try_send(AppCommand::CreatorNextStep);
                }
            } else {
                let _ = ghost_button(ui, "Next: Post-install →");
            }
        });
    });
}

#[allow(dead_code)]
fn form_field(
    ui: &mut egui::Ui,
    opt: &ProjectOption,
    form_values: &mut std::collections::HashMap<String, String>,
    tld: &str,
    is_name: bool,
) {
    ui.vertical(|ui| {
        // Label row
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&opt.label)
                    .size(11.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            if opt.required {
                ui.label(
                    RichText::new(" *")
                        .size(11.0)
                        .color(Colors::DANGER),
                );
            }
        });

        let value = form_values
            .entry(opt.key.clone())
            .or_insert_with(|| opt.default_value.clone());

        match &opt.option_type {
            OptionType::Text => {
                ui.add_sized(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::TextEdit::singleline(value),
                );
            }
            OptionType::Password => {
                ui.add_sized(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::TextEdit::singleline(value).password(true),
                );
            }
            OptionType::Select(opts) => {
                let current = value.clone();
                egui::ComboBox::from_id_salt(format!("creator_field_{}", opt.key))
                    .selected_text(current)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for s in opts {
                            ui.selectable_value(value, s.clone(), s);
                        }
                    });
            }
            OptionType::Toggle => {
                let mut is_on = value == "true";
                if ui.checkbox(&mut is_on, "").changed() {
                    *value = if is_on { "true".to_string() } else { "false".to_string() };
                }
            }
            OptionType::Dir => {
                ui.horizontal(|ui| {
                    let btn_w = 32.0;
                    let edit_w = (ui.available_width() - btn_w - 4.0).max(40.0);
                    ui.add_sized(
                        egui::vec2(edit_w, 28.0),
                        egui::TextEdit::singleline(value),
                    );
                    if ghost_button(ui, "…").clicked() {
                        if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                            *value = picked.to_string_lossy().to_string();
                        }
                    }
                });
            }
        }

        // For `name` field — show domain preview
        if is_name {
            ui.add_space(2.0);
            let preview = if value.trim().is_empty() {
                format!("→ .{}", tld)
            } else {
                format!("→ {}.{}", value, tld)
            };
            ui.label(
                RichText::new(preview)
                    .size(11.0)
                    .color(Colors::ACCENT),
            );
        } else {
            ui.add_space(2.0);
        }
    });
}

// ── Step 3: PostInstall ─────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_post_install(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    step_indicator(ui, 3);

    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.label(
            RichText::new("Post-install options")
                .size(14.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
    });
    ui.add_space(12.0);

    // Available PHP versions
    let php_versions: Vec<String> = if state.php_versions.is_empty() {
        vec![
            "8.1".to_string(),
            "8.2".to_string(),
            "8.3".to_string(),
            "8.4".to_string(),
        ]
    } else {
        state.php_versions.iter().map(|v| v.version.clone()).collect()
    };

    // Link
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.checkbox(&mut state.creator.post_install_link, "Link site to Valet");
    });
    ui.add_space(8.0);

    // Secure
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.checkbox(&mut state.creator.post_install_secure, "Secure with TLS");
    });
    ui.add_space(8.0);

    // Isolate
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.checkbox(&mut state.creator.post_install_isolate, "Isolate PHP version");
        if state.creator.post_install_isolate {
            ui.add_space(6.0);
            let mut current = state
                .creator
                .isolate_php_version
                .clone()
                .unwrap_or_else(|| php_versions.first().cloned().unwrap_or_default());
            egui::ComboBox::from_id_salt("creator_isolate_php")
                .selected_text(current.clone())
                .show_ui(ui, |ui| {
                    for v in &php_versions {
                        ui.selectable_value(&mut current, v.clone(), v);
                    }
                });
            state.creator.isolate_php_version = Some(current);
        }
    });
    ui.add_space(8.0);

    // Open in browser
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.checkbox(
            &mut state.creator.post_install_open_browser,
            "Open in browser after",
        );
    });
    ui.add_space(8.0);

    // Open in editor
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.checkbox(
            &mut state.creator.post_install_open_editor,
            "Open in editor after",
        );
    });

    // Footer
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        if ghost_button(ui, "← Back").clicked() {
            state.creator.step = CreatorStep::Configure;
            let _ = cmd_tx.try_send(AppCommand::CreatorStepBack);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(22.0);
            if accent_button(ui, "✓ Create").clicked() {
                if let Some(pt) = state.creator.selected_type.clone() {
                    let req = AppCreationRequest {
                        type_id: pt.id.clone(),
                        form_values: state.creator.form_values.clone(),
                        post_install_secure: state.creator.post_install_secure,
                        post_install_php_version: if state.creator.post_install_isolate {
                            state.creator.isolate_php_version.clone()
                        } else {
                            None
                        },
                        post_install_open_browser: state.creator.post_install_open_browser,
                    };
                    let _ = cmd_tx.try_send(AppCommand::CreateApp(req));
                    state.creator.is_running = true;
                    state.creator.step = CreatorStep::Progress;
                    let _ = cmd_tx.try_send(AppCommand::CreatorNextStep);
                }
            }
        });
    });
}

// ── Step 4: Progress ────────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_progress(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    let name = state
        .creator
        .form_values
        .get("name")
        .cloned()
        .unwrap_or_default();
    let display_name = state
        .creator
        .selected_type
        .as_ref()
        .map(|p| p.display_name.clone())
        .unwrap_or_default();

    // Header row
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        ui.label(
            RichText::new(&display_name)
                .size(14.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.label(
            RichText::new(" · ")
                .size(14.0)
                .color(Colors::TEXT_TERTIARY),
        );
        ui.label(
            RichText::new(&name)
                .size(14.0)
                .color(Colors::TEXT_SECONDARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(22.0);
            if danger_button(ui, "✕ Cancel").clicked() {
                state.creator.cancel_requested = true;
                let _ = cmd_tx.try_send(AppCommand::CancelCreation);
            }
        });
    });
    ui.add_space(12.0);

    // Progress checklist (placeholder — all pending)
    let steps_labels = [
        "Downloading",
        "Installing dependencies",
        "Configuring",
        "Linking",
        "Securing",
    ];
    for label in steps_labels.iter() {
        ui.horizontal(|ui| {
            ui.add_space(22.0);
            ui.label(
                RichText::new("○")
                    .size(12.0)
                    .color(Colors::TEXT_TERTIARY),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(*label)
                    .size(12.0)
                    .color(Colors::TEXT_SECONDARY),
            );
        });
        ui.add_space(6.0);
    }
    ui.add_space(16.0);

    // Terminal output
    ui.horizontal(|ui| {
        ui.add_space(22.0);
        let w = ui.available_width() - 22.0;
        ui.allocate_ui(egui::vec2(w, 0.0), |ui| {
            crate::ui::components::terminal_output::render(
                ui,
                &state.creator.output_lines,
                true,
            );
        });
    });
}

// ── Step 5: Complete ────────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_complete(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    let name = state
        .creator
        .form_values
        .get("name")
        .cloned()
        .unwrap_or_default();
    let directory = state
        .creator
        .form_values
        .get("directory")
        .cloned()
        .unwrap_or_default();
    let domain = format!("{}.{}", name, state.tld);

    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        // Checkmark circle
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(56.0, 56.0),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.circle_filled(rect.center(), 24.0, Colors::ACCENT_DARK);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::proportional(20.0),
                Colors::ACCENT,
            );
        }
        ui.add_space(12.0);

        ui.label(
            RichText::new("Site created!")
                .size(16.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(6.0);

        // Domain link
        let resp = ui.add(
            egui::Button::new(
                RichText::new(&domain)
                    .size(14.0)
                    .color(Colors::ACCENT),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE),
        );
        if resp.clicked() {
            let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(domain.clone()));
        }
        ui.add_space(4.0);

        // Node.js extensions: second URL + dev server label + systemd indicator
        let is_node = state
            .creator
            .selected_type
            .as_ref()
            .map(|t| t.group == ProjectGroup::Node)
            .unwrap_or(false);
        if is_node {
            let port: u16 = state
                .creator
                .form_values
                .get("port")
                .and_then(|p| p.parse::<u16>().ok())
                .or_else(|| {
                    state
                        .creator
                        .selected_type
                        .as_ref()
                        .and_then(|t| t.default_port)
                })
                .unwrap_or(3000);

            ui.label(
                RichText::new("Dev server (via npm run dev)")
                    .size(10.5)
                    .color(Colors::TEXT_TERTIARY),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(format!("http://localhost:{}", port))
                    .size(13.0)
                    .color(Colors::INFO)
                    .monospace(),
            );
            ui.add_space(4.0);

            let systemd_on = state
                .creator
                .form_values
                .get("with_systemd")
                .map(|v| v == "true")
                .unwrap_or(false);
            if systemd_on {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✓")
                            .size(12.0)
                            .color(Colors::ACCENT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Systemd service created")
                            .size(12.0)
                            .color(Colors::TEXT_SECONDARY),
                    );
                });
                ui.add_space(4.0);
            }
        }

        // Directory path
        ui.label(
            RichText::new(&directory)
                .size(12.0)
                .color(Colors::TEXT_TERTIARY)
                .monospace(),
        );
        ui.add_space(16.0);

        // Actions
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if accent_button(ui, "↗ Open in browser").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenSiteInBrowser(domain.clone()));
            }
            if ghost_button(ui, "Open in editor").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenSiteInEditor(name.clone()));
            }
            if ghost_button(ui, "← New app").clicked() {
                state.creator.step = CreatorStep::SelectType;
                state.creator.selected_type = None;
                state.creator.form_values.clear();
                state.creator.output_lines.clear();
                state.creator.error = None;
            }
        });
    });
}

// ── Step 5b: Error ──────────────────────────────────────────────────────────
#[allow(dead_code)]
fn render_step_error(
    ui: &mut egui::Ui,
    state: &mut AppState,
    cmd_tx: &Sender<AppCommand>,
) {
    let err_msg = state.creator.error.clone().unwrap_or_default();

    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        // X circle
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(56.0, 56.0),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.circle_filled(rect.center(), 24.0, Colors::DANGER);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                egui::FontId::proportional(20.0),
                Colors::DEEP_BG,
            );
        }
        ui.add_space(12.0);

        ui.label(
            RichText::new("Creation failed")
                .size(16.0)
                .color(Colors::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(12.0);

        // Error message card (max 600 wide)
        ui.allocate_ui(egui::vec2(600.0, 0.0), |ui| {
            card_frame().show(ui, |ui| {
                ui.label(
                    RichText::new(&err_msg)
                        .size(12.0)
                        .color(Colors::WARNING)
                        .monospace(),
                );
            });
        });
        ui.add_space(16.0);

        // Action row
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if ghost_button(ui, "← Try again").clicked() {
                state.creator.step = CreatorStep::SelectType;
                state.creator.selected_type = None;
                state.creator.form_values.clear();
                state.creator.output_lines.clear();
                state.creator.error = None;
            }
            if ghost_button(ui, "View logs").clicked() {
                let _ = cmd_tx.try_send(AppCommand::OpenPanel(Panel::Logs));
            }
        });
    });

    // Cosmetic: suppress unused Margin warning if it ever appears
    let _ = Margin::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::creator_state::CreatorState;

    #[test]
    fn creator_state_has_new_wizard_fields() {
        let s = CreatorState::default();
        assert_eq!(s.active_group, ProjectGroup::WordPress);
        assert!(s.post_install_link);
        assert!(s.post_install_secure);
        assert!(!s.post_install_isolate);
        assert!(s.isolate_php_version.is_none());
        assert!(!s.post_install_open_browser);
        assert!(!s.post_install_open_editor);
    }

    #[test]
    fn step_indicator_signature_compiles() {
        // Sanity: confirm step_indicator accepts a u8 between 1..=5.
        // Full egui rendering requires a paint pass; we just verify the
        // function pointer + arg types resolve at compile time.
        let _f: fn(&mut egui::Ui, u8) = step_indicator;
    }
}
