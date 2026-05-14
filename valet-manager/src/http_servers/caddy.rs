#![allow(dead_code, unused_variables)]

use crate::site_config::models::HttpServerSiteConfig;
use crate::valet::site_scanner::ValetSite;

pub async fn is_installed() -> bool {
    which::which("caddy").is_ok()
}

/// Stub: renders a per-site Caddyfile and writes to /etc/caddy/sites/{name}.caddy.
pub async fn write_site_config(
    site: &ValetSite,
    config: &HttpServerSiteConfig,
) -> anyhow::Result<()> {
    Ok(())
}

pub async fn reload() -> anyhow::Result<()> {
    Ok(())
}
