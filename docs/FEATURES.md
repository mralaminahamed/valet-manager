# Valet Manager — Feature Inventory
## 48 features across 14 categories

Status key: 🔵 Planned · ✅ In scope · ⭐ Exceeds PHPMon

---

## PHP Management (11 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 1 | Detect installed PHP versions | ✓ | ✅ |
| 2 | Switch global PHP version | ✓ | ✅ |
| 3 | Per-site PHP isolation (valet isolate / .valetrc) | ✓ | ✅ |
| 4 | Install PHP versions via GUI (apt/dnf/pacman) | ✓ (Homebrew) | ✅ |
| 5 | Remove PHP versions via GUI | ✓ | ✅ |
| 6 | PHP-FPM start / stop / restart | ✓ | ✅ |
| 7 | PHP extension manager: list, enable, disable | ✓ | ✅ |
| 8 | PHP extension install from package manager | ✓ | ✅ |
| 9 | PHP INI editor: section-based + raw + validation | ⚠ basic | ⭐ |
| 10 | Per-site PHP INI overrides via .user.ini | ✗ | ⭐ |
| 11 | phpinfo() in-app viewer (searchable) | ✓ | ✅ |

---

## Site Management (11 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 12 | Domain list with framework auto-detection (22 types) | ✓ | ✅ |
| 13 | Link / unlink sites | ✓ | ✅ |
| 14 | Park / forget directories | ⚠ | ✅ |
| 15 | TLS secure / unsecure | ✓ | ✅ |
| 16 | Inline PHP version dropdown per site | ✓ | ✅ |
| 17 | Favorite / bookmark sites (sort to top) | ✓ | ✅ |
| 18 | Site env vars editor (.valet-env.php) | ✗ | ⭐ |
| 19 | .env file editor (grouped, masked, validated) | ✗ | ⭐ |
| 20 | Open in browser / editor / file manager | ✓ | ✅ |
| 21 | inotify auto-refresh on site list changes | N/A (macOS) | ⭐ |
| 22 | PHP compatibility checker (composer.json) | ✓ | ✅ |

---

## HTTP Server Config (8 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 23 | Nginx site config viewer and editor | ✗ | ⭐ |
| 24 | Per-site Nginx directive injection (sentinel blocks) | ✗ | ⭐ |
| 25 | FrankenPHP: standalone + Octane proxy mode | ✗ | ⭐ |
| 26 | Caddy: per-site Caddyfile generation | ✗ | ⭐ |
| 27 | Apache: VirtualHost config + .htaccess editor | ✗ | ⭐ |
| 28 | Basic auth: per-site htpasswd management | ✗ | ⭐ |
| 29 | Redirect rules per site | ✗ | ⭐ |
| 30 | HTTP server auto-detection at startup | ✗ | ⭐ |

---

## Proxy & dnsmasq (4 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 31 | Proxy manager: add/remove/test valet proxy entries | ✗ | ⭐ |
| 32 | dnsmasq TLD changer + DNS resolution tester | ✗ | ⭐ |
| 33 | Nginx port override (valet port) | ✗ | ⭐ |
| 34 | SSL certificate dashboard: expiry monitoring + regenerate | ✗ | ⭐ |

---

## Services & Diagnostics (5 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 35 | Service start / stop / restart (nginx, fpm, dnsmasq) | ✓ | ✅ |
| 36 | Live service status polling every 5 seconds | ✓ | ✅ |
| 37 | Log viewer: nginx, php-fpm, valet logs | ⚠ | ⭐ |
| 38 | Diagnostics panel (valet diagnose streaming) | ✓ | ✅ |
| 39 | Command history & audit log (SQLite, every subprocess) | ✓ | ✅ |

---

## Developer Tooling (7 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 40 | Artisan runner: discover, autocomplete, stream, history | ✗ | ⭐ |
| 41 | Database manager: MySQL / PostgreSQL / SQLite | ✗ | ⭐ |
| 42 | Laravel migrations GUI (migrate / fresh / seed / rollback) | ✗ | ⭐ |
| 43 | Queue worker manager (systemd user services) | ✗ | ⭐ |
| 44 | Xdebug quick toggle: per-version, mode, IDE key | ✗ | ⭐ |
| 45 | Mail catcher: Mailpit / MailHog, SMTP auto-config | ✗ | ⭐ |
| 46 | phpMyAdmin per-site (site-DB-only or all-DB) | ✗ | ⭐ |

---

## WordPress-Specific (5 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 47 | WordPress version detection and display | ✓ | ✅ |
| 48 | Multisite enable/disable (subdomain / subdirectory) | ✗ | ⭐ |
| 49 | wp-config.php constants editor with backup | ✗ | ⭐ |
| 50 | Network site management (list / create / delete) | ✗ | ⭐ |
| 51 | Plugin activate / deactivate (network-wide) | ✗ | ⭐ |

---

## Laravel-Specific (5 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 52 | Installed package detection (Horizon, Telescope, Pulse, Reverb) | ✗ | ⭐ |
| 53 | Laravel Octane: start / stop / restart (Swoole / RoadRunner / FrankenPHP) | ✗ | ⭐ |
| 54 | Horizon dashboard shortcut | ✗ | ⭐ |
| 55 | Telescope shortcut | ✗ | ⭐ |
| 56 | Reverb WebSocket server control | ✗ | ⭐ |

---

## App Creator (6 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 57 | 5-step wizard with streaming CLI output | ✗ | ⭐ |
| 58 | WordPress: wp valet new (all 14 options), wp valet destroy | ✗ | ⭐ |
| 59 | Laravel: blank / Breeze (all stacks) / Jetstream / Filament / API | ✗ | ⭐ |
| 60 | 12 PHP frameworks via composer create-project | ✗ | ⭐ |
| 61 | 6 Node.js frameworks with auto-proxy + systemd dev service | ✗ | ⭐ |
| 62 | Post-install: auto link, secure, isolate PHP, open browser | ✗ | ⭐ |

---

## Per-Site Config (4 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 63 | .valet-manager.toml per site (portable, git-committable) | ✗ | ⭐ |
| 64 | Centralized site config with site-root override (field-level merge) | ✗ | ⭐ |
| 65 | WordPress config (multisite, WP_DEBUG, wp-config.php extras) | ✗ | ⭐ |
| 66 | Laravel config (Octane server, packages, environment) | ✗ | ⭐ |

---

## Sharing (4 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 67 | Site sharing via ngrok | ✓ | ✅ |
| 68 | Site sharing via Expose | ✗ | ⭐ |
| 69 | Site sharing via cloudflared | ✗ | ⭐ |
| 70 | Share-tool selector + ngrok auth token config | ✓ | ✅ |

---

## System Integration & UX (8 features)

| # | Feature | PHPMon | Status |
|---|---|---|---|
| 71 | System tray: PHP version, quick switcher, services, favorites | ✓ | ✅ |
| 72 | Desktop notifications (opt-in, 6 event types) | ✓ | ✅ |
| 73 | Command palette (Ctrl+K): fuzzy search all actions | ✗ | ⭐ |
| 74 | valet-manager:// deep-link protocol (Raycast / Alfred) | ✓ (phpmon://) | ✅ |
| 75 | Built-in app updater (GitHub releases API) | ✓ | ✅ |
| 76 | First-run onboarding wizard | ✓ | ✅ |
| 77 | Custom Valet driver manager (list / edit / create) | ✗ | ⭐ |
| 78 | Multi-valet-fork support (3 Linux forks, auto-detected) | ✗ (macOS only) | ⭐ |

---

## Summary

| Category | Total | Shared with PHPMon | Exclusive to Valet Manager |
|---|---|---|---|
| PHP Management | 11 | 9 | 2 |
| Site Management | 11 | 7 | 4 |
| HTTP Server Config | 8 | 0 | 8 |
| Proxy & dnsmasq | 4 | 0 | 4 |
| Services & Diagnostics | 5 | 4 | 1 |
| Developer Tooling | 7 | 0 | 7 |
| WordPress-specific | 5 | 1 | 4 |
| Laravel-specific | 5 | 0 | 5 |
| App Creator | 6 | 0 | 6 |
| Per-Site Config | 4 | 0 | 4 |
| Sharing | 4 | 2 | 2 |
| System Integration | 8 | 5 | 3 |
| **Total** | **78** | **28** | **50** |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
