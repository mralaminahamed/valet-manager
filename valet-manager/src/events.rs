use crate::php::detector::PhpVersion;
use crate::php::types::{IniSection, IniType, PhpExtension};
use crate::services::monitor::{ManagedService, ServiceStatus};
use crate::valet::variant::{ValetPaths, ValetVariant};

#[derive(Debug)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::php::types::IniType;

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
}
