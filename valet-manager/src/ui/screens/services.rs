use egui::RichText;
use tokio::sync::mpsc::Sender;
use crate::commands::AppCommand;
use crate::state::app_state::AppState;
use crate::ui::theme::Colors;

pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>) {
    ui.label(RichText::new("Services — coming in M3").color(Colors::TEXT_TERTIARY));
    let _ = (state, cmd_tx);
}
