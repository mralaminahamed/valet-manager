use crate::php::types::IniType;
use crate::state::app_state::Panel;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppCommand {
    // Phase 1
    RefreshAll,
    RefreshServiceStatus,
    SwitchGlobalPhp(String),
    OpenPanel(Panel),
    OpenCommandPalette,
    // Phase 2 — PHP FPM
    StartPhpFpm(String),
    StopPhpFpm(String),
    RestartPhpFpm(String),
    // Phase 2 — Extensions
    EnableExtension { version: String, extension: String },
    DisableExtension { version: String, extension: String },
    LoadExtensions(String),
    // Phase 2 — INI
    LoadIni { version: String, ini_type: IniType },
    SaveIni { version: String, ini_type: IniType, content: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_debug() {
        let cmd = AppCommand::StartPhpFpm("8.3".to_string());
        assert!(format!("{:?}", cmd).contains("8.3"));
    }

    #[test]
    fn enable_extension_has_fields() {
        let cmd = AppCommand::EnableExtension {
            version: "8.3".to_string(),
            extension: "xdebug".to_string(),
        };
        if let AppCommand::EnableExtension { version, extension } = cmd {
            assert_eq!(version, "8.3");
            assert_eq!(extension, "xdebug");
        } else {
            panic!("wrong variant");
        }
    }
}
