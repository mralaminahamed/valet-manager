# Valet Manager

> Native Linux desktop application for managing Laravel Valet environments.
> Built with Rust + egui. Inspired by PHPMon for macOS — and goes far beyond it.

[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://rustup.rs)
[![License](https://img.shields.io/badge/license-MIT-teal)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-blue)](https://github.com/mralaminahamed/valet-manager)

---

## What it is

Valet Manager is a full-window Linux desktop GUI that replaces the need to
run dozens of `valet`, `php`, `wp`, `artisan`, and `systemctl` commands by hand.
It manages every aspect of a local PHP development environment in one place.

```
PHP versions · Extensions · INI config · phpinfo()
Valet sites · TLS certs · Nginx configs · dnsmasq
WordPress multisite · Laravel Octane · Artisan runner
Database manager · .env editor · Queue workers
phpMyAdmin per-site · FrankenPHP · Caddy · Apache
App Creator: 30+ frameworks with one-click scaffolding
```

## Why not PHPMon?

PHPMon is macOS-only. Valet Manager is Linux-native.
Beyond platform, Valet Manager adds an App Creator wizard, database manager,
Artisan runner, per-site phpMyAdmin, multi-server support, and a command palette.
See [COMPARISON.md](COMPARISON.md) for the full feature-by-feature breakdown.

---

## Supported Valet forks

| Fork | Config path | Status |
|---|---|---|
| `cpriego/valet-linux` | `~/.valet/` | ✅ Full support |
| `laravel/valet` (on Linux) | `~/.config/valet/` | ✅ Full support |
| `genesisweb/valet-linux-plus` | `~/.config/valet/` | ✅ Full support |

---

## Feature highlights

### PHP Management
- Switch global PHP version with one click
- Per-site PHP isolation via `valet isolate` or `.valetrc`
- Install / remove PHP versions through the GUI (apt / dnf / pacman)
- Enable / disable extensions per PHP version
- Section-based INI editor with live validation
- Per-site INI overrides via `.user.ini` (no root needed)
- phpinfo() viewer with search
- PHP compatibility checker (reads `composer.json require.php`)
- Xdebug quick toggle per PHP version with mode and IDE key presets

### Site Management
- Full table of all parked, linked, and proxy sites
- Framework auto-detection (22 frameworks)
- Favorite sites (always sort to top)
- Inline PHP version dropdown per site
- TLS secure / unsecure with expiry warnings
- Site context menu: browser, editor, file manager, Nginx config, .env, destroy

### Developer Tools
- Artisan runner with command autocomplete and history
- Database manager: create/drop/migrate/seed (MySQL, PostgreSQL, SQLite)
- `.env` editor: grouped, secret-masked, validated against `.env.example`
- Queue worker manager as systemd user services
- Mail catcher (Mailpit / MailHog) with SMTP auto-config
- phpMyAdmin per-site (site-DB-only or all-DB mode)

### App Creator
30+ project types created through a 5-step wizard with streaming CLI output:
Laravel (blank / Breeze / Jetstream / Filament / API), WordPress (standard /
SQLite / Bedrock / WooCommerce / Multisite), Symfony, CakePHP, Craft, Slim,
Drupal, Joomla, Statamic, Kirby, OctoberCMS, Next.js, Nuxt, React/Vite,
Vue, SvelteKit, Astro, Static HTML.

### HTTP Server Config
Per-site configuration for Nginx, FrankenPHP, Caddy, and Apache.
Custom directive injection, basic auth, redirect rules, timeout overrides.

### Per-Site Config
Every site stores its configuration in `.valet-manager.toml` (portable, committable
to git) or `~/.config/valet-manager/sites/{name}.toml` (centralized). Config covers
PHP, framework version, WordPress multisite, Laravel Octane, HTTP server, phpMyAdmin,
and development environment settings.

---

## Technology stack

| Layer | Choice |
|---|---|
| Language | Rust 2021 edition (stable) |
| GUI | egui 0.31 + eframe (wgpu backend) |
| Async | tokio (full features) |
| Database (history) | SQLite via rusqlite |
| File watching | notify (inotify) |
| Templates | Handlebars |
| Packaging | cargo-deb, cargo-rpm, AppImage |

---

## Project status

Currently in architecture and design phase. Implementation follows the
[12-phase roadmap](ROADMAP.md) over approximately 40 weeks.

---

## Documentation index
### Claude Design command

Paste this into Claude to implement the full UI:

```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```


| File | Contents |
|---|---|
| [ROADMAP.md](ROADMAP.md) | 12-phase development timeline |
| [FEATURES.md](FEATURES.md) | Complete feature inventory (45 features) |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Full technical architecture |
| [COMPARISON.md](COMPARISON.md) | PHPMon vs Valet Manager |
| [design/DESIGN-SYSTEM.md](design/DESIGN-SYSTEM.md) | Colour tokens, typography, egui components |
| [specs/SITE-CONFIG.md](specs/SITE-CONFIG.md) | Per-site config data models and I/O |
| [specs/PHPMYADMIN.md](specs/PHPMYADMIN.md) | phpMyAdmin per-site service spec |
| [specs/HTTP-SERVERS.md](specs/HTTP-SERVERS.md) | Nginx / FrankenPHP / Caddy / Apache |
| [prompts/OVERVIEW.md](prompts/OVERVIEW.md) | How to use the Claude Code prompts |

---

## Author

Al Amin Ahamed · [@mralaminahamed](https://github.com/mralaminahamed) · Codexpert Inc., Dhaka
