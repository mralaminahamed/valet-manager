use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One entry per PHP minor version (8.1, 8.2, 8.3, 8.4, 8.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PhpVersionInfo {
    pub minor: String,                       // "8.3"
    pub latest_patch: String,                // "8.3.14"
    pub active_support_until: NaiveDate,
    pub security_support_until: NaiveDate,
    pub release_url: String,
}

impl PhpVersionInfo {
    #[allow(dead_code)]
    pub fn days_until_eol(&self) -> i64 {
        let today = Utc::now().date_naive();
        self.security_support_until
            .signed_duration_since(today)
            .num_days()
    }
    #[allow(dead_code)]
    pub fn is_eol(&self) -> bool {
        self.days_until_eol() < 0
    }
    /// EOL bucket. Active > 1y, Warning < 1y, Critical < 90d, Eol passed.
    #[allow(dead_code)]
    pub fn eol_status(&self) -> EolStatus {
        let d = self.days_until_eol();
        if d < 0 {
            EolStatus::Eol
        } else if d < 90 {
            EolStatus::Critical
        } else if d < 365 {
            EolStatus::Warning
        } else {
            EolStatus::Active
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EolStatus {
    Active,
    Warning,
    Critical,
    Eol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FrameworkVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub min_php: Option<String>,
    pub release_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ServerVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub release_url: Option<String>,
}

impl From<FrameworkVersionInfo> for ServerVersionInfo {
    fn from(f: FrameworkVersionInfo) -> Self {
        ServerVersionInfo {
            name: f.name,
            latest_version: f.latest_version,
            release_url: f.release_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToolVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub release_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct VersionRegistry {
    pub last_refreshed: Option<DateTime<Utc>>,
    pub php: Vec<PhpVersionInfo>,
    pub frameworks: HashMap<String, FrameworkVersionInfo>,
    pub servers: HashMap<String, ServerVersionInfo>,
    pub tools: HashMap<String, ToolVersionInfo>,
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self {
            last_refreshed: None,
            php: default_php_versions(),
            frameworks: default_frameworks(),
            servers: default_servers(),
            tools: default_tools(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct VersionRegistryCache {
    pub registry: VersionRegistry,
    pub fetched_at: DateTime<Utc>,
    pub ttl_hours: u32,
}

impl VersionRegistryCache {
    #[allow(dead_code)]
    pub fn is_stale(&self) -> bool {
        let age = (Utc::now() - self.fetched_at).num_hours();
        age >= self.ttl_hours as i64
    }
}

// ────────────────────────────────────────────────────────────────────
// Bundled fallback data
// ────────────────────────────────────────────────────────────────────

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid bundled date")
}

#[allow(dead_code)]
pub fn default_php_versions() -> Vec<PhpVersionInfo> {
    vec![
        PhpVersionInfo {
            minor: "8.1".into(),
            latest_patch: "8.1.31".into(),
            active_support_until: date(2023, 11, 25),
            security_support_until: date(2025, 11, 25),
            release_url: "https://www.php.net/releases/8_1_0.php".into(),
        },
        PhpVersionInfo {
            minor: "8.2".into(),
            latest_patch: "8.2.26".into(),
            active_support_until: date(2024, 12, 31),
            security_support_until: date(2026, 12, 31),
            release_url: "https://www.php.net/releases/8_2_0.php".into(),
        },
        PhpVersionInfo {
            minor: "8.3".into(),
            latest_patch: "8.3.14".into(),
            active_support_until: date(2025, 11, 23),
            security_support_until: date(2027, 11, 23),
            release_url: "https://www.php.net/releases/8_3_0.php".into(),
        },
        PhpVersionInfo {
            minor: "8.4".into(),
            latest_patch: "8.4.1".into(),
            active_support_until: date(2026, 12, 31),
            security_support_until: date(2028, 12, 31),
            release_url: "https://www.php.net/releases/8_4_0.php".into(),
        },
        PhpVersionInfo {
            minor: "8.5".into(),
            latest_patch: "8.5.0".into(),
            active_support_until: date(2027, 12, 31),
            security_support_until: date(2029, 12, 31),
            release_url: "https://www.php.net/releases/8_5_0.php".into(),
        },
    ]
}

fn fw(name: &str, ver: &str, min_php: Option<&str>, url: Option<&str>) -> FrameworkVersionInfo {
    FrameworkVersionInfo {
        name: name.into(),
        latest_version: ver.into(),
        min_php: min_php.map(|s| s.into()),
        release_url: url.map(|s| s.into()),
    }
}

#[allow(dead_code)]
pub fn default_frameworks() -> HashMap<String, FrameworkVersionInfo> {
    let mut m = HashMap::new();
    m.insert(
        "laravel".into(),
        fw(
            "Laravel",
            "11.0.0",
            Some("8.2"),
            Some("https://github.com/laravel/framework/releases/latest"),
        ),
    );
    m.insert(
        "wordpress".into(),
        fw(
            "WordPress",
            "6.6",
            Some("7.2.24"),
            Some("https://wordpress.org/download/releases/"),
        ),
    );
    m.insert(
        "wordpress_multisite".into(),
        fw(
            "WordPress Multisite",
            "6.6",
            Some("7.2.24"),
            Some("https://wordpress.org/download/releases/"),
        ),
    );
    m.insert(
        "symfony".into(),
        fw(
            "Symfony",
            "7.1.0",
            Some("8.2"),
            Some("https://packagist.org/packages/symfony/framework-bundle"),
        ),
    );
    m.insert(
        "cakephp".into(),
        fw(
            "CakePHP",
            "5.1.0",
            Some("8.1"),
            Some("https://packagist.org/packages/cakephp/cakephp"),
        ),
    );
    m.insert(
        "drupal".into(),
        fw(
            "Drupal",
            "11.0.0",
            Some("8.3"),
            Some("https://packagist.org/packages/drupal/core"),
        ),
    );
    m.insert(
        "craft".into(),
        fw(
            "Craft CMS",
            "5.0.0",
            Some("8.2"),
            Some("https://packagist.org/packages/craftcms/cms"),
        ),
    );
    m.insert(
        "statamic".into(),
        fw(
            "Statamic",
            "5.0.0",
            Some("8.2"),
            Some("https://packagist.org/packages/statamic/cms"),
        ),
    );
    m.insert(
        "slim".into(),
        fw(
            "Slim",
            "4.14.0",
            Some("7.4"),
            Some("https://packagist.org/packages/slim/slim"),
        ),
    );
    m.insert(
        "joomla".into(),
        fw(
            "Joomla",
            "5.2.0",
            Some("8.1"),
            Some("https://downloads.joomla.org/"),
        ),
    );
    m.insert(
        "kirby".into(),
        fw(
            "Kirby",
            "5.0.0",
            Some("8.1"),
            Some("https://getkirby.com/releases"),
        ),
    );
    m.insert(
        "bedrock".into(),
        fw(
            "Bedrock",
            "14.0.0",
            Some("8.0"),
            Some("https://github.com/roots/bedrock/releases/latest"),
        ),
    );
    m.insert(
        "magento".into(),
        fw(
            "Magento",
            "2.4.7",
            Some("8.2"),
            Some("https://magento.com/tech-resources/download"),
        ),
    );
    m.insert(
        "laminas".into(),
        fw(
            "Laminas",
            "3.10.0",
            Some("8.2"),
            Some("https://packagist.org/packages/laminas/laminas-mvc"),
        ),
    );
    m.insert(
        "concretecms".into(),
        fw(
            "Concrete CMS",
            "9.3.0",
            Some("8.1"),
            Some("https://www.concretecms.com/download"),
        ),
    );
    m.insert(
        "contao".into(),
        fw(
            "Contao",
            "5.3.0",
            Some("8.1"),
            Some("https://contao.org/en/download"),
        ),
    );
    m.insert(
        "octobercms".into(),
        fw(
            "October CMS",
            "3.5.0",
            Some("8.0"),
            Some("https://octobercms.com/download"),
        ),
    );
    m.insert(
        "jigsaw".into(),
        fw(
            "Jigsaw",
            "1.7.0",
            Some("8.0"),
            Some("https://github.com/tightenco/jigsaw/releases/latest"),
        ),
    );
    m.insert(
        "sculpin".into(),
        fw(
            "Sculpin",
            "3.2.0",
            Some("7.4"),
            Some("https://github.com/sculpin/sculpin/releases/latest"),
        ),
    );
    m.insert(
        "expressionengine".into(),
        fw(
            "ExpressionEngine",
            "7.5.0",
            Some("8.1"),
            Some("https://expressionengine.com/download"),
        ),
    );
    m.insert(
        "static_html".into(),
        fw("Static HTML", "1.0", None, None),
    );
    m
}

fn srv(name: &str, ver: &str, url: Option<&str>) -> ServerVersionInfo {
    ServerVersionInfo {
        name: name.into(),
        latest_version: ver.into(),
        release_url: url.map(|s| s.into()),
    }
}

#[allow(dead_code)]
pub fn default_servers() -> HashMap<String, ServerVersionInfo> {
    let mut m = HashMap::new();
    m.insert(
        "nginx".into(),
        srv("Nginx", "1.27.0", Some("https://nginx.org/")),
    );
    m.insert(
        "frankenphp".into(),
        srv(
            "FrankenPHP",
            "1.3.0",
            Some("https://github.com/php/frankenphp/releases/latest"),
        ),
    );
    m.insert(
        "caddy".into(),
        srv(
            "Caddy",
            "2.8.0",
            Some("https://github.com/caddyserver/caddy/releases/latest"),
        ),
    );
    m.insert(
        "apache".into(),
        srv("Apache", "2.4.0", Some("https://httpd.apache.org/")),
    );
    m
}

fn tool(name: &str, ver: &str, url: Option<&str>) -> ToolVersionInfo {
    ToolVersionInfo {
        name: name.into(),
        latest_version: ver.into(),
        release_url: url.map(|s| s.into()),
    }
}

#[allow(dead_code)]
pub fn default_tools() -> HashMap<String, ToolVersionInfo> {
    let mut m = HashMap::new();
    m.insert(
        "composer".into(),
        tool(
            "Composer",
            "2.8.0",
            Some("https://github.com/composer/composer/releases/latest"),
        ),
    );
    m.insert(
        "wpcli".into(),
        tool(
            "WP-CLI",
            "2.11.0",
            Some("https://github.com/wp-cli/wp-cli/releases/latest"),
        ),
    );
    m.insert(
        "mailpit".into(),
        tool(
            "Mailpit",
            "1.20.0",
            Some("https://github.com/axllent/mailpit/releases/latest"),
        ),
    );
    m.insert(
        "phpmyadmin".into(),
        tool(
            "phpMyAdmin",
            "5.2.0",
            Some("https://github.com/phpmyadmin/phpmyadmin/releases/latest"),
        ),
    );
    m.insert(
        "node_lts".into(),
        tool(
            "Node.js LTS",
            "22.0.0",
            Some("https://nodejs.org/en/download/"),
        ),
    );
    m
}

/// Best PHP minor version for a framework: highest installed PHP that meets the
/// framework's min_php requirement. Falls back to `default` if no match.
#[allow(dead_code)]
pub fn best_php_for_framework(
    registry: &VersionRegistry,
    framework_id: &str,
    installed: &[String],
    default: &str,
) -> String {
    let Some(fw) = registry.frameworks.get(framework_id) else {
        return default.to_string();
    };
    let Some(min) = fw.min_php.as_deref() else {
        return installed
            .iter()
            .max_by(|a, b| compare_minor(a, b))
            .cloned()
            .unwrap_or_else(|| default.to_string());
    };
    let min_tuple = parse_minor(min).unwrap_or((8, 0));
    installed
        .iter()
        .filter_map(|v| parse_minor(v).map(|t| (t, v.clone())))
        .filter(|(t, _)| *t >= min_tuple)
        .max_by_key(|(t, _)| *t)
        .map(|(_, v)| v)
        .unwrap_or_else(|| default.to_string())
}

fn parse_minor(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.split('.');
    let a = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;
    Some((a, b))
}

fn compare_minor(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_minor(a).unwrap_or((0, 0));
    let pb = parse_minor(b).unwrap_or((0, 0));
    pa.cmp(&pb)
}
