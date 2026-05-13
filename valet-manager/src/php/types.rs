// Re-export ServiceStatus so callers don't need two imports
#[allow(unused_imports)]
pub use crate::services::monitor::ServiceStatus;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionType {
    Core,
    Bundled,
    Pecl,
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub ext_type: ExtensionType,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IniType {
    #[default]
    Cli,
    Fpm,
}

impl IniType {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            IniType::Cli => "CLI",
            IniType::Fpm => "FPM",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IniEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IniSection {
    pub name: String,
    pub entries: Vec<IniEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_type_default_is_other() {
        let ext = PhpExtension {
            name: "myext".to_string(),
            enabled: true,
            ext_type: ExtensionType::Other,
        };
        assert_eq!(ext.ext_type, ExtensionType::Other);
    }

    #[test]
    fn ini_type_default_is_cli() {
        assert_eq!(IniType::default(), IniType::Cli);
    }

    #[test]
    fn ini_section_can_hold_entries() {
        let section = IniSection {
            name: "PHP".to_string(),
            entries: vec![IniEntry {
                key: "memory_limit".to_string(),
                value: "128M".to_string(),
                comment: None,
            }],
        };
        assert_eq!(section.entries[0].key, "memory_limit");
    }
}
