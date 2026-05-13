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
