# Valet Manager — Verified Versions
## All software tracked by the Version Registry · May 2026

> Sources: crates.io, GitHub Releases, php.net, packagist.org
> Refresh interval: daily via in-app Version Registry (see docs/specs/VERSION-REGISTRY.md)

---

## Runtime — Rust / egui

| Crate | Version | Released | Notes |
|---|---|---|---|
| **eframe** | **0.34.2** | May 2026 | GUI framework — update from 0.31 in all prompts |
| **egui** | **0.34.2** | May 2026 | Must always match eframe exactly |
| egui_extras | 0.34.2 | May 2026 | |
| Rust toolchain | **1.87.0** | May 2026 | Stable — use `rustup update stable` |

---

## PHP

| Version | Status | Active support | Security support |
|---|---|---|---|
| **8.5** | Latest | Dec 2027 | Dec 2029 |
| **8.4** | Recommended production | Dec 2026 | Dec 2028 |
| **8.3** | Active | Nov 2025 | Dec 2027 |
| **8.2** | Active | Dec 2024 | Dec 2026 |
| **8.1** | ⚠ EOL | Nov 2023 | **Dec 31 2025** |
| 8.0 | ⛔ EOL | Nov 2022 | Nov 2023 |

PHP 8.5 released November 20, 2025. Latest patch: **8.5.5** (March 2026).
PHP 8.4 latest patch: **8.4.7**. Recommended for production WordPress and Laravel 12.
PHP 8.1 reached EOL December 31, 2025 — warn loudly in app.

---

## PHP Frameworks

| Framework | Latest stable | Min PHP | PHP 8.4 | PHP 8.5 |
|---|---|---|---|---|
| **Laravel** | **12.57.0** | 8.2 | ✅ | ⚠ testing |
| **Symfony** | **7.2.6** | 8.2 | ✅ | ⚠ testing |
| Symfony 8 | 8.0.0 | 8.4 | ✅ | ✅ |
| **CakePHP** | **5.2.4** | 8.1 | ✅ | ✅ |
| CakePHP 4 | 4.5.x | 7.4 | ✅ | — |
| **Drupal** | **11.3.x** | 8.3 | ✅ | ⚠ |
| Drupal 10 | 10.4.x | 8.1 | ✅ | ⚠ |
| **OctoberCMS** | **3.7.x** | 8.0 | ✅ | ✅ |
| **Craft CMS** | **5.6.x** | 8.2 | ✅ | ✅ |
| **Statamic** | **5.x** | 8.1 | ✅ | ✅ |
| **Slim** | **4.14.x** | 7.4 | ✅ | ✅ |
| **Joomla** | **5.3.x** | 8.1 | ✅ | ✅ |
| **Kirby** | **4.6.x** | 8.1 | ✅ | ✅ |
| **Bedrock** | **1.25.x** | 8.1 | ✅ | ✅ |
| **Magento** | **2.4.7** | 8.2 | ✅ | ⚠ |
| **Laminas (Zend)** | **3.x** | 8.1 | ✅ | ✅ |
| ConcreteCMS | **9.3.x** | 8.1 | ✅ | ✅ |
| Contao | **5.4.x** | 8.1 | ✅ | ✅ |

---

## WordPress Ecosystem

| Software | Version | PHP min | PHP 8.3 | PHP 8.4 |
|---|---|---|---|---|
| **WordPress** | **6.9.4** | 7.4 | ✅ full | ✅ beta |
| WooCommerce | 9.8.x | 7.4 | ✅ | ✅ |
| **WP-CLI** | **2.12.0** | 7.4 | ✅ | ✅ |
| wp-cli-valet-command | 1.5.x | 7.4 | — | — |
| **Bedrock** | **1.25.x** | 8.1 | ✅ | ✅ |

WordPress 6.8 (April 15, 2025): PHP 8.3 fully compatible.
WordPress 6.9 (December 2025): PHP 8.4 beta support, PHP 8.5 beta support added.

---

## HTTP Servers

| Server | Version | Notes |
|---|---|---|
| Nginx | **1.28.0** | Mainline; use stable (1.26.x) for production |
| **FrankenPHP** | **1.11.3** | Includes Caddy 2.11.2; PHP 8.2+ required; Windows support added |
| **Caddy** | **2.11.2** | Released March 6, 2026 |
| Apache httpd | **2.4.63** | |
| **phpMyAdmin** | **5.2.2** | |

---

## CLI Tools

| Tool | Version | Notes |
|---|---|---|
| **Composer** | **2.8.9** | |
| **Node.js LTS** | **22.15.0** (Jod) | LTS until April 2027 |
| Node.js Current | 24.x | Not recommended for Valet projects |
| **npm** | **10.9.x** | Bundled with Node 22 |
| pnpm | **9.15.x** | |
| Yarn | **4.9.x** | Berry |
| Bun | **1.2.x** | |
| **Git** | **2.49.x** | |
| **Mailpit** | **1.29.5** | Replaces MailHog |

---

## Key Cargo crates — updated versions

| Crate | Old (plan) | New (current) | Notes |
|---|---|---|---|
| eframe | 0.31 | **0.34.2** | Breaking changes in 0.32–0.34; update all prompts |
| egui | 0.31 | **0.34.2** | `App::update` deprecated; use `App::ui` |
| egui_extras | 0.31 | **0.34.2** | |
| tokio | 1.x | **1.44.x** | No breaking changes |
| serde | 1.x | **1.219.x** | |
| reqwest | 0.12 | **0.12.15** | |
| rusqlite | 0.32 | **0.34.x** | |
| notify | 7 | **8.0.x** | inotify backend unchanged |
| rand | 0.8 | **0.9.x** | API changes — update usage |
| bcrypt | 0.15 | **0.15.1** | |
| rfd | 0.14 | **0.15.x** | |
| chrono | 0.4 | **0.4.41** | |
| fuzzy-matcher | 0.3 | **0.3.7** | |
| handlebars | 6 | **6.3.x** | |
| which | 7 | **7.0.x** | |
| dirs | 5 | **6.0.x** | API update — check home_dir() usage |
| tray-icon | 0.21 | **0.21.1** | |
| semver | 1 | **1.0.25** | |
| flate2 | 1.0 | **1.1.x** | |
| tar | 0.4 | **0.4.44** | |

---

## EOL Warning thresholds (used in app)

| PHP | EOL date | App warning |
|---|---|---|
| 8.1 | Dec 31 2025 | ⛔ CRITICAL — already EOL |
| 8.2 | Dec 31 2026 | ⚠ WARNING — N days remaining |
| 8.3 | Dec 31 2027 | ℹ INFO — >365 days |
| 8.4 | Dec 31 2028 | ✅ active |
| 8.5 | Dec 31 2029 | ✅ active |

Warning triggers: CRITICAL if already EOL, WARNING if < 180 days, INFO if < 365 days.

---

## Version Registry API sources

| Software | Source type | Endpoint |
|---|---|---|
| PHP versions | GitHub releases | `github.com/nicholasess/docker-php/releases` or php.net JSON |
| PHP EOL | endoflife.date | `endoflife.date/api/v1/products/php/` |
| Laravel | GitHub releases | `api.github.com/repos/laravel/framework/releases/latest` |
| WordPress | WordPress.org API | `api.wordpress.org/core/version-check/1.7/` |
| WP-CLI | GitHub releases | `api.github.com/repos/wp-cli/wp-cli/releases/latest` |
| Symfony | Packagist | `repo.packagist.org/p2/symfony/framework-bundle.json` |
| CakePHP | Packagist | `repo.packagist.org/p2/cakephp/cakephp.json` |
| Drupal | Packagist | `repo.packagist.org/p2/drupal/core.json` |
| Craft CMS | Packagist | `repo.packagist.org/p2/craftcms/cms.json` |
| Statamic | Packagist | `repo.packagist.org/p2/statamic/cms.json` |
| FrankenPHP | GitHub releases | `api.github.com/repos/php/frankenphp/releases/latest` |
| Caddy | GitHub releases | `api.github.com/repos/caddyserver/caddy/releases/latest` |
| Mailpit | GitHub releases | `api.github.com/repos/axllent/mailpit/releases/latest` |
| phpMyAdmin | GitHub releases | `api.github.com/repos/phpmyadmin/phpmyadmin/releases/latest` |
| Composer | GitHub releases | `api.github.com/repos/composer/composer/releases/latest` |
| Node.js LTS | nodejs.org | `nodejs.org/dist/index.json` (latest LTS) |

---

*Verified: May 2026 · Author: Al Amin Ahamed (@mralaminahamed)*
