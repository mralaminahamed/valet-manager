use std::collections::HashMap;
use crate::php::detector::PhpVersion;
use crate::php::types::{IniSection, IniType, PhpExtension};
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
    // Phase 2
    pub extensions: Vec<PhpExtension>,
    pub ini_sections: Vec<IniSection>,
    pub ini_raw: String,
    pub ini_last_saved: Option<String>,
    pub ui: UiState,
    // Phase 3
    pub sites: Vec<crate::valet::site_scanner::ValetSite>,
    pub parks: Vec<std::path::PathBuf>,
    pub nginx_configs: std::collections::HashMap<String, String>,
    pub nginx_selected: Option<String>,
    pub site_search: String,
    pub site_filter_favorites: bool,
    pub site_sort_favorites_top: bool,
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
            extensions: Vec::new(),
            ini_sections: Vec::new(),
            ini_raw: String::new(),
            ini_last_saved: None,
            ui: UiState::default(),
            sites: Vec::new(),
            parks: Vec::new(),
            nginx_configs: std::collections::HashMap::new(),
            nginx_selected: None,
            site_search: String::new(),
            site_filter_favorites: false,
            site_sort_favorites_top: true,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UiState {
    pub active_panel: Panel,
    pub loading: bool,
    pub toast_queue: Vec<String>,
    pub last_error: Option<String>,
    // Phase 2: PHP panels shared state
    pub selected_php_version: Option<String>,
    pub php_search_query: String,
    pub php_show_core_ext: bool,
    pub php_ini_type: IniType,
    pub php_raw_edit_mode: bool,
    pub selected_ini_section: usize,
    pub ini_pending: HashMap<(String, String), String>,
    pub php_install_modal: bool,
    pub confirm_delete_php: Option<String>,
    pub selected_extension: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            active_panel: Panel::default(),
            loading: false,
            toast_queue: Vec::new(),
            last_error: None,
            selected_php_version: None,
            php_search_query: String::new(),
            php_show_core_ext: false,
            php_ini_type: IniType::Cli,
            php_raw_edit_mode: false,
            selected_ini_section: 0,
            ini_pending: HashMap::new(),
            php_install_modal: false,
            confirm_delete_php: None,
            selected_extension: None,
        }
    }
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

    #[test]
    fn ui_state_php_search_query_defaults_empty() {
        assert_eq!(UiState::default().php_search_query, "");
    }

    #[test]
    fn ui_state_selected_php_version_defaults_none() {
        assert!(UiState::default().selected_php_version.is_none());
    }

    #[test]
    fn app_state_extensions_defaults_empty() {
        assert!(AppState::default().extensions.is_empty());
    }

    #[test]
    fn app_state_ini_raw_defaults_empty() {
        assert_eq!(AppState::default().ini_raw, "");
    }
}
