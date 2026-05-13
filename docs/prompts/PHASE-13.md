> **Claude Design command** (run before implementing any panel in this phase):
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

---

# Claude Code Prompt — Framework Support
## Phase 13: All 21 Valet-Supported Frameworks

```
You are implementing comprehensive framework support for Valet Manager.
Phases 1–12 are complete.

Read these files before writing any code:
  src/valet/site_scanner.rs          — existing detect_framework() stub
  src/creator/project_types.rs       — existing project type definitions
  src/ui/theme.rs                    — framework_badge_colors()
  src/site_config/models.rs          — SiteConfig struct
  src/php/user_ini.rs                — user_ini_path() fn
  docs/specs/FRAMEWORKS.md          — the complete spec (PRIMARY REFERENCE)

══════════════════════════════════════════════════════════
PART 1 — FRAMEWORK DETECTION
══════════════════════════════════════════════════════════

Replace the stub detect_framework() in src/valet/site_scanner.rs with the
complete implementation from docs/specs/FRAMEWORKS.md §1.

The function signature stays:
  pub fn detect_framework(path: &Path) -> DetectedFramework

Critical ordering rules (read before implementing):
  1. Magento (bin/magento) before any artisan check
  2. Craft (craft file) before any PHP check
  3. Bedrock (web/wp/) before WordPress (wp-admin/)
  4. OctoberCMS (artisan + modules/backend/) before Laravel (artisan only)
  5. Statamic (artisan + vendor/statamic/) before Laravel
  6. Jigsaw (config.php + source/) before Symfony (bin/console)
  7. Sculpin (sculpin.json) before Symfony
  8. Joomla (administrator/ + configuration.php) after Drupal

All probes use std::path::Path methods — no subprocess calls here.
For Katana: read config.php content and check for "Katana" string.
For Slim/Zend: read composer.json and check for package name strings.

Add unit tests in tests/integration/framework_detection_test.rs:

  fn make_site(files: &[&str]) -> TempDir
    Creates a tempdir and touches each file path (creating parent dirs).
    Use tempfile::tempdir().

  Test all 21 frameworks:
  #[test] fn detects_laravel()        → make_site(&["artisan","public/index.php"])
  #[test] fn detects_bedrock()        → make_site(&["web/wp/wp-settings.php","config/application.php"])
  #[test] fn detects_wordpress()      → make_site(&["wp-admin/index.php"])
  #[test] fn detects_cakephp()        → make_site(&["config/app.php","src/Application.php"])
  #[test] fn detects_concretecms()    → make_site(&["concrete/index.php","index.php"])
  #[test] fn detects_contao()         → make_site(&["system/modules/.gitkeep"])
  #[test] fn detects_craft()          → make_site(&["craft"])
  #[test] fn detects_drupal()         → make_site(&["web/core/lib/Drupal.php"])
  #[test] fn detects_expressionengine() → make_site(&["system/ee/legacy/boot/basic.php"])
  #[test] fn detects_jigsaw()         → make_site(&["config.php","source/index.blade.php"])
  #[test] fn detects_joomla()         → make_site(&["administrator/index.php","configuration.php"])
  #[test] fn detects_kirby()          → make_site(&["kirby/bootstrap.php"])
  #[test] fn detects_magento()        → make_site(&["bin/magento"])
  #[test] fn detects_octobercms()     → make_site(&["artisan","modules/backend/Module.php"])
  #[test] fn detects_sculpin()        → make_site(&["sculpin.json"])
  #[test] fn detects_slim()           → {
    let dir = tempdir()?;
    write composer.json: {"require": {"slim/slim": "^4.0"}}
    assert_eq!(detect_framework(dir.path()), DetectedFramework::Slim);
  }
  #[test] fn detects_statamic()       → make_site(&["artisan","vendor/statamic/cms/README.md","public/index.php"])
  #[test] fn detects_static_html()    → make_site(&["index.html"])
  #[test] fn detects_symfony()        → make_site(&["bin/console","config/routes.yaml","public/index.php"])
  #[test] fn detects_zend()           → make_site(&["module/Application/src/Module.php","public/index.php"])
  #[test] fn detects_unknown()        → make_site(&["random.txt"])

  Priority tests (ensure no false positives):
  #[test] fn october_not_laravel()    → make_site(&["artisan","modules/backend/x.php"])
                                         assert == OctoberCms, not Laravel
  #[test] fn bedrock_not_wordpress()  → make_site(&["web/wp/wp-settings.php","config/application.php"])
                                         assert == Bedrock, not WordPress
  #[test] fn statamic_not_laravel()   → make_site(&["artisan","vendor/statamic/cms/x.php","public/index.php"])
                                         assert == Statamic, not Laravel

══════════════════════════════════════════════════════════
PART 2 — BADGE COLOURS
══════════════════════════════════════════════════════════

Replace framework_badge_colors() and add framework_display_name() in
src/ui/theme.rs with the complete implementation from docs/specs/FRAMEWORKS.md §2.

All 21 variants must be handled — no wildcard match fallthrough for known variants.

Add a test:
  #[test] fn all_frameworks_have_badge_color()
    for fw in DetectedFramework::all_variants():  // implement all_variants() iterator
      let (bg, text) = framework_badge_colors(&fw);
      assert!(bg.a() > 0, "bg alpha must be > 0 for {fw:?}");
      assert!(text.a() == 255, "text must be fully opaque for {fw:?}");

══════════════════════════════════════════════════════════
PART 3 — DOCUMENT ROOT MAPPING
══════════════════════════════════════════════════════════

Update src/php/user_ini.rs — replace the fn user_ini_path() with the
complete mapping from docs/specs/FRAMEWORKS.md §6:

  pub fn user_ini_path(site: &ValetSite) -> PathBuf
    Match site.framework:
      Laravel | Statamic | Slim | Symfony | Zend | Jigsaw-post-build → site.path.join("public/.user.ini")
      Bedrock | Craft | Drupal | Contao               → site.path.join("web/.user.ini")
      CakePHP                                           → site.path.join("webroot/.user.ini")
      WordPress | Joomla | Kirby | ConcreteCms
        | Magento | OctoberCms | ExpressionEngine
        | StaticHtml | Unknown                          → site.path.join(".user.ini")
      Jigsaw  → site.path.join("build_local/.user.ini")
      Sculpin → site.path.join("output_dev/.user.ini")
      Katana  → site.path.join("_output/.user.ini")

══════════════════════════════════════════════════════════
PART 4 — RECOMMENDED PHP VERSION
══════════════════════════════════════════════════════════

Add to src/valet/site_scanner.rs (or src/php/detector.rs):

  pub fn recommended_php_version(fw: &DetectedFramework) -> &'static str
    Full implementation from docs/specs/FRAMEWORKS.md §7.

Use this in the App Creator wizard when auto-selecting the PHP version
based on the chosen project type. Also used in the compatibility checker
suggestion message: "Consider isolating to PHP 8.2 (minimum for Drupal 10)".

══════════════════════════════════════════════════════════
PART 5 — APP CREATOR PROJECT TYPES
══════════════════════════════════════════════════════════

Replace all_project_types() in src/creator/project_types.rs with the
complete implementation from docs/specs/FRAMEWORKS.md §3.

Key implementation notes:

A. HELPER FUNCTIONS (define before all_project_types):

  fn text(key, label, default_val) -> ProjectOption
    Returns ProjectOption { key, label, option_type: Text, default: Text(default_val) }

  fn password(key, label, default_val) -> ProjectOption
    Returns ProjectOption { option_type: Password, ... }

  fn select(key, label, choices: &[(&str, &str)]) -> ProjectOption
    Returns ProjectOption { option_type: Select, choices: Some(vec), ... }

  fn toggle(key, label, default_val: bool) -> ProjectOption
    Returns ProjectOption { option_type: Toggle, default: Bool(default_val) }

  fn php_version_option() -> ProjectOption
    select("php_version", "PHP version", built from installed versions)
    Default: recommended_php_version for the framework

  fn db_options() -> Vec<ProjectOption>
    Returns vec![
      text("dbname",  "Database name", ""),
      text("dbuser",  "DB user",       "root"),
      password("dbpass", "DB password", ""),
      text("dbhost",  "DB host",       "127.0.0.1"),
    ]

  fn wp_standard_options() -> Vec<ProjectOption>
    Returns the standard 9 wp valet new options (version, locale, db type,
    dbname, dbuser, dbpass, admin_user, admin_password, admin_email)

B. COMMAND INTERPOLATION
   install_command strings use {name}, {flags}, {version} placeholders.
   The command builder in each runner resolves these from form_values HashMap.
   For wp valet new: build flags from all wp_standard_options values.

C. DURATION WARNINGS
   Magento warns: "Installation can take 10–20 minutes"
   Display this in Step 2 below the framework name badge.

D. POST-INSTALL NOTES
   Frameworks that need browser setup (Contao, Drupal, Joomla, ConcreteCMS)
   show a callout box on Step 5 (Complete screen):
     "⚠ To finish setup, visit: https://{domain}/{path}"
   This is driven by a `setup_url_suffix: Option<&'static str>` field on ProjectType.

E. STATIC HTML RUNNER
   For id="static-html", do NOT call stream_command.
   Instead, in-process:
     1. Create directory at parent_dir/name
     2. Write index.html from template based on selected template option
     3. Send synthetic OutputLine messages: "Creating directory...", "Writing index.html...", "Done"
   Template content:
     blank:     <!DOCTYPE html><html><head><title>{name}</title></head><body><h1>{name}</h1></body></html>
     tailwind:  Add <script src="https://cdn.tailwindcss.com"></script>
     bootstrap: Add Bootstrap CDN link

F. JIGSAW SPECIAL CASE
   Jigsaw is installed into the project directory, not as a project:
     cd {parent_dir}/{name}
     composer init --no-interaction --name="{name}/site"
     composer require tightenco/jigsaw
     vendor/bin/jigsaw init {starter}
   Stream each step. The `starter` option maps to jigsaw init templates.

G. SCULPIN SPECIAL CASE
   Sculpin has no skeleton project. Create manually:
     mkdir -p {name}/source/_layouts {name}/source/_posts
     cd {name} && composer init --no-interaction
     composer require sculpin/sculpin
   Then generate initial config and sample post.

H. EXPRESSIONENGINE SPECIAL CASE
   ExpressionEngine has no composer package. Show a download prompt:
     "ExpressionEngine requires a manual download from expressionengine.com"
     Show "Download ExpressionEngine" link button
     Show manual extraction instructions in the terminal widget
   Set install_command = "" and show_manual_download = true.

I. KATANA NOTE
   Katana is listed for detection only (the project was abandoned).
   Do NOT include it in all_project_types() — omit from App Creator.
   Keep DetectedFramework::Katana for detection of existing Katana sites.

══════════════════════════════════════════════════════════
PART 6 — FRAMEWORK CLI RUNNER
══════════════════════════════════════════════════════════

The Artisan panel currently only handles Laravel-style artisan.
Extend src/artisan/runner.rs to support other framework CLIs:

Create src/artisan/framework_cli.rs:

  #[derive(Debug, Clone, PartialEq)]
  pub enum FrameworkCli {
    Artisan,               // php artisan (Laravel, OctoberCMS, Statamic)
    SymfonyCli,            // php bin/console
    MagentoCli,            // php bin/magento
    DrushCli,              // vendor/bin/drush (Drupal)
    BinMagento,            // bin/magento (alias)
  }

  pub fn detect_cli(site: &ValetSite) -> Option<FrameworkCli>
    Check in order:
      artisan file exists → Artisan
      bin/console exists → SymfonyCli
      bin/magento exists → BinMagento (= MagentoCli)
      vendor/bin/drush exists → DrushCli
      None otherwise

  pub fn cli_binary(cli: &FrameworkCli, php_bin: &str) -> Vec<String>
    Returns the command + args prefix:
      Artisan      → vec![php_bin.to_string(), "artisan".to_string()]
      SymfonyCli   → vec![php_bin.to_string(), "bin/console".to_string()]
      BinMagento   → vec![php_bin.to_string(), "bin/magento".to_string()]
      DrushCli     → vec!["vendor/bin/drush".to_string()]

  pub async fn discover_commands(
    site: &ValetSite,
    cli: &FrameworkCli,
    php_bin: &str,
  ) -> Vec<ArtisanCommand>
    Match cli:
      Artisan | OctoberCms | Statamic →
        existing discover() using "php artisan list --format=json"
      SymfonyCli →
        "php bin/console list --format=json" — same JSON structure
      BinMagento →
        "php bin/magento list --format=json" — same structure
      DrushCli →
        "vendor/bin/drush --format=json list" → map to ArtisanCommand

In the Artisan panel UI:
  - Show panel for ALL sites where detect_cli() returns Some (not just Laravel)
  - Panel header shows the CLI type: "Artisan" / "bin/console" / "bin/magento" / "drush"
  - No functional changes to discover/run/history logic

══════════════════════════════════════════════════════════
PART 7 — COMPATIBILITY CHECKER UPDATES
══════════════════════════════════════════════════════════

In src/compat/checker.rs, update check_site() to handle frameworks
without composer.json:

  WordPress: Read wp-includes/version.php for $wp_version, then check
    known PHP requirements per WP version:
      WP 6.4+: PHP 7.2.24+
      WP 6.6+: PHP 7.2.24+  
      WP 6.7+: PHP 7.2.24+ (tested on 8.x)
    Return CompatStatus::Compatible if PHP >= 7.2.

  Magento: Read app/etc/di.xml or composer.json for PHP constraint.

  Drupal: Try web/core/composer.json if root composer.json missing.

  Joomla: No programmatic version detection — return NoRequirement.
    Show note: "Check Joomla system requirements page manually"

  ExpressionEngine: Return NoRequirement.

  Katana/Sculpin/Jigsaw/StaticHtml: Return NoRequirement.

Add recommended_php_version() to the suggestion text:
  If incompatible: "Compatible with PHP {min_version}+. Suggest isolating to PHP {recommended}"
  Use recommended_php_version(fw) from Part 4.

══════════════════════════════════════════════════════════
PART 8 — SITE CONFIG FRAMEWORK PANELS
══════════════════════════════════════════════════════════

Update src/site_config/models.rs — add framework-specific config structs
from docs/specs/FRAMEWORKS.md §5:

  // Add to SiteConfig:
  #[serde(default)]
  pub craft: Option<CraftConfig>,
  #[serde(default)]
  pub concretecms: Option<ConcreteCmsConfig>,
  #[serde(default)]
  pub drupal: Option<DrupalConfig>,
  #[serde(default)]
  pub joomla: Option<JoomlaConfig>,
  #[serde(default)]
  pub magento: Option<MagentoConfig>,
  #[serde(default)]
  pub octobercms: Option<OctoberCmsConfig>,
  #[serde(default)]
  pub statamic: Option<StatamicConfig>,

Implement each struct (Serialize + Deserialize + Default):

  CraftConfig:     environment, license_key, db_driver, use_project_config
  ConcreteCmsConfig: environment, cache_enabled, pretty_urls
  DrupalConfig:    environment, trusted_host_patterns, cache_bins
  JoomlaConfig:    error_reporting, sef_urls, debug, cache_enabled
  MagentoConfig:   mode, indexer_mode
  OctoberCmsConfig: debug_mode, backend_path
  StatamicConfig:  flat_file, git_integration, api_enabled

In the site config panel (src/ui/panels/site_config.rs), the "Framework"
tab already exists. Wire these framework-specific config sections:
  - Only show the section matching site.framework
  - Show the relevant fields as inputs (selects/toggles/text)
  - Each change updates the in-memory SiteConfig and marks dirty
  - Save persists via SaveSiteConfig command

For Magento, add action buttons (dispatching subprocess commands):
  "Flush cache" → stream "php bin/magento cache:flush"
  "Reindex"     → stream "php bin/magento indexer:reindex"
  "Set dev mode" → stream "php bin/magento deploy:mode:set developer"

For Drupal, add:
  "Rebuild cache" → stream "vendor/bin/drush cr"
  "Update DB"     → stream "vendor/bin/drush updb -y"

For Statamic:
  "Run git sync" → stream "php artisan statamic:git:sync" (if git_integration)

══════════════════════════════════════════════════════════
PART 9 — FRAMEWORK INFO IN SITES PANEL
══════════════════════════════════════════════════════════

In src/ui/panels/sites.rs, update the framework badge column:

  1. Call theme::framework_badge(ui, &site.framework) — already exists,
     but ensure it uses framework_display_name() from the updated theme.rs

  2. Add a tooltip on the badge (shown on hover):
       "{framework_display_name} · PHP {recommended_php_version}"
     If site.php_version is Some and doesn't match recommended:
       Tooltip adds: " (site uses PHP {actual}, consider {recommended})"

  3. In the ⋮ context menu, add "Open framework docs" linking to:
       Laravel → laravel.com/docs
       WordPress → developer.wordpress.org
       Symfony → symfony.com/doc/current
       CakePHP → book.cakephp.org
       Drupal → drupal.org/docs
       etc.
     Use a HashMap<DetectedFramework, &'static str> constant for doc URLs.

══════════════════════════════════════════════════════════
PART 10 — COMMAND PALETTE UPDATES
══════════════════════════════════════════════════════════

In src/ui/command_palette.rs, update build_index() to add framework-
specific actions when a site is selected:

  Magento sites:
    "Flush Magento cache" → RunFrameworkCli("php bin/magento cache:flush")
    "Reindex Magento"     → RunFrameworkCli("php bin/magento indexer:reindex")

  Drupal sites:
    "Drupal cache rebuild" → RunFrameworkCli("vendor/bin/drush cr")
    "Drupal update DB"     → RunFrameworkCli("vendor/bin/drush updb -y")

  Symfony sites:
    "Clear Symfony cache"  → RunArtisanCommand("cache:clear", via bin/console)

  OctoberCMS sites:
    "OctoberCMS migrate"   → RunArtisanCommand("october:migrate")

══════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 13
══════════════════════════════════════════════════════════
  □ cargo test --workspace — all 21 framework detection tests pass
  □ Priority ordering tests pass (OctoberCMS not mistaken for Laravel, etc.)
  □ All 21 DetectedFramework variants have a non-empty badge color pair
  □ framework_display_name() returns a clean label for all variants
  □ user_ini_path() returns correct path for all 21 frameworks
  □ App Creator shows 20 project types (Katana excluded) in correct groups
  □ Static HTML creator produces valid index.html without subprocess
  □ Jigsaw install uses composer require then vendor/bin/jigsaw init
  □ ExpressionEngine shows download prompt, not a command
  □ Magento install shows duration warning and streams setup:install
  □ detect_cli() returns correct CLI for Laravel, Symfony, Magento, Drupal
  □ Artisan panel shows for Symfony (bin/console) and Drupal (drush) sites
  □ Compatibility checker handles WordPress (reads wp-includes/version.php)
  □ Framework tab in site config shows correct fields for each framework
  □ Sites panel badge tooltip includes recommended PHP version
  □ "Open framework docs" context menu item opens correct URL
  □ Command palette shows framework-specific quick actions for Magento/Drupal
```

---

## Update INDEX.md

Add to docs/INDEX.md under `specs/`:
```
specs/FRAMEWORKS.md    All 21 Valet-supported frameworks: detection, badges, App Creator, config
```

Add to the prompts table:
```
PHASE-13.md    Framework support implementation (Phase 13 prompt)
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
