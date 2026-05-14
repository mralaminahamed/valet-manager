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
    // Phase 2 — Site isolation
    IsolateSite { site: String, version: String },
    UnisolateSite(String),
    // Phase 2 — PHP version management
    InstallPhpVersion(String),
    RemovePhpVersion(String),
    // Phase 3 — Sites
    RefreshSites,
    ParkDirectory(std::path::PathBuf),
    ForgetDirectory(std::path::PathBuf),
    LinkSite { name: String, path: std::path::PathBuf },
    UnlinkSite(String),
    SecureSite(String),
    UnsecureSite(String),
    ToggleFavoriteSite(String),
    OpenSiteInBrowser(String),
    OpenSiteInEditor(String),
    // Phase 3 — Nginx
    SaveNginxConfig { site: String, content: String },
    ReloadNginx,
    // Phase 4 — App Creator
    CreateApp(AppCreationRequest),
    CancelCreation,
    CreatorStepBack,
    CreatorSelectType { type_id: String },
    CreatorUpdateField { key: String, value: String },
    CreatorNextStep,
    // Phase 5 — Proxies
    RefreshProxies,
    AddProxy { domain: String, target: String, secure: bool },
    RemoveProxy(String),
    TestProxy(String),
    // Phase 5 — Dnsmasq / TLD
    ChangeTld(String),
    TestDns(String),
    // Phase 5 — Sharing
    StartSharing { site: String, tool: crate::valet::sharing::ShareTool, token: String },
    StopSharing,
    // Phase 5 — Logs
    LoadLogs(crate::system::log_reader::LogSource),
    // Phase 5 — Diagnostics
    RunDiagnostics,
    TrustValet,
    RestartAllServices,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppCreationRequest {
    pub type_id: String,
    pub form_values: std::collections::HashMap<String, String>,
    pub post_install_secure: bool,
    pub post_install_php_version: Option<String>,
    pub post_install_open_browser: bool,
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

    #[test]
    fn isolate_site_has_fields() {
        let cmd = AppCommand::IsolateSite {
            site: "myapp".to_string(),
            version: "8.3".to_string(),
        };
        if let AppCommand::IsolateSite { site, version } = cmd {
            assert_eq!(site, "myapp");
            assert_eq!(version, "8.3");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn unisolate_site_has_fields() {
        let cmd = AppCommand::UnisolateSite("myapp".to_string());
        if let AppCommand::UnisolateSite(site) = cmd {
            assert_eq!(site, "myapp");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn install_php_version_has_fields() {
        let cmd = AppCommand::InstallPhpVersion("8.3".to_string());
        if let AppCommand::InstallPhpVersion(version) = cmd {
            assert_eq!(version, "8.3");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn remove_php_version_has_fields() {
        let cmd = AppCommand::RemovePhpVersion("8.1".to_string());
        if let AppCommand::RemovePhpVersion(version) = cmd {
            assert_eq!(version, "8.1");
        } else {
            panic!("wrong variant");
        }
    }
}
