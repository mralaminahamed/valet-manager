pub mod cache;
pub mod github_fetcher;
pub mod models;
pub mod nodejs_fetcher;
pub mod packagist_fetcher;
pub mod php_fetcher;
pub mod wordpress_fetcher;

#[allow(unused_imports)]
pub use models::{
    EolStatus, FrameworkVersionInfo, PhpVersionInfo, ServerVersionInfo, ToolVersionInfo,
    VersionRegistry, VersionRegistryCache,
};

use crate::events::AppEvent;
use tokio::sync::mpsc::Sender;

/// Fetch every source concurrently with bundled-defaults fallback per source.
/// Saves to cache and sends `VersionRegistryRefreshed` event on success.
#[allow(dead_code)]
pub async fn refresh(
    config: &crate::config::AppConfig,
    force: bool,
    event_tx: Sender<AppEvent>,
) -> anyhow::Result<VersionRegistry> {
    if !force {
        if let Ok(c) = cache::load().await {
            if !c.is_stale() {
                let reg = c.registry.clone();
                let _ = event_tx
                    .send(AppEvent::VersionRegistryRefreshed(reg.clone()))
                    .await;
                return Ok(reg);
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("valet-manager/1.0")
        .build()?;

    let (
        php_v,
        wp_r,
        laravel_r,
        symfony_r,
        cakephp_r,
        drupal_r,
        craft_r,
        statamic_r,
        frankenphp_r,
        caddy_r,
        mailpit_r,
        phpmyadmin_r,
        composer_r,
        nodejs_r,
    ) = tokio::join!(
        php_fetcher::fetch(&client),
        wordpress_fetcher::fetch(&client),
        github_fetcher::fetch(&client, "laravel/framework", "Laravel", Some("8.2")),
        packagist_fetcher::fetch(&client, "symfony/framework-bundle", "Symfony"),
        packagist_fetcher::fetch(&client, "cakephp/cakephp", "CakePHP"),
        packagist_fetcher::fetch(&client, "drupal/core", "Drupal"),
        packagist_fetcher::fetch(&client, "craftcms/cms", "Craft CMS"),
        packagist_fetcher::fetch(&client, "statamic/cms", "Statamic"),
        github_fetcher::fetch(&client, "php/frankenphp", "FrankenPHP", None),
        github_fetcher::fetch(&client, "caddyserver/caddy", "Caddy", None),
        github_fetcher::fetch(&client, "axllent/mailpit", "Mailpit", None),
        github_fetcher::fetch(&client, "phpmyadmin/phpmyadmin", "phpMyAdmin", None),
        github_fetcher::fetch(&client, "composer/composer", "Composer", None),
        nodejs_fetcher::fetch_lts(&client),
    );

    let defaults = VersionRegistry::default();
    let mut frameworks = defaults.frameworks.clone();
    let mut servers = defaults.servers.clone();
    let mut tools = defaults.tools.clone();

    if let Ok(v) = wp_r {
        frameworks.insert("wordpress".into(), v.clone());
        frameworks.insert("wordpress_multisite".into(), v);
    }
    if let Ok(v) = laravel_r {
        frameworks.insert("laravel".into(), v);
    }
    if let Ok(v) = symfony_r {
        frameworks.insert("symfony".into(), v);
    }
    if let Ok(v) = cakephp_r {
        frameworks.insert("cakephp".into(), v);
    }
    if let Ok(v) = drupal_r {
        frameworks.insert("drupal".into(), v);
    }
    if let Ok(v) = craft_r {
        frameworks.insert("craft".into(), v);
    }
    if let Ok(v) = statamic_r {
        frameworks.insert("statamic".into(), v);
    }
    if let Ok(v) = frankenphp_r {
        servers.insert("frankenphp".into(), v.into());
    }
    if let Ok(v) = caddy_r {
        servers.insert("caddy".into(), v.into());
    }
    if let Ok(v) = mailpit_r {
        tools.insert(
            "mailpit".into(),
            ToolVersionInfo {
                name: v.name,
                latest_version: v.latest_version,
                release_url: v.release_url,
            },
        );
    }
    if let Ok(v) = phpmyadmin_r {
        tools.insert(
            "phpmyadmin".into(),
            ToolVersionInfo {
                name: v.name,
                latest_version: v.latest_version,
                release_url: v.release_url,
            },
        );
    }
    if let Ok(v) = composer_r {
        tools.insert(
            "composer".into(),
            ToolVersionInfo {
                name: v.name,
                latest_version: v.latest_version,
                release_url: v.release_url,
            },
        );
    }
    if let Ok(v) = nodejs_r {
        tools.insert("node_lts".into(), v);
    }

    let php = if php_v.is_empty() { defaults.php } else { php_v };

    let registry = VersionRegistry {
        last_refreshed: Some(chrono::Utc::now()),
        php,
        frameworks,
        servers,
        tools,
    };

    let cache = VersionRegistryCache {
        registry: registry.clone(),
        fetched_at: chrono::Utc::now(),
        ttl_hours: config.version_registry_ttl_hours as u32,
    };
    let _ = cache::save(&cache).await;
    let _ = event_tx
        .send(AppEvent::VersionRegistryRefreshed(registry.clone()))
        .await;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::models::{
        best_php_for_framework, default_frameworks, default_php_versions, default_servers,
        default_tools, EolStatus, FrameworkVersionInfo, PhpVersionInfo, ServerVersionInfo,
        ToolVersionInfo, VersionRegistry, VersionRegistryCache,
    };
    use chrono::{Duration as ChronoDuration, NaiveDate, Utc};

    fn php_info_with_eol_in_days(days: i64) -> PhpVersionInfo {
        let today = Utc::now().date_naive();
        let eol = today + ChronoDuration::days(days);
        PhpVersionInfo {
            minor: "8.3".into(),
            latest_patch: "8.3.14".into(),
            active_support_until: eol,
            security_support_until: eol,
            release_url: "x".into(),
        }
    }

    #[test]
    fn default_registry_is_populated() {
        let reg = VersionRegistry::default();
        assert!(!reg.php.is_empty(), "php empty");
        assert!(reg.frameworks.contains_key("laravel"));
        assert!(reg.frameworks.contains_key("wordpress"));
        assert!(reg.last_refreshed.is_none());
    }

    #[test]
    fn default_php_includes_supported_minors() {
        let php = default_php_versions();
        let minors: Vec<&str> = php.iter().map(|p| p.minor.as_str()).collect();
        for v in ["8.1", "8.2", "8.3", "8.4", "8.5"] {
            assert!(minors.contains(&v), "missing PHP {v}");
        }
    }

    #[test]
    fn php_eol_status_active_when_far() {
        let info = php_info_with_eol_in_days(500);
        assert_eq!(info.eol_status(), EolStatus::Active);
        assert!(!info.is_eol());
    }

    #[test]
    fn php_eol_status_warning_within_year() {
        let info = php_info_with_eol_in_days(200);
        assert_eq!(info.eol_status(), EolStatus::Warning);
    }

    #[test]
    fn php_eol_status_critical_within_90_days() {
        let info = php_info_with_eol_in_days(30);
        assert_eq!(info.eol_status(), EolStatus::Critical);
    }

    #[test]
    fn php_eol_status_eol_when_past() {
        let info = php_info_with_eol_in_days(-1);
        assert_eq!(info.eol_status(), EolStatus::Eol);
        assert!(info.is_eol());
    }

    #[test]
    fn cache_stale_after_ttl() {
        let cache = VersionRegistryCache {
            registry: VersionRegistry::default(),
            fetched_at: Utc::now() - ChronoDuration::hours(25),
            ttl_hours: 24,
        };
        assert!(cache.is_stale());
    }

    #[test]
    fn cache_fresh_within_ttl() {
        let cache = VersionRegistryCache {
            registry: VersionRegistry::default(),
            fetched_at: Utc::now() - ChronoDuration::hours(1),
            ttl_hours: 24,
        };
        assert!(!cache.is_stale());
    }

    #[test]
    fn default_frameworks_contains_all_21_keys() {
        let f = default_frameworks();
        for key in [
            "laravel",
            "wordpress",
            "wordpress_multisite",
            "symfony",
            "cakephp",
            "drupal",
            "craft",
            "statamic",
            "slim",
            "joomla",
            "kirby",
            "bedrock",
            "magento",
            "laminas",
            "concretecms",
            "contao",
            "octobercms",
            "jigsaw",
            "sculpin",
            "expressionengine",
            "static_html",
        ] {
            assert!(f.contains_key(key), "missing framework key {key}");
        }
        assert_eq!(f.len(), 21);
    }

    #[test]
    fn default_servers_has_4_keys() {
        let s = default_servers();
        for k in ["nginx", "frankenphp", "caddy", "apache"] {
            assert!(s.contains_key(k), "missing server {k}");
        }
    }

    #[test]
    fn default_tools_has_5_keys() {
        let t = default_tools();
        for k in ["composer", "wpcli", "mailpit", "phpmyadmin", "node_lts"] {
            assert!(t.contains_key(k), "missing tool {k}");
        }
    }

    #[test]
    fn framework_into_server_preserves_fields() {
        let f = FrameworkVersionInfo {
            name: "Caddy".into(),
            latest_version: "2.8.0".into(),
            min_php: None,
            release_url: Some("https://example".into()),
        };
        let s: ServerVersionInfo = f.into();
        assert_eq!(s.name, "Caddy");
        assert_eq!(s.latest_version, "2.8.0");
        assert_eq!(s.release_url, Some("https://example".into()));
    }

    #[test]
    fn registry_round_trips_through_json() {
        let reg = VersionRegistry::default();
        let json = serde_json::to_string(&reg).expect("serialize");
        let parsed: VersionRegistry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.php.len(), reg.php.len());
        assert_eq!(parsed.frameworks.len(), reg.frameworks.len());
    }

    #[test]
    fn best_php_picks_highest_meeting_min() {
        let reg = VersionRegistry::default();
        let installed = vec!["8.1".into(), "8.2".into(), "8.3".into(), "8.4".into()];
        // Laravel min_php = 8.2 → highest meeting is 8.4
        let v = best_php_for_framework(&reg, "laravel", &installed, "8.2");
        assert_eq!(v, "8.4");
    }

    #[test]
    fn best_php_falls_back_to_default_when_none_match() {
        let reg = VersionRegistry::default();
        let installed = vec!["7.4".into()];
        let v = best_php_for_framework(&reg, "laravel", &installed, "8.3");
        assert_eq!(v, "8.3");
    }

    #[test]
    fn php_info_days_until_eol_is_positive_for_future() {
        let info = PhpVersionInfo {
            minor: "8.4".into(),
            latest_patch: "8.4.1".into(),
            active_support_until: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(),
            security_support_until: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(),
            release_url: "x".into(),
        };
        assert!(info.days_until_eol() > 0);
    }

    #[test]
    fn cache_roundtrips_through_json() {
        let cache = VersionRegistryCache {
            registry: VersionRegistry::default(),
            fetched_at: Utc::now(),
            ttl_hours: 24,
        };
        let json = serde_json::to_string(&cache).unwrap();
        let parsed: VersionRegistryCache = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ttl_hours, 24);
    }

    #[test]
    fn default_tool_node_lts_has_name() {
        let t = default_tools();
        let n = t.get("node_lts").unwrap();
        assert_eq!(n.name, "Node.js LTS");
        assert!(!n.latest_version.is_empty());
    }

    #[test]
    fn _suppress_unused() {
        // Touch otherwise-unused items so dead-code warnings don't snowball.
        let _t: ToolVersionInfo = ToolVersionInfo {
            name: "x".into(),
            latest_version: "1".into(),
            release_url: None,
        };
    }
}
