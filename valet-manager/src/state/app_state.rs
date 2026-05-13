use crate::php::detector::PhpVersion;
use crate::services::monitor::ManagedService;
use crate::valet::variant::{ValetPaths, ValetVariant};

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Dashboard,
    PhpVersions,
    PhpExtensions,
    PhpIni,
    PhpInfo,
    PhpCompat,
    Sites,
    Parks,
    Nginx,
    Proxies,
    Dnsmasq,
    SslCerts,
    Database,
    EnvEditor,
    Artisan,
    QueueWorkers,
    Xdebug,
    MailCatcher,
    Sharing,
    Drivers,
    Logs,
    History,
    Diagnostics,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Panel::Dashboard
    }
}

#[derive(Debug)]
pub struct AppState {
    pub valet_variant: Option<ValetVariant>,
    pub valet_paths: Option<ValetPaths>,
    pub php_versions: Vec<PhpVersion>,
    pub active_php: String,
    pub services: Vec<ManagedService>,
    pub cli_tools: Vec<String>,
    pub tld: String,
    pub site_count: usize,
    pub ui: UiState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            valet_variant: None,
            valet_paths: None,
            php_versions: Vec::new(),
            active_php: "unknown".to_string(),
            services: Vec::new(),
            cli_tools: Vec::new(),
            tld: "test".to_string(),
            site_count: 0,
            ui: UiState::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub active_panel: Panel,
    pub loading: bool,
    pub toast_queue: Vec<String>,
    pub last_error: Option<String>,
}
