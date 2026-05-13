use crate::state::app_state::Panel;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppCommand {
    RefreshAll,
    RefreshServiceStatus,
    SwitchGlobalPhp(String),
    OpenPanel(Panel),
    OpenCommandPalette,
}
