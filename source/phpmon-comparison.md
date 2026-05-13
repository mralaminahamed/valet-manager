# PHPMon vs Valet Manager
## A Comprehensive Feature Comparison

> **PHPMon** — Lightweight macOS menu bar app for PHP version management with Laravel Valet integration.
> **Valet Manager** — Full-window Linux desktop application for complete Valet environment management, developer tooling, and project scaffolding.

---

## At a Glance

| | PHPMon | Valet Manager |
|---|---|---|
| **Platform** | macOS only | Linux (Ubuntu, Debian, Fedora, Arch) |
| **Language** | Swift (native macOS) | Rust (`egui` + `eframe`) |
| **UI paradigm** | Menu bar app (status bar) | Full-window desktop app + system tray |
| **Valet support** | `laravel/valet` (Homebrew) | `cpriego/valet-linux`, `laravel/valet` on Linux, `genesisweb/valet-linux-plus` |
| **License** | MIT, open source, free | MIT, open source, free |
| **First release** | 2020 | 2025 |
| **GitHub stars** | ~3,200 | — |
| **Total downloads** | ~141,000 | — |
| **Binary size** | ~15 MB | ~12 MB (single binary, zero runtime deps) |
| **RAM footprint** | ~40 MB | ~18 MB |
| **Project scope** | PHP version management + Valet companion | Full-stack Valet environment GUI + project creation platform |

---

## Platform & Installation

| | PHPMon | Valet Manager |
|---|---|---|
| macOS 13.5+ | ✅ | ❌ |
| Linux (Ubuntu 20.04+) | ❌ | ✅ |
| Linux (Debian 11+) | ❌ | ✅ |
| Linux (Fedora 36+) | ❌ | ✅ |
| Linux (Arch / AUR) | ❌ | ✅ |
| Apple Silicon (ARM) | ✅ native | ❌ |
| Linux aarch64 | ❌ | ✅ |
| Homebrew install | ✅ | ❌ |
| `.deb` package | ❌ | ✅ |
| `.rpm` package | ❌ | ✅ |
| AppImage | ❌ | ✅ |
| Built-in updater | ✅ | ✅ |

---

## PHP Management

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Detect installed PHP versions | ✅ | ✅ | |
| Global PHP version switcher | ✅ | ✅ | |
| Per-site PHP isolation | ✅ | ✅ | Uses `valet isolate` or `.valetrc` |
| Install PHP versions via GUI | ✅ (Homebrew) | ✅ (apt / dnf / pacman) | |
| Remove PHP versions via GUI | ✅ | ✅ | |
| PHP-FPM start / stop / restart | ✅ | ✅ | |
| PHP extension manager — list | ✅ | ✅ | |
| PHP extension — enable / disable | ✅ | ✅ | One-click toggle |
| PHP extension — install new | ✅ | ✅ | From package manager |
| PHP INI — locate config files | ✅ | ✅ | |
| PHP INI — in-app editor | ⚠️ Basic | ✅ Section-based + validation | Section tree, type checking |
| phpinfo() viewer | ✅ | ✅ | Searchable section tree |
| PHP compatibility checker | ✅ | ✅ | Reads `composer.json` `require.php` |
| PHP version auto-fix | ✅ First Aid | ⚠️ Planned | |
| PHP version alias detection | ✅ | ✅ | |

**Legend:** ✅ Full  ⚠️ Partial  ❌ Missing

---

## Site & Domain Management

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Domain list overview | ✅ | ✅ | |
| Framework auto-detection | ✅ | ✅ | 22 frameworks |
| Link / unlink sites | ✅ | ✅ | |
| Park / forget directories | ⚠️ Partial | ✅ | Full parks panel |
| TLS secure / unsecure | ✅ | ✅ | |
| Per-site PHP version (inline) | ✅ | ✅ | Dropdown per row |
| Wildcard subdomain support | ✅ | ✅ | |
| Favorite / bookmark domains | ✅ | ✅ | Sorts to top |
| Default site configuration | ✅ | ✅ | `config.json` key |
| Site-specific env vars | ❌ | ✅ | `.valet-env.php` table editor |
| `.env` file editor | ❌ | ✅ | Grouped, masked secrets, `.env.example` diff |
| Directory listing toggle | ✅ | ✅ | |
| Open in browser | ✅ | ✅ | |
| Open in editor / IDE | ✅ | ✅ | Configurable, scan_apps support |
| Open in file manager | ❌ | ✅ | |
| Copy domain URL | ✅ | ✅ | |
| Right-click context menu | ✅ | ✅ | |
| inotify auto-refresh (Linux) | N/A | ✅ | Real-time site list updates |

---

## Nginx, Proxy & dnsmasq

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Nginx site config viewer | ❌ | ✅ | Syntax-highlighted read view |
| Nginx site config editor | ❌ | ✅ | In-app edit + save + reload |
| Nginx reload / restart | ✅ (via services) | ✅ | |
| Proxy add / remove | ❌ | ✅ | Full proxy manager panel |
| Proxy with TLS (`--secure`) | ❌ | ✅ | |
| Proxy test (HTTP check) | ❌ | ✅ | Live test button per proxy |
| Proxy list | ❌ | ✅ | |
| dnsmasq TLD changer | ❌ | ✅ | `valet domain` |
| dnsmasq config viewer | ❌ | ✅ | |
| dnsmasq DNS resolution tester | ❌ | ✅ | Runs `dig`, shows result inline |
| Nginx port override | ❌ | ✅ | `valet port` (valet-linux) |

---

## Services & Diagnostics

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Service status monitoring | ✅ | ✅ | |
| Start / stop / restart all | ✅ | ✅ | |
| Start / stop individual service | ✅ | ✅ | |
| Diagnostics (`valet diagnose`) | ✅ | ✅ | |
| First Aid — auto-fix | ✅ | ⚠️ Planned v2 | |
| Log viewer | ⚠️ Partial | ✅ | Multi-source: nginx, fpm, valet |
| Command history & audit log | ✅ | ✅ | Per-command timing, exit codes |
| Startup integrity checks | ✅ | ✅ | |
| Mail catcher service | ❌ | ✅ | Mailpit / MailHog control |
| Mail catcher unread count | ❌ | ✅ | Shown in tray + dashboard |

---

## SSL Certificates

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Secure site with TLS | ✅ | ✅ | `valet secure` |
| Unsecure site | ✅ | ✅ | |
| SSL cert details viewer | ❌ | ✅ | Issuer, expiry, SANs |
| Expiry warning notifications | ❌ | ✅ | 14-day threshold, configurable |
| Dashboard expiry alert | ❌ | ✅ | Critical / warning banners |
| One-click cert regenerate | ❌ | ✅ | Re-runs `valet secure` |
| Export PEM | ❌ | ✅ | |
| Valet CA system install | ❌ | ✅ | Adds CA to system trust store |

---

## Sharing

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Site sharing | ✅ | ✅ | |
| ngrok integration | ✅ | ✅ | `valet share` |
| Expose integration | ❌ | ✅ | |
| Cloudflared integration | ❌ | ✅ | |
| Share tool selector | ✅ | ✅ | `valet share-tool` |
| ngrok auth token config | ✅ | ✅ | |
| Public URL display | ✅ | ✅ | Copied to clipboard |
| Stop sharing | ✅ | ✅ | |

---

## Database Management

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Database manager GUI | ❌ | ✅ | MySQL, PostgreSQL, SQLite |
| Create / drop database | ❌ | ✅ | |
| List tables + row counts | ❌ | ✅ | |
| Import / export SQL | ❌ | ✅ | `mysqldump`, `pg_dump` |
| Laravel migrations GUI | ❌ | ✅ | `artisan migrate`, fresh, rollback |
| Laravel seeders GUI | ❌ | ✅ | `artisan db:seed` |
| Streamed migration output | ❌ | ✅ | Real-time output widget |
| DBngin integration (macOS) | ✅ Docs mention | N/A | macOS-only tool |

---

## Developer Tooling

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Artisan command runner | ❌ | ✅ | Discover + autocomplete + history |
| Artisan quick commands | ❌ | ✅ | Configurable per-site one-click buttons |
| Queue worker manager | ❌ | ✅ | systemd user services per site |
| Queue job stats | ❌ | ✅ | Redis LLEN + failed_jobs table |
| Xdebug quick toggle | ❌ | ✅ | One-click per PHP version |
| Xdebug mode preset | ❌ | ✅ | debug / profile / coverage / trace |
| Xdebug IDE key preset | ❌ | ✅ | VSCODE / PHPSTORM / custom |
| Command palette (Ctrl+K) | ❌ | ✅ | Fuzzy search across all actions |
| phpinfo() viewer | ✅ | ✅ | Searchable table |
| PHP compatibility checker | ✅ | ✅ | composer.json semver comparison |

---

## App Creator (Project Scaffolding)

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| New project wizard | ❌ | ✅ | 5-step GUI wizard |
| **Laravel ecosystem** | | | |
| Laravel (blank) | ❌ | ✅ | `laravel new` |
| Laravel + Breeze | ❌ | ✅ | All stacks: React, Vue, Livewire, Blade, API |
| Laravel + Jetstream | ❌ | ✅ | Livewire + Inertia, optional teams |
| Laravel + Filament | ❌ | ✅ | Admin panel scaffolding |
| Laravel API (headless) | ❌ | ✅ | |
| Lumen | ❌ | ✅ | |
| Statamic | ❌ | ✅ | Flat-file + Eloquent |
| **WordPress ecosystem** | | | |
| WordPress (standard) | ❌ | ✅ | `wp valet new` — all 14 options |
| WordPress (SQLite / portable) | ❌ | ✅ | |
| Bedrock | ❌ | ✅ | Roots.io `wp valet new --project=bedrock` |
| WooCommerce site | ❌ | ✅ | WordPress + `wp plugin install woocommerce` |
| WordPress Multisite | ❌ | ✅ | `wp core multisite-install` |
| Destroy WordPress site | ❌ | ✅ | `wp valet destroy` — drops DB + files |
| **Other PHP frameworks** | | | |
| Symfony (full / micro) | ❌ | ✅ | |
| CakePHP 4 | ❌ | ✅ | |
| Slim 4 | ❌ | ✅ | |
| Craft CMS 5 | ❌ | ✅ | |
| Kirby 4 | ❌ | ✅ | |
| OctoberCMS | ❌ | ✅ | |
| Drupal | ❌ | ✅ | |
| Joomla | ❌ | ✅ | |
| Magento 2 | ❌ | ✅ | |
| Static HTML | ❌ | ✅ | |
| **Node.js / Frontend** | | | |
| Next.js + auto-proxy | ❌ | ✅ | Creates Valet proxy to :3000 |
| Nuxt 4 + auto-proxy | ❌ | ✅ | |
| React (Vite) + auto-proxy | ❌ | ✅ | |
| Vue 3 (Vite) + auto-proxy | ❌ | ✅ | |
| SvelteKit + auto-proxy | ❌ | ✅ | |
| Astro + auto-proxy | ❌ | ✅ | |
| **Post-install automation** | | | |
| Auto valet link | ❌ | ✅ | If not in parked directory |
| Auto valet secure | ❌ | ✅ | Optional TLS on create |
| Auto PHP isolation | ❌ | ✅ | Per-framework recommended version |
| Streaming CLI output | ❌ | ✅ | Real-time stdout in terminal widget |
| CLI tool prerequisite check | ❌ | ✅ | Install prompts for missing tools |

---

## Custom Drivers

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| Custom driver list | ❌ | ✅ | |
| Custom driver editor | ❌ | ✅ | In-app code editor |
| Local driver support | ❌ | ✅ | `LocalValetDriver.php` at site root |
| Driver serving detection | ❌ | ✅ | Shows which sites use each driver |

---

## System Integration & UX

| Feature | PHPMon | Valet Manager | Notes |
|---|---|---|---|
| System tray / menu bar | ✅ Menu bar | ✅ System tray | |
| Tray — quick PHP switcher | ✅ | ✅ | |
| Tray — service status | ✅ | ✅ | |
| Tray — favorites section | ✅ | ✅ | |
| Tray — mail unread count | ❌ | ✅ | |
| Desktop notifications | ✅ | ✅ | Opt-in |
| Notification — PHP switch | ✅ | ✅ | |
| Notification — service failure | ✅ | ✅ | |
| Notification — SSL expiry | ❌ | ✅ | |
| Notification — update available | ✅ | ✅ | Non-intrusive banner |
| Notification — queue failure | ❌ | ✅ | |
| Dark / Light / System theme | ✅ | ✅ | |
| Multiple editor support | ✅ | ✅ | VSCode, Cursor, Zed, PhpStorm, Neovim |
| Editor — scan_apps config | ✅ | ✅ | Custom editors via config.json / config.toml |
| Command palette (Ctrl+K) | ❌ | ✅ | Spotlight-style fuzzy search |
| Third-party protocol | ✅ `phpmon://` | ✅ `valet-manager://` | For Alfred, Raycast, shell aliases |
| Raycast extension | ✅ Official | ✅ Planned | |
| Alfred workflow | ✅ Official | ⚠️ Planned | |
| Built-in app updater | ✅ | ✅ | |
| First-run onboarding | ✅ | ✅ | Guided setup wizard |
| Localisation / i18n | ✅ Multiple languages | ⚠️ English only (v1) | |
| Crash reporter | ✅ | ⚠️ Planned | |
| Verbose logging mode | ✅ | ✅ | `~/.config/valet-manager/last_session.log` |

---

## Multi-Valet-Fork Support

| Valet Fork | PHPMon | Valet Manager |
|---|---|---|
| `laravel/valet` (Homebrew, macOS) | ✅ | N/A (macOS only) |
| `laravel/valet` (on Linux) | ❌ | ✅ |
| `cpriego/valet-linux` | ❌ | ✅ |
| `genesisweb/valet-linux-plus` | ❌ | ✅ |
| Config path `~/.valet/` | N/A | ✅ (`cpriego`) |
| Config path `~/.config/valet/` | N/A | ✅ (`laravel` + `genesisweb`) |
| `valet domain` (TLD changer) | ❌ | ✅ |
| `valet port` (port override) | ❌ | ✅ |
| `valet isolate` (per-site PHP) | ✅ | ✅ |
| `.valetrc` file | ✅ | ✅ |
| `.valetphprc` file (legacy) | ✅ | ✅ |

---

## Valet Command Coverage

| Command | PHPMon | Valet Manager |
|---|---|---|
| `valet park` | ⚠️ | ✅ |
| `valet forget` | ⚠️ | ✅ |
| `valet paths` | ⚠️ | ✅ |
| `valet link [name]` | ✅ | ✅ |
| `valet unlink` | ✅ | ✅ |
| `valet links` | ✅ | ✅ |
| `valet secure` | ✅ | ✅ |
| `valet unsecure` | ✅ | ✅ |
| `valet isolate php@X.X` | ✅ | ✅ |
| `valet unisolate` | ✅ | ✅ |
| `valet isolated` | ✅ | ✅ |
| `valet use php@X.X` | ✅ | ✅ |
| `valet proxy domain url` | ❌ | ✅ |
| `valet unproxy` | ❌ | ✅ |
| `valet proxies` | ❌ | ✅ |
| `valet share` | ✅ | ✅ |
| `valet share-tool` | ✅ | ✅ |
| `valet set-ngrok-token` | ✅ | ✅ |
| `valet start` | ✅ | ✅ |
| `valet stop` | ✅ | ✅ |
| `valet restart` | ✅ | ✅ |
| `valet status` | ✅ | ✅ |
| `valet log` | ⚠️ | ✅ |
| `valet diagnose` | ✅ | ✅ |
| `valet trust` | ✅ | ✅ |
| `valet domain [tld]` | ❌ | ✅ (valet-linux) |
| `valet port [n]` | ❌ | ✅ (valet-linux) |
| `valet directory-listing` | ✅ | ✅ |
| `valet forget` | ✅ | ✅ |
| `valet uninstall` | ✅ | ✅ |
| `valet which-php` | ✅ | ✅ |
| **Coverage** | ~73% | **100%** |

---

## Supported Frameworks (Valet Drivers)

Both apps support all 22 native Valet drivers. Valet Manager additionally
auto-detects the framework in the Sites panel and displays it as a badge.

| Framework | PHPMon | Valet Manager |
|---|---|---|
| Laravel | ✅ | ✅ |
| WordPress | ✅ | ✅ |
| Bedrock | ✅ | ✅ |
| Symfony | ✅ | ✅ |
| CakePHP 3 | ✅ | ✅ |
| Craft CMS | ✅ | ✅ |
| Statamic | ✅ | ✅ |
| Jigsaw | ✅ | ✅ |
| Drupal | ✅ | ✅ |
| Joomla | ✅ | ✅ |
| Magento | ✅ | ✅ |
| Slim | ✅ | ✅ |
| Kirby | ✅ | ✅ |
| OctoberCMS | ✅ | ✅ |
| ConcreteCMS | ✅ | ✅ |
| Contao | ✅ | ✅ |
| ExpressionEngine | ✅ | ✅ |
| Katana | ✅ | ✅ |
| Sculpin | ✅ | ✅ |
| Zend Framework | ✅ | ✅ |
| Static HTML | ✅ | ✅ |
| Custom driver | ✅ | ✅ (+ GUI editor) |

---

## Feature Count Summary

| Category | PHPMon | Valet Manager | Δ |
|---|---|---|---|
| PHP management | 10 / 11 | 11 / 11 | +1 |
| Site & domain management | 11 / 18 | 18 / 18 | +7 |
| Nginx, proxy & dnsmasq | 2 / 11 | 11 / 11 | +9 |
| Services & diagnostics | 7 / 10 | 10 / 10 | +3 |
| SSL certificates | 2 / 8 | 8 / 8 | +6 |
| Sharing | 5 / 8 | 8 / 8 | +3 |
| Database management | 0 / 8 | 8 / 8 | +8 |
| Developer tooling | 2 / 10 | 10 / 10 | +8 |
| App Creator | 0 / 28 | 28 / 28 | +28 |
| Custom drivers | 0 / 4 | 4 / 4 | +4 |
| System integration & UX | 16 / 23 | 21 / 23 | +5 |
| Valet command coverage | 22 / 30 | 30 / 30 | +8 |
| **Total** | **77 / 169** | **167 / 169** | **+90** |

---

## Architecture Comparison

| Aspect | PHPMon | Valet Manager |
|---|---|---|
| **Language** | Swift | Rust |
| **UI framework** | SwiftUI + AppKit | egui (immediate-mode, wgpu) |
| **UI paradigm** | Menu bar / popover windows | Full-window + tray |
| **Async** | Swift Concurrency (async/await) | tokio |
| **Background work** | Actor model | tokio tasks + mpsc channels |
| **Subprocess** | Foundation `Process` | `tokio::process::Command` |
| **Configuration** | `~/.config/phpmon/config.json` | `~/.config/valet-manager/config.toml` |
| **History storage** | In-memory window | SQLite (`rusqlite`) |
| **File watching** | `FSNotifier` (kqueue/FSEvents) | `notify` (inotify) |
| **Privilege escalation** | macOS admin dialogs | Polkit + helper binary |
| **Package manager** | Homebrew | apt / dnf / pacman |
| **Binary distribution** | `.app` bundle (Homebrew cask) | `.deb` / `.rpm` / AppImage |
| **Build system** | Xcode | Cargo |

---

## Where PHPMon Leads

PHPMon has years of production use, a mature codebase (1,873 commits), and a dedicated
author who ships monthly releases. Specific areas where it currently has an edge:

- **First Aid / auto-fix diagnostics** — Detects and automatically corrects broken
  PHP symlinks, bad Homebrew state, and PHP-FPM socket issues. Valet Manager has
  `valet diagnose` output but not an automated fix layer yet (planned post-v1).
- **Localisation** — Available in English, French, German, Dutch, Spanish, Italian,
  Chinese, Japanese, and more. Valet Manager ships English-only in v1.
- **Ecosystem maturity** — Official Raycast extension, Alfred workflow, and 3+ years
  of community-reported edge cases handled.
- **macOS polish** — Native SwiftUI animations, macOS accessibility APIs, and Dock
  icon attention requests are unavailable on Linux equivalents.

---

## Where Valet Manager Leads

Valet Manager is purpose-built for Linux and designed with a broader scope from the start.

- **Linux-native** — The only full-featured GUI for Laravel Valet on Linux.
  PHPMon does not run on Linux at all.
- **Full-window UI** — A dedicated window with 26 panels versus a menu bar popover.
  Complex workflows (INI editing, database management, app creation) benefit enormously.
- **App Creator wizard** — 30+ project types across Laravel, WordPress, Symfony, CakePHP,
  Craft, Drupal, Next.js, Nuxt, Astro, and more. PHPMon has no project scaffolding.
- **Database manager** — Create/drop databases, run migrations, seed, view table
  structure. PHPMon has no database tooling.
- **Artisan runner** — Discover, autocomplete, run, and history-track artisan commands
  per site. PHPMon has no artisan integration.
- **Queue worker manager** — Start/stop Laravel queue workers as systemd user services
  with job stats. PHPMon has no queue tooling.
- **Xdebug quick toggle** — One-click enable/disable with mode presets and IDE key
  selection per PHP version. PHPMon has no Xdebug tooling.
- **Mail catcher** — Mailpit / MailHog install, control, unread count, and auto-SMTP
  configuration per site. PHPMon has no mail tooling.
- **SSL cert dashboard** — Per-site cert expiry monitoring, 14-day warning notifications,
  one-click regenerate. PHPMon shows TLS status but no cert details.
- **`.env` editor** — Grouped, validated, secret-masked `.env` editor with
  `.env.example` diff. PHPMon has no `.env` tooling.
- **Command palette** — Ctrl+K spotlight-style access to every action in the app.
  PHPMon uses native macOS menu bar navigation only.
- **Nginx config editor** — In-app syntax-highlighted Nginx site config viewer and
  editor with diff and reload. PHPMon has no Nginx config tooling.
- **Proxy manager** — Full GUI for `valet proxy` / `valet unproxy` / `valet proxies`
  with live HTTP test. PHPMon has no proxy tooling.
- **Multi-fork support** — Handles `cpriego/valet-linux`, `laravel/valet` on Linux,
  and `genesisweb/valet-linux-plus` with automatic detection and path resolution.
- **100% Valet command coverage** — Every documented Valet command has a GUI
  equivalent. PHPMon covers approximately 73%.

---

## Target Audience

### PHPMon is the right choice if you:
- Work on **macOS** and use Homebrew + `laravel/valet`
- Want a **lightweight** menu bar accessory with minimal footprint
- Primarily need **PHP version switching** and basic Valet management
- Value **macOS-native aesthetics** and SwiftUI integration
- Are already in the **established PHPMon community** (Raycast, Alfred, etc.)

### Valet Manager is the right choice if you:
- Develop on **Linux** and use any valet-linux variant
- Want a **comprehensive workstation GUI** rather than a menu bar app
- Create new projects frequently and want **one-click scaffolding**
- Work with **WordPress + WP-CLI** and want `wp valet new` exposed as a form
- Need **database, artisan, queue, Xdebug, and mail** tools without switching
  to a terminal
- Prefer **keyboard-first workflows** via a command palette
- Manage **multiple framework types** (Laravel, WordPress, Node.js) on the same machine

---

## Compatibility Notes

| Scenario | PHPMon | Valet Manager |
|---|---|---|
| Pure PHP management (no Valet) | ✅ Works without Valet | ⚠️ Requires Valet for most features |
| WordPress + WP-CLI development | ⚠️ Site listing only | ✅ Full wp valet integration |
| Node.js projects alongside PHP | ❌ | ✅ Auto-proxy creation |
| Multiple PHP versions simultaneously | ✅ | ✅ |
| Docker alongside Valet | ❌ | ✅ Via proxy manager |
| Remote server management | ❌ | ❌ Both are local-only |

---

## Roadmap Maturity

| Milestone | PHPMon | Valet Manager |
|---|---|---|
| Stable v1.0 | ✅ Released 2020 | 🔄 In development |
| Monthly releases | ✅ | Planned |
| Public issue tracker | ✅ GitHub Issues | ✅ GitHub Issues |
| Community contributions | ✅ 61 forks | Planned |
| Documentation site | ✅ phpmon.app | Planned (mdBook) |
| Test coverage | ✅ XCTest + UI tests | Planned (Rust integration tests) |
| CI / CD pipeline | ✅ GitHub Actions | ✅ GitHub Actions |

---

*Comparison accurate as of May 2026.*
*PHPMon version: 26.03.2 · Valet Manager version: architecture v3.0 (pre-release)*

*Author: Al Amin Ahamed (@mralaminahamed)*
