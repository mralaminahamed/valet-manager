# Framework Support Spec
## All 21 Valet-Supported Frameworks
## Detection · App Creator · Badges · Config · Drivers

---

## Framework inventory

| # | Framework | Valet driver | App Creator | Artisan | Notes |
|---|---|---|---|---|---|
| 1 | Laravel | LaravelValetDriver | ✅ | ✅ | Core framework |
| 2 | Bedrock | BedrockValetDriver | ✅ | ✗ | WordPress via roots.io |
| 3 | CakePHP 3 | CakePhpValetDriver | ✅ | ✗ | webroot = webroot/ |
| 4 | ConcreteCMS | Concrete5ValetDriver | ✅ | ✗ | webroot = root |
| 5 | Contao | ContaoValetDriver | ✅ | ✗ | webroot = web/ |
| 6 | Craft CMS | CraftValetDriver | ✅ | ✗ | needs `craft setup` |
| 7 | Drupal | DrupalValetDriver | ✅ | ✗ | webroot = web/ |
| 8 | ExpressionEngine | ExpressionEngineValetDriver | ⚠ download | ✗ | no Composer package |
| 9 | Jigsaw | JigsawValetDriver | ✅ | ✗ | static site generator |
| 10 | Joomla | JoomlaValetDriver | ✅ | ✗ | webroot = root |
| 11 | Katana | KatanaValetDriver | ✗ | ✗ | abandoned, detect only |
| 12 | Kirby | KirbyValetDriver | ✅ | ✗ | flat-file CMS |
| 13 | Magento | MagentoValetDriver | ✅ | ✅ (bin/magento) | heavy, long install |
| 14 | OctoberCMS | OctoberValetDriver | ✅ | ✅ | artisan-based |
| 15 | Sculpin | SculpinValetDriver | ✅ | ✗ | static site generator |
| 16 | Slim | SlimValetDriver | ✅ | ✗ | micro-framework |
| 17 | Statamic | StatamicValetDriver | ✅ | ✅ | flat-file or Eloquent |
| 18 | Static HTML | BasicValetDriver | ✅ | ✗ | bare HTML |
| 19 | Symfony | SymfonyValetDriver | ✅ | ✅ (bin/console) | |
| 20 | WordPress | WordPressValetDriver | ✅ | ✗ (WP-CLI) | |
| 21 | Zend | ZendValetDriver | ✅ | ✗ | Laminas fork preferred |

---

## 1. Detection heuristics

Detection order matters. Probe in this exact sequence to avoid false positives.

```rust
// src/valet/site_scanner.rs

pub fn detect_framework(path: &Path) -> DetectedFramework {
    // ── Priority 1: unambiguous single-file signatures ────────────────
    // Magento: bin/magento exists (before any artisan check)
    if path.join("bin/magento").exists() {
        return DetectedFramework::Magento;
    }

    // Craft CMS: craft binary at root
    if path.join("craft").exists() {
        return DetectedFramework::Craft;
    }

    // ExpressionEngine: system/ee/ directory
    if path.join("system/ee").is_dir() {
        return DetectedFramework::ExpressionEngine;
    }

    // Katana: config.php containing class SiteConfiguration or Katana string
    if path.join("config.php").exists()
        && path.join("posts").is_dir()
        && !path.join("source").is_dir() {
        let content = fs::read_to_string(path.join("config.php")).unwrap_or_default();
        if content.contains("Katana") || content.contains("SiteConfiguration") {
            return DetectedFramework::Katana;
        }
    }

    // ── Priority 2: WordPress family ─────────────────────────────────
    // Bedrock: web/wp/ directory (check BEFORE generic WordPress)
    if path.join("web/wp").is_dir() && path.join("config/application.php").exists() {
        return DetectedFramework::Bedrock;
    }

    // WordPress: wp-admin/ directory
    if path.join("wp-admin").is_dir() {
        return DetectedFramework::WordPress;
    }

    // ── Priority 3: artisan-based frameworks ─────────────────────────
    if path.join("artisan").exists() {
        let artisan = fs::read_to_string(path.join("artisan")).unwrap_or_default();

        // OctoberCMS: artisan + modules/backend/ (check BEFORE Laravel)
        if path.join("modules/backend").is_dir() {
            return DetectedFramework::OctoberCms;
        }

        // Statamic: artisan + statamic in vendor or composer.json
        if path.join("vendor/statamic").is_dir() {
            return DetectedFramework::Statamic;
        }
        let composer = fs::read_to_string(path.join("composer.json")).unwrap_or_default();
        if composer.contains("\"statamic/cms\"") || composer.contains("statamic/statamic") {
            return DetectedFramework::Statamic;
        }

        // Laravel: artisan + public/index.php
        if path.join("public/index.php").exists() {
            return DetectedFramework::Laravel;
        }
    }

    // ── Priority 4: Symfony family ────────────────────────────────────
    // Jigsaw: config.php + source/ directory (BEFORE Symfony — no bin/console)
    if path.join("config.php").exists() && path.join("source").is_dir() {
        return DetectedFramework::Jigsaw;
    }

    // Sculpin: sculpin.json OR app/SculpinKernel.php
    if path.join("sculpin.json").exists()
        || path.join("app/SculpinKernel.php").exists() {
        return DetectedFramework::Sculpin;
    }

    // Symfony: bin/console + config/ + public/index.php
    if path.join("bin/console").exists()
        && path.join("config").is_dir()
        && path.join("public/index.php").exists() {
        return DetectedFramework::Symfony;
    }

    // ── Priority 5: CMS platforms ────────────────────────────────────
    // Drupal: web/core/lib/Drupal.php OR core/lib/Drupal.php
    if path.join("web/core/lib/Drupal.php").exists()
        || path.join("core/lib/Drupal.php").exists() {
        return DetectedFramework::Drupal;
    }

    // Joomla: administrator/ + includes/defines.php OR configuration.php
    if path.join("administrator").is_dir()
        && (path.join("includes/defines.php").exists()
            || path.join("configuration.php").exists()) {
        return DetectedFramework::Joomla;
    }

    // ConcreteCMS: concrete/ directory + index.php at root
    if path.join("concrete").is_dir() && path.join("index.php").exists() {
        let index = fs::read_to_string(path.join("index.php")).unwrap_or_default();
        if index.contains("concrete") || index.contains("Concrete") {
            return DetectedFramework::ConcreteCms;
        }
    }

    // Contao: system/modules/ OR vendor/contao/ directory
    if path.join("system/modules").is_dir()
        || path.join("vendor/contao").is_dir()
        || path.join("system/config/constants.php").exists() {
        return DetectedFramework::Contao;
    }

    // Kirby: kirby/ directory
    if path.join("kirby").is_dir() {
        return DetectedFramework::Kirby;
    }

    // ── Priority 6: micro-frameworks ─────────────────────────────────
    // CakePHP 3/4: config/app.php + src/
    if path.join("config/app.php").exists() && path.join("src").is_dir() {
        return DetectedFramework::CakePHP;
    }

    // Slim: composer.json with slim/slim dependency
    if path.join("composer.json").exists() {
        let composer = fs::read_to_string(path.join("composer.json")).unwrap_or_default();
        if composer.contains("\"slim/slim\"") {
            return DetectedFramework::Slim;
        }
        // Zend/Laminas: module/ + public/index.php OR zendframework in composer
        if (path.join("module").is_dir() && path.join("public/index.php").exists())
            || composer.contains("zendframework")
            || composer.contains("laminas") {
            return DetectedFramework::Zend;
        }
    }

    // ── Priority 7: static sites ─────────────────────────────────────
    // Static HTML: index.html at root, no PHP files at root
    if path.join("index.html").exists() && !path.join("index.php").exists() {
        return DetectedFramework::StaticHtml;
    }

    DetectedFramework::Unknown
}
```

---

## 2. Badge colours (src/ui/theme.rs)

```rust
pub fn framework_badge_colors(fw: &DetectedFramework) -> (Color32, Color32) {
    // Returns (background_rgba, text_color)
    use DetectedFramework::*;
    match fw {
        // Amber family
        Laravel    => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,30), Color32::from_rgb(0xBA,0x75,0x17)),
        Magento    => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,25), Color32::from_rgb(0x85,0x4F,0x0B)),
        Joomla     => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,22), Color32::from_rgb(0x85,0x4F,0x0B)),
        ExpressionEngine => (Color32::from_rgba_unmultiplied(0xEF,0x9F,0x27,20), Color32::from_rgb(0x63,0x38,0x06)),

        // Blue family
        WordPress  => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,30), Color32::from_rgb(0x18,0x5F,0xA5)),
        Bedrock    => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,22), Color32::from_rgb(0x0C,0x44,0x7C)),
        Drupal     => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,35), Color32::from_rgb(0x18,0x5F,0xA5)),
        Symfony    => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,20), Color32::from_rgb(0x0C,0x44,0x7C)),
        Contao     => (Color32::from_rgba_unmultiplied(0x37,0x8A,0xDD,18), Color32::from_rgb(0x0C,0x44,0x7C)),

        // Teal family
        Craft      => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,25), Color32::from_rgb(0x08,0x50,0x41)),
        Statamic   => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,30), Color32::from_rgb(0x0F,0x6E,0x56)),
        Slim       => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,20), Color32::from_rgb(0x08,0x50,0x41)),
        Jigsaw     => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,22), Color32::from_rgb(0x0F,0x6E,0x56)),
        Sculpin    => (Color32::from_rgba_unmultiplied(0x5D,0xCA,0xA5,18), Color32::from_rgb(0x08,0x50,0x41)),

        // Red / coral family
        CakePHP    => (Color32::from_rgba_unmultiplied(0xE2,0x4B,0x4A,25), Color32::from_rgb(0xA3,0x2D,0x2D)),
        Kirby      => (Color32::from_rgba_unmultiplied(0xE2,0x4B,0x4A,22), Color32::from_rgb(0xA3,0x2D,0x2D)),

        // Purple family
        OctoberCms => (Color32::from_rgba_unmultiplied(0x7F,0x77,0xDD,25), Color32::from_rgb(0x53,0x4A,0xB7)),
        Katana     => (Color32::from_rgba_unmultiplied(0x7F,0x77,0xDD,20), Color32::from_rgb(0x3C,0x34,0x89)),

        // Pink family
        ConcreteCms => (Color32::from_rgba_unmultiplied(0xD4,0x53,0x7E,22), Color32::from_rgb(0x99,0x35,0x56)),

        // Gray
        Zend       => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,25), Color32::from_rgb(0x5F,0x5E,0x5A)),
        StaticHtml => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,15), Color32::from_rgb(0x5F,0x5E,0x5A)),
        Unknown    => (Color32::from_rgba_unmultiplied(0x88,0x87,0x80,10), Color32::from_rgb(0x44,0x44,0x41)),
    }
}

/// Short display name for the framework badge
pub fn framework_display_name(fw: &DetectedFramework) -> &'static str {
    use DetectedFramework::*;
    match fw {
        Laravel       => "Laravel",
        WordPress     => "WordPress",
        Bedrock       => "Bedrock",
        CakePHP       => "CakePHP",
        ConcreteCms   => "Concrete5",
        Contao        => "Contao",
        Craft         => "Craft CMS",
        Drupal        => "Drupal",
        ExpressionEngine => "ExpressionEngine",
        Jigsaw        => "Jigsaw",
        Joomla        => "Joomla",
        Katana        => "Katana",
        Kirby         => "Kirby",
        Magento       => "Magento",
        OctoberCms    => "OctoberCMS",
        Sculpin       => "Sculpin",
        Slim          => "Slim",
        Statamic      => "Statamic",
        StaticHtml    => "Static HTML",
        Symfony       => "Symfony",
        Zend          => "Zend/Laminas",
        Unknown       => "PHP",
    }
}
```

---

## 3. App Creator entries (src/creator/project_types.rs)

Complete `all_project_types()` function with all 21 frameworks plus variants.

```rust
pub fn all_project_types() -> Vec<ProjectType> {
    vec![

    // ══════════════════════════════════════════════════════════════════
    // GROUP: Laravel Ecosystem
    // ══════════════════════════════════════════════════════════════════

    ProjectType {
        id: "laravel-blank",
        display_name: "Laravel",
        description: "Clean Laravel installation with no starter kit",
        group: ProjectGroup::LaravelEcosystem,
        required_tools: vec![CliTool::LaravelInstaller, CliTool::Composer],
        install_command: "laravel new {name}",
        post_install: PostInstall::ValetLink | PostInstall::Secure,
        options: vec![
            select("php_version", "PHP version", php_version_choices()),
        ],
    },
    ProjectType {
        id: "laravel-breeze",
        display_name: "Laravel + Breeze",
        description: "Laravel with Breeze auth scaffolding",
        group: ProjectGroup::LaravelEcosystem,
        required_tools: vec![CliTool::LaravelInstaller, CliTool::Npm],
        install_command: "laravel new {name} --breeze --stack={stack} --pest",
        options: vec![
            select("stack", "Frontend stack", &[
                ("blade",    "Blade (simple)"),
                ("react",    "React + Inertia"),
                ("vue",      "Vue + Inertia"),
                ("livewire", "Livewire"),
                ("api",      "API only"),
            ]),
            toggle("dark_mode", "Dark mode", false),
            toggle("ssr", "SSR (React/Vue only)", false),
        ],
    },
    ProjectType {
        id: "laravel-jetstream",
        display_name: "Laravel + Jetstream",
        description: "Laravel with Jetstream auth (teams + roles)",
        group: ProjectGroup::LaravelEcosystem,
        required_tools: vec![CliTool::LaravelInstaller, CliTool::Npm],
        install_command: "laravel new {name} --jet --stack={stack}",
        options: vec![
            select("stack", "Stack", &[("livewire","Livewire"),("inertia","Inertia")]),
            toggle("teams", "Enable teams", false),
        ],
    },
    ProjectType {
        id: "laravel-api",
        display_name: "Laravel API",
        description: "Headless API — no frontend scaffolding",
        group: ProjectGroup::LaravelEcosystem,
        required_tools: vec![CliTool::LaravelInstaller],
        install_command: "laravel new {name} --api",
        options: vec![],
    },
    ProjectType {
        id: "statamic",
        display_name: "Statamic",
        description: "Flat-file CMS built on Laravel",
        group: ProjectGroup::LaravelEcosystem,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project statamic/statamic {name}",
        post_install: PostInstall::ValetLink | PostInstall::Secure | PostInstall::RunArtisan("make:user"),
        options: vec![
            select("edition", "Edition", &[
                ("free", "Free"),
                ("pro",  "Pro (license required)"),
            ]),
        ],
    },

    // ══════════════════════════════════════════════════════════════════
    // GROUP: WordPress Ecosystem
    // ══════════════════════════════════════════════════════════════════

    ProjectType {
        id: "wordpress",
        display_name: "WordPress",
        description: "Full WordPress install via wp valet new",
        group: ProjectGroup::WordPressEcosystem,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        install_command: "wp valet new {name} {wp_flags}",
        options: wp_standard_options(),
    },
    ProjectType {
        id: "wordpress-sqlite",
        display_name: "WordPress (SQLite)",
        description: "Portable WordPress — no MySQL required",
        group: ProjectGroup::WordPressEcosystem,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        install_command: "wp valet new {name} --db=sqlite --unsecure",
        options: vec![
            text("locale", "Locale", "en_US"),
            text("admin_email", "Admin email", "admin@example.com"),
        ],
    },
    ProjectType {
        id: "bedrock",
        display_name: "Bedrock",
        description: "Modern WordPress stack by Roots.io (12-factor)",
        group: ProjectGroup::WordPressEcosystem,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand, CliTool::Composer],
        install_command: "wp valet new {name} --project=bedrock",
        post_install: PostInstall::ValetLink | PostInstall::Secure | PostInstall::WriteEnv,
        options: wp_standard_options(),
    },
    ProjectType {
        id: "wordpress-woocommerce",
        display_name: "WooCommerce",
        description: "WordPress + WooCommerce pre-installed",
        group: ProjectGroup::WordPressEcosystem,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        install_command: "wp valet new {name} {wp_flags}",
        post_install_extra: "wp plugin install woocommerce --activate",
        options: wp_standard_options(),
    },
    ProjectType {
        id: "wordpress-multisite",
        display_name: "WordPress Multisite",
        description: "WordPress network installation",
        group: ProjectGroup::WordPressEcosystem,
        required_tools: vec![CliTool::WpCli],
        // Multi-step: standard install, then multisite-install
        install_command: "wp core download && wp config create {db_flags} && wp core install {wp_flags}",
        post_install_extra: "wp core multisite-install --subdomains={subdomains}",
        options: vec![
            select("multisite_type", "Network type", &[
                ("subdomain",    "Subdomain (blog1.site.test)"),
                ("subdirectory", "Subdirectory (site.test/blog1)"),
            ]),
            // Plus standard WP options
            text("admin_email", "Admin email", "admin@example.com"),
            text("dbuser",      "DB user",     "root"),
            password("dbpass",  "DB password", ""),
        ],
    },

    // ══════════════════════════════════════════════════════════════════
    // GROUP: PHP Frameworks
    // ══════════════════════════════════════════════════════════════════

    ProjectType {
        id: "symfony-full",
        display_name: "Symfony (full)",
        description: "Full Symfony webapp skeleton",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project symfony/website-skeleton {name}",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "symfony-micro",
        display_name: "Symfony (micro)",
        description: "Minimal Symfony skeleton for APIs/microservices",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project symfony/skeleton {name}",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "cakephp",
        display_name: "CakePHP 4",
        description: "CakePHP rapid development framework",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project cakephp/app:{version} {name}",
        options: vec![
            select("version", "Version", &[
                ("5.*", "CakePHP 5 (PHP 8.1+)"),
                ("4.*", "CakePHP 4 (PHP 7.4+)"),
            ]),
            php_version_option(),
        ],
    },
    ProjectType {
        id: "concretecms",
        display_name: "ConcreteCMS",
        description: "ConcreteCMS (formerly Concrete5)",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project concrete5/composer {name}",
        options: vec![
            db_options(),
            php_version_option(),
        ],
    },
    ProjectType {
        id: "contao",
        display_name: "Contao",
        description: "Contao open source CMS",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project contao/managed-edition {name}",
        post_install_note: "Run Contao install tool at https://{domain}/contao/install to complete setup",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "craft",
        display_name: "Craft CMS",
        description: "Craft CMS — content-first CMS",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project craftcms/craft {name}",
        post_install_extra: "php craft setup",
        options: vec![
            db_options(),
            php_version_option(),
        ],
    },
    ProjectType {
        id: "drupal",
        display_name: "Drupal",
        description: "Drupal CMS",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project drupal/recommended-project {name}",
        post_install_note: "Visit https://{domain}/install.php to complete Drupal setup",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "jigsaw",
        display_name: "Jigsaw",
        description: "Static site generator by Tighten",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer, CliTool::Npm],
        install_command: "composer create-project tightenco/jigsaw {name}",
        post_install_extra: "npm install && npm run dev",
        options: vec![
            select("starter", "Starter template", &[
                ("blank", "Blank"),
                ("blog",  "Blog"),
                ("docs",  "Documentation"),
            ]),
        ],
    },
    ProjectType {
        id: "joomla",
        display_name: "Joomla",
        description: "Joomla CMS",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project joomla/joomla-cms {name}",
        post_install_note: "Visit https://{domain}/installation/index.php to complete Joomla setup",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "kirby",
        display_name: "Kirby",
        description: "Kirby — file-based CMS",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project getkirby/starterkit {name}",
        options: vec![
            select("edition", "Edition", &[
                ("starterkit", "Starterkit"),
                ("plainkit",   "Plainkit (minimal)"),
            ]),
        ],
    },
    ProjectType {
        id: "magento",
        display_name: "Magento 2",
        description: "Magento 2 e-commerce platform",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project magento/project-community-edition {name}",
        post_install_extra: "php bin/magento setup:install --base-url=https://{domain}/ --db-host=localhost --db-name={dbname} --db-user={dbuser} --db-password={dbpass} --admin-firstname=Admin --admin-lastname=User --admin-email={admin_email} --admin-user=admin --admin-password=Admin123! --backend-frontname=admin",
        duration_warning: "Installation takes 5–15 minutes",
        options: vec![
            db_options(),
            text("admin_email", "Admin email", "admin@example.com"),
        ],
    },
    ProjectType {
        id: "octobercms",
        display_name: "OctoberCMS",
        description: "OctoberCMS built on Laravel",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project october/october {name}",
        post_install_extra: "php artisan october:migrate",
        options: vec![db_options(), php_version_option()],
    },
    ProjectType {
        id: "sculpin",
        display_name: "Sculpin",
        description: "Static site generator for PHP developers",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        // Sculpin is typically installed locally in the project
        install_command: "mkdir -p {name}/source && cd {name} && composer init --no-interaction && composer require sculpin/sculpin",
        post_install_extra: "vendor/bin/sculpin generate --watch --server",
        options: vec![],
    },
    ProjectType {
        id: "slim",
        display_name: "Slim Framework",
        description: "Slim — PHP micro-framework",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project slim/slim-skeleton {name}",
        options: vec![php_version_option()],
    },
    ProjectType {
        id: "zend-laminas",
        display_name: "Laminas (Zend)",
        description: "Laminas Project (formerly Zend Framework)",
        group: ProjectGroup::OtherPhp,
        required_tools: vec![CliTool::Composer],
        install_command: "composer create-project laminas/mvc-skeleton {name}",
        options: vec![php_version_option()],
    },

    // ══════════════════════════════════════════════════════════════════
    // GROUP: Static
    // ══════════════════════════════════════════════════════════════════

    ProjectType {
        id: "static-html",
        display_name: "Static HTML",
        description: "Plain HTML/CSS/JS — no PHP",
        group: ProjectGroup::Static,
        required_tools: vec![],
        // Created in-process, no subprocess
        install_command: "",
        options: vec![
            select("template", "Starter template", &[
                ("blank",     "Blank (just index.html)"),
                ("tailwind",  "Tailwind CDN"),
                ("bootstrap", "Bootstrap CDN"),
            ]),
        ],
    },

    ] // end vec!
}
```

---

## 4. Post-install steps per framework

| Framework | Auto-link | Auto-secure | Extra command | Browser note |
|---|---|---|---|---|
| Laravel | ✅ | ✅ | — | Opens https://name.test |
| Bedrock | ✅ | ✅ | `cp .env.example .env && php artisan key:generate` | |
| CakePHP | ✅ | ✅ | — | |
| ConcreteCMS | ✅ | ✅ | — | Visit /_install to finish |
| Contao | ✅ | ✅ | — | Visit /contao/install |
| Craft CMS | ✅ | ✅ | `php craft setup` (prompts via terminal widget) | |
| Drupal | ✅ | ✅ | — | Visit /install.php |
| ExpressionEngine | ✅ | ✅ | — | Manual upload required |
| Jigsaw | ✅ | ✅ | `npm install && npm run dev` | |
| Joomla | ✅ | ✅ | — | Visit /installation/index.php |
| Kirby | ✅ | ✅ | — | Opens https://name.test |
| Magento | ✅ | ✅ | `bin/magento setup:install ...` | Long install — progress bar |
| OctoberCMS | ✅ | ✅ | `php artisan october:migrate` | |
| Sculpin | ✅ | ✅ | `vendor/bin/sculpin generate` | |
| Slim | ✅ | ✅ | — | |
| Statamic | ✅ | ✅ | `php artisan make:user` | |
| Symfony | ✅ | ✅ | — | |
| WordPress | Auto via wp valet new | Auto via wp valet new | — | |
| Zend/Laminas | ✅ | ✅ | — | |
| Static HTML | ✅ | ✅ | — | |

---

## 5. Framework-specific site config options

Beyond the common PHP/server/database config, each framework adds to the site config panel.

### Craft CMS

```toml
[craft]
environment = "dev"         # dev | staging | production
license_key = ""
db_driver   = "mysql"       # mysql | pgsql
db_port     = 3306
use_project_config = true
```

**Panel additions:** Run `craft up` button, cache clear, project config sync.

### ConcreteCMS

```toml
[concretecms]
environment = "development"  # development | production
cache_enabled = false
pretty_urls = true
```

**Panel additions:** Cache clear button, update check.

### Drupal

```toml
[drupal]
environment = "local"
trusted_host_patterns = true
cache_bins = "null"          # null | default (disables cache in dev)
```

**Panel additions:** Cache rebuild (`drush cr`), update DB (`drush updb`).

### Joomla

```toml
[joomla]
error_reporting = "maximum"  # minimum | normal | maximum | development
sef_urls = true
sef_suffix = false
debug = true
cache_enabled = false
```

### Magento

```toml
[magento]
mode = "developer"           # developer | production | default
indexer_mode = "realtime"    # realtime | schedule
```

**Panel additions:**
- Mode switcher (`bin/magento deploy:mode:set`)
- Cache flush (`bin/magento cache:flush`)
- Reindex (`bin/magento indexer:reindex`)
- Admin URL display

### OctoberCMS

```toml
[octobercms]
debug_mode = true
backend_path = "backend"
```

**Panel additions:** Backend URL display, plugin list.

### Statamic

```toml
[statamic]
flat_file = true             # true = flat-file, false = Eloquent
git_integration = false
api_enabled = false
```

### Sculpin / Jigsaw (static generators)

```toml
[static_generator]
output_dir = "_site"         # Sculpin default
watch = false                # auto-rebuild on change
```

**Panel additions:** "Build" button, "Watch" toggle → runs generator in background.

---

## 6. Valet driver → document root mapping

Important for `.user.ini` placement and Nginx `root` directive:

| Framework | Document root | .user.ini location |
|---|---|---|
| Laravel | `public/` | `public/.user.ini` |
| Bedrock | `web/` | `web/.user.ini` |
| CakePHP 3/4 | `webroot/` | `webroot/.user.ini` |
| ConcreteCMS | `/` (root) | `.user.ini` |
| Contao | `web/` or `public/` | `web/.user.ini` |
| Craft CMS | `web/` | `web/.user.ini` |
| Drupal | `web/` | `web/.user.ini` |
| ExpressionEngine | `/` | `.user.ini` |
| Jigsaw | `build_local/` | `build_local/.user.ini` |
| Joomla | `/` | `.user.ini` |
| Katana | `_output/` | `_output/.user.ini` |
| Kirby | `/` | `.user.ini` |
| Magento | `/` | `.user.ini` |
| OctoberCMS | `/` | `.user.ini` |
| Sculpin | `output_dev/` | `output_dev/.user.ini` |
| Slim | `public/` | `public/.user.ini` |
| Statamic | `public/` | `public/.user.ini` |
| Static HTML | `/` | `.user.ini` |
| Symfony | `public/` | `public/.user.ini` |
| WordPress | `/` | `.user.ini` |
| Zend/Laminas | `public/` | `public/.user.ini` |

---

## 7. Default recommended PHP version per framework

```rust
pub fn recommended_php_version(fw: &DetectedFramework) -> &'static str {
    use DetectedFramework::*;
    match fw {
        Laravel | Statamic | Slim | Symfony      => "8.3",
        Bedrock | WordPress | OctoberCms          => "8.2",
        CakePHP | Kirby | Craft                   => "8.2",
        Drupal                                     => "8.2",
        Joomla | ConcreteCms                       => "8.1",
        Magento                                    => "8.2",
        Contao                                     => "8.2",
        ExpressionEngine                           => "8.1",
        Jigsaw | Sculpin                           => "8.2",
        Zend                                       => "8.1",
        Katana | StaticHtml | Unknown             => "8.3",
    }
}
```

---

## 8. Artisan support per framework

Not just Laravel — some frameworks have artisan-compatible CLIs:

| Framework | CLI binary | Common commands |
|---|---|---|
| Laravel | `php artisan` | migrate, serve, make:*, cache:clear, queue:work |
| OctoberCMS | `php artisan` | october:migrate, october:up, make:component |
| Statamic | `php artisan` (partial) | statamic:install, make:collection |
| Symfony | `php bin/console` | doctrine:migrations:migrate, cache:clear, make:* |
| Magento | `php bin/magento` | setup:upgrade, cache:flush, indexer:reindex |
| Drupal | `vendor/bin/drush` | cr, updb, en, dis |
| Joomla | `php cli/joomla.php` (v4+) | extension:install |

The Artisan Runner panel already handles Laravel and OctoberCMS.
For non-artisan frameworks, add a "Framework CLI" runner in the same panel
that detects and uses the correct binary.

---

## 9. Composer.json `require.php` constraints by framework

Used by the PHP compatibility checker:

| Framework / Version | `require.php` |
|---|---|
| Laravel 11 | `^8.2` |
| Laravel 10 | `^8.1` |
| Symfony 7 | `>=8.2` |
| Symfony 6 | `>=8.1` |
| CakePHP 5 | `>=8.1` |
| CakePHP 4 | `>=7.4` |
| Drupal 10 | `>=8.1` |
| Drupal 9 | `>=7.3` |
| Magento 2.4.7 | `~8.2.0\|\|~8.3.0` |
| WordPress (no composer.json) | auto-detect from readme |
| Bedrock | `>=8.1` |
| OctoberCMS 3 | `>=8.0` |
| Craft CMS 5 | `>=8.2` |
| Statamic 4 | `>=8.1` |
| Joomla 4 | `>=7.2.5` |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
