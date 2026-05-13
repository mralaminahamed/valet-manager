use crate::state::app_state::Panel;

#[derive(Debug, Clone)]
pub enum AppCommand {
    RefreshAll,
    RefreshServiceStatus,
    SwitchGlobalPhp(String),
    OpenPanel(Panel),
    OpenCommandPalette,
}
