use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    pub accent_index: usize,
    pub density: String,
    pub sidebar_width: f32,
    pub window_radius: f32,
    pub show_stats: bool,
    pub show_badges: bool,
    pub dot_glow: bool,
    pub mono_numerals: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_index: 0,
            density: "compact".to_string(),
            sidebar_width: 220.0,
            window_radius: 12.0,
            show_stats: true,
            show_badges: true,
            dot_glow: true,
            mono_numerals: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct AppConfig {
    #[serde(default = "default_editor")]
    pub editor_command: String,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default = "default_file_manager")]
    pub file_manager: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_poll_interval")]
    pub service_poll_interval_secs: u64,
    #[serde(default = "default_debounce")]
    pub site_scan_debounce_ms: u64,
    #[serde(default = "default_parent_dir")]
    pub default_parent_directory: String,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default = "default_ttl")]
    pub version_registry_ttl_hours: u64,
    #[serde(default = "default_true")]
    pub version_registry_auto_refresh: bool,
    #[serde(default)]
    pub notifications: crate::notifications::NotificationPrefs,
    #[serde(default)]
    pub skip_version: Option<String>,
    // Phase 12 — phpMyAdmin
    /// Lazily-generated phpMyAdmin blowfish_secret. None means "regenerate on first use".
    #[serde(default)]
    pub blowfish_secret: Option<String>,
    /// Default MySQL user used when site .env/wp-config does not provide one.
    #[serde(default)]
    pub mysql_user: String,
    /// Default MySQL password used when site .env/wp-config does not provide one.
    #[serde(default)]
    pub mysql_pass: String,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    // M5 — Behavior / PHP / TLS / Paths / Updates
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_true")]
    pub show_tray_icon: bool,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    #[serde(default = "default_true")]
    pub auto_restart_services: bool,
    #[serde(default = "default_php")]
    pub default_php_version: String,
    #[serde(default = "default_memory")]
    pub default_memory_limit: String,
    #[serde(default)]
    pub xdebug_enabled: bool,
    #[serde(default = "default_true")]
    pub auto_renew_certs: bool,
    #[serde(default = "default_true")]
    pub auto_update: bool,
    #[serde(default = "default_config_dir")]
    pub config_directory: String,
    #[serde(default = "default_log_dir")]
    pub log_directory: String,
    #[serde(default = "default_composer")]
    pub composer_binary: String,
}

fn default_editor() -> String { "code".to_string() }
fn default_terminal() -> String { "gnome-terminal".to_string() }
fn default_file_manager() -> String { "nautilus".to_string() }
fn default_theme() -> String { "dark".to_string() }
fn default_font_size() -> f32 { 14.0 }
fn default_poll_interval() -> u64 { 5 }
fn default_debounce() -> u64 { 500 }
fn default_parent_dir() -> String { "~/Sites".to_string() }
fn default_ttl() -> u64 { 24 }
fn default_true() -> bool { true }
fn default_php() -> String { "8.3".to_string() }
fn default_memory() -> String { "512M".to_string() }
fn default_config_dir() -> String { "~/.config/valet-manager".to_string() }
fn default_log_dir() -> String { "~/.local/share/valet-manager/logs".to_string() }
fn default_composer() -> String { "/usr/local/bin/composer".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            editor_command: default_editor(),
            terminal: default_terminal(),
            file_manager: default_file_manager(),
            theme: default_theme(),
            font_size: default_font_size(),
            service_poll_interval_secs: default_poll_interval(),
            site_scan_debounce_ms: default_debounce(),
            default_parent_directory: default_parent_dir(),
            favorites: Vec::new(),
            version_registry_ttl_hours: default_ttl(),
            version_registry_auto_refresh: true,
            notifications: crate::notifications::NotificationPrefs::default(),
            skip_version: None,
            blowfish_secret: None,
            mysql_user: String::new(),
            mysql_pass: String::new(),
            appearance: AppearanceConfig::default(),
            launch_at_login: false,
            show_tray_icon: true,
            confirm_destructive: true,
            auto_restart_services: true,
            default_php_version: default_php(),
            default_memory_limit: default_memory(),
            xdebug_enabled: false,
            auto_renew_certs: true,
            auto_update: true,
            config_directory: default_config_dir(),
            log_directory: default_log_dir(),
            composer_binary: default_composer(),
        }
    }
}

/// Path of the onboarding sentinel file (touched on completion).
#[allow(dead_code)]
pub fn onboarded_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config")
        .join("valet-manager")
        .join(".onboarded")
}

#[allow(dead_code)]
pub fn is_onboarded() -> bool {
    onboarded_path().exists()
}

#[allow(dead_code)]
pub fn mark_onboarded() -> anyhow::Result<()> {
    let path = onboarded_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, b"1")?;
    Ok(())
}

#[allow(dead_code)]
pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config")
        .join("valet-manager")
        .join("config.toml")
}

#[allow(dead_code)]
pub fn load() -> AppConfig {
    let path = config_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

#[allow(dead_code)]
pub fn save(cfg: &AppConfig) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_editor_is_code() {
        assert_eq!(AppConfig::default().editor_command, "code");
    }

    #[test]
    fn default_theme_is_dark() {
        assert_eq!(AppConfig::default().theme, "dark");
    }

    #[test]
    fn default_font_size_is_14() {
        assert!((AppConfig::default().font_size - 14.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_poll_interval_is_5() {
        assert_eq!(AppConfig::default().service_poll_interval_secs, 5);
    }

    #[test]
    fn toml_roundtrip() {
        let cfg = AppConfig::default();
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(cfg.editor_command, restored.editor_command);
        assert!((cfg.font_size - restored.font_size).abs() < f32::EPSILON);
        assert_eq!(cfg.service_poll_interval_secs, restored.service_poll_interval_secs);
    }

    #[test]
    fn default_version_registry_ttl_is_24() {
        assert_eq!(AppConfig::default().version_registry_ttl_hours, 24);
    }

    #[test]
    fn default_version_registry_auto_refresh_is_true() {
        assert!(AppConfig::default().version_registry_auto_refresh);
    }

    #[test]
    fn config_path_ends_with_config_toml() {
        let p = config_path();
        assert!(p.to_string_lossy().ends_with("config.toml"));
        assert!(p.to_string_lossy().contains("valet-manager"));
    }

    #[test]
    fn default_notification_prefs_roundtrip_via_appconfig() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let r: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.notifications, r.notifications);
        assert!(cfg.notifications.php_switched);
        assert!(!cfg.notifications.update_available);
    }

    #[test]
    fn onboarded_path_ends_with_sentinel() {
        let p = onboarded_path();
        assert!(p.to_string_lossy().ends_with(".onboarded"));
    }

    #[test]
    fn appearance_config_defaults() {
        let a = AppearanceConfig::default();
        assert_eq!(a.density, "compact");
        assert!((a.sidebar_width - 220.0).abs() < f32::EPSILON);
        assert!(a.show_stats);
        assert!(a.dot_glow);
        assert_eq!(a.accent_index, 0);
    }

    #[test]
    fn app_config_has_appearance() {
        let c = AppConfig::default();
        assert_eq!(c.appearance.accent_index, 0);
        assert_eq!(c.appearance.density, "compact");
    }

    #[test]
    fn appearance_config_toml_roundtrip() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let r: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.appearance.accent_index, r.appearance.accent_index);
        assert!((cfg.appearance.sidebar_width - r.appearance.sidebar_width).abs() < f32::EPSILON);
    }

    #[test]
    fn new_appconfig_fields_have_correct_defaults() {
        let cfg = AppConfig::default();
        assert!(!cfg.launch_at_login);
        assert!(cfg.show_tray_icon);
        assert!(cfg.confirm_destructive);
        assert!(cfg.auto_restart_services);
        assert_eq!(cfg.default_php_version, "8.3");
        assert_eq!(cfg.default_memory_limit, "512M");
        assert!(!cfg.xdebug_enabled);
        assert!(cfg.auto_renew_certs);
        assert!(cfg.auto_update);
        assert_eq!(cfg.config_directory, "~/.config/valet-manager");
        assert_eq!(cfg.log_directory, "~/.local/share/valet-manager/logs");
        assert_eq!(cfg.composer_binary, "/usr/local/bin/composer");
    }

    #[test]
    fn appconfig_new_fields_toml_roundtrip() {
        let cfg = AppConfig::default();
        let s = toml::to_string(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.default_php_version, back.default_php_version);
        assert_eq!(cfg.composer_binary, back.composer_binary);
    }

    #[test]
    fn appconfig_omitted_new_fields_deserialize_to_defaults() {
        let toml = r#"tld = "test""#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert!(cfg.show_tray_icon);
        assert_eq!(cfg.default_php_version, "8.3");
    }
}
