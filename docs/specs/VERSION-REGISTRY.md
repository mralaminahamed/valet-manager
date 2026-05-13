# Version Registry Spec
## In-app software version tracking with refresh

---

## What the registry tracks

The registry holds current version info for every tool Valet Manager uses
or manages: PHP versions + EOL dates, all 21 supported frameworks, HTTP
servers, CLI tools, and key Cargo crates.

It is the single source of truth for:
- Recommended PHP per framework (in App Creator)
- Compatibility check PHP constraints
- EOL warnings in the dashboard
- Version badges next to installed PHP versions
- "Update available" indicators for phpMyAdmin and Mailpit

---

## Data models (src/version_registry/models.rs)

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionRegistry {
    /// ISO 8601 timestamp of last successful fetch
    pub last_refreshed: Option<chrono::DateTime<chrono::Utc>>,
    pub php: Vec<PhpVersionInfo>,
    pub frameworks: HashMap<String, FrameworkVersionInfo>,  // keyed by framework id
    pub servers: HashMap<String, ServerVersionInfo>,
    pub tools: HashMap<String, ToolVersionInfo>,
}

/// One entry per PHP minor version (8.1, 8.2, 8.3, 8.4, 8.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpVersionInfo {
    pub minor: String,          // "8.4"
    pub latest_patch: String,   // "8.4.7"
    pub active_support_until: NaiveDate,
    pub security_support_until: NaiveDate,
    pub release_url: String,
}

impl PhpVersionInfo {
    pub fn is_eol(&self) -> bool {
        chrono::Utc::now().date_naive() > self.security_support_until
    }
    pub fn days_until_eol(&self) -> i64 {
        (self.security_support_until - chrono::Utc::now().date_naive()).num_days()
    }
    pub fn eol_status(&self) -> EolStatus {
        match self.days_until_eol() {
            d if d < 0   => EolStatus::Eol,
            d if d < 180 => EolStatus::Critical,
            d if d < 365 => EolStatus::Warning,
            _            => EolStatus::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EolStatus { Active, Warning, Critical, Eol }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub min_php: String,       // "8.2"
    pub release_url: String,
    pub changelog_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionInfo {
    pub name: String,
    pub latest_version: String,
    pub release_url: String,
}

/// Cached on disk at ~/.config/valet-manager/version-registry.json
/// Stale after configured TTL (default 24h)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistryCache {
    pub registry: VersionRegistry,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub ttl_hours: u32,
}

impl VersionRegistryCache {
    pub fn is_stale(&self) -> bool {
        let age = chrono::Utc::now() - self.fetched_at;
        age.num_hours() >= self.ttl_hours as i64
    }
}
```

---

## Fetcher modules (src/version_registry/)

```
src/version_registry/
├── mod.rs              orchestrator: refresh(), load_cache(), save_cache()
├── models.rs           all structs above
├── cache.rs            read/write version-registry.json
├── php_fetcher.rs      php.net JSON + endoflife.date
├── github_fetcher.rs   generic GitHub releases/latest → FrameworkVersionInfo
├── packagist_fetcher.rs Packagist p2 API → FrameworkVersionInfo
├── wordpress_fetcher.rs WordPress.org version-check API
└── nodejs_fetcher.rs   nodejs.org dist index → LTS version
```

### mod.rs — public API

```rust
/// Load cache from disk if fresh, else fetch all sources concurrently
pub async fn refresh(
    config: &AppConfig,
    force: bool,
    tx: mpsc::Sender<VersionRegistryEvent>,
) -> anyhow::Result<VersionRegistry>

    // 1. Try loading cache
    if !force:
        if let Ok(cache) = cache::load().await:
            if !cache.is_stale():
                return Ok(cache.registry)

    // 2. Fetch all sources concurrently (tokio::join! or futures::join_all)
    tx.send(Started { sources: 14 }).await?;
    let (php, wp, laravel, symfony, cakephp, drupal, craft, statamic,
         frankenphp, caddy, mailpit, phpmyadmin, composer, nodejs) =
        tokio::join!(
            php_fetcher::fetch(),
            wordpress_fetcher::fetch(),
            github_fetcher::fetch("laravel/framework"),
            packagist_fetcher::fetch("symfony/framework-bundle"),
            packagist_fetcher::fetch("cakephp/cakephp"),
            packagist_fetcher::fetch("drupal/core"),
            packagist_fetcher::fetch("craftcms/cms"),
            packagist_fetcher::fetch("statamic/cms"),
            github_fetcher::fetch("php/frankenphp"),
            github_fetcher::fetch("caddyserver/caddy"),
            github_fetcher::fetch("axllent/mailpit"),
            github_fetcher::fetch("phpmyadmin/phpmyadmin"),
            github_fetcher::fetch("composer/composer"),
            nodejs_fetcher::fetch_lts(),
        );
    // 3. Assemble registry (use Ok or default on individual fetch failures)
    // 4. Save to cache
    // 5. Send VersionRegistryEvent::Refreshed(registry)
    // 6. Return registry
```

---

## Fetcher implementations

### php_fetcher.rs

```rust
const ENDOFLIFE_URL: &str = "https://endoflife.date/api/v1/products/php/";
const PHP_NET_URL: &str = "https://www.php.net/releases/index.php?json&version=8";

pub async fn fetch() -> Vec<PhpVersionInfo>

    // 1. GET endoflife.date/api/v1/products/php/
    //    Returns JSON array: [{cycle, eol, support, latest, latestReleaseDate, ...}]
    // 2. Filter to PHP 8.x cycles only
    // 3. For each cycle, build PhpVersionInfo:
    //    minor = cycle,  latest_patch = latest,
    //    active_support_until = parse(support) as NaiveDate,
    //    security_support_until = parse(eol) as NaiveDate,
    //    release_url = "https://www.php.net/releases/8_{minor_minor}_0.php"
```

JSON response shape from endoflife.date:
```json
[
  {
    "cycle": "8.5",
    "eol": "2029-12-31",
    "support": "2027-12-31",
    "latest": "8.5.5",
    "latestReleaseDate": "2026-03-27",
    "releaseDate": "2025-11-20",
    "lts": false
  },
  ...
]
```

### github_fetcher.rs

```rust
pub async fn fetch(repo: &str) -> anyhow::Result<FrameworkVersionInfo>
    // GET https://api.github.com/repos/{repo}/releases/latest
    // Headers: User-Agent: "valet-manager/1.0"
    //          Accept: application/vnd.github.v3+json
    // Parse: tag_name → strip leading "v" → latest_version
    //        html_url → release_url
    // Rate limit: GitHub allows 60 unauthenticated requests/hour
    //   If rate-limited: return cached value + log warning
```

### packagist_fetcher.rs

```rust
pub async fn fetch(package: &str) -> anyhow::Result<FrameworkVersionInfo>
    // GET https://repo.packagist.org/p2/{package}.json
    // Parse packages[{package}][0].version → latest stable
    //   (find first version not containing "dev", "alpha", "beta", "RC")
    // php constraint: packages[{package}][0].require.php
```

### wordpress_fetcher.rs

```rust
pub async fn fetch() -> anyhow::Result<FrameworkVersionInfo>
    // GET https://api.wordpress.org/core/version-check/1.7/
    // Parse: offers[0].version, offers[0].download

pub async fn fetch_wpcli() -> anyhow::Result<ToolVersionInfo>
    // github_fetcher::fetch("wp-cli/wp-cli")
```

### nodejs_fetcher.rs

```rust
pub async fn fetch_lts() -> anyhow::Result<ToolVersionInfo>
    // GET https://nodejs.org/dist/index.json
    // Find first entry where lts != false (most recent LTS)
    // Return version + date
```

---

## AppState integration

```rust
// Add to src/state/app_state.rs
pub version_registry: VersionRegistry,
pub version_registry_loading: bool,

// Add to AppConfig
pub version_registry_ttl_hours: u32,  // default 24
pub version_registry_auto_refresh: bool,  // default true
```

## New AppCommand variants

```rust
RefreshVersionRegistry,                 // manual trigger (button)
VersionRegistryRefreshed(VersionRegistry),  // result event
```

---

## UI integration points

### 1. Dashboard — PHP EOL banners

```rust
// In dashboard render_alert_banners():
for php in &state.version_registry.php {
    match php.eol_status() {
        EolStatus::Eol      => alert_banner(DANGER,  "⛔", format!(
            "PHP {} reached end-of-life on {}. Remove it or upgrade.",
            php.minor, php.security_support_until)),
        EolStatus::Critical => alert_banner(DANGER,  "⚠", format!(
            "PHP {} EOL in {} days ({}). Plan your upgrade.",
            php.minor, php.days_until_eol(), php.security_support_until)),
        EolStatus::Warning  => alert_banner(WARNING, "⚠", format!(
            "PHP {} security support ends {}.",
            php.minor, php.security_support_until)),
        EolStatus::Active   => {}
    }
}
```

### 2. PHP Versions panel — patch version badge

Each PHP version card shows:
```
"8.3.12 installed  ·  8.3.21 available  ↑"
```
Fetch installed version from `php8.3 --version` (already done).
Fetch latest patch from `version_registry.php` for the matching minor.
If installed < latest patch: show "↑ Update available" in WARNING color.

### 3. App Creator — recommended PHP auto-select

```rust
// When user selects a project type in Step 1:
let fw_info = state.version_registry.frameworks.get(project_type.framework_id);
let min_php = fw_info.map(|f| &f.min_php).unwrap_or("8.2");
// Pre-select the highest installed PHP that meets min_php requirement
// instead of hardcoding from recommended_php_version()
```

### 4. Settings panel — Version Registry section

New section in settings panel:
```
[Version registry]
  Last updated: "2 hours ago" / "Never"
  ↺ Refresh now  (accent button → dispatches RefreshVersionRegistry)
  Auto-refresh:  [✓] Refresh automatically every [24] hours
  Open cache file  (ghost button → xdg-open the JSON cache file)
```

### 5. Sites panel — framework version tooltip

Each framework badge tooltip shows:
```
"WordPress 6.9.4 available (you have 6.8.1)"
```
Fetch current installed version from site scan (reads wp-includes/version.php, composer.json, etc.).
Compare against `state.version_registry.frameworks["wordpress"].latest_version`.

### 6. Dashboard quick stats — "Registry fresh" indicator

In the title bar right section (already shows "PHP 8.3 · valet-linux"):
Add: "Versions ↺" ghost button — clicking dispatches RefreshVersionRegistry.
While loading: spinner dot in WARNING color. When stale: dot in TEXT_TERTIARY.

---

## Startup behaviour

On app startup:
1. Load cache from disk.
2. If cache is fresh (< TTL) → use it, skip network.
3. If cache is stale or missing AND auto_refresh = true → dispatch RefreshVersionRegistry as background task.
4. Refresh result arrives as VersionRegistryRefreshed event → update AppState.

```rust
// In src/main.rs after state initialisation
if config.version_registry_auto_refresh {
    let age_hours = cache_age_hours();  // 0 if no cache
    if age_hours >= config.version_registry_ttl_hours {
        cmd_tx.try_send(AppCommand::RefreshVersionRegistry);
    }
}
```

---

## Cache file

```
~/.config/valet-manager/version-registry.json
```

```json
{
  "registry": {
    "last_refreshed": "2026-05-13T08:00:00Z",
    "php": [
      {
        "minor": "8.5",
        "latest_patch": "8.5.5",
        "active_support_until": "2027-12-31",
        "security_support_until": "2029-12-31",
        "release_url": "https://www.php.net/releases/8_5_0.php"
      }
    ],
    "frameworks": {
      "laravel": {
        "name": "Laravel",
        "latest_version": "12.57.0",
        "min_php": "8.2",
        "release_url": "https://github.com/laravel/framework/releases/latest"
      },
      "wordpress": {
        "name": "WordPress",
        "latest_version": "6.9.4",
        "min_php": "7.2",
        "release_url": "https://wordpress.org/news/category/releases/"
      }
    },
    "servers": {
      "frankenphp": { "name": "FrankenPHP", "latest_version": "1.11.3", ... },
      "caddy":      { "name": "Caddy",      "latest_version": "2.11.2", ... },
      "mailpit":    { "name": "Mailpit",    "latest_version": "1.29.5", ... },
      "phpmyadmin": { "name": "phpMyAdmin", "latest_version": "5.2.2",  ... }
    },
    "tools": {
      "composer": { "name": "Composer", "latest_version": "2.8.9", ... },
      "wpcli":    { "name": "WP-CLI",   "latest_version": "2.12.0", ... },
      "node_lts": { "name": "Node.js LTS", "latest_version": "22.15.0", ... }
    }
  },
  "fetched_at": "2026-05-13T08:00:00Z",
  "ttl_hours": 24
}
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
