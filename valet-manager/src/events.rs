use crate::php::detector::PhpVersion;
use crate::services::monitor::ManagedService;
use crate::valet::variant::{ValetPaths, ValetVariant};

#[derive(Debug)]
pub enum AppEvent {
    ServiceStatusUpdated(Vec<ManagedService>),
    PhpVersionsRefreshed(Vec<PhpVersion>),
    ValetDetected(ValetVariant, ValetPaths),
    Error(String),
}
