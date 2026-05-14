use notify_rust::Notification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NotificationKind {
    PhpSwitched,
    ServiceFailed,
    CreationComplete,
    SslExpiring,
    UpdateAvailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct NotificationPrefs {
    #[serde(default = "default_true")]
    pub php_switched: bool,
    #[serde(default = "default_true")]
    pub service_failed: bool,
    #[serde(default = "default_true")]
    pub creation_complete: bool,
    #[serde(default = "default_true")]
    pub ssl_expiring: bool,
    #[serde(default)]
    pub update_available: bool,
}

fn default_true() -> bool { true }

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            php_switched: true,
            service_failed: true,
            creation_complete: true,
            ssl_expiring: true,
            update_available: false,
        }
    }
}

impl NotificationPrefs {
    pub fn enabled(&self, kind: NotificationKind) -> bool {
        match kind {
            NotificationKind::PhpSwitched      => self.php_switched,
            NotificationKind::ServiceFailed    => self.service_failed,
            NotificationKind::CreationComplete => self.creation_complete,
            NotificationKind::SslExpiring      => self.ssl_expiring,
            NotificationKind::UpdateAvailable  => self.update_available,
        }
    }
}

/// Fire a desktop notification respecting the prefs.
#[allow(dead_code)]
pub fn notify(kind: NotificationKind, title: &str, body: &str, prefs: &NotificationPrefs) {
    if !prefs.enabled(kind) { return; }
    let _ = Notification::new()
        .summary(title)
        .body(body)
        .appname("Valet Manager")
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_prefs_have_update_available_off() {
        assert!(!NotificationPrefs::default().update_available);
        assert!(NotificationPrefs::default().php_switched);
    }
    #[test]
    fn enabled_respects_per_kind_flag() {
        let mut p = NotificationPrefs::default();
        p.ssl_expiring = false;
        assert!(!p.enabled(NotificationKind::SslExpiring));
        assert!(p.enabled(NotificationKind::PhpSwitched));
    }
    #[test]
    fn prefs_roundtrip_toml() {
        let p = NotificationPrefs::default();
        let s = toml::to_string(&p).unwrap();
        let r: NotificationPrefs = toml::from_str(&s).unwrap();
        assert_eq!(p, r);
    }
}
