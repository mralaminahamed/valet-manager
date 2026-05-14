#![allow(dead_code)]

use crate::site_config::models::HttpServerType;

/// Detect which HTTP servers are installed locally.
pub async fn detect_installed() -> Vec<HttpServerType> {
    let mut out = Vec::new();
    if which::which("nginx").is_ok() || std::path::Path::new("/usr/sbin/nginx").exists() {
        out.push(HttpServerType::Nginx);
    }
    if which::which("frankenphp").is_ok() {
        out.push(HttpServerType::FrankenPhp);
    }
    if which::which("caddy").is_ok() {
        out.push(HttpServerType::Caddy);
    }
    if which::which("apache2").is_ok() || which::which("httpd").is_ok() {
        out.push(HttpServerType::Apache);
    }
    out
}

/// Inspect site config files to guess which server is currently fronting a site.
/// Defaults to Nginx if no Caddy/Apache file is found.
pub async fn detect_site_server(site_name: &str) -> HttpServerType {
    if tokio::fs::metadata(format!("/etc/caddy/sites/{site_name}.caddy")).await.is_ok() {
        return HttpServerType::Caddy;
    }
    if tokio::fs::metadata(format!("/etc/apache2/sites-enabled/{site_name}.conf")).await.is_ok() {
        return HttpServerType::Apache;
    }
    if tokio::fs::metadata(format!("/etc/frankenphp/conf.d/{site_name}.caddy")).await.is_ok() {
        return HttpServerType::FrankenPhp;
    }
    HttpServerType::Nginx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detect_installed_is_a_vec() {
        // We can't assume any server is installed in CI; just confirm it returns
        // without panic and yields a Vec.
        let v = detect_installed().await;
        let _len = v.len();
    }

    #[tokio::test]
    async fn detect_site_server_defaults_to_nginx() {
        // Use a site name that definitely doesn't exist in /etc.
        let s = detect_site_server("vm-no-such-site-xyz").await;
        assert_eq!(s, HttpServerType::Nginx);
    }
}
