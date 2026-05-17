//! Framework CLI detection helper used by the Artisan panel.
//!
//! Many PHP frameworks ship an artisan-like command runner — but not always
//! `php artisan`. This module classifies the binary so the panel header and
//! quick-action sets can adapt per framework.
//!
//! Detection is purely path-based and synchronous — no subprocess calls.

#![allow(dead_code)]

use crate::valet::site_scanner::ValetSite;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameworkCli {
    /// Laravel/OctoberCMS/Statamic `php artisan`
    Artisan,
    /// Symfony `php bin/console`
    SymfonyCli,
    /// Magento `php bin/magento`
    BinMagento,
    /// Drupal `vendor/bin/drush`
    DrushCli,
}

impl FrameworkCli {
    /// Short human label shown in the Artisan panel header.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Artisan    => "Artisan",
            Self::SymfonyCli => "bin/console",
            Self::BinMagento => "bin/magento",
            Self::DrushCli   => "drush",
        }
    }
}

/// Inspect the site directory and return the first matching CLI runner.
///
/// Order: `artisan` → `bin/console` → `bin/magento` → `vendor/bin/drush`.
pub fn detect_cli(site: &ValetSite) -> Option<FrameworkCli> {
    let p = &site.path;
    if p.join("artisan").exists()            { return Some(FrameworkCli::Artisan); }
    if p.join("bin/console").exists()        { return Some(FrameworkCli::SymfonyCli); }
    if p.join("bin/magento").exists()        { return Some(FrameworkCli::BinMagento); }
    if p.join("vendor/bin/drush").exists()   { return Some(FrameworkCli::DrushCli); }
    None
}

/// Return the (binary, args-prefix) tuple for invoking the given CLI.
/// `php_bin` is used for the three PHP-script variants; for `drush` we
/// invoke the vendor binary directly and pass an empty prefix.
pub fn cli_binary(cli: FrameworkCli, php_bin: &str) -> (String, Vec<String>) {
    match cli {
        FrameworkCli::Artisan    => (php_bin.to_string(), vec!["artisan".to_string()]),
        FrameworkCli::SymfonyCli => (php_bin.to_string(), vec!["bin/console".to_string()]),
        FrameworkCli::BinMagento => (php_bin.to_string(), vec!["bin/magento".to_string()]),
        FrameworkCli::DrushCli   => ("vendor/bin/drush".to_string(), vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::DetectedFramework;
    use crate::valet::site_scanner::{SiteType, ValetSite};
    use std::path::PathBuf;

    fn make_site_dir() -> PathBuf {
        use rand::Rng;
        let id: u32 = rand::rng().random();
        let dir = std::env::temp_dir().join(format!("vm-fwcli-{}", id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &PathBuf, rel: &str) {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, "").unwrap();
    }

    fn cleanup(dir: &PathBuf) { let _ = std::fs::remove_dir_all(dir); }

    fn site_at(path: PathBuf) -> ValetSite {
        ValetSite {
            name: "demo".into(),
            domain: "demo.test".into(),
            path,
            site_type: SiteType::Linked,
            framework: DetectedFramework::Unknown,
            php_version: None,
            is_secured: false,
            ssl_expiry: None,
            is_favorite: false,
            status: crate::valet::site_scanner::SiteStatus::Unknown,
            last_hit: None,
            proxy_target: None,
        }
    }

    #[test]
    fn detect_cli_finds_artisan() {
        let d = make_site_dir();
        touch(&d, "artisan");
        let s = site_at(d.clone());
        assert_eq!(detect_cli(&s), Some(FrameworkCli::Artisan));
        cleanup(&d);
    }

    #[test]
    fn detect_cli_finds_console() {
        let d = make_site_dir();
        touch(&d, "bin/console");
        let s = site_at(d.clone());
        assert_eq!(detect_cli(&s), Some(FrameworkCli::SymfonyCli));
        cleanup(&d);
    }

    #[test]
    fn detect_cli_finds_magento() {
        let d = make_site_dir();
        touch(&d, "bin/magento");
        let s = site_at(d.clone());
        assert_eq!(detect_cli(&s), Some(FrameworkCli::BinMagento));
        cleanup(&d);
    }

    #[test]
    fn detect_cli_finds_drush() {
        let d = make_site_dir();
        touch(&d, "vendor/bin/drush");
        let s = site_at(d.clone());
        assert_eq!(detect_cli(&s), Some(FrameworkCli::DrushCli));
        cleanup(&d);
    }

    #[test]
    fn detect_cli_returns_none_for_empty_site() {
        let d = make_site_dir();
        let s = site_at(d.clone());
        assert!(detect_cli(&s).is_none());
        cleanup(&d);
    }

    #[test]
    fn detect_cli_artisan_wins_over_console() {
        let d = make_site_dir();
        touch(&d, "artisan");
        touch(&d, "bin/console");
        let s = site_at(d.clone());
        assert_eq!(detect_cli(&s), Some(FrameworkCli::Artisan));
        cleanup(&d);
    }

    #[test]
    fn cli_binary_artisan_uses_php_bin() {
        let (bin, args) = cli_binary(FrameworkCli::Artisan, "php8.3");
        assert_eq!(bin, "php8.3");
        assert_eq!(args, vec!["artisan".to_string()]);
    }

    #[test]
    fn cli_binary_drush_uses_vendor_path() {
        let (bin, args) = cli_binary(FrameworkCli::DrushCli, "php");
        assert_eq!(bin, "vendor/bin/drush");
        assert!(args.is_empty());
    }

    #[test]
    fn cli_binary_symfony_uses_php_bin() {
        let (bin, args) = cli_binary(FrameworkCli::SymfonyCli, "php8.2");
        assert_eq!(bin, "php8.2");
        assert_eq!(args, vec!["bin/console".to_string()]);
    }

    #[test]
    fn cli_binary_magento_uses_php_bin() {
        let (bin, args) = cli_binary(FrameworkCli::BinMagento, "php");
        assert_eq!(bin, "php");
        assert_eq!(args, vec!["bin/magento".to_string()]);
    }

    #[test]
    fn label_returns_unique_per_variant() {
        assert_eq!(FrameworkCli::Artisan.label(),    "Artisan");
        assert_eq!(FrameworkCli::SymfonyCli.label(), "bin/console");
        assert_eq!(FrameworkCli::BinMagento.label(), "bin/magento");
        assert_eq!(FrameworkCli::DrushCli.label(),   "drush");
    }
}
