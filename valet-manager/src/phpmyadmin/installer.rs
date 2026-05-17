#![allow(dead_code)]

//! phpMyAdmin installer detection.
//!
//! For Phase 12 scope: detection only reads filesystem; installation paths
//! are stubbed and surface user-facing messages instead of running `sudo apt`.

use std::path::{Path, PathBuf};

/// Common locations Debian-family distros install phpMyAdmin to.
pub const COMMON_PATHS: &[&str] = &[
    "/usr/share/phpmyadmin",
    "/usr/local/share/phpmyadmin",
    "/opt/phpmyadmin",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmaInstall {
    pub path: PathBuf,
    pub version: Option<String>,
}

/// Look for an installed phpMyAdmin in known locations.
pub async fn detect() -> Option<PmaInstall> {
    for candidate in COMMON_PATHS {
        let p = PathBuf::from(candidate);
        if is_pma_directory(&p).await {
            let version = read_version(&p).await;
            return Some(PmaInstall { path: p, version });
        }
    }
    None
}

pub async fn is_installed() -> bool {
    detect().await.is_some()
}

async fn is_pma_directory(path: &Path) -> bool {
    if !tokio::fs::metadata(path).await.map(|m| m.is_dir()).unwrap_or(false) {
        return false;
    }
    // The canonical entry point.
    tokio::fs::metadata(path.join("index.php"))
        .await
        .map(|m| m.is_file())
        .unwrap_or(false)
}

/// Reads README or composer.json for a version hint. Best-effort.
pub async fn read_version(path: &Path) -> Option<String> {
    if let Ok(content) = tokio::fs::read_to_string(path.join("README")).await {
        return parse_readme_version(&content);
    }
    if let Ok(content) = tokio::fs::read_to_string(path.join("composer.json")).await {
        return parse_composer_version(&content);
    }
    None
}

pub(crate) fn parse_readme_version(content: &str) -> Option<String> {
    // README typically starts with: "phpMyAdmin - Version 5.2.1"
    let re = regex::Regex::new(r"(?i)version\s+([0-9]+\.[0-9]+(?:\.[0-9]+)?)").ok()?;
    re.captures(content).map(|c| c[1].to_string())
}

pub(crate) fn parse_composer_version(content: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(content).ok()?;
    v.get("version").and_then(|x| x.as_str()).map(|s| s.to_string())
}

/// Latest known stable upstream version. Stub used by the UI when offline.
pub fn get_latest_version() -> &'static str {
    "5.2.1"
}

/// Stub: installation requires root. For SCOPE: surface a friendly message.
pub fn install_command_hint() -> &'static str {
    "Run: sudo apt-get install -y phpmyadmin"
}

pub fn manual_install_hint() -> &'static str {
    "Download from https://www.phpmyadmin.net/downloads/ and extract to /usr/share/phpmyadmin"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_from_readme() {
        let s = "phpMyAdmin - A web interface for MySQL/MariaDB\nVersion 5.2.1\n";
        assert_eq!(parse_readme_version(s).as_deref(), Some("5.2.1"));
    }

    #[test]
    fn parses_version_from_composer_json() {
        let s = r#"{ "name": "phpmyadmin/phpmyadmin", "version": "5.2.2" }"#;
        assert_eq!(parse_composer_version(s).as_deref(), Some("5.2.2"));
    }

    #[test]
    fn parse_readme_returns_none_when_absent() {
        assert!(parse_readme_version("no version here").is_none());
    }

    #[test]
    fn common_paths_includes_usr_share() {
        assert!(COMMON_PATHS.contains(&"/usr/share/phpmyadmin"));
    }

    #[test]
    fn install_hints_are_non_empty() {
        assert!(!install_command_hint().is_empty());
        assert!(!manual_install_hint().is_empty());
    }
}
