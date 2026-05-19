#![allow(dead_code)]

//! phpMyAdmin per-site service (Phase 12).
//!
//! Orchestrates credential resolution, config.inc.php generation,
//! nginx sentinel-block injection, and per-site state reporting.

pub mod installer;
pub mod config_generator;
pub mod nginx_integration;
pub mod global_site;

use std::path::PathBuf;

use crate::config::AppConfig;
use crate::creator::output_streamer::{OutputLine, Stream};
use crate::site_config::models::{
    PhpMyAdminSiteConfig, PmaAccessMode, PmaDbScope, SiteConfig,
};
use crate::state::app_state::PmaSiteStatus;
use crate::valet::site_scanner::ValetSite;
use crate::valet::variant::ValetPaths;

#[derive(Debug, Clone)]
pub struct ResolvedDb {
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub source: DbSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbSource {
    Env,
    WpConfig,
    AppDefaults,
}

impl DbSource {
    pub fn label(&self) -> &'static str {
        match self {
            DbSource::Env => "Detected from .env",
            DbSource::WpConfig => "Detected from wp-config.php",
            DbSource::AppDefaults => "Using app defaults",
        }
    }
}

/// Walks the site to find DB credentials. Order: `.env` → `wp-config.php` → app defaults.
pub async fn resolve_db_credentials(
    site: &ValetSite,
    site_config: &SiteConfig,
    app_config: &AppConfig,
) -> ResolvedDb {
    let pma = &site_config.phpmyadmin;

    // 1) .env
    let env_path = site.path.join(".env");
    if env_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&env_path).await {
            let entries = crate::env_file::parse(&content);
            let lookup = |key: &str| -> Option<String> {
                entries.iter().find_map(|e| match e {
                    crate::env_file::EnvEntry::Kv { key: k, value, .. } if k == key => {
                        Some(value.clone())
                    }
                    _ => None,
                })
            };
            if lookup("DB_DATABASE").is_some()
                || lookup("DB_USERNAME").is_some()
                || lookup("DB_PASSWORD").is_some()
            {
                let name = pma
                    .db_name_override
                    .clone()
                    .or_else(|| lookup("DB_DATABASE"))
                    .unwrap_or_else(|| format!("{}_db", site.name));
                let user = pma
                    .db_user_override
                    .clone()
                    .or_else(|| lookup("DB_USERNAME"))
                    .unwrap_or_else(|| default_user(app_config));
                let password = lookup("DB_PASSWORD").unwrap_or_else(|| app_config.mysql_pass.clone());
                let host = lookup("DB_HOST").unwrap_or_else(|| "127.0.0.1".into());
                let port = lookup("DB_PORT").and_then(|p| p.parse().ok()).unwrap_or(3306);
                return ResolvedDb { name, user, password, host, port, source: DbSource::Env };
            }
        }
    }

    // 2) wp-config.php
    let wp_path = site.path.join("wp-config.php");
    if wp_path.exists() {
        if let Ok(map) = crate::wordpress::config_editor::read_constants(&site.path).await {
            if map.contains_key("DB_NAME") || map.contains_key("DB_USER") {
                let name = pma
                    .db_name_override
                    .clone()
                    .or_else(|| map.get("DB_NAME").cloned())
                    .unwrap_or_else(|| format!("wp_{}", site.name));
                let user = pma
                    .db_user_override
                    .clone()
                    .or_else(|| map.get("DB_USER").cloned())
                    .unwrap_or_else(|| default_user(app_config));
                let password = map
                    .get("DB_PASSWORD")
                    .cloned()
                    .unwrap_or_else(|| app_config.mysql_pass.clone());
                let host = map.get("DB_HOST").cloned().unwrap_or_else(|| "127.0.0.1".into());
                return ResolvedDb {
                    name,
                    user,
                    password,
                    host,
                    port: 3306,
                    source: DbSource::WpConfig,
                };
            }
        }
    }

    // 3) App defaults
    let name = pma
        .db_name_override
        .clone()
        .unwrap_or_else(|| format!("{}_db", site.name));
    let user = pma
        .db_user_override
        .clone()
        .unwrap_or_else(|| default_user(app_config));
    ResolvedDb {
        name,
        user,
        password: app_config.mysql_pass.clone(),
        host: "127.0.0.1".into(),
        port: 3306,
        source: DbSource::AppDefaults,
    }
}

fn default_user(app_config: &AppConfig) -> String {
    if app_config.mysql_user.is_empty() {
        "root".into()
    } else {
        app_config.mysql_user.clone()
    }
}

/// Compute the per-site access URL based on the configured mode.
pub fn access_url(
    site_domain: &str,
    pma: &PhpMyAdminSiteConfig,
    scope: PmaDbScope,
    tld: &str,
) -> String {
    match (pma.access_mode, scope) {
        (_, PmaDbScope::AllDatabases) => format!("https://phpmyadmin.{}/", tld),
        (PmaAccessMode::GlobalOnly, _) => format!("https://phpmyadmin.{}/", tld),
        (PmaAccessMode::Subdomain, _) => nginx_integration::subdomain_url(site_domain),
        (PmaAccessMode::PathAlias, _) => {
            let alias = pma.path_alias.trim();
            let alias = if alias.starts_with('/') {
                alias.to_string()
            } else {
                format!("/{}", alias)
            };
            format!("https://{}{}/", site_domain, alias.trim_end_matches('/'))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigureRequest<'a> {
    pub site: &'a ValetSite,
    pub site_config: &'a SiteConfig,
    pub app_config: &'a AppConfig,
    pub valet_paths: &'a ValetPaths,
    pub tld: &'a str,
    pub pma_install_path: PathBuf,
    pub php_fpm_socket: PathBuf,
    pub blowfish_secret: String,
}

/// Orchestrate per-site configuration. Streams progress through `out_tx`
/// (capped externally) and returns the final status.
pub async fn configure_for_site(
    req: ConfigureRequest<'_>,
    out_tx: tokio::sync::mpsc::Sender<OutputLine>,
) -> anyhow::Result<PmaSiteStatus> {
    let pma = &req.site_config.phpmyadmin;
    log_line(&out_tx, Stream::Stdout, format!("Configuring phpMyAdmin for '{}'…", req.site.name)).await;

    // Resolve credentials.
    let resolved =
        resolve_db_credentials(req.site, req.site_config, req.app_config).await;
    log_line(
        &out_tx,
        Stream::Stdout,
        format!(
            "Detected DB: name='{}' user='{}' host='{}' port={} ({})",
            resolved.name, resolved.user, resolved.host, resolved.port, resolved.source.label()
        ),
    ).await;

    // Render config (no secrets logged).
    let db_name = match pma.db_scope {
        PmaDbScope::SiteOnly => Some(resolved.name.clone()),
        PmaDbScope::AllDatabases => None,
    };
    let server = config_generator::PmaServerConfig {
        host: resolved.host.clone(),
        port: resolved.port,
        user: resolved.user.clone(),
        password: resolved.password.clone(),
        db_name,
    };
    let content = config_generator::render_site_config(&req.blowfish_secret, &server);

    // Write config.inc.php (0640).
    let cfg_path = config_generator::write_site_config(&req.site.name, &content).await?;
    log_line(
        &out_tx,
        Stream::Stdout,
        format!("Wrote {} (mode 0640)", cfg_path.display()),
    ).await;

    // Inject nginx block when access mode requires it.
    if matches!(pma.access_mode, PmaAccessMode::PathAlias) {
        let body = nginx_integration::render_path_alias_block(
            &pma.path_alias,
            &req.pma_install_path,
            &cfg_path,
            &req.php_fpm_socket,
        );
        nginx_integration::inject_into_site(req.site, &body, req.valet_paths).await?;
        log_line(
            &out_tx,
            Stream::Stdout,
            format!("Injected nginx phpmyadmin block (idempotent) for {}", req.site.name),
        ).await;
    } else {
        log_line(
            &out_tx,
            Stream::Stdout,
            format!("Skipping nginx injection (mode = {})", pma.access_mode.display_name()),
        ).await;
    }

    let url = access_url(&req.site.domain, pma, pma.db_scope, req.tld);

    Ok(PmaSiteStatus {
        enabled: true,
        access_url: url,
        config_path: cfg_path,
        db_name: resolved.name,
        access_mode: pma.access_mode,
        last_configured: Some(chrono::Local::now()),
    })
}

async fn log_line(tx: &tokio::sync::mpsc::Sender<OutputLine>, stream: Stream, text: String) {
    let _ = tx
        .send(OutputLine {
            text,
            stream,
            timestamp: chrono::Local::now(),
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site_config::models::{PhpMyAdminSiteConfig, PmaAccessMode, PmaDbScope};

    fn pma_default() -> PhpMyAdminSiteConfig {
        PhpMyAdminSiteConfig::default()
    }

    #[test]
    fn access_url_path_alias_uses_site_domain() {
        let mut pma = pma_default();
        pma.access_mode = PmaAccessMode::PathAlias;
        pma.path_alias = "/phpmyadmin".into();
        let u = access_url("myapp.test", &pma, PmaDbScope::SiteOnly, "test");
        assert_eq!(u, "https://myapp.test/phpmyadmin/");
    }

    #[test]
    fn access_url_subdomain_prefixes_pma() {
        let mut pma = pma_default();
        pma.access_mode = PmaAccessMode::Subdomain;
        let u = access_url("myapp.test", &pma, PmaDbScope::SiteOnly, "test");
        assert_eq!(u, "https://pma.myapp.test/");
    }

    #[test]
    fn access_url_global_only_uses_global() {
        let mut pma = pma_default();
        pma.access_mode = PmaAccessMode::GlobalOnly;
        let u = access_url("myapp.test", &pma, PmaDbScope::SiteOnly, "test");
        assert_eq!(u, "https://phpmyadmin.test/");
    }

    #[test]
    fn access_url_all_databases_forces_global() {
        let mut pma = pma_default();
        pma.access_mode = PmaAccessMode::PathAlias;
        let u = access_url("myapp.test", &pma, PmaDbScope::AllDatabases, "test");
        assert_eq!(u, "https://phpmyadmin.test/");
    }

    #[test]
    fn access_url_path_alias_without_leading_slash_normalizes() {
        let mut pma = pma_default();
        pma.access_mode = PmaAccessMode::PathAlias;
        pma.path_alias = "pma".into();
        let u = access_url("myapp.test", &pma, PmaDbScope::SiteOnly, "test");
        assert_eq!(u, "https://myapp.test/pma/");
    }

    #[test]
    fn db_source_labels_distinct() {
        assert_ne!(DbSource::Env.label(), DbSource::WpConfig.label());
        assert_ne!(DbSource::Env.label(), DbSource::AppDefaults.label());
    }

    fn make_site(path: std::path::PathBuf) -> ValetSite {
        ValetSite {
            name: "myapp".into(),
            domain: "myapp.test".into(),
            path,
            site_type: crate::valet::site_scanner::SiteType::Parked,
            php_version: None,
            framework: crate::ui::DetectedFramework::Unknown,
            is_secured: false,
            is_favorite: false,
            ssl_expiry: None,
            tls_expiry_days: None,
            status: crate::valet::site_scanner::SiteStatus::Unknown,
            last_hit: None,
            proxy_target: None,
        }
    }

    #[tokio::test]
    async fn resolve_credentials_prefers_env() {
        let tmp = std::env::temp_dir().join(format!(
            "vm-pma-env-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join(".env"),
            "DB_DATABASE=envdb\nDB_USERNAME=envuser\nDB_PASSWORD=envpw\nDB_HOST=db.local\nDB_PORT=3307\n",
        ).unwrap();
        let site = make_site(tmp.clone());
        let sc = SiteConfig::default();
        let ac = AppConfig::default();
        let r = resolve_db_credentials(&site, &sc, &ac).await;
        assert_eq!(r.source, DbSource::Env);
        assert_eq!(r.name, "envdb");
        assert_eq!(r.user, "envuser");
        assert_eq!(r.password, "envpw");
        assert_eq!(r.host, "db.local");
        assert_eq!(r.port, 3307);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn resolve_credentials_falls_back_to_wp_config() {
        let tmp = std::env::temp_dir().join(format!(
            "vm-pma-wp-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("wp-config.php"),
            "<?php\ndefine('DB_NAME', 'wpdb');\ndefine('DB_USER', 'wpuser');\ndefine('DB_PASSWORD', 'wppw');\ndefine('DB_HOST', 'localhost');\n",
        ).unwrap();
        let site = make_site(tmp.clone());
        let sc = SiteConfig::default();
        let ac = AppConfig::default();
        let r = resolve_db_credentials(&site, &sc, &ac).await;
        assert_eq!(r.source, DbSource::WpConfig);
        assert_eq!(r.name, "wpdb");
        assert_eq!(r.user, "wpuser");
        assert_eq!(r.password, "wppw");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn resolve_credentials_uses_app_defaults_when_nothing_present() {
        let tmp = std::env::temp_dir().join(format!(
            "vm-pma-def-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let site = make_site(tmp.clone());
        let sc = SiteConfig::default();
        let mut ac = AppConfig::default();
        ac.mysql_user = "rooty".into();
        ac.mysql_pass = "letmein".into();
        let r = resolve_db_credentials(&site, &sc, &ac).await;
        assert_eq!(r.source, DbSource::AppDefaults);
        assert_eq!(r.name, "myapp_db");
        assert_eq!(r.user, "rooty");
        assert_eq!(r.password, "letmein");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn resolve_credentials_honors_db_name_override() {
        let tmp = std::env::temp_dir().join(format!(
            "vm-pma-override-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join(".env"), "DB_DATABASE=envdb\nDB_USERNAME=envuser\n").unwrap();
        let site = make_site(tmp.clone());
        let mut sc = SiteConfig::default();
        sc.phpmyadmin.db_name_override = Some("overridden".into());
        let ac = AppConfig::default();
        let r = resolve_db_credentials(&site, &sc, &ac).await;
        assert_eq!(r.name, "overridden");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
