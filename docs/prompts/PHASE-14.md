> **Claude Design command** (run before the egui migration work in Step 0):
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

---

# Claude Code Prompt — Version Registry & Crate Updates
## Phase 14: Live version tracking + refresh button

```
You are implementing Phase 14 of Valet Manager — the Version Registry.
All previous phases are complete.

Read these files before writing:
  src/state/app_state.rs
  src/commands.rs
  src/config.rs
  src/ui/theme.rs               (Colors::*)
  src/ui/panels/dashboard.rs    (alert banners)
  src/ui/panels/settings.rs     (add registry section)
  docs/specs/VERSION-REGISTRY.md  (PRIMARY SPEC)
  docs/VERSIONS.md              (all current verified versions)

══════════════════════════════════════════════════════════
STEP 0 — UPDATE CARGO.TOML VERSIONS FIRST
══════════════════════════════════════════════════════════

Update valet-manager/Cargo.toml with these version pins.
Make every change; do not leave any of the "old" versions.

eframe      = { version = "0.34.2", features = ["default_fonts", "wgpu"] }
egui        = "0.34.2"
egui_extras = { version = "0.34.2", features = ["all_loaders"] }
tokio       = { version = "1.44", features = ["full"] }
reqwest     = { version = "0.12.15", features = ["json"] }
rusqlite    = { version = "0.34", features = ["bundled"] }
notify      = "8"
rand        = { version = "0.9", features = ["std"] }
dirs        = "6"
rfd         = "0.15"
flate2      = "1.1"
chrono      = { version = "0.4.41", features = ["serde"] }
semver      = "1.0.25"
fuzzy-matcher = "0.3.7"
handlebars  = "6.3"
bcrypt      = "0.15.1"

egui API NOTE — 0.34 breaking changes from 0.31:
  - App::update() is deprecated. Replace with App::ui(ui: &mut Ui).
  - ui.ctx() is now just ui (Ui derefs to Context).
  - Code like ui.ctx().input(|i| ...) becomes ui.input(|i| ...).
  - ViewportBuilder stays the same.
  - Fix all compile errors before proceeding to later steps.

rand 0.9 NOTE:
  - rand::distributions::Alphanumeric moved to rand::distr::Alphanumeric
  - Fix usage in src/phpmyadmin/config_generator.rs generate_blowfish_secret()

dirs 6.0 NOTE:
  - dirs::config_dir() still works unchanged.
  - dirs::home_dir() still works.

notify 8 NOTE:
  - RecommendedWatcher API unchanged; imports may differ.
  - Check src/valet/watcher.rs for import paths.

══════════════════════════════════════════════════════════
STEP 1 — DATA MODELS
══════════════════════════════════════════════════════════

Create src/version_registry/models.rs with ALL structs from the spec:
  VersionRegistry, PhpVersionInfo, EolStatus, FrameworkVersionInfo,
  ServerVersionInfo, ToolVersionInfo, VersionRegistryCache.

Implement on PhpVersionInfo:
  pub fn is_eol(&self) -> bool
  pub fn days_until_eol(&self) -> i64
  pub fn eol_status(&self) -> EolStatus

Implement Default for VersionRegistry that returns hardcoded fallback data
from docs/VERSIONS.md so the app works offline with no cache:

  Default::default() for VersionRegistry must return a populated struct
  with the verified versions from docs/VERSIONS.md, not an empty one.
  This is the "bundled baseline" that runs before the first refresh.

  fn default_php_versions() -> Vec<PhpVersionInfo>
    Returns 5 entries (8.1 through 8.5) with EOL dates from docs/VERSIONS.md.

  fn default_frameworks() -> HashMap<String, FrameworkVersionInfo>
    Returns entries for all 21 frameworks from docs/VERSIONS.md.
    Keys: "laravel", "wordpress", "symfony", "cakephp", "drupal", "craft",
          "statamic", "slim", "joomla", "kirby", "bedrock", "magento",
          "laminas", "concretecms", "contao", "octobercms", "jigsaw",
          "sculpin", "expressionengine", "static_html", "wordpress_multisite"

  fn default_servers() -> HashMap<String, ServerVersionInfo>
    Keys: "nginx", "frankenphp", "caddy", "apache"

  fn default_tools() -> HashMap<String, ToolVersionInfo>
    Keys: "composer", "wpcli", "mailpit", "phpmyadmin", "node_lts"

══════════════════════════════════════════════════════════
STEP 2 — CACHE MODULE
══════════════════════════════════════════════════════════

Create src/version_registry/cache.rs:

  fn cache_path() -> PathBuf
    dirs::config_dir()
      .unwrap()
      .join("valet-manager/version-registry.json")

  pub async fn load() -> anyhow::Result<VersionRegistryCache>
    Read and deserialize cache_path(). Error if missing.

  pub async fn save(cache: &VersionRegistryCache) -> anyhow::Result<()>
    Serialize with serde_json::to_string_pretty + atomic write.

══════════════════════════════════════════════════════════
STEP 3 — FETCHER MODULES
══════════════════════════════════════════════════════════

Create src/version_registry/php_fetcher.rs:

  const EOL_URL: &str = "https://endoflife.date/api/v1/products/php/";

  pub async fn fetch(client: &reqwest::Client) -> Vec<PhpVersionInfo>
    GET EOL_URL with User-Agent header.
    Parse JSON array. Filter: cycle starts with "8." AND parse as f32 >= 8.1.
    For each entry:
      PhpVersionInfo {
        minor: entry["cycle"].as_str(),
        latest_patch: entry["latest"].as_str(),
        active_support_until: parse entry["support"] as NaiveDate,
        security_support_until: parse entry["eol"] as NaiveDate,
        release_url: format!("https://www.php.net/releases/{}_0.php",
          entry["cycle"].replace('.', '_')),
      }
    On any error: return default_php_versions() (fallback to bundled data)
    Never propagate network errors — log them and return defaults.

Create src/version_registry/github_fetcher.rs:

  const GITHUB_API: &str = "https://api.github.com/repos";

  pub async fn fetch(
    client: &reqwest::Client,
    repo: &str,
    name: &str,
    min_php: Option<&str>,
  ) -> anyhow::Result<FrameworkVersionInfo>
    GET {GITHUB_API}/{repo}/releases/latest
    Headers: User-Agent: "valet-manager/1.0", Accept: application/vnd.github.v3+json
    Parse: tag_name (strip leading "v"), html_url.
    Handle 403/429 (rate-limit): log warning, return Err so caller can use fallback.

Create src/version_registry/packagist_fetcher.rs:

  pub async fn fetch(
    client: &reqwest::Client,
    package: &str,
    name: &str,
  ) -> anyhow::Result<FrameworkVersionInfo>
    GET https://repo.packagist.org/p2/{package}.json
    Parse packages[{package}][0] — skip dev/alpha/beta/RC entries.
    Return latest stable version string and require.php.
    On error: return Err.

Create src/version_registry/wordpress_fetcher.rs:

  pub async fn fetch(client: &reqwest::Client) -> anyhow::Result<FrameworkVersionInfo>
    GET https://api.wordpress.org/core/version-check/1.7/
    Parse: body.offers[0].version.

Create src/version_registry/nodejs_fetcher.rs:

  pub async fn fetch_lts(client: &reqwest::Client) -> anyhow::Result<ToolVersionInfo>
    GET https://nodejs.org/dist/index.json
    Find first entry where "lts" is not false (JSON value, not null).
    Return version string (strip leading "v").

══════════════════════════════════════════════════════════
STEP 4 — ORCHESTRATOR
══════════════════════════════════════════════════════════

Create src/version_registry/mod.rs:

  pub enum VersionRegistryEvent {
    Started { sources: usize },
    SourceFetched { source: String },
    Refreshed(VersionRegistry),
    Failed(String),
  }

  pub async fn refresh(
    config: &AppConfig,
    force: bool,
    event_tx: mpsc::Sender<AppEvent>,
  ) -> anyhow::Result<VersionRegistry>

    // Step 1: load cache if fresh and not force
    if !force {
        if let Ok(cache) = cache::load().await {
            if !cache.is_stale() {
                return Ok(cache.registry);
            }
        }
    }

    // Step 2: create shared reqwest::Client with timeout 10s and User-Agent
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("valet-manager/1.0")
        .build()?;

    // Step 3: concurrent fetch — use tokio::join! for all sources
    let (php_result, wp_result, laravel, symfony, cakephp, drupal,
         craft, statamic, frankenphp, caddy, mailpit, phpmyadmin,
         composer_result, nodejs) = tokio::join!(
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

    // Step 4: build registry — use Ok() values, fall back to defaults for Err
    let defaults = VersionRegistry::default();

    let mut frameworks = defaults.frameworks.clone();
    let insert_fw = |map: &mut HashMap<String, FrameworkVersionInfo>, key, result| {
        if let Ok(info) = result { map.insert(key, info); }
    };
    insert_fw(&mut frameworks, "laravel".into(), laravel);
    // ... repeat for all frameworks

    let mut servers = defaults.servers.clone();
    if let Ok(info) = frankenphp { servers.insert("frankenphp".into(), info.into()); }
    // ... etc

    let registry = VersionRegistry {
        last_refreshed: Some(chrono::Utc::now()),
        php: if php_result.is_empty() { defaults.php } else { php_result },
        frameworks,
        servers,
        tools: { /* similarly */ },
    };

    // Step 5: save cache
    let cache = VersionRegistryCache {
        registry: registry.clone(),
        fetched_at: chrono::Utc::now(),
        ttl_hours: config.version_registry_ttl_hours,
    };
    let _ = cache::save(&cache).await; // log but don't fail on cache write error

    // Step 6: send event
    event_tx.send(AppEvent::VersionRegistryRefreshed(registry.clone())).await?;
    Ok(registry)

══════════════════════════════════════════════════════════
STEP 5 — STATE + COMMAND + CONFIG UPDATES
══════════════════════════════════════════════════════════

src/state/app_state.rs — add fields:
  pub version_registry: VersionRegistry,      // Default::default() on startup
  pub version_registry_loading: bool,

src/commands.rs — add:
  RefreshVersionRegistry,

src/events.rs — add:
  VersionRegistryRefreshed(VersionRegistry),

src/config.rs — add to AppConfig:
  #[serde(default = "default_ttl")]
  pub version_registry_ttl_hours: u32,

  #[serde(default = "default_true")]
  pub version_registry_auto_refresh: bool,

  fn default_ttl() -> u32 { 24 }

══════════════════════════════════════════════════════════
STEP 6 — DISPATCHER WIRING
══════════════════════════════════════════════════════════

In run_dispatcher() match arms:

  RefreshVersionRegistry => {
    state.write().await.version_registry_loading = true;
    let config = state.read().await.config.clone();
    let etx = event_tx.clone();
    tokio::spawn(async move {
        match version_registry::refresh(&config, true, etx).await {
            Ok(reg) => { /* event sent inside refresh() */ }
            Err(e)  => { let _ = etx.send(AppEvent::Error(e.to_string())).await; }
        }
    });
  }

In the event handler (where AppEvents are applied to AppState):
  VersionRegistryRefreshed(registry) => {
    state.version_registry = registry;
    state.version_registry_loading = false;
  }

══════════════════════════════════════════════════════════
STEP 7 — STARTUP AUTO-REFRESH
══════════════════════════════════════════════════════════

In src/main.rs, after cmd_tx and initial state setup:

  // Load cached registry immediately (no network)
  if let Ok(cache) = version_registry::cache::load().await {
      state.write().await.version_registry = cache.registry;
  }
  // Auto-refresh in background if stale
  if config.version_registry_auto_refresh {
      let is_stale = version_registry::cache::load().await
          .map(|c| c.is_stale())
          .unwrap_or(true);
      if is_stale {
          cmd_tx.try_send(AppCommand::RefreshVersionRegistry).ok();
      }
  }

══════════════════════════════════════════════════════════
STEP 8 — UI INTEGRATION
══════════════════════════════════════════════════════════

A. DASHBOARD — PHP EOL banners
   In src/ui/panels/dashboard.rs render_alert_banners():
   Replace any hardcoded PHP EOL checks with:

   for php in &state.version_registry.php {
       let installed = state.php_versions.iter()
           .find(|v| v.version == php.minor);
       if installed.is_none() { continue; }   // not installed, skip

       match php.eol_status() {
           EolStatus::Eol => alert_banner(ui, Colors::DANGER, "⛔",
               &format!("PHP {} is EOL since {}. Uninstall or upgrade.",
                   php.minor, php.security_support_until), None, cmd_tx),
           EolStatus::Critical => alert_banner(ui, Colors::DANGER, "⚠",
               &format!("PHP {} EOL in {} days. Plan your upgrade.",
                   php.minor, php.days_until_eol()),
               Some(("Learn more", AppCommand::OpenUrl(php.release_url.clone()))), cmd_tx),
           EolStatus::Warning => alert_banner(ui, Colors::WARNING, "ℹ",
               &format!("PHP {} security support ends {}.",
                   php.minor, php.security_support_until), None, cmd_tx),
           EolStatus::Active => {}
       }
   }

B. PHP VERSIONS PANEL — patch update badge
   In src/ui/panels/php_versions.rs, in the CARD MIDDLE row:
   Look up php.minor in state.version_registry.php.
   If installed_full_version < latest_patch (semver compare):
     Append: RichText "  ↑ {latest_patch} available" in WARNING color.
   If up to date:
     Append nothing (or show "Up to date" in TEXT_TERTIARY).

   Use semver::Version::parse() for comparison.

C. TITLE BAR — refresh indicator + button
   In src/app.rs title bar (TopBottomPanel), RIGHT section:
   After the existing "PHP X.X · valet-linux" text:

   if state.version_registry_loading {
       // Animated dot (alternate color each frame using ctx.request_repaint())
       painter.circle_filled(pos, 4.0, Colors::WARNING);
   }
   // Small refresh button
   if ghost_button(ui, "↺").clicked() {
       cmd_tx.try_send(AppCommand::RefreshVersionRegistry).ok();
   }
   // Last refreshed: "updated 2h ago" in TEXT_TERTIARY 11px
   if let Some(ts) = &state.version_registry.last_refreshed {
       let mins = (chrono::Utc::now() - ts).num_minutes();
       let label = if mins < 60 { format!("{}m ago", mins) }
                   else { format!("{}h ago", mins / 60) };
       ui.label(RichText::new(label).size(10.0).color(Colors::TEXT_TERTIARY));
   }

D. SETTINGS PANEL — Version Registry section
   In src/ui/panels/settings.rs, add a new settings_section("version registry"):
   
   CARD content:
   Row 1: "Last refreshed" label LEFT | timestamp or "Never" TEXT_SECONDARY RIGHT
   Row 2: "Auto-refresh every" label LEFT | input (24, u32, suffix "hours") + save RIGHT
   Row 3: "Refresh now" accent_button (full row span)
           → cmd_tx.try_send(RefreshVersionRegistry)
           While loading: button disabled, shows "Refreshing…"
   
   Below the card: LIST of tracked sources in two columns:
   PHP versions, Laravel, WordPress, Symfony, CakePHP, Drupal, Craft,
   Statamic, FrankenPHP, Caddy, Mailpit, phpMyAdmin, Composer, Node.js LTS
   Each shows: name + fetched version + small green/amber dot.
   "version unknown" if not in registry (network failure).

E. APP CREATOR — PHP version auto-select
   In src/ui/panels/app_creator.rs Step 2 Configure, PHP version ComboBox:
   
   fn best_php_for_framework(fw_id: &str, registry: &VersionRegistry,
       installed: &[PhpVersion]) -> Option<String>
     Get fw info from registry.frameworks.get(fw_id)
     Parse min_php as semver::Version
     Find installed versions where version >= min_php, highest first
     Return the highest installed version meeting the requirement
   
   Pre-select this version as the default in the ComboBox.
   Show below the ComboBox (11px TEXT_TERTIARY):
     "Requires PHP {min_php}+ · {framework_name} {latest_version} available"

══════════════════════════════════════════════════════════
TESTS
══════════════════════════════════════════════════════════

tests/unit/version_registry_test.rs:

  #[test] fn default_registry_is_populated()
    let reg = VersionRegistry::default();
    assert!(!reg.php.is_empty(), "PHP versions must be populated");
    assert!(reg.frameworks.contains_key("laravel"));
    assert!(reg.frameworks.contains_key("wordpress"));

  #[test] fn php_eol_status_correct()
    let mut info = PhpVersionInfo { ...; security_support_until: NaiveDate::from_ymd(2025, 12, 31) };
    assert_eq!(info.eol_status(), EolStatus::Eol);
    info.security_support_until = chrono::Utc::now().date_naive() + chrono::Duration::days(90);
    assert_eq!(info.eol_status(), EolStatus::Critical);
    info.security_support_until = chrono::Utc::now().date_naive() + chrono::Duration::days(200);
    assert_eq!(info.eol_status(), EolStatus::Warning);
    info.security_support_until = chrono::Utc::now().date_naive() + chrono::Duration::days(400);
    assert_eq!(info.eol_status(), EolStatus::Active);

  #[test] fn cache_stale_detection()
    let cache = VersionRegistryCache {
        registry: VersionRegistry::default(),
        fetched_at: chrono::Utc::now() - chrono::Duration::hours(25),
        ttl_hours: 24,
    };
    assert!(cache.is_stale());
    let fresh_cache = VersionRegistryCache {
        fetched_at: chrono::Utc::now() - chrono::Duration::hours(1),
        ttl_hours: 24,
        ..cache
    };
    assert!(!fresh_cache.is_stale());

  #[tokio::test] async fn refresh_uses_cache_when_fresh()
    // Create a fresh cache file in tempdir
    // Call refresh(config, false, tx)
    // Assert no network calls made (check that tokio runtime doesn't spawn any tasks)
    // (Use a mock or check that the cache is returned)

══════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 14
══════════════════════════════════════════════════════════
  □ cargo build --workspace compiles — no errors after Cargo.toml version bumps
  □ egui 0.34.2 API migration complete: App::ui() used, no App::update() warnings
  □ rand 0.9 migration: distr::Alphanumeric used
  □ VersionRegistry::default() returns populated data for all 21 frameworks
  □ PHP EOL banners appear in dashboard when 8.1 (EOL) is installed
  □ PHP Versions panel shows "↑ X.Y.Z available" when patch update exists
  □ Title bar shows "↺" button and "Xm ago" last-refreshed label
  □ Settings panel Version Registry section shows all 14 tracked sources
  □ RefreshVersionRegistry command fetches all sources concurrently
  □ Network failure on any individual source falls back to bundled default
  □ Cache saved to ~/.config/valet-manager/version-registry.json after refresh
  □ Auto-refresh triggers on startup when cache is stale or missing
  □ App Creator PHP dropdown pre-selects best version for chosen framework
  □ cargo test --workspace — all new tests pass
  □ cargo clippy -- -D warnings — zero warnings
```

---

## Summary of Cargo.toml changes required

Paste this into the `[dependencies]` section of `valet-manager/Cargo.toml`.
Run `cargo update` after editing.

```toml
eframe      = { version = "0.34.2", features = ["default_fonts", "wgpu"] }
egui        = "0.34.2"
egui_extras = { version = "0.34.2", features = ["all_loaders"] }
tokio       = { version = "1.44", features = ["full"] }
serde       = { version = "1.219", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"
toml_edit   = "0.22"
anyhow      = "1"
thiserror   = "2"
tracing     = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dirs        = "6"
which       = "7"
chrono      = { version = "0.4.41", features = ["serde"] }
notify      = "8"
tray-icon   = "0.21.1"
notify-rust = "4"
nix         = { version = "0.29", features = ["process", "signal", "fs"] }
sysinfo     = "0.33"
regex       = "1"
futures     = "0.3"
fuzzy-matcher = "0.3.7"
reqwest     = { version = "0.12.15", features = ["json"] }
rusqlite    = { version = "0.34", features = ["bundled"] }
handlebars  = "6.3"
rand        = { version = "0.9", features = ["std"] }
bcrypt      = "0.15.1"
flate2      = "1.1"
tar         = "0.4.44"
rfd         = "0.15"
semver      = "1.0.25"
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
