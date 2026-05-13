use crate::php::detector::PhpVersion;
use crate::services::monitor::ManagedService;
use crate::valet::variant::{ValetPaths, ValetVariant};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Panel {
    #[default]
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

#[derive(Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct UiState {
    pub active_panel: Panel,
    pub loading: bool,
    pub toast_queue: Vec<String>,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_active_panel_is_dashboard() {
        let state = AppState::default();
        assert_eq!(state.ui.active_panel, Panel::Dashboard);
    }

    #[test]
    fn default_active_php_is_unknown() {
        assert_eq!(AppState::default().active_php, "unknown");
    }

    #[test]
    fn default_tld_is_test() {
        assert_eq!(AppState::default().tld, "test");
    }

    #[test]
    fn default_site_count_is_zero() {
        assert_eq!(AppState::default().site_count, 0);
    }

    #[test]
    fn panel_default_is_dashboard() {
        assert_eq!(Panel::default(), Panel::Dashboard);
    }

    #[test]
    fn ui_state_loading_defaults_false() {
        assert!(!UiState::default().loading);
    }
}
