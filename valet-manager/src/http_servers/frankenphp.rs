#![allow(dead_code, unused_variables)]

use crate::site_config::models::{HttpServerSiteConfig, LaravelConfig};
use crate::valet::site_scanner::ValetSite;

pub async fn is_installed() -> bool {
    which::which("frankenphp").is_ok()
}

/// Stub: would render a Caddyfile and write to /etc/frankenphp/conf.d/{name}.caddy.
pub async fn write_standalone_config(
    site: &ValetSite,
    config: &HttpServerSiteConfig,
) -> anyhow::Result<()> {
    Ok(())
}

/// Render a Caddyfile string for FrankenPHP Octane mode.
pub fn generate_octane_caddyfile(site: &ValetSite, config: &LaravelConfig) -> String {
    let port = config.octane_port.unwrap_or(8000);
    format!(
        "{} {{\n    reverse_proxy 127.0.0.1:{}\n}}\n",
        site.domain, port,
    )
}

pub async fn reload() -> anyhow::Result<()> {
    Ok(())
}
