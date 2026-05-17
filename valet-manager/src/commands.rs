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
    // Phase 6 — Settings & Onboarding
    LoadSettings,
    SaveSettings(crate::config::AppConfig),
    MarkOnboardingComplete,
    // Phase 7 — phpinfo
    LoadPhpInfo(String),
    // Phase 7 — Compatibility
    CheckCompat,
    // Phase 7 — History
    LoadHistory,
    RerunHistory(i64),
    // Phase 7 — Updater
    CheckForUpdates,
    SkipUpdate(String),
    OpenUpdatePage,
    // Phase 9 — .env editor
    LoadEnvFile(String),
    SaveEnvFile { site: String, content: String },
    // Phase 9 — Artisan
    LoadArtisanCommands(String),
    RunArtisan { site: String, command: String, args: String },
    // Phase 9 — Database
    RefreshDatabases,
    SelectDatabase(String),
    RunMigration { site: String, action: String },
    // Phase 10 — SSL certs
    RefreshSslCerts,
    TrustValetCa,
    RevokeSiteCert(String),
    // Phase 10 — Xdebug
    RefreshXdebug,
    InstallXdebug(String),
    SetXdebugMode { version: String, mode: crate::php::xdebug::XdebugMode },
    SetXdebugIdeKey { version: String, ide_key: String },
    SetXdebugPort   { version: String, port: u16 },
    // Phase 10 — Mail catcher
    StartMailCatcher,
    StopMailCatcher,
    RefreshMailUnread,
    ApplyMailEnv(String),
    // Phase 10 — Queue workers
    RefreshQueueWorkers,
    AddQueueWorker { site: String, connection: String, queue: String, start_on_boot: bool },
    StartQueueWorker(String),
    StopQueueWorker(String),
    RemoveQueueWorker(String),
    // Phase 11 — Per-site config
    LoadSiteConfig(String),
    SaveSiteConfig {
        site: String,
        config: crate::site_config::models::SiteConfig,
    },
    SaveSiteConfigToProject {
        site: String,
        config: crate::site_config::models::SiteConfig,
    },
    ResetSiteConfig(String),
    ApplyPhpIniOverrides { site: String },
    SetSitePhpVersion { site: String, version: Option<String> },
    EnableWpMultisite {
        site: String,
        multisite_type: crate::site_config::models::MultisiteType,
    },
    DisableWpMultisite(String),
    SetWpConfigConstants {
        site: String,
        config: crate::site_config::models::WordPressConfig,
    },
    FetchWpNetworkSites(String),
    SetLaravelOctane { site: String, enabled: bool },
    DetectInstalledServers,
    SetSiteHttpServer {
        site: String,
        server_type: crate::site_config::models::HttpServerType,
    },
    AddBasicAuth {
        site: String,
        username: String,
        password: String,
    },
    RemoveBasicAuth { site: String, username: String },
    DetectLaravelPackages(String),
    // Phase 12 — phpMyAdmin
    CheckPhpMyAdminInstalled,
    InstallPhpMyAdmin,
    UninstallPhpMyAdmin,
    ConfigurePhpMyAdminForSite(String),
    RemovePhpMyAdminFromSite(String),
    OpenPhpMyAdmin {
        site: String,
        scope: crate::site_config::models::PmaDbScope,
    },
    SetupGlobalPhpMyAdminSite,
    SetPmaAccessMode {
        site: String,
        mode: crate::site_config::models::PmaAccessMode,
    },
    SetPmaDbScope {
        site: String,
        scope: crate::site_config::models::PmaDbScope,
    },
    RefreshPhpMyAdminStatus,
    // Phase 14 — Version registry
    RefreshVersionRegistry,
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
