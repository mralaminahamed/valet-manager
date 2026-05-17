use egui::Context;
use tokio::sync::mpsc::Sender;
use crate::commands::AppCommand;
use crate::state::app_state::AppState;

pub fn render(_ctx: &Context, _state: &mut AppState, _cmd_tx: &Sender<AppCommand>) {}
