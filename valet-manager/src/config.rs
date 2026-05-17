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
    pub editor_command: String,
    pub terminal: String,
    pub file_manager: String,
    pub theme: String,
    pub font_size: f32,
    pub service_poll_interval_secs: u64,
    pub site_scan_debounce_ms: u64,
    pub default_parent_directory: String,
    pub favorites: Vec<String>,
    pub version_registry_ttl_hours: u64,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            editor_command: "code".to_string(),
            terminal: "gnome-terminal".to_string(),
            file_manager: "nautilus".to_string(),
            theme: "dark".to_string(),
            font_size: 14.0,
            service_poll_interval_secs: 5,
            site_scan_debounce_ms: 500,
            default_parent_directory: "~/Sites".to_string(),
            favorites: Vec::new(),
            version_registry_ttl_hours: 24,
            version_registry_auto_refresh: true,
            notifications: crate::notifications::NotificationPrefs::default(),
            skip_version: None,
            blowfish_secret: None,
            mysql_user: String::new(),
            mysql_pass: String::new(),
            appearance: AppearanceConfig::default(),
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
}
