use std::collections::HashMap;
use crate::php::detector::PhpVersion;
use crate::php::types::{IniSection, IniType, PhpExtension};
use crate::services::monitor::ManagedService;
use crate::valet::variant::{ValetPaths, ValetVariant};
use crate::state::creator_state::CreatorState;

#[derive(Debug, Clone, Default)]
pub struct MailpitMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub received: String,
    pub read: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Screen {
    #[default]
    Sites,
    Services,
    Logs,
    Dns,
    Settings,
}

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
    AppCreator,
    Settings,
    SiteConfig,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SiteStatusFilter {
    #[default]
    All,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Behavior,
    Php,
    Tls,
    Paths,
    Updates,
    About,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsAlias {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AddSiteTab {
    #[default]
    Park,
    Create,
    Proxy,
}

#[derive(Debug, Clone)]
pub struct AddSiteFormState {
    // Park tab
    pub park_path: String,
    pub park_domain: String,
    pub park_domain_touched: bool,
    pub park_framework: String,
    pub park_php: String,
    pub park_http_server: String,
    pub park_secure: bool,
    pub park_create_db: bool,
    pub park_scoped_pma: bool,
    pub park_bulk: bool,
    // Create tab
    pub create_template: String,
    pub create_name: String,
    pub create_location: String,
    pub create_php: String,
    pub create_git: bool,
    pub create_install: bool,
    pub create_migrate: bool,
    // Proxy tab
    pub proxy_domain: String,
    pub proxy_target: String,
    pub proxy_secure: bool,
    pub proxy_websocket: bool,
}

impl Default for AddSiteFormState {
    fn default() -> Self {
        Self {
            park_path: "~/Code/new-project".to_string(),
            park_domain: "new-project".to_string(),
            park_domain_touched: false,
            park_framework: "Laravel".to_string(),
            park_php: "8.3".to_string(),
            park_http_server: "nginx".to_string(),
            park_secure: true,
            park_create_db: true,
            park_scoped_pma: true,
            park_bulk: false,
            create_template: "laravel".to_string(),
            create_name: "my-app".to_string(),
            create_location: "~/Code".to_string(),
            create_php: "8.3".to_string(),
            create_git: true,
            create_install: true,
            create_migrate: true,
            proxy_domain: "studio".to_string(),
            proxy_target: "http://localhost:5173".to_string(),
            proxy_secure: true,
            proxy_websocket: true,
        }
    }
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
    // Phase 4
    pub creator: CreatorState,
    // Phase 5 — Proxies
    pub proxies: Vec<crate::nginx::proxy_manager::ValetProxy>,
    pub dns_aliases: Vec<DnsAlias>,
    pub proxy_status: std::collections::HashMap<String, crate::nginx::proxy_manager::HttpProbe>,
    // Phase 5 — Dnsmasq
    pub dns_tester_host: String,
    pub dns_tester_output: Vec<crate::creator::output_streamer::OutputLine>,
    pub tld_input: String,
    // Phase 5 — Sharing
    pub share_tool: crate::valet::sharing::ShareTool,
    pub share_tokens: std::collections::HashMap<String, String>,
    pub share_site: Option<String>,
    pub share_active: Option<crate::valet::sharing::ShareSession>,
    pub share_output: Vec<crate::creator::output_streamer::OutputLine>,
    // Phase 5 — Logs
    pub logs_source: crate::system::log_reader::LogSource,
    pub logs_lines: Vec<String>,
    pub logs_tail: bool,
    // Phase 5 — Diagnostics
    pub diagnostics_output: Vec<crate::creator::output_streamer::OutputLine>,
    pub diagnostics_running: bool,
    // Phase 6 — Toasts, Settings, Onboarding
    pub toasts: Vec<crate::ui::components::toast::Toast>,
    pub config: crate::config::AppConfig,
    pub config_draft: crate::config::AppConfig,
    pub onboarding_complete: bool,
    pub onboarding_step: u8,
    pub onboarding_diagnose_output: Vec<crate::creator::output_streamer::OutputLine>,
    // Phase 7 — phpinfo
    pub phpinfo_sections: Vec<crate::php::phpinfo_parser::PhpInfoSection>,
    pub phpinfo_selected_section: usize,
    pub phpinfo_search: String,
    // Phase 7 — compatibility
    pub site_compat: Vec<crate::php::compat_checker::SiteCompat>,
    // Phase 7 — history
    pub history: Vec<crate::history::HistoryEntry>,
    pub history_expanded: Option<i64>,
    // Phase 7 — updater
    pub update_info: Option<crate::updater::UpdateInfo>,
    // Phase 9 — .env editor
    pub env_selected_site: Option<String>,
    pub env_entries: Vec<crate::env_file::EnvEntry>,
    pub env_example_entries: Vec<crate::env_file::EnvEntry>,
    pub env_active_group: String,
    pub env_show_secrets: bool,
    // Phase 9 — Artisan
    pub artisan_site: Option<String>,
    pub artisan_tool: Option<crate::artisan::ArtisanTool>,
    pub artisan_commands: Vec<crate::artisan::ArtisanCommand>,
    pub artisan_input: String,
    pub artisan_args: String,
    pub artisan_autocomplete_selected: usize,
    pub artisan_output: Vec<crate::creator::output_streamer::OutputLine>,
    // Phase 9 — Database
    pub db_engine: crate::database::DbEngine,
    pub db_credentials: crate::database::DbCredentials,
    pub db_selected_site: Option<String>,
    pub databases: Vec<crate::database::DbDatabase>,
    pub db_selected_database: Option<String>,
    pub db_tables: Vec<crate::database::DbTable>,
    pub db_migration_output: Vec<crate::creator::output_streamer::OutputLine>,
    pub db_migration_running: bool,
    // Phase 10 — SSL certs
    pub ssl_certs: Vec<crate::ssl::cert_reader::CertInfo>,
    pub ssl_ca_trusted: bool,
    // Phase 10 — Xdebug
    pub xdebug_configs: std::collections::HashMap<String, crate::php::xdebug::XdebugConfig>,
    pub xdebug_install_output: Vec<crate::creator::output_streamer::OutputLine>,
    // Phase 10 — Mail catcher
    pub mail_status: crate::mail::mailpit::MailStatus,
    pub mail_apply_site: Option<String>,
    // Phase 10 — Queue workers
    pub queue_workers: Vec<crate::queue::QueueWorker>,
    pub queue_add_modal: bool,
    pub queue_add_site: Option<String>,
    pub queue_add_connection: String,
    pub queue_add_queue: String,
    pub queue_add_start_on_boot: bool,
    // Phase 11 — Site config
    pub site_configs: std::collections::HashMap<String, crate::site_config::models::SiteConfig>,
    pub site_config_selected: Option<String>,
    pub site_config_draft: Option<crate::site_config::models::SiteConfig>,
    pub wp_network_sites: std::collections::HashMap<String, Vec<crate::site_config::models::WpNetworkSite>>,
    pub installed_http_servers: Vec<crate::site_config::models::HttpServerType>,
    pub site_config_output: Vec<crate::creator::output_streamer::OutputLine>,
    pub laravel_packages: std::collections::HashMap<String, crate::laravel::packages::LaravelPackages>,
    pub octane_processes: std::collections::HashMap<String, crate::site_config::models::OctaneProcess>,
    // Phase 12 — phpMyAdmin
    pub pma_state: PhpMyAdminState,
    // Phase 14 — Version registry
    pub version_registry: crate::version_registry::models::VersionRegistry,
    pub version_registry_loading: bool,
    // M1 — Shell
    pub shell_output: Vec<String>,
    pub shell_input: String,
    pub shell_site: Option<String>,
    // M8 — Mailpit inbox
    pub mailpit_messages: Vec<MailpitMessage>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PhpMyAdminState {
    pub installed: bool,
    pub install_path: Option<std::path::PathBuf>,
    pub version: Option<String>,
    pub global_site_configured: bool,
    pub global_site_domain: String,
    pub sites: HashMap<String, PmaSiteStatus>,
    pub output: Vec<crate::creator::output_streamer::OutputLine>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PmaSiteStatus {
    pub enabled: bool,
    pub access_url: String,
    pub config_path: std::path::PathBuf,
    pub db_name: String,
    pub access_mode: crate::site_config::models::PmaAccessMode,
    pub last_configured: Option<chrono::DateTime<chrono::Local>>,
}

impl Default for PmaSiteStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            access_url: String::new(),
            config_path: std::path::PathBuf::new(),
            db_name: String::new(),
            access_mode: crate::site_config::models::PmaAccessMode::default(),
            last_configured: None,
        }
    }
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
            creator: CreatorState::default(),
            proxies: Vec::new(),
            dns_aliases: Vec::new(),
            proxy_status: std::collections::HashMap::new(),
            dns_tester_host: String::new(),
            dns_tester_output: Vec::new(),
            tld_input: String::new(),
            share_tool: crate::valet::sharing::ShareTool::Ngrok,
            share_tokens: std::collections::HashMap::new(),
            share_site: None,
            share_active: None,
            share_output: Vec::new(),
            logs_source: crate::system::log_reader::LogSource::NginxError,
            logs_lines: Vec::new(),
            logs_tail: true,
            diagnostics_output: Vec::new(),
            diagnostics_running: false,
            toasts: Vec::new(),
            config: crate::config::load(),
            config_draft: crate::config::load(),
            onboarding_complete: crate::config::is_onboarded(),
            onboarding_step: 0,
            onboarding_diagnose_output: Vec::new(),
            phpinfo_sections: Vec::new(),
            phpinfo_selected_section: 0,
            phpinfo_search: String::new(),
            site_compat: Vec::new(),
            history: Vec::new(),
            history_expanded: None,
            update_info: None,
            // Phase 9 — .env editor
            env_selected_site: None,
            env_entries: Vec::new(),
            env_example_entries: Vec::new(),
            env_active_group: "APP".to_string(),
            env_show_secrets: false,
            // Phase 9 — Artisan
            artisan_site: None,
            artisan_tool: None,
            artisan_commands: Vec::new(),
            artisan_input: String::new(),
            artisan_args: String::new(),
            artisan_autocomplete_selected: 0,
            artisan_output: Vec::new(),
            // Phase 9 — Database
            db_engine: crate::database::DbEngine::MySql,
            db_credentials: crate::database::DbCredentials::default(),
            db_selected_site: None,
            databases: Vec::new(),
            db_selected_database: None,
            db_tables: Vec::new(),
            db_migration_output: Vec::new(),
            db_migration_running: false,
            // Phase 10 — SSL certs
            ssl_certs: Vec::new(),
            ssl_ca_trusted: true,
            // Phase 10 — Xdebug
            xdebug_configs: std::collections::HashMap::new(),
            xdebug_install_output: Vec::new(),
            // Phase 10 — Mail catcher
            mail_status: crate::mail::mailpit::MailStatus {
                tool: crate::mail::mailpit::MailTool::Mailpit,
                running: false,
                smtp_port: 1025,
                http_port: 8025,
                unread: 0,
            },
            mail_apply_site: None,
            // Phase 10 — Queue workers
            queue_workers: Vec::new(),
            queue_add_modal: false,
            queue_add_site: None,
            queue_add_connection: "redis".to_string(),
            queue_add_queue: "default".to_string(),
            queue_add_start_on_boot: false,
            // Phase 11
            site_configs: std::collections::HashMap::new(),
            site_config_selected: None,
            site_config_draft: None,
            wp_network_sites: std::collections::HashMap::new(),
            installed_http_servers: Vec::new(),
            site_config_output: Vec::new(),
            laravel_packages: std::collections::HashMap::new(),
            octane_processes: std::collections::HashMap::new(),
            // Phase 12 — phpMyAdmin
            pma_state: PhpMyAdminState {
                global_site_domain: "phpmyadmin.test".to_string(),
                ..Default::default()
            },
            // Phase 14 — Version registry
            version_registry: crate::version_registry::models::VersionRegistry::default(),
            version_registry_loading: false,
            // M1 — Shell
            shell_output: Vec::new(),
            shell_input: String::new(),
            shell_site: None,
            // M8 — Mailpit inbox
            mailpit_messages: Vec::new(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UiState {
    pub active_panel: Panel,
    pub active_screen: Screen,
    // Floating windows
    pub shell_open: bool,
    pub mailpit_open: bool,
    pub pma_open: bool,
    // Add-site modal
    pub add_site_modal_open: bool,
    pub add_site_modal_tab: AddSiteTab,
    pub add_site_form: AddSiteFormState,
    // Site status filter
    pub site_status_filter: SiteStatusFilter,
    // Settings left-nav
    pub settings_section: SettingsSection,
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
    // Phase 6 — responsive sidebar
    pub mobile_sidebar_open: bool,
    // Phase 9 — command palette
    pub palette: crate::ui::command_palette::PaletteState,
    // Phase 11 — Site config panel UI state
    pub site_config_tab: u8,
    pub site_config_new_auth_user: String,
    pub site_config_new_auth_pass: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            active_panel: Panel::default(),
            active_screen: Screen::default(),
            shell_open: false,
            mailpit_open: false,
            pma_open: false,
            add_site_modal_open: false,
            add_site_modal_tab: AddSiteTab::default(),
            add_site_form: AddSiteFormState::default(),
            site_status_filter: SiteStatusFilter::default(),
            settings_section: SettingsSection::default(),
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
            mobile_sidebar_open: false,
            palette: crate::ui::command_palette::PaletteState::default(),
            site_config_tab: 0,
            site_config_new_auth_user: String::new(),
            site_config_new_auth_pass: String::new(),
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

    // Phase 10 default-state assertions
    #[test]
    fn app_state_ssl_certs_defaults_empty() {
        assert!(AppState::default().ssl_certs.is_empty());
    }

    #[test]
    fn app_state_ssl_ca_trusted_defaults_true() {
        assert!(AppState::default().ssl_ca_trusted);
    }

    #[test]
    fn app_state_xdebug_configs_defaults_empty() {
        assert!(AppState::default().xdebug_configs.is_empty());
    }

    #[test]
    fn app_state_mail_status_defaults_stopped() {
        let s = AppState::default();
        assert!(!s.mail_status.running);
        assert_eq!(s.mail_status.smtp_port, 1025);
        assert_eq!(s.mail_status.http_port, 8025);
        assert_eq!(s.mail_status.unread, 0);
    }

    #[test]
    fn app_state_queue_workers_defaults_empty() {
        let s = AppState::default();
        assert!(s.queue_workers.is_empty());
        assert!(!s.queue_add_modal);
        assert_eq!(s.queue_add_connection, "redis");
        assert_eq!(s.queue_add_queue, "default");
    }

    // Phase 14 — Version registry defaults
    #[test]
    fn app_state_version_registry_defaults_populated() {
        let s = AppState::default();
        assert!(!s.version_registry.php.is_empty());
        assert!(s.version_registry.frameworks.contains_key("laravel"));
        assert!(s.version_registry.last_refreshed.is_none());
        assert!(!s.version_registry_loading);
    }

    #[test]
    fn default_screen_is_sites() {
        let state = AppState::default();
        assert_eq!(state.ui.active_screen, Screen::Sites);
    }

    #[test]
    fn default_site_filter_is_all() {
        let state = AppState::default();
        assert_eq!(state.ui.site_status_filter, SiteStatusFilter::All);
    }

    #[test]
    fn default_settings_section_is_appearance() {
        let state = AppState::default();
        assert_eq!(state.ui.settings_section, SettingsSection::Appearance);
    }

    #[test]
    fn floating_windows_default_closed() {
        let state = AppState::default();
        assert!(!state.ui.shell_open);
        assert!(!state.ui.mailpit_open);
        assert!(!state.ui.pma_open);
    }
}
