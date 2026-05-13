# Valet Manager — Roadmap
## 12 Phases · ~40 Weeks

---

## Timeline overview

```
Weeks  1–3   Phase 1   Foundation & design system
Weeks  4–6   Phase 2   PHP management + PHP panels UI
Weeks  7–9   Phase 3   Sites & Nginx + site/park/Nginx UI
Weeks 10–12  Phase 4   App Creator: WordPress & Laravel
Weeks 13–15  Phase 5   App Creator: all 30+ frameworks
Weeks 16–18  Phase 6   Advanced features panels
Weeks 19–21  Phase 7   Polish, tray, notifications, packaging
Weeks 22–25  Phase 8   PHPMon gap-closing
Weeks 26–30  Phase 9   DX Tier 1: palette, .env, Artisan, Database
Weeks 31–34  Phase 10  DX Tier 2: SSL, Xdebug, Mail, Queue
Weeks 35–38  Phase 11  Per-site config & HTTP server abstraction
Weeks 38–40  Phase 12  phpMyAdmin per-site service
```

---

## Phase 1 — Foundation + Design System
**Weeks 1–3**

### Architecture
- Cargo workspace: `valet-manager` + `valet-manager-helper`
- Valet variant auto-detection (cpriego / official / genesisweb)
- Linux distro detection + package manager abstraction (apt / dnf / pacman)
- PHP version detection from `/usr/bin/php*` and `update-alternatives`
- Service status polling via `systemctl is-active` every 5 seconds
- App state with `Arc<RwLock<AppState>>` + tokio mpsc command dispatcher
- Config persistence at `~/.config/valet-manager/config.toml`
- `AppCommand` enum + `AppEvent` enum

### Design System
- `src/ui/theme.rs` with all `Colors::*` constants (15 colour tokens)
- `apply_dark()` sets every `egui::Visuals` field
- Helper functions: `status_dot`, `framework_badge`, `accent_button`, `ghost_button`, `card_frame`, `section_label`, `divider`
- Sidebar with logo V mark, navigation sections, service status
- 36px title bar with traffic-light dots
- Dashboard panel: alert banners, stats row, service grid

**Deliverable:** App opens with dark teal theme, sidebar, live service dots.

---

## Phase 2 — PHP Management
**Weeks 4–6**

### Architecture
- Privilege helper binary (`pkexec` + Polkit policy)
- `valet use php@X.X` global switcher + `update-alternatives` fallback
- Per-site isolation via `valet isolate` and `.valetrc`
- PHP-FPM start / stop / restart via privilege helper
- Extension list / enable / disable by renaming `.ini` files
- PHP INI section parser + renderer + value validator

### UI
- PHP Versions panel: version cards, active indicator (3px teal left border), FPM controls
- PHP Extensions panel: table with toggle squares, type badges, detail expand
- PHP INI panel: section tree (left) + key-value table (right), raw edit mode, save/revert

**Deliverable:** PHP version switching works end-to-end.

---

## Phase 3 — Sites & Nginx
**Weeks 7–9**

### Architecture
- Valet config reader (`config.json` both path variants)
- Site scanner: parked + linked + proxy, framework detection (22 frameworks), `.valetrc` parsing
- Nginx config parser: server_name, root, fastcgi_pass, SSL fields
- inotify watcher → auto-refresh on `Sites/` and `Nginx/` changes
- `ToggleFavoriteSite` persists to config, favorites sort to top

### UI
- Sites panel: table with ★, domain, path, PHP dropdown, framework badge, TLS icon, ⋮ menu
- Parks panel: directory list, add via `rfd::FileDialog`, remove with confirm
- Nginx panel: site list (left), syntax-highlighted raw config (right), save & reload

**Deliverable:** All sites visible, PHP dropdown changes isolation, Nginx editor saves.

---

## Phase 4 — App Creator: WordPress & Laravel
**Weeks 10–12**

### Architecture
- CLI tool registry: probe wp, wp-cli-valet-command, laravel, composer, npm, git
- Output streamer: tokio process + piped stdout/stderr → `OutputLine` channel
- WordPress runner: `wp valet new` with all 14 options, `wp valet destroy`
- Laravel runner: blank / Breeze (all stacks) / Jetstream / API
- Post-install: `valet link`, `valet secure`, `valet isolate` in sequence

### UI
- 5-step wizard: SelectType → Configure → PostInstall → Progress → Complete/Error
- Step indicator circles, framework cards, dynamic form driven by `ProjectType.options`
- Terminal output widget: `DEEP_BG` monospace, timestamps, stderr in amber
- Cancel sends SIGTERM to child process

**Deliverable:** WordPress and Laravel sites created in GUI with streaming output.

---

## Phase 5 — App Creator: All Frameworks
**Weeks 13–15**

- Composer runner for 12 PHP frameworks (Symfony, CakePHP, Craft, Slim, Drupal, Joomla, Kirby, OctoberCMS, Statamic, Magento, Bedrock, Static HTML)
- Node.js runner (Next.js, Nuxt 4, React/Vite, Vue/Vite, SvelteKit, Astro) + auto-proxy creation
- Optional systemd user dev-server service per Node project
- Prerequisite installer modal shows exact install command before running
- App Creator UI shows all 30+ project types in correct groups with availability badges

**Deliverable:** Any framework created with one wizard run.

---

## Phase 6 — Advanced Features
**Weeks 16–18**

- Proxies panel: add/remove/test `valet proxy` entries, live HTTP status check
- Site env vars editor: `.valet-env.php` table editor (key/value pairs)
- Custom Drivers panel: list/edit/create Valet PHP driver files
- dnsmasq panel: TLD changer, DNS resolution tester (`dig @127.0.0.1`)
- Sharing panel: ngrok / Expose / cloudflared, token config, public URL display
- Logs panel: multi-source log viewer with color-coded lines
- Diagnostics panel: `valet diagnose` streaming output

**Deliverable:** All valet commands reachable from the GUI.

---

## Phase 7 — Polish & Release
**Weeks 19–21**

- System tray icon: active PHP in tooltip, quick switcher menu, service status, favorites
- Desktop notifications (opt-in) for PHP switch, service failure, creation complete, update
- Toast component: auto-dismiss 4s, bottom-right stack, fade animation
- Dark / Light / System theme switching
- First-run onboarding wizard (5 steps from no-Valet to ready)
- Settings panel: appearance, editor, notifications, app creator defaults, danger zone
- Responsive sidebar: icon-only mode below 1000px, hamburger below 800px
- `.deb` package (`cargo-deb`), `.desktop` file, `com.valetmanager.policy` Polkit
- GitHub Actions release workflow: lint → test → build → `.deb` + `.rpm` + AppImage

**Deliverable:** App ships as an installable `.deb` package.

---

## Phase 8 — PHPMon Gap-Closing
**Weeks 22–25**

- phpinfo() viewer: searchable section tree, copy-value per row
- PHP compatibility checker: reads `composer.json require.php`, batch concurrent check, fix suggestions
- Command history & audit log: SQLite, every subprocess logged with timing + exit code
- Favorite domains: `config.toml` persistence, sort-to-top, tray submenu
- `valet-manager://` deep-link protocol: registered in `.desktop`, Raycast/Alfred compatible
- Built-in updater: GitHub releases API, non-intrusive dashboard banner
- Onboarding wizard polish: guided Valet install, `valet diagnose` step
- Dashboard alert banners: compat issues, SSL expiry, update available

**Deliverable:** Feature parity with PHPMon on every dimension.

---

## Phase 9 — DX Tier 1
**Weeks 26–30**

- Command palette (Ctrl+K): fuzzy search across PHP versions, sites, services, artisan, panels
- `.env` file editor: grouped by prefix, secret masking, `.env.example` diff, atomic write
- Artisan runner: `php artisan list --format=json` discovery, autocomplete, quick commands, history
- Database manager: MySQL/PostgreSQL/SQLite, create/drop/list, migrate/seed/rollback, streaming output
- Dashboard alert banners complete: SSL expiry + compat + update + diagnostics

**Deliverable:** No more switching to terminal for common Laravel/WordPress tasks.

---

## Phase 10 — DX Tier 2
**Weeks 31–34**

- SSL certificate dashboard: per-cert expiry, color-coded status, 14-day warning notifications
- Xdebug quick toggle: per-PHP-version cards, mode presets, IDE key selector
- Mail catcher (Mailpit / MailHog): install, start/stop, unread count, SMTP auto-config per site
- Queue worker manager: systemd user services, Redis LLEN polling, failed job count
- Sites panel ⋮ menu: phpMyAdmin shortcuts appear for enabled sites
- All 26 panels routable from sidebar and command palette

**Deliverable:** Complete local dev environment toolbox, no PHPMon features missing.

---

## Phase 11 — Per-Site Config & HTTP Server Abstraction
**Weeks 35–38**

### Architecture
- `SiteConfig` TOML struct covering PHP, framework, server, WordPress, Laravel, database, development
- Dual config file: `{site_root}/.valet-manager.toml` (portable) + `~/.config/valet-manager/sites/{name}.toml` (centralized)
- Field-level merge: site-root wins over centralized
- `.user.ini` writer for per-site PHP INI overrides (no privilege needed)
- `wp-config.php` constant reader/writer with backup-before-modify
- WordPress multisite enablement flow: backup → constants → `wp core multisite-install` → Nginx rewrites → reload
- `WpNetworkSite` management: list / create / delete via `wp-cli`
- Laravel Octane: start/stop as background process, Nginx upstream switch
- Laravel package detection: Horizon, Telescope, Pulse, Reverb, Octane, Filament
- HTTP server detector: Nginx / FrankenPHP / Caddy / Apache auto-detect
- Per-site Nginx directive injection with sentinel comments (idempotent)
- FrankenPHP: standalone Caddyfile + Octane proxy mode
- Caddy: per-site Caddyfile generation
- Apache: VirtualHost config + `.htaccess` editor
- Basic auth: htpasswd generation, Nginx / Caddy / Apache injection
- Redirect rules per site

### UI wiring (Phase 11b, Weeks 38–40)
- Site config panel tabs: PHP · WordPress · Laravel · Nginx/Server · Database · Development
- All form controls wired to config state and dispatch commands
- Dirty-state tracking: "● Unsaved changes" badge, confirm on navigate-away
- "Save to project" option writes `.valet-manager.toml` to site root
- Multisite enable flow shows streaming `wp core multisite-install` output
- Octane start/stop shows live process status

**Deliverable:** Every per-site setting configurable without touching a config file manually.

---

## Phase 12 — phpMyAdmin Per-Site Service
**Weeks 38–40** *(overlaps with Phase 11b)*

- phpMyAdmin detection at standard install paths
- Install via `apt` (with non-interactive debconf pre-seeding) or manual tar.gz download
- Per-site `config.inc.php` generation in `~/.config/valet-manager/phpmyadmin/sites/{name}/`
- `PMA_CONFIG_DIR` passed via Nginx `fastcgi_param` → per-site config isolation
- **Site-only mode**: `only_db` restriction + `config` auth (auto-login)
- **All-databases mode**: no `only_db` + `cookie` auth (login prompt, security)
- Three access modes: path alias (`/_pma/`), subdomain (`pma.{site}.test`), global only
- Nginx sentinel-block injection (idempotent, re-apply replaces not duplicates)
- Global `phpmyadmin.{tld}` valet site for all-DB access
- Credential resolution: `.env` first, `wp-config.php` second, app config defaults last
- `blowfish_secret` generated once, persisted in `AppConfig`
- Dashboard quick action button when any site has phpMyAdmin enabled
- Sites panel ⋮ menu shortcuts: "Open phpMyAdmin" + "Open phpMyAdmin (all DBs)"

**Deliverable:** Per-site phpMyAdmin accessible at `https://mysite.test/_pma/`.

---

## Milestone summary

| Phase | Weeks | Key deliverable |
|---|---|---|
| 1 | 1–3 | Themed window opens, service dots live |
| 2 | 4–6 | PHP switching end-to-end |
| 3 | 7–9 | All sites visible, Nginx editor |
| 4 | 10–12 | WordPress + Laravel created in GUI |
| 5 | 13–15 | All 30+ frameworks in creator |
| 6 | 16–18 | Every valet command in GUI |
| 7 | 19–21 | Installable .deb package |
| 8 | 22–25 | Full PHPMon parity |
| 9 | 26–30 | No terminal needed for daily tasks |
| 10 | 31–34 | Complete toolbox |
| 11 | 35–40 | Per-site TOML config system |
| 12 | 38–40 | phpMyAdmin per-site |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
