#![allow(dead_code, unused_variables)]

use crate::site_config::models::HttpServerSiteConfig;
use crate::valet::site_scanner::ValetSite;

pub async fn is_installed() -> bool {
    which::which("apache2").is_ok() || which::which("httpd").is_ok()
}

/// Stub: renders an Apache VirtualHost and writes to /etc/apache2/sites-available.
pub async fn write_vhost(
    site: &ValetSite,
    config: &HttpServerSiteConfig,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn read_htaccess(site: &ValetSite) -> Option<String> {
    tokio::fs::read_to_string(site.path.join(".htaccess")).await.ok()
}

pub async fn write_htaccess(site: &ValetSite, content: &str) -> anyhow::Result<()> {
    let path = site.path.join(".htaccess");
    tokio::fs::write(&path, content).await?;
    Ok(())
}

pub async fn reload() -> anyhow::Result<()> {
    Ok(())
}
