#![allow(dead_code)]

use std::path::Path;

use crate::site_config::models::{LaravelConfig, OctaneProcess};
use crate::valet::site_scanner::ValetSite;

pub async fn is_octane_installed(site_path: &Path) -> bool {
    let composer = site_path.join("composer.json");
    let content = tokio::fs::read_to_string(&composer).await.unwrap_or_default();
    content.contains("laravel/octane")
}

/// Stub: produces an OctaneProcess record without actually spawning a child.
/// A future iteration will spawn `php artisan octane:start` and capture PID.
pub async fn start(
    site: &ValetSite,
    config: &LaravelConfig,
    _php_bin: &str,
) -> anyhow::Result<OctaneProcess> {
    Ok(OctaneProcess {
        site: site.name.clone(),
        server: config.octane_server,
        pid: None,
        port: config.octane_port.unwrap_or(8000),
        running: false,
    })
}

/// Stub: marks the process stopped. No-op if not actually started.
pub async fn stop(_site_name: &str, _process: &OctaneProcess) -> anyhow::Result<()> {
    Ok(())
}

pub async fn detect_installed_packages(site_path: &Path) -> super::packages::LaravelPackages {
    super::packages::detect(site_path).await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site_config::models::OctaneServer;

    #[tokio::test]
    async fn start_returns_process_with_configured_port() {
        let site = ValetSite {
            name: "demo".into(),
            domain: "demo.test".into(),
            path: std::env::temp_dir(),
            site_type: crate::valet::site_scanner::SiteType::Parked,
            framework: crate::ui::DetectedFramework::Laravel,
            php_version: None,
            is_secured: false,
            ssl_expiry: None,
            is_favorite: false,
        };
        let mut cfg = LaravelConfig::default();
        cfg.octane_server = OctaneServer::Swoole;
        cfg.octane_port = Some(9001);
        let p = start(&site, &cfg, "php").await.unwrap();
        assert_eq!(p.site, "demo");
        assert_eq!(p.port, 9001);
        assert_eq!(p.server, OctaneServer::Swoole);
        assert!(!p.running);
    }

    #[tokio::test]
    async fn stop_does_not_error() {
        let proc = OctaneProcess::default();
        stop("demo", &proc).await.unwrap();
    }

    #[tokio::test]
    async fn is_octane_installed_false_for_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("vm-octane-detect-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(&tmp).await.unwrap();
        assert!(!is_octane_installed(&tmp).await);
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }

    #[tokio::test]
    async fn is_octane_installed_true_when_in_composer() {
        let tmp = std::env::temp_dir().join(format!("vm-octane-detect-true-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(&tmp).await.unwrap();
        tokio::fs::write(tmp.join("composer.json"), r#"{"require":{"laravel/octane":"^2.0"}}"#).await.unwrap();
        assert!(is_octane_installed(&tmp).await);
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }
}
