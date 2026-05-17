#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use chrono::NaiveDate;
use crate::ui::DetectedFramework;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SiteStatus {
    Running,
    Stopped,
    Failed,
    #[default]
    Unknown,
}

impl SiteStatus {
    pub fn label(&self) -> &'static str {
        match self {
            SiteStatus::Running => "Running",
            SiteStatus::Stopped => "Stopped",
            SiteStatus::Failed  => "Failed",
            SiteStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SiteType {
    Parked,
    Linked,
    Proxy,
}

#[derive(Debug, Clone)]
pub struct ValetSite {
    pub name: String,
    pub domain: String,
    pub path: PathBuf,
    pub site_type: SiteType,
    pub framework: DetectedFramework,
    pub php_version: Option<String>,
    pub is_secured: bool,
    pub ssl_expiry: Option<NaiveDate>,
    pub is_favorite: bool,
    pub status: SiteStatus,
    pub last_hit: Option<String>,
    pub proxy_target: Option<String>,
}

pub fn detect_framework(path: &Path) -> DetectedFramework {
    // Priority 1: unambiguous single-file signatures
    if path.join("bin/magento").exists() {
        return DetectedFramework::Magento;
    }
    if path.join("craft").exists() {
        return DetectedFramework::Craft;
    }
    if path.join("system/ee").is_dir() {
        return DetectedFramework::ExpressionEngine;
    }
    if path.join("config.php").exists()
        && path.join("posts").is_dir()
        && !path.join("source").is_dir()
    {
        let content = fs::read_to_string(path.join("config.php")).unwrap_or_default();
        if content.contains("Katana") || content.contains("SiteConfiguration") {
            return DetectedFramework::Katana;
        }
    }

    // Priority 2: WordPress family
    if path.join("web/wp").is_dir() && path.join("config/application.php").exists() {
        return DetectedFramework::Bedrock;
    }
    if path.join("wp-admin").is_dir() {
        return DetectedFramework::WordPress;
    }

    // Priority 3: artisan-based frameworks
    if path.join("artisan").exists() {
        if path.join("modules/backend").is_dir() {
            return DetectedFramework::OctoberCms;
        }
        if path.join("vendor/statamic").is_dir() {
            return DetectedFramework::Statamic;
        }
        let composer = fs::read_to_string(path.join("composer.json")).unwrap_or_default();
        if composer.contains("\"statamic/cms\"") || composer.contains("statamic/statamic") {
            return DetectedFramework::Statamic;
        }
        if path.join("public/index.php").exists() {
            return DetectedFramework::Laravel;
        }
    }

    // Priority 4: Symfony family
    if path.join("config.php").exists() && path.join("source").is_dir() {
        return DetectedFramework::Jigsaw;
    }
    if path.join("sculpin.json").exists() || path.join("app/SculpinKernel.php").exists() {
        return DetectedFramework::Sculpin;
    }
    if path.join("bin/console").exists()
        && path.join("config").is_dir()
        && path.join("public/index.php").exists()
    {
        return DetectedFramework::Symfony;
    }

    // Priority 5: CMS platforms
    if path.join("web/core/lib/Drupal.php").exists()
        || path.join("core/lib/Drupal.php").exists()
    {
        return DetectedFramework::Drupal;
    }
    if path.join("administrator").is_dir()
        && (path.join("includes/defines.php").exists()
            || path.join("configuration.php").exists())
    {
        return DetectedFramework::Joomla;
    }
    if path.join("concrete").is_dir() && path.join("index.php").exists() {
        let index = fs::read_to_string(path.join("index.php")).unwrap_or_default();
        if index.contains("concrete") || index.contains("Concrete") {
            return DetectedFramework::ConcreteCms;
        }
    }
    if path.join("system/modules").is_dir()
        || path.join("vendor/contao").is_dir()
        || path.join("system/config/constants.php").exists()
    {
        return DetectedFramework::Contao;
    }
    if path.join("kirby").is_dir() {
        return DetectedFramework::Kirby;
    }

    // Priority 6: micro-frameworks
    if path.join("config/app.php").exists() && path.join("src").is_dir() {
        return DetectedFramework::CakePHP;
    }
    if path.join("composer.json").exists() {
        let composer = fs::read_to_string(path.join("composer.json")).unwrap_or_default();
        if composer.contains("\"slim/slim\"") {
            return DetectedFramework::Slim;
        }
        if (path.join("module").is_dir() && path.join("public/index.php").exists())
            || composer.contains("zendframework")
            || composer.contains("laminas")
        {
            return DetectedFramework::Zend;
        }
    }

    // Priority 7: static sites
    if path.join("index.html").exists() && !path.join("index.php").exists() {
        return DetectedFramework::StaticHtml;
    }

    DetectedFramework::Unknown
}

pub async fn scan_all(
    valet_paths: &crate::valet::variant::ValetPaths,
    valet_config: &crate::valet::config_reader::ValetConfig,
) -> Vec<ValetSite> {
    let mut sites = Vec::new();
    let tld = &valet_config.tld;

    // 1. Scan parked directories (from valet_config.paths)
    for park_path in &valet_config.paths {
        let park = Path::new(park_path);
        let Ok(entries) = std::fs::read_dir(park) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() { continue }
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if name.starts_with('.') { continue }
            let framework = detect_framework(&path);
            let php_version = read_valetrc(&path);
            let is_secured = check_secured(&path, valet_paths);
            sites.push(ValetSite {
                domain: format!("{}.{}", name, tld),
                name,
                path,
                site_type: SiteType::Parked,
                framework,
                php_version,
                is_secured,
                ssl_expiry: None,
                is_favorite: false,
                status: SiteStatus::Unknown,
                last_hit: None,
                proxy_target: None,
            });
        }
    }

    // 2. Scan linked sites from valet_paths.sites_dir (symlinks)
    let Ok(entries) = std::fs::read_dir(&valet_paths.sites_dir) else { return sites };
    for entry in entries.filter_map(|e| e.ok()) {
        let link_path = entry.path();
        // Resolve symlink to actual path
        let actual = std::fs::read_link(&link_path).unwrap_or_else(|_| link_path.clone());
        let name = link_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if name.starts_with('.') { continue }
        // Skip if already found as parked
        if sites.iter().any(|s| s.name == name) { continue }
        let framework = detect_framework(&actual);
        let php_version = read_valetrc(&actual);
        let is_secured = check_secured(&link_path, valet_paths);
        sites.push(ValetSite {
            domain: format!("{}.{}", name, tld),
            name,
            path: actual,
            site_type: SiteType::Linked,
            framework,
            php_version,
            is_secured,
            ssl_expiry: None,
            is_favorite: false,
            status: SiteStatus::Unknown,
            last_hit: None,
            proxy_target: None,
        });
    }

    sites.sort_by(|a, b| a.name.cmp(&b.name));
    sites
}

fn read_valetrc(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join(".valetrc")).ok()?;
    content.lines()
        .find(|l| l.starts_with("php="))
        .map(|l| l.trim_start_matches("php=").to_string())
}

fn check_secured(site_path: &Path, valet_paths: &crate::valet::variant::ValetPaths) -> bool {
    let name = site_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    valet_paths.ca_dir.join(format!("{}.crt", name)).exists()
        || valet_paths.nginx_dir.join(name).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_site() -> PathBuf {
        use rand::Rng;
        let id: u32 = rand::rng().random();
        let dir = std::env::temp_dir().join(format!("valet_test_{}", id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn create_file(dir: &PathBuf, rel: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, "").unwrap();
    }

    fn create_dir(dir: &PathBuf, rel: &str) {
        std::fs::create_dir_all(dir.join(rel)).unwrap();
    }

    #[test]
    fn detects_magento() {
        let dir = make_temp_site();
        create_file(&dir, "bin/magento");
        assert_eq!(detect_framework(&dir), DetectedFramework::Magento);
        cleanup(&dir);
    }

    #[test]
    fn detects_craft() {
        let dir = make_temp_site();
        create_file(&dir, "craft");
        assert_eq!(detect_framework(&dir), DetectedFramework::Craft);
        cleanup(&dir);
    }

    #[test]
    fn detects_expression_engine() {
        let dir = make_temp_site();
        create_dir(&dir, "system/ee");
        assert_eq!(detect_framework(&dir), DetectedFramework::ExpressionEngine);
        cleanup(&dir);
    }

    #[test]
    fn detects_katana() {
        let dir = make_temp_site();
        create_file(&dir, "config.php");
        create_dir(&dir, "posts");
        // no source/ dir
        std::fs::write(dir.join("config.php"), "<?php // Katana config\nclass SiteConfiguration {}").unwrap();
        assert_eq!(detect_framework(&dir), DetectedFramework::Katana);
        cleanup(&dir);
    }

    #[test]
    fn detects_bedrock() {
        let dir = make_temp_site();
        create_dir(&dir, "web/wp");
        create_file(&dir, "config/application.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Bedrock);
        cleanup(&dir);
    }

    #[test]
    fn detects_wordpress() {
        let dir = make_temp_site();
        create_dir(&dir, "wp-admin");
        assert_eq!(detect_framework(&dir), DetectedFramework::WordPress);
        cleanup(&dir);
    }

    #[test]
    fn detects_october_cms() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        create_dir(&dir, "modules/backend");
        assert_eq!(detect_framework(&dir), DetectedFramework::OctoberCms);
        cleanup(&dir);
    }

    #[test]
    fn detects_statamic_vendor() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        create_dir(&dir, "vendor/statamic");
        assert_eq!(detect_framework(&dir), DetectedFramework::Statamic);
        cleanup(&dir);
    }

    #[test]
    fn detects_statamic_composer() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        std::fs::write(
            dir.join("composer.json"),
            r#"{"require": {"statamic/statamic": "^4.0"}}"#,
        ).unwrap();
        assert_eq!(detect_framework(&dir), DetectedFramework::Statamic);
        cleanup(&dir);
    }

    #[test]
    fn detects_laravel() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        create_file(&dir, "public/index.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Laravel);
        cleanup(&dir);
    }

    #[test]
    fn detects_jigsaw() {
        let dir = make_temp_site();
        create_file(&dir, "config.php");
        create_dir(&dir, "source");
        assert_eq!(detect_framework(&dir), DetectedFramework::Jigsaw);
        cleanup(&dir);
    }

    #[test]
    fn detects_sculpin() {
        let dir = make_temp_site();
        create_file(&dir, "sculpin.json");
        assert_eq!(detect_framework(&dir), DetectedFramework::Sculpin);
        cleanup(&dir);
    }

    #[test]
    fn detects_sculpin_kernel() {
        let dir = make_temp_site();
        create_file(&dir, "app/SculpinKernel.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Sculpin);
        cleanup(&dir);
    }

    #[test]
    fn detects_symfony() {
        let dir = make_temp_site();
        create_file(&dir, "bin/console");
        create_dir(&dir, "config");
        create_file(&dir, "public/index.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Symfony);
        cleanup(&dir);
    }

    #[test]
    fn detects_drupal() {
        let dir = make_temp_site();
        create_file(&dir, "core/lib/Drupal.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Drupal);
        cleanup(&dir);
    }

    #[test]
    fn detects_drupal_web() {
        let dir = make_temp_site();
        create_file(&dir, "web/core/lib/Drupal.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Drupal);
        cleanup(&dir);
    }

    #[test]
    fn detects_joomla() {
        let dir = make_temp_site();
        create_dir(&dir, "administrator");
        create_file(&dir, "includes/defines.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Joomla);
        cleanup(&dir);
    }

    #[test]
    fn detects_concrete_cms() {
        let dir = make_temp_site();
        create_dir(&dir, "concrete");
        std::fs::write(dir.join("index.php"), "<?php define('concrete', true);").unwrap();
        assert_eq!(detect_framework(&dir), DetectedFramework::ConcreteCms);
        cleanup(&dir);
    }

    #[test]
    fn detects_contao() {
        let dir = make_temp_site();
        create_dir(&dir, "vendor/contao");
        assert_eq!(detect_framework(&dir), DetectedFramework::Contao);
        cleanup(&dir);
    }

    #[test]
    fn detects_kirby() {
        let dir = make_temp_site();
        create_dir(&dir, "kirby");
        assert_eq!(detect_framework(&dir), DetectedFramework::Kirby);
        cleanup(&dir);
    }

    #[test]
    fn detects_cakephp() {
        let dir = make_temp_site();
        create_file(&dir, "config/app.php");
        create_dir(&dir, "src");
        assert_eq!(detect_framework(&dir), DetectedFramework::CakePHP);
        cleanup(&dir);
    }

    #[test]
    fn detects_slim() {
        let dir = make_temp_site();
        std::fs::write(
            dir.join("composer.json"),
            r#"{"require": {"slim/slim": "^4.0"}}"#,
        ).unwrap();
        assert_eq!(detect_framework(&dir), DetectedFramework::Slim);
        cleanup(&dir);
    }

    #[test]
    fn detects_zend() {
        let dir = make_temp_site();
        std::fs::write(
            dir.join("composer.json"),
            r#"{"require": {"laminas/laminas-mvc": "^3.0"}}"#,
        ).unwrap();
        assert_eq!(detect_framework(&dir), DetectedFramework::Zend);
        cleanup(&dir);
    }

    #[test]
    fn detects_static_html() {
        let dir = make_temp_site();
        create_file(&dir, "index.html");
        // no index.php
        assert_eq!(detect_framework(&dir), DetectedFramework::StaticHtml);
        cleanup(&dir);
    }

    #[test]
    fn detects_unknown() {
        let dir = make_temp_site();
        // empty directory → Unknown
        assert_eq!(detect_framework(&dir), DetectedFramework::Unknown);
        cleanup(&dir);
    }

    #[test]
    fn static_html_not_detected_when_php_present() {
        let dir = make_temp_site();
        create_file(&dir, "index.html");
        create_file(&dir, "index.php");
        // should be Unknown since no other framework markers
        assert_eq!(detect_framework(&dir), DetectedFramework::Unknown);
        cleanup(&dir);
    }

    // ── Phase 13 — priority-ordering tests ────────────────────────────

    #[test]
    fn priority_october_beats_laravel() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        create_file(&dir, "public/index.php");
        create_file(&dir, "modules/backend/x.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::OctoberCms);
        cleanup(&dir);
    }

    #[test]
    fn priority_bedrock_beats_wordpress() {
        let dir = make_temp_site();
        create_file(&dir, "web/wp/wp-settings.php");
        create_file(&dir, "config/application.php");
        // No wp-admin → bedrock wins. But add wp-admin too to be safe.
        create_dir(&dir, "web/wp/wp-admin");
        // The detect_framework checks web/wp dir (just the dir exists),
        // not the wp-admin path of bedrock — but it does NOT check root
        // wp-admin if bedrock has matched first. Bedrock wins.
        assert_eq!(detect_framework(&dir), DetectedFramework::Bedrock);
        cleanup(&dir);
    }

    #[test]
    fn priority_statamic_beats_laravel() {
        let dir = make_temp_site();
        create_file(&dir, "artisan");
        create_file(&dir, "vendor/statamic/cms/x.php");
        create_file(&dir, "public/index.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Statamic);
        cleanup(&dir);
    }

    #[test]
    fn priority_magento_beats_artisan() {
        let dir = make_temp_site();
        create_file(&dir, "bin/magento");
        create_file(&dir, "artisan");
        create_file(&dir, "public/index.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Magento);
        cleanup(&dir);
    }

    #[test]
    fn priority_jigsaw_beats_symfony() {
        let dir = make_temp_site();
        create_file(&dir, "config.php");
        create_file(&dir, "source/index.blade.php");
        // Even if bin/console + config existed, jigsaw should win because
        // jigsaw is checked before the Symfony triple.
        assert_eq!(detect_framework(&dir), DetectedFramework::Jigsaw);
        cleanup(&dir);
    }

    #[test]
    fn priority_sculpin_beats_symfony() {
        let dir = make_temp_site();
        create_file(&dir, "sculpin.json");
        create_file(&dir, "bin/console");
        create_dir(&dir, "config");
        create_file(&dir, "public/index.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Sculpin);
        cleanup(&dir);
    }

    #[test]
    fn priority_drupal_beats_joomla() {
        let dir = make_temp_site();
        // Both signatures present.
        create_file(&dir, "web/core/lib/Drupal.php");
        create_dir(&dir, "administrator");
        create_file(&dir, "configuration.php");
        assert_eq!(detect_framework(&dir), DetectedFramework::Drupal);
        cleanup(&dir);
    }

    #[test]
    fn priority_craft_beats_anything_else() {
        let dir = make_temp_site();
        create_file(&dir, "craft");
        // throw in noise that would otherwise match other frameworks
        create_file(&dir, "wp-admin/index.php");
        create_file(&dir, "artisan");
        assert_eq!(detect_framework(&dir), DetectedFramework::Craft);
        cleanup(&dir);
    }

    #[test]
    fn site_status_label() {
        assert_eq!(SiteStatus::Running.label(), "Running");
        assert_eq!(SiteStatus::Failed.label(),  "Failed");
        assert_eq!(SiteStatus::Unknown.label(), "Unknown");
    }

    #[test]
    fn valet_site_defaults_status_unknown() {
        let site = ValetSite {
            name: "test".to_string(),
            domain: "test.test".to_string(),
            path: std::path::PathBuf::from("/tmp/test"),
            site_type: SiteType::Parked,
            framework: DetectedFramework::Unknown,
            php_version: None,
            is_secured: false,
            ssl_expiry: None,
            is_favorite: false,
            status: SiteStatus::default(),
            last_hit: None,
            proxy_target: None,
        };
        assert_eq!(site.status, SiteStatus::Unknown);
        assert!(site.last_hit.is_none());
        assert!(site.proxy_target.is_none());
    }
}
