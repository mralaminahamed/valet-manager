use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum XdebugMode { Off, Debug, Profile, Coverage, Trace }

impl XdebugMode {
    pub fn ini_value(&self) -> &'static str {
        match self {
            XdebugMode::Off      => "off",
            XdebugMode::Debug    => "debug",
            XdebugMode::Profile  => "profile",
            XdebugMode::Coverage => "coverage",
            XdebugMode::Trace    => "trace",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            XdebugMode::Off      => "Off",
            XdebugMode::Debug    => "Debug",
            XdebugMode::Profile  => "Profile",
            XdebugMode::Coverage => "Coverage",
            XdebugMode::Trace    => "Trace",
        }
    }
    pub fn all() -> [XdebugMode; 5] {
        [XdebugMode::Off, XdebugMode::Debug, XdebugMode::Profile, XdebugMode::Coverage, XdebugMode::Trace]
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct XdebugConfig {
    pub php_version: String,
    pub installed: bool,
    pub mode: XdebugMode,
    pub ide_key: String,
    pub port: u16,
}

impl XdebugConfig {
    #[allow(dead_code)]
    pub fn ini_path(version: &str) -> PathBuf {
        PathBuf::from(format!("/etc/php/{}/mods-available/xdebug.ini", version))
    }
}

#[allow(dead_code)]
pub fn render_ini(cfg: &XdebugConfig) -> String {
    format!(
        "zend_extension=xdebug.so\nxdebug.mode={mode}\nxdebug.client_host=127.0.0.1\nxdebug.client_port={port}\nxdebug.idekey={ide}\nxdebug.start_with_request=trigger\n",
        mode = cfg.mode.ini_value(),
        port = cfg.port,
        ide  = cfg.ide_key,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ini_value_for_mode() {
        assert_eq!(XdebugMode::Debug.ini_value(), "debug");
        assert_eq!(XdebugMode::Off.ini_value(), "off");
    }
    #[test]
    fn all_modes_distinct() {
        let all = XdebugMode::all();
        assert_eq!(all.len(), 5);
        let unique: std::collections::HashSet<_> = all.iter().collect();
        assert_eq!(unique.len(), 5);
    }
    #[test]
    fn render_ini_contains_mode_and_port() {
        let cfg = XdebugConfig { php_version: "8.3".into(), installed: true, mode: XdebugMode::Profile, ide_key: "PHPSTORM".into(), port: 9003 };
        let s = render_ini(&cfg);
        assert!(s.contains("xdebug.mode=profile"));
        assert!(s.contains("xdebug.client_port=9003"));
        assert!(s.contains("xdebug.idekey=PHPSTORM"));
    }
    #[test]
    fn ini_path_per_version() {
        assert_eq!(XdebugConfig::ini_path("8.3"), PathBuf::from("/etc/php/8.3/mods-available/xdebug.ini"));
    }
}
