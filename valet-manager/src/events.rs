use crate::php::detector::PhpVersion;
use crate::php::types::{IniSection, IniType, PhpExtension};
use crate::services::monitor::{ManagedService, ServiceStatus};
use crate::valet::variant::{ValetPaths, ValetVariant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppEvent {
    // Phase 1
    ServiceStatusUpdated(Vec<ManagedService>),
    PhpVersionsRefreshed(Vec<PhpVersion>),
    ValetDetected(ValetVariant, ValetPaths),
    Error(String),
    // Phase 2
    PhpSwitched(String),
    PhpFpmStatusChanged { version: String, status: ServiceStatus },
    ExtensionsLoaded { version: String, extensions: Vec<PhpExtension> },
    IniLoaded { version: String, ini_type: IniType, sections: Vec<IniSection>, raw: String },
    IniSaved,
    SiteIsolated { site: String, version: String },
    SiteUnisolated(String),
    PhpVersionInstalled(String),
    PhpVersionRemoved(String),
    // Phase 3
    SitesRefreshed(Vec<crate::valet::site_scanner::ValetSite>),
    NginxConfigLoaded { site: String, content: String },
    NginxReloaded,
    ParksUpdated(Vec<std::path::PathBuf>),
    // Phase 4 — App Creator
    CreatorOutputLine(crate::creator::output_streamer::OutputLine),
    CreatorComplete { site_name: String, domain: String },
    CreatorFailed(String),
    // Phase 5 — Proxies
    ProxiesRefreshed(Vec<crate::nginx::proxy_manager::ValetProxy>),
    ProxyAdded(String),
    ProxyRemoved(String),
    ProxyProbed { domain: String, result: crate::nginx::proxy_manager::HttpProbe },
    // Phase 5 — Dnsmasq
    TldChanged(String),
    DnsOutputLine(crate::creator::output_streamer::OutputLine),
    // Phase 5 — Sharing
    SharingStarted(crate::valet::sharing::ShareSession),
    SharingStopped,
    SharingOutputLine(crate::creator::output_streamer::OutputLine),
    // Phase 5 — Logs
    LogsLoaded { source: crate::system::log_reader::LogSource, lines: Vec<String> },
    // Phase 5 — Diagnostics
    DiagnosticsOutputLine(crate::creator::output_streamer::OutputLine),
    DiagnosticsComplete,
    // Phase 6 — Settings & Onboarding
    SettingsLoaded(crate::config::AppConfig),
    SettingsSaved,
    OnboardingComplete,
    // Phase 7 — phpinfo
    PhpInfoLoaded {
        version: String,
        sections: Vec<crate::php::phpinfo_parser::PhpInfoSection>,
    },
    // Phase 7 — Compatibility
    CompatChecked(Vec<crate::php::compat_checker::SiteCompat>),
    // Phase 7 — History
    HistoryLoaded(Vec<crate::history::HistoryEntry>),
    // Phase 7 — Updater
    UpdateAvailable(crate::updater::UpdateInfo),
    // Phase 9 — .env editor
    EnvFileLoaded {
        site: String,
        entries: Vec<crate::env_file::EnvEntry>,
        example: Vec<crate::env_file::EnvEntry>,
    },
    EnvFileSaved,
    // Phase 9 — Artisan
    ArtisanCommandsLoaded {
        site: String,
        tool: crate::artisan::ArtisanTool,
        commands: Vec<crate::artisan::ArtisanCommand>,
    },
    ArtisanOutputLine(crate::creator::output_streamer::OutputLine),
    ArtisanComplete,
    // Phase 9 — Database
    DatabasesLoaded(Vec<crate::database::DbDatabase>),
    TablesLoaded {
        database: String,
        tables: Vec<crate::database::DbTable>,
    },
    MigrationOutput(crate::creator::output_streamer::OutputLine),
    MigrationComplete,
    // Phase 10 — SSL certs
    SslCertsLoaded(Vec<crate::ssl::cert_reader::CertInfo>),
    // Phase 10 — Xdebug
    XdebugConfigsLoaded(std::collections::HashMap<String, crate::php::xdebug::XdebugConfig>),
    XdebugInstallOutput(crate::creator::output_streamer::OutputLine),
    XdebugConfigUpdated(crate::php::xdebug::XdebugConfig),
    // Phase 10 — Mail catcher
    MailStatusUpdated(crate::mail::mailpit::MailStatus),
    MailEnvApplied(String),
    // Phase 10 — Queue workers
    QueueWorkersLoaded(Vec<crate::queue::QueueWorker>),
    QueueWorkerChanged(String),
    // Phase 11 — Per-site config
    SiteConfigLoaded {
        site: String,
        config: crate::site_config::models::SiteConfig,
    },
    SiteConfigSaved(String),
    SiteConfigReset(String),
    WpMultisiteEnabled { site: String, output: Vec<String> },
    WpMultisiteDisabled(String),
    WpNetworkSitesLoaded {
        site: String,
        sites: Vec<crate::site_config::models::WpNetworkSite>,
    },
    InstalledServersDetected(Vec<crate::site_config::models::HttpServerType>),
    SiteConfigOutputLine(crate::creator::output_streamer::OutputLine),
    LaravelPackagesDetected {
        site: String,
        packages: crate::laravel::packages::LaravelPackages,
    },
    OctaneProcessUpdated(crate::site_config::models::OctaneProcess),
    // Phase 12 — phpMyAdmin
    PhpMyAdminInstalledCheck {
        installed: bool,
        path: Option<std::path::PathBuf>,
        version: Option<String>,
    },
    PhpMyAdminConfigured {
        site: String,
        status: crate::state::app_state::PmaSiteStatus,
    },
    PhpMyAdminRemoved(String),
    PhpMyAdminOutputLine(crate::creator::output_streamer::OutputLine),
    PhpMyAdminGlobalReady(String),
    // Phase 14 — Version registry
    VersionRegistryRefreshed(crate::version_registry::models::VersionRegistry),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_debug() {
        let e = AppEvent::PhpSwitched("8.3".to_string());
        assert!(format!("{:?}", e).contains("8.3"));
    }

    #[test]
    fn ini_saved_event() {
        let e = AppEvent::IniSaved;
        assert!(format!("{:?}", e).contains("IniSaved"));
    }

    #[test]
    fn site_isolated_event() {
        let e = AppEvent::SiteIsolated {
            site: "myapp".to_string(),
            version: "8.3".to_string(),
        };
        let debug = format!("{:?}", e);
        assert!(debug.contains("myapp"));
        assert!(debug.contains("8.3"));
    }

    #[test]
    fn site_unisolated_event() {
        let e = AppEvent::SiteUnisolated("myapp".to_string());
        assert!(format!("{:?}", e).contains("myapp"));
    }

    #[test]
    fn php_version_installed_event() {
        let e = AppEvent::PhpVersionInstalled("8.3".to_string());
        assert!(format!("{:?}", e).contains("8.3"));
    }

    #[test]
    fn php_version_removed_event() {
        let e = AppEvent::PhpVersionRemoved("8.1".to_string());
        assert!(format!("{:?}", e).contains("8.1"));
    }
}
