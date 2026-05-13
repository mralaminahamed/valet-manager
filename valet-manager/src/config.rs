use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        }
    }
}
