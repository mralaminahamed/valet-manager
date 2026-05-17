#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root per-site configuration. Stored as TOML.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SiteConfig {
    #[serde(default)]
    pub php: PhpSiteConfig,
    #[serde(default)]
    pub framework: FrameworkSiteConfig,
    #[serde(default)]
    pub server: HttpServerSiteConfig,
    #[serde(default)]
    pub wordpress: Option<WordPressConfig>,
    #[serde(default)]
    pub laravel: Option<LaravelConfig>,
    #[serde(default)]
    pub database: DatabaseSiteConfig,
    #[serde(default)]
    pub development: DevSiteConfig,
    #[serde(default)]
    pub phpmyadmin: PhpMyAdminSiteConfig,
    // Phase 13 — framework-specific sub-configs (state-only this phase; no
    // dedicated panel rendering yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub craft: Option<CraftConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concretecms: Option<ConcreteCmsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drupal: Option<DrupalConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joomla: Option<JoomlaConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magento: Option<MagentoConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub octobercms: Option<OctoberCmsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statamic: Option<StatamicConfig>,
}

// ── FRAMEWORK-SPECIFIC SITE CONFIG (Phase 13) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CraftConfig {
    pub environment: Option<String>,
    pub license_key: Option<String>,
    pub db_driver: Option<String>,
    #[serde(default)]
    pub use_project_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ConcreteCmsConfig {
    pub environment: Option<String>,
    #[serde(default)]
    pub cache_enabled: bool,
    #[serde(default = "default_true")]
    pub pretty_urls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DrupalConfig {
    pub environment: Option<String>,
    #[serde(default)]
    pub trusted_host_patterns: Vec<String>,
    #[serde(default)]
    pub cache_bins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct JoomlaConfig {
    pub error_reporting: Option<String>,
    #[serde(default)]
    pub sef_urls: bool,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub cache_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MagentoConfig {
    /// "developer" | "production" | "default"
    pub mode: Option<String>,
    /// "realtime" | "schedule"
    pub indexer_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OctoberCmsConfig {
    #[serde(default)]
    pub debug_mode: bool,
    pub backend_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StatamicConfig {
    #[serde(default = "default_true")]
    pub flat_file: bool,
    #[serde(default)]
    pub git_integration: bool,
    #[serde(default)]
    pub api_enabled: bool,
}

// ── PHPMYADMIN ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PhpMyAdminSiteConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub access_mode: PmaAccessMode,
    #[serde(default = "default_pma_path_alias")]
    pub path_alias: String,
    #[serde(default)]
    pub db_scope: PmaDbScope,
    /// Optional override; when None, credentials are detected from .env / wp-config.php.
    pub db_name_override: Option<String>,
    /// Optional override; mirrors db_name_override. None falls back to detected user.
    pub db_user_override: Option<String>,
}

impl Default for PhpMyAdminSiteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            access_mode: PmaAccessMode::default(),
            path_alias: default_pma_path_alias(),
            db_scope: PmaDbScope::default(),
            db_name_override: None,
            db_user_override: None,
        }
    }
}

fn default_pma_path_alias() -> String {
    "/phpmyadmin".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PmaAccessMode {
    #[default]
    PathAlias,
    Subdomain,
    GlobalOnly,
}

impl PmaAccessMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            PmaAccessMode::PathAlias => "Path alias",
            PmaAccessMode::Subdomain => "Subdomain",
            PmaAccessMode::GlobalOnly => "Global only",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PmaDbScope {
    #[default]
    SiteOnly,
    AllDatabases,
}

impl PmaDbScope {
    pub fn display_name(&self) -> &'static str {
        match self {
            PmaDbScope::SiteOnly => "Site only",
            PmaDbScope::AllDatabases => "All databases",
        }
    }
}

// ── PHP ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PhpSiteConfig {
    /// None = inherit global active version
    pub version: Option<String>,
    #[serde(default)]
    pub ini_overrides: PhpIniOverrides,
    #[serde(default)]
    pub xdebug_enabled: bool,
    #[serde(default)]
    pub xdebug_mode: XdebugMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PhpIniOverrides {
    pub memory_limit: Option<String>,
    pub upload_max_filesize: Option<String>,
    pub post_max_size: Option<String>,
    pub max_execution_time: Option<u32>,
    pub max_input_vars: Option<u32>,
    pub max_file_uploads: Option<u32>,
    pub session_gc_maxlifetime: Option<u32>,
    pub date_timezone: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl PhpIniOverrides {
    pub fn is_empty(&self) -> bool {
        self.memory_limit.is_none()
            && self.upload_max_filesize.is_none()
            && self.post_max_size.is_none()
            && self.max_execution_time.is_none()
            && self.max_input_vars.is_none()
            && self.max_file_uploads.is_none()
            && self.session_gc_maxlifetime.is_none()
            && self.date_timezone.is_none()
            && self.extra.is_empty()
    }

    /// Render as valid PHP .user.ini content
    pub fn render(&self) -> String {
        let mut lines = vec![
            "; Generated by Valet Manager — do not edit manually".to_string(),
        ];
        if let Some(v) = &self.memory_limit         { lines.push(format!("memory_limit = {v}")); }
        if let Some(v) = &self.upload_max_filesize  { lines.push(format!("upload_max_filesize = {v}")); }
        if let Some(v) = &self.post_max_size        { lines.push(format!("post_max_size = {v}")); }
        if let Some(v) = self.max_execution_time    { lines.push(format!("max_execution_time = {v}")); }
        if let Some(v) = self.max_input_vars        { lines.push(format!("max_input_vars = {v}")); }
        if let Some(v) = self.max_file_uploads      { lines.push(format!("max_file_uploads = {v}")); }
        if let Some(v) = self.session_gc_maxlifetime { lines.push(format!("session.gc_maxlifetime = {v}")); }
        if let Some(v) = &self.date_timezone        { lines.push(format!("date.timezone = \"{v}\"")); }
        // Sort extra keys deterministically
        let mut extra: Vec<(&String, &String)> = self.extra.iter().collect();
        extra.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in extra                         { lines.push(format!("{k} = {v}")); }
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum XdebugMode {
    #[default]
    Off,
    Develop,
    Debug,
    Profile,
    Trace,
    Coverage,
}

// ── FRAMEWORK ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FrameworkSiteConfig {
    pub framework_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_version: Option<String>,
}

// ── HTTP SERVER ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HttpServerSiteConfig {
    #[serde(default)]
    pub server_type: HttpServerType,
    #[serde(default)]
    pub custom_directives: String,
    pub client_max_body_size: Option<String>,
    pub read_timeout: Option<u32>,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
    #[serde(default)]
    pub basic_auth: BasicAuthConfig,
    #[serde(default)]
    pub redirects: Vec<RedirectRule>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum HttpServerType {
    #[default]
    Nginx,
    #[serde(rename = "frankenphp")]
    FrankenPhp,
    Caddy,
    Apache,
}

impl HttpServerType {
    pub fn display_name(&self) -> &'static str {
        match self {
            HttpServerType::Nginx => "Nginx",
            HttpServerType::FrankenPhp => "FrankenPHP",
            HttpServerType::Caddy => "Caddy",
            HttpServerType::Apache => "Apache",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BasicAuthConfig {
    pub enabled: bool,
    #[serde(default)]
    pub realm: String,
    #[serde(default)]
    pub users: Vec<BasicAuthUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasicAuthUser {
    pub username: String,
    /// bcrypt hash — never stored as plaintext
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedirectRule {
    pub from: String,
    pub to: String,
    pub code: u16,
    pub enabled: bool,
}

// ── WORDPRESS ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordPressConfig {
    #[serde(default)]
    pub multisite_enabled: bool,
    #[serde(default)]
    pub multisite_type: MultisiteType,
    pub domain_current_site: Option<String>,

    #[serde(default)]
    pub wp_debug: bool,
    #[serde(default = "default_true")]
    pub wp_debug_log: bool,
    #[serde(default)]
    pub wp_debug_display: bool,
    #[serde(default)]
    pub script_debug: bool,
    #[serde(default)]
    pub savequeries: bool,

    pub wp_home: Option<String>,
    pub wp_siteurl: Option<String>,

    #[serde(default = "default_wp_prefix")]
    pub table_prefix: String,

    pub wp_cache: Option<bool>,
    pub wp_memory_limit: Option<String>,
    pub wp_max_memory_limit: Option<String>,

    pub disallow_file_edit: Option<bool>,
    pub disallow_file_mods: Option<bool>,
    pub force_ssl_admin: Option<bool>,

    #[serde(default)]
    pub extra_config: String,
}

impl Default for WordPressConfig {
    fn default() -> Self {
        Self {
            multisite_enabled: false,
            multisite_type: MultisiteType::default(),
            domain_current_site: None,
            wp_debug: false,
            wp_debug_log: default_true(),
            wp_debug_display: false,
            script_debug: false,
            savequeries: false,
            wp_home: None,
            wp_siteurl: None,
            table_prefix: default_wp_prefix(),
            wp_cache: None,
            wp_memory_limit: None,
            wp_max_memory_limit: None,
            disallow_file_edit: None,
            disallow_file_mods: None,
            force_ssl_admin: None,
            extra_config: String::new(),
        }
    }
}

fn default_true() -> bool { true }
fn default_wp_prefix() -> String { "wp_".to_string() }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MultisiteType {
    #[default]
    Subdomain,
    Subdirectory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WpNetworkSite {
    pub blog_id: u32,
    pub url: String,
    pub title: String,
    pub registered: String,
    pub last_updated: String,
    pub public: bool,
    pub archived: bool,
    pub spam: bool,
    pub deleted: bool,
}

// ── LARAVEL ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LaravelConfig {
    pub environment: Option<String>,

    #[serde(default)]
    pub octane_enabled: bool,
    #[serde(default)]
    pub octane_server: OctaneServer,
    pub octane_host: Option<String>,
    pub octane_port: Option<u16>,
    pub octane_workers: Option<u32>,

    #[serde(default)]
    pub horizon_enabled: bool,
    #[serde(default)]
    pub telescope_enabled: bool,
    #[serde(default)]
    pub pulse_enabled: bool,
    #[serde(default)]
    pub reverb_enabled: bool,
    pub reverb_port: Option<u16>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OctaneServer {
    #[default]
    Swoole,
    #[serde(rename = "roadrunner")]
    RoadRunner,
    #[serde(rename = "frankenphp")]
    FrankenPhp,
}

impl OctaneServer {
    pub fn display_name(&self) -> &'static str {
        match self {
            OctaneServer::Swoole => "Swoole",
            OctaneServer::RoadRunner => "RoadRunner",
            OctaneServer::FrankenPhp => "FrankenPHP",
        }
    }
    pub fn cli_value(&self) -> &'static str {
        match self {
            OctaneServer::Swoole => "swoole",
            OctaneServer::RoadRunner => "roadrunner",
            OctaneServer::FrankenPhp => "frankenphp",
        }
    }
}

// ── DATABASE ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseSiteConfig {
    pub engine: Option<String>,
    #[serde(default = "default_true")]
    pub backup_before_destroy: bool,
    pub backup_path: Option<String>,
}

impl Default for DatabaseSiteConfig {
    fn default() -> Self {
        Self {
            engine: None,
            backup_before_destroy: true,
            backup_path: None,
        }
    }
}

// ── DEVELOPMENT ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DevSiteConfig {
    pub node_version: Option<String>,
    #[serde(default)]
    pub package_manager: NodePackageManager,
    pub dev_server_port: Option<u16>,
    #[serde(default)]
    pub dev_env: HashMap<String, String>,
    pub start_command: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodePackageManager {
    #[default]
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

impl NodePackageManager {
    pub fn display_name(&self) -> &'static str {
        match self {
            NodePackageManager::Npm => "npm",
            NodePackageManager::Pnpm => "pnpm",
            NodePackageManager::Yarn => "yarn",
            NodePackageManager::Bun => "bun",
        }
    }
}

// ── OCTANE PROCESS (runtime state, not config) ────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct OctaneProcess {
    pub site: String,
    pub server: OctaneServer,
    pub pid: Option<u32>,
    pub port: u16,
    pub running: bool,
}

impl Default for OctaneProcess {
    fn default() -> Self {
        Self {
            site: String::new(),
            server: OctaneServer::default(),
            pid: None,
            port: 8000,
            running: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_config_round_trips() {
        let mut cfg = SiteConfig::default();
        cfg.php.version = Some("8.3".into());
        cfg.php.ini_overrides.memory_limit = Some("512M".into());
        cfg.php.ini_overrides.max_execution_time = Some(120);
        cfg.server.server_type = HttpServerType::Nginx;
        cfg.server.custom_directives = "add_header X-Foo bar;".into();
        cfg.wordpress = Some(WordPressConfig {
            wp_debug: true,
            ..Default::default()
        });

        let serialized = toml::to_string_pretty(&cfg).expect("serialize");
        let parsed: SiteConfig = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn php_ini_overrides_is_empty_when_default() {
        assert!(PhpIniOverrides::default().is_empty());
    }

    #[test]
    fn php_ini_overrides_render_emits_header() {
        let mut o = PhpIniOverrides::default();
        o.memory_limit = Some("256M".into());
        let s = o.render();
        assert!(s.contains("Generated by Valet Manager"));
        assert!(s.contains("memory_limit = 256M"));
    }

    #[test]
    fn php_ini_overrides_render_includes_all_set_fields() {
        let o = PhpIniOverrides {
            memory_limit: Some("512M".into()),
            upload_max_filesize: Some("64M".into()),
            post_max_size: Some("64M".into()),
            max_execution_time: Some(120),
            max_input_vars: Some(2000),
            max_file_uploads: Some(20),
            session_gc_maxlifetime: Some(1440),
            date_timezone: Some("Asia/Dhaka".into()),
            extra: HashMap::new(),
        };
        let s = o.render();
        assert!(s.contains("memory_limit = 512M"));
        assert!(s.contains("upload_max_filesize = 64M"));
        assert!(s.contains("post_max_size = 64M"));
        assert!(s.contains("max_execution_time = 120"));
        assert!(s.contains("max_input_vars = 2000"));
        assert!(s.contains("max_file_uploads = 20"));
        assert!(s.contains("session.gc_maxlifetime = 1440"));
        assert!(s.contains("date.timezone = \"Asia/Dhaka\""));
    }

    #[test]
    fn wordpress_default_table_prefix_is_wp() {
        let wp = WordPressConfig::default();
        assert_eq!(wp.table_prefix, "wp_");
        assert!(wp.wp_debug_log);
    }

    #[test]
    fn database_default_backup_is_true() {
        assert!(DatabaseSiteConfig::default().backup_before_destroy);
    }

    #[test]
    fn http_server_type_serializes_lowercase() {
        let cfg = HttpServerSiteConfig { server_type: HttpServerType::FrankenPhp, ..Default::default() };
        let s = toml::to_string(&cfg).unwrap();
        assert!(s.contains("frankenphp"));
    }

    #[test]
    fn octane_server_cli_value() {
        assert_eq!(OctaneServer::Swoole.cli_value(), "swoole");
        assert_eq!(OctaneServer::RoadRunner.cli_value(), "roadrunner");
        assert_eq!(OctaneServer::FrankenPhp.cli_value(), "frankenphp");
    }

    #[test]
    fn multisite_type_default_is_subdomain() {
        assert_eq!(MultisiteType::default(), MultisiteType::Subdomain);
    }

    #[test]
    fn package_manager_default_is_npm() {
        assert_eq!(NodePackageManager::default(), NodePackageManager::Npm);
    }

    #[test]
    fn xdebug_mode_default_is_off() {
        assert_eq!(XdebugMode::default(), XdebugMode::Off);
    }

    // ── Phase 13 — framework-specific config round-trips ──────────────

    #[test]
    fn craft_config_round_trips() {
        let cfg = CraftConfig {
            environment: Some("dev".into()),
            license_key: Some("abc".into()),
            db_driver: Some("mysql".into()),
            use_project_config: true,
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: CraftConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn concretecms_config_round_trips() {
        let cfg = ConcreteCmsConfig {
            environment: Some("production".into()),
            cache_enabled: true,
            pretty_urls: false,
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: ConcreteCmsConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn drupal_config_round_trips() {
        let cfg = DrupalConfig {
            environment: Some("local".into()),
            trusted_host_patterns: vec!["^example\\.test$".into()],
            cache_bins: vec!["null".into()],
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: DrupalConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn joomla_config_round_trips() {
        let cfg = JoomlaConfig {
            error_reporting: Some("maximum".into()),
            sef_urls: true,
            debug: true,
            cache_enabled: false,
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: JoomlaConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn magento_config_round_trips() {
        let cfg = MagentoConfig {
            mode: Some("developer".into()),
            indexer_mode: Some("realtime".into()),
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: MagentoConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn octobercms_config_round_trips() {
        let cfg = OctoberCmsConfig {
            debug_mode: true,
            backend_path: Some("backend".into()),
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: OctoberCmsConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn statamic_config_round_trips() {
        let cfg = StatamicConfig {
            flat_file: true,
            git_integration: true,
            api_enabled: false,
        };
        let s = toml::to_string(&cfg).unwrap();
        let parsed: StatamicConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn statamic_default_flat_file_is_true() {
        // The serde default for flat_file is true via `default_true`.
        let toml_no_flat = "git_integration = true\napi_enabled = false\n";
        let parsed: StatamicConfig = toml::from_str(toml_no_flat).unwrap();
        assert!(parsed.flat_file);
        assert!(parsed.git_integration);
    }

    #[test]
    fn concretecms_default_pretty_urls_is_true() {
        let parsed: ConcreteCmsConfig = toml::from_str("").unwrap();
        assert!(parsed.pretty_urls);
        assert!(!parsed.cache_enabled);
    }

    #[test]
    fn site_config_includes_framework_subconfigs() {
        let mut cfg = SiteConfig::default();
        cfg.craft = Some(CraftConfig {
            environment: Some("dev".into()),
            ..Default::default()
        });
        cfg.magento = Some(MagentoConfig {
            mode: Some("developer".into()),
            ..Default::default()
        });
        let s = toml::to_string(&cfg).unwrap();
        let parsed: SiteConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.craft, cfg.craft);
        assert_eq!(parsed.magento, cfg.magento);
    }
}
