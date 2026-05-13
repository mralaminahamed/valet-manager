# Valet Manager — Linux Desktop Application
## Architecture & Implementation Plan v2.0 (Rust)

> A native Linux desktop GUI for managing Laravel Valet (valet-linux / valet-linux-plus),
> PHP versions, extensions, Nginx, dnsmasq, PHP INI configuration, and a full
> App Creator wizard powered by WP-CLI, Laravel CLI, Composer, and more.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Technology Stack](#2-technology-stack)
3. [Valet Variant Matrix](#3-valet-variant-matrix)
4. [Complete Valet Command Coverage](#4-complete-valet-command-coverage)
5. [Supported Application Types](#5-supported-application-types)
6. [Application Architecture](#6-application-architecture)
7. [Crate & Module Structure](#7-crate--module-structure)
8. [Core Domain Models](#8-core-domain-models)
9. [Module Specifications](#9-module-specifications)
   - [9.1 PHP Manager](#91-php-manager)
   - [9.2 Nginx Manager](#92-nginx-manager)
   - [9.3 Valet Manager](#93-valet-manager)
   - [9.4 dnsmasq Manager](#94-dnsmasq-manager)
   - [9.5 Service Monitor](#95-service-monitor)
   - [9.6 App Creator Module](#96-app-creator-module)
   - [9.7 CLI Tool Registry](#97-cli-tool-registry)
   - [9.8 Proxy Manager](#98-proxy-manager)
   - [9.9 Site Environment Variables](#99-site-environment-variables)
   - [9.10 Custom Driver Manager](#910-custom-driver-manager)
   - [9.11 Sharing Manager](#911-sharing-manager)
10. [GUI Layout & Screens](#10-gui-layout--screens)
11. [System Integration Layer](#11-system-integration-layer)
12. [State Management](#12-state-management)
13. [Configuration & Persistence](#13-configuration--persistence)
14. [Privilege Escalation Strategy](#14-privilege-escalation-strategy)
15. [Packaging & Distribution](#15-packaging--distribution)
16. [Phased Development Roadmap](#16-phased-development-roadmap)
17. [Dependencies Reference](#17-dependencies-reference)

---

## 1. Project Overview

**Application Name:** `valet-manager`
**Binary:** `valet-manager`
**Target Platform:** Linux (Ubuntu 20.04+, Debian 11+, Fedora 36+, Arch)
**Target Valet Forks:**
- `laravel/valet` (macOS-origin, config: `~/.config/valet/`)
- `cpriego/valet-linux` (config: `~/.valet/`)
- `genesisweb/valet-linux-plus` (config: `~/.config/valet/` or `~/.valet/`)

**Inspiration:** PHPMon for macOS — extended significantly with an App Creator wizard
that orchestrates WP-CLI, Laravel CLI, Composer, and other toolchains.

---

## 2. Technology Stack

| Layer | Choice | Rationale |
|---|---|---|
| Language | Rust 2021 edition (stable) | Single binary, memory safety, async |
| GUI | `egui` + `eframe` (wgpu backend) | Zero runtime deps, immediate mode suits live service status |
| Async | `tokio` (full features) | Background subprocess, file watching, polling |
| IPC | `tokio::sync::mpsc` channels | GUI ↔ backend without blocking render thread |
| File watch | `notify` (inotify on Linux) | React to site/config changes in real time |
| Serialization | `serde` + `serde_json` + `toml` | Valet JSON config + app TOML config |
| Templates | `handlebars` | Nginx config templating |
| System tray | `tray-icon` | Quick PHP switcher without opening full app |
| Notifications | `notify-rust` | Desktop notification on service state changes |
| Privilege | Polkit + dedicated helper binary | Never run GUI as root |
| Packaging | `cargo-deb`, `cargo-rpm`, AppImage | Multi-distro distribution |

---

## 3. Valet Variant Matrix

The app auto-detects the installed Valet variant at startup and adjusts all paths and
available commands accordingly. Detection priority: probe config paths → check composer
global installed packages → run `valet --version`.

| Property | `cpriego/valet-linux` | `laravel/valet` (on Linux) | `genesisweb/valet-linux-plus` |
|---|---|---|---|
| Config root | `~/.valet/` | `~/.config/valet/` | `~/.config/valet/` |
| Nginx configs | `~/.valet/Nginx/` | `~/.config/valet/Nginx/` | `~/.config/valet/Nginx/` |
| Sites symlinks | `~/.valet/Sites/` | `~/.config/valet/Sites/` | `~/.config/valet/Sites/` |
| Drivers dir | `~/.valet/Drivers/` | `~/.config/valet/Drivers/` | `~/.config/valet/Drivers/` |
| dnsmasq conf | `/etc/dnsmasq.d/valet` | `/etc/dnsmasq.d/valet` | `/etc/dnsmasq.d/valet` |
| FPM socket | `~/.valet/valet.sock` | `~/.config/valet/valet.sock` | `~/.config/valet/valet.sock` |
| Per-site PHP | `.valetphprc` | `.valetrc` (`php=php@8.x`) | `.valetrc` |
| `isolate` cmd | ❌ | ✅ | ✅ |
| `port` cmd | ✅ | ❌ | ✅ |
| `domain` cmd | ✅ | ❌ | ✅ |
| `status` cmd | ✅ | ❌ | ✅ |
| Isolated drivers | ❌ | ❌ | ✅ |
| Share support | ngrok only | ngrok / Expose / cloudflared | ngrok / Expose |

---

## 4. Complete Valet Command Coverage

Every Valet command is represented as a typed `ValetCommand` enum. The app wraps these
commands with pre-flight validation, result parsing, and GUI state updates.

### 4.1 Site Serving

| Valet Command | GUI Equivalent | Notes |
|---|---|---|
| `valet park` | Parks panel → "Park Directory" button | Directory picker dialog |
| `valet park [path]` | Parks panel → drag-and-drop or path input | |
| `valet forget` | Parks panel → remove row action | Run from parked dir |
| `valet paths` | Parks panel → parked paths list | Read-only display |
| `valet link [name]` | Sites panel → "Link Site" button | Optional name input |
| `valet link subdomain.name` | Sites panel → "Link with subdomain" | Subdomain support |
| `valet unlink [name]` | Sites panel → "Unlink" row action | |
| `valet links` | Sites panel → linked sites list | |

### 4.2 TLS / Security

| Valet Command | GUI Equivalent |
|---|---|
| `valet secure [site]` | Sites panel → lock icon toggle per row |
| `valet unsecure [site]` | Sites panel → lock icon toggle per row |

### 4.3 Per-Site PHP Isolation (valet v4 / valet-linux-plus)

| Valet Command | GUI Equivalent |
|---|---|
| `valet isolate php@8.3 [--site=name]` | Sites panel → PHP version dropdown per row |
| `valet unisolate [--site=name]` | Sites panel → "Reset to global" action |
| `valet isolated` | Sites panel → isolated sites filter |
| `valet which-php` | Sites panel → tooltip showing resolved PHP binary |
| `valet php` | Terminal panel (proxy to site PHP) |
| `valet composer` | App Creator → "Run Composer" action using site PHP |

### 4.4 PHP Global Version

| Valet Command | GUI Equivalent |
|---|---|
| `valet use php@8.3` | PHP Versions panel → "Set Global" button |
| `valet use php` | PHP Versions panel → "Reset to System PHP" |

### 4.5 Service Control

| Valet Command | GUI Equivalent |
|---|---|
| `valet start` | Dashboard → "Start All" button |
| `valet stop` | Dashboard → "Stop All" button |
| `valet restart` | Dashboard → "Restart All" button |
| `valet status` | Dashboard → service status grid |

### 4.6 Sharing

| Valet Command | GUI Equivalent |
|---|---|
| `valet share` | Sites panel → "Share" button → opens Sharing panel |
| `valet share-tool ngrok\|expose\|cloudflared` | Sharing settings → tool selector |
| `valet set-ngrok-token TOKEN` | Sharing settings → token input field |

### 4.7 Proxying

| Valet Command | GUI Equivalent |
|---|---|
| `valet proxy domain http://127.0.0.1:PORT` | Proxies panel → "Add Proxy" dialog |
| `valet proxy domain http://... --secure` | Proxies panel → TLS toggle |
| `valet unproxy domain` | Proxies panel → delete row action |
| `valet proxies` | Proxies panel → proxy list |

### 4.8 Site Environment Variables

| Valet Command | GUI Equivalent |
|---|---|
| `.valet-env.php` file editing | Sites panel → "Env Vars" button → in-app editor |

### 4.9 Configuration

| Valet Command | GUI Equivalent |
|---|---|
| `valet domain [tld]` | dnsmasq panel → TLD changer (valet-linux) |
| `valet port [number]` | Settings panel → Nginx port (valet-linux) |
| `valet directory-listing on\|off` | Settings panel → toggle |
| `valet log` | Logs panel → log file viewer |
| `valet diagnose` | Tools panel → "Run Diagnostics" |
| `valet trust` | Settings panel → "Trust Valet" button |
| `valet uninstall [--force]` | Settings panel → danger zone |

### 4.10 Default Site

| Config | GUI Equivalent |
|---|---|
| `config.json` → `"default": "/path"` | Settings panel → "Default Site" path picker |

---

## 5. Supported Application Types

All frameworks natively supported by Valet drivers, plus additional app types handled
through the App Creator wizard using their respective CLI tools.

### 5.1 Native Valet Driver Support (auto-served)

| Framework / CMS | Detection Logic | Valet Driver |
|---|---|---|
| **Laravel** | `artisan` file + `public/index.php` | `LaravelValetDriver` |
| **WordPress** | `wp-admin/` directory | `WordPressValetDriver` |
| **Bedrock (WordPress)** | `web/wp/` + `composer.json` with roots/wordpress | `BedrockValetDriver` |
| **Symfony** | `config/` + `public/index.php` + symfony deps | `SymfonyValetDriver` |
| **CakePHP 3** | `config/app.php` + `src/` | `CakePHP3ValetDriver` |
| **Craft CMS** | `craft` file at root | `CraftValetDriver` |
| **Statamic** | `statamic/` dir or `artisan` with statamic dep | `StatamicValetDriver` |
| **Jigsaw** | `config.php` + `source/` (Tighten Jigsaw) | `JigsawValetDriver` |
| **Drupal** | `core/lib/Drupal.php` | `DrupalValetDriver` |
| **Joomla** | `administrator/` + `configuration.php` | `JoomlaValetDriver` |
| **Magento** | `app/Mage.php` or `bin/magento` | `MagentoValetDriver` |
| **Slim** | `composer.json` with slim/slim dep | `SlimValetDriver` |
| **Kirby** | `kirby/` directory | `KirbyValetDriver` |
| **OctoberCMS** | `artisan` + `modules/backend/` | `OctoberValetDriver` |
| **ConcreteCMS** | `concrete/` dir + `index.php` | `ConcreteCMSValetDriver` |
| **Contao** | `system/` + `config/localconfig.php` | `ContaoValetDriver` |
| **ExpressionEngine** | `system/ee/` directory | `ExpressionEngineValetDriver` |
| **Katana** | `config.php` (Katana static generator) | `KatanaValetDriver` |
| **Sculpin** | `sculpin.json` + `source/` | `SculpinValetDriver` |
| **Zend Framework** | `module/` + `public/index.php` | `ZendValetDriver` |
| **Static HTML** | `index.html` at root | `BasicValetDriver` |
| **Custom Driver** | `LocalValetDriver.php` in root | `LocalValetDriver` |

### 5.2 App Creator — CLI-Powered Project Types

These are the project types the App Creator wizard can scaffold, each backed by
a specific CLI toolchain.

#### Group A: Laravel Ecosystem

| Project Type | CLI Command | PHP Required | Notes |
|---|---|---|---|
| Laravel (blank) | `laravel new {name}` | ≥8.1 | Laravel Installer |
| Laravel + Breeze | `laravel new {name} --breeze --stack={react\|vue\|blade\|livewire\|api}` | ≥8.2 | Auth scaffolding |
| Laravel + Jetstream | `laravel new {name} --jet --stack={livewire\|inertia} [--teams]` | ≥8.2 | |
| Laravel + Filament | `laravel new {name}` → `composer require filament/filament` | ≥8.1 | Admin panel |
| Laravel API (headless) | `laravel new {name} --api` | ≥8.2 | API-only template |
| Lumen | `composer create-project laravel/lumen {name}` | ≥8.1 | Micro-framework |
| Statamic (flat-file) | `composer create-project statamic/statamic {name}` | ≥8.1 | |
| Statamic + Eloquent | `statamic new {name}` | ≥8.1 | Statamic CLI |

#### Group B: WordPress Ecosystem

| Project Type | CLI Command | Requires | Notes |
|---|---|---|---|
| WordPress (standard) | `wp valet new {name} --db=mysql` | WP-CLI + wp-cli-valet-command | Full install |
| WordPress (SQLite) | `wp valet new {name} --db=sqlite --unsecure` | WP-CLI + wp-cli-valet-command | Portable |
| WordPress (secured) | `wp valet new {name}` (default is https) | WP-CLI + wp-cli-valet-command | Auto TLS |
| Bedrock | `wp valet new {name} --project=bedrock` | WP-CLI + wp-cli-valet-command | Roots.io Bedrock |
| WordPress (manual) | `wp core download && wp config create && wp core install` | WP-CLI | Full WP-CLI control |
| WooCommerce site | WordPress install + `wp plugin install woocommerce --activate` | WP-CLI | |
| Multisite | `wp core multisite-install` | WP-CLI | |

**`wp valet new` Full Options Surface (exposed in GUI):**

```
Name:           <name>               — domain slug (required)
Project:        wp | bedrock         — project type
Directory:      [--in=<dir>]         — parent path (default: current parked dir)
WP Version:     [--version=latest]   — WordPress version
Locale:         [--locale=<locale>]  — e.g. en_US, de_DE, fr_FR
Database:       mysql | sqlite       — driver
DB Name:        [--dbname=wp_name]
DB User:        [--dbuser=root]
DB Password:    [--dbpass=]
DB Host:        [--dbhost=localhost]
DB Prefix:      [--dbprefix=wp_]
Admin User:     [--admin_user=admin]
Admin Password: [--admin_password=admin]
Admin Email:    [--admin_email=]
Unsecure:       [--unsecure]         — HTTP only
Portable:       [--portable]         — sqlite + unsecure
```

**`wp valet destroy` surface:**
```
Name:    <name>   — domain slug
Confirm: [--yes]  — skip confirmation prompt
```

#### Group C: Other PHP Frameworks

| Project Type | CLI Command | Notes |
|---|---|---|
| Symfony (full) | `composer create-project symfony/website-skeleton {name}` | |
| Symfony (micro) | `composer create-project symfony/skeleton {name}` | |
| CakePHP 4 | `composer create-project cakephp/app:{4.*} {name}` | |
| Slim 4 | `composer create-project slim/slim-skeleton {name}` | |
| Craft CMS 5 | `composer create-project craftcms/craft {name}` | Requires `craft setup` post-install |
| Kirby 4 | `composer create-project getkirby/starterkit {name}` | |
| OctoberCMS | `composer create-project october/october {name}` | |
| Drupal | `composer create-project drupal/recommended-project {name}` | |
| Joomla | `composer create-project joomla/joomla-cms {name}` | |
| Magento 2 | `composer create-project magento/project-community-edition {name}` | |
| Static HTML | Create `index.html` + optional TailwindCSS CDN | No CLI needed |

#### Group D: Node.js / Frontend Projects (served as static or proxy)

| Project Type | CLI Command | Serve via Valet |
|---|---|---|
| Next.js | `npx create-next-app {name}` | `valet proxy` → `localhost:3000` |
| Nuxt 4 | `npx nuxi init {name}` | `valet proxy` → `localhost:3000` |
| React (Vite) | `npm create vite@latest {name} -- --template react-ts` | `valet proxy` → `localhost:5173` |
| Vue 3 (Vite) | `npm create vite@latest {name} -- --template vue-ts` | `valet proxy` → `localhost:5173` |
| SvelteKit | `npx sv create {name}` | `valet proxy` → `localhost:5173` |
| Astro | `npm create astro@latest {name}` | `valet proxy` → `localhost:4321` |

For Node.js projects, the App Creator also generates a `start` script and optionally
a systemd user service to keep the dev server alive.

---

## 6. Application Architecture

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                             valet-manager binary                              │
│                                                                               │
│  ┌────────────────────────┐      ┌──────────────────────────────────────────┐ │
│  │      GUI Thread        │      │           App State                      │ │
│  │   (egui / eframe)      │◄────►│   Arc<RwLock<AppState>>                  │ │
│  │                        │ msgs │   PhpState, NginxState, ValetState,      │ │
│  │   Panels / Components  │      │   ServiceState, DnsmasqState,            │ │
│  │   Theme / Layout       │      │   ProxyState, AppCreatorState,           │ │
│  └───────────┬────────────┘      │   DriverState, ShareState                │ │
│              │ AppCommand enum   └──────────────────────────────────────────┘ │
│              ▼                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐   │
│  │                    Command Dispatcher (tokio runtime)                  │   │
│  └──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬───────────────┘   │
│         │      │      │      │      │      │      │      │                   │
│    ┌────▼─┐ ┌──▼──┐ ┌─▼──┐ ┌▼───┐ ┌▼────┐ ┌▼───┐ ┌▼───┐ ┌▼──────────┐     │
│    │ PHP  │ │Nginx│ │Vale│ │DNS │ │Svc  │ │Prxy│ │Shrg│ │App Creator│     │
│    │ Mgr  │ │ Mgr │ │t   │ │masq│ │ Mon │ │ Mgr│ │ Mgr│ │  Module   │     │
│    └──────┘ └─────┘ └────┘ └────┘ └─────┘ └────┘ └────┘ └─────┬─────┘     │
│                                                                  │           │
│                                               ┌──────────────────▼─────────┐ │
│                                               │      CLI Tool Registry     │ │
│                                               │  wp-cli, laravel, composer │ │
│                                               │  npm, npx, valet, artisan  │ │
│                                               └──────────────┬─────────────┘ │
│                                                              │               │
│                             ┌────────────────────────────────▼─────────────┐ │
│                             │              System Bridge                   │ │
│                             │   subprocess, pkexec/polkit, fs, inotify     │ │
│                             └──────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Crate & Module Structure

```
valet-manager/
├── Cargo.toml
├── Cargo.lock
├── build.rs
├── assets/
│   ├── icons/                          # SVG icons, compiled via build.rs
│   ├── fonts/
│   └── templates/
│       ├── nginx-site.conf.hbs
│       ├── nginx-proxy.conf.hbs
│       ├── php-fpm-pool.conf.hbs
│       └── valet-env.php.hbs
├── src/
│   ├── main.rs
│   ├── app.rs                          # Root eframe::App impl
│   ├── commands.rs                     # AppCommand enum (all dispatchable ops)
│   ├── events.rs                       # AppEvent enum (all responses to GUI)
│   ├── config.rs                       # App-level config (~/.config/valet-manager/)
│   │
│   ├── state/
│   │   ├── mod.rs
│   │   ├── app_state.rs
│   │   ├── php_state.rs
│   │   ├── nginx_state.rs
│   │   ├── valet_state.rs              # Sites, parked paths, config
│   │   ├── service_state.rs
│   │   ├── dnsmasq_state.rs
│   │   ├── proxy_state.rs
│   │   ├── share_state.rs
│   │   ├── driver_state.rs
│   │   └── creator_state.rs            # App Creator wizard state
│   │
│   ├── valet/
│   │   ├── mod.rs
│   │   ├── variant.rs                  # Detect valet variant + config paths
│   │   ├── config_reader.rs
│   │   ├── site_scanner.rs
│   │   ├── commands.rs                 # Typed wrappers for every valet CLI command
│   │   ├── driver_manager.rs
│   │   └── env_vars.rs                 # .valet-env.php editor
│   │
│   ├── php/
│   │   ├── mod.rs
│   │   ├── detector.rs
│   │   ├── switcher.rs
│   │   ├── extension_manager.rs
│   │   ├── ini_manager.rs
│   │   └── fpm_manager.rs
│   │
│   ├── nginx/
│   │   ├── mod.rs
│   │   ├── site_manager.rs
│   │   ├── service.rs
│   │   └── template.rs
│   │
│   ├── dnsmasq/
│   │   ├── mod.rs
│   │   ├── config_reader.rs
│   │   └── tld_manager.rs
│   │
│   ├── proxy/
│   │   ├── mod.rs
│   │   └── proxy_manager.rs
│   │
│   ├── sharing/
│   │   ├── mod.rs
│   │   ├── ngrok.rs
│   │   ├── expose.rs
│   │   └── cloudflared.rs
│   │
│   ├── creator/
│   │   ├── mod.rs
│   │   ├── wizard.rs                   # Wizard step state machine
│   │   ├── project_types.rs            # All supported project type definitions
│   │   ├── runners/
│   │   │   ├── mod.rs
│   │   │   ├── laravel.rs              # laravel new, starter kits
│   │   │   ├── wordpress.rs            # wp valet new, wp core install
│   │   │   ├── composer.rs             # composer create-project
│   │   │   ├── node.rs                 # npm create, npx
│   │   │   └── static_html.rs
│   │   ├── post_install.rs             # valet link, secure, isolate after creation
│   │   └── output_streamer.rs          # Stream CLI output to GUI terminal panel
│   │
│   ├── cli_tools/
│   │   ├── mod.rs
│   │   ├── registry.rs                 # Detect installed CLI tools
│   │   ├── wp_cli.rs                   # WP-CLI wrapper
│   │   ├── laravel_cli.rs              # Laravel Installer wrapper
│   │   ├── composer.rs                 # Composer wrapper
│   │   ├── npm.rs                      # npm / npx / pnpm wrapper
│   │   └── artisan.rs                  # artisan command runner
│   │
│   ├── services/
│   │   ├── mod.rs
│   │   └── monitor.rs
│   │
│   ├── system/
│   │   ├── mod.rs
│   │   ├── subprocess.rs
│   │   ├── privilege.rs
│   │   ├── package_manager.rs
│   │   ├── filesystem.rs
│   │   └── distro.rs
│   │
│   ├── tray/
│   │   ├── mod.rs
│   │   └── tray_icon.rs
│   │
│   ├── notifications.rs
│   │
│   └── ui/
│       ├── mod.rs
│       ├── main_window.rs
│       ├── sidebar.rs
│       ├── theme.rs
│       ├── panels/
│       │   ├── mod.rs
│       │   ├── dashboard.rs
│       │   ├── php_versions.rs
│       │   ├── php_extensions.rs
│       │   ├── php_ini.rs
│       │   ├── sites.rs
│       │   ├── parks.rs
│       │   ├── nginx.rs
│       │   ├── dnsmasq.rs
│       │   ├── proxies.rs
│       │   ├── sharing.rs
│       │   ├── drivers.rs
│       │   ├── logs.rs
│       │   ├── app_creator.rs          # App Creator wizard panel
│       │   └── settings.rs
│       └── components/
│           ├── service_badge.rs
│           ├── php_version_card.rs
│           ├── site_row.rs
│           ├── code_editor.rs
│           ├── terminal_output.rs      # Streaming CLI output widget
│           ├── confirm_dialog.rs
│           ├── file_picker.rs
│           └── toast.rs
│
├── helper/                             # Privilege helper binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── handlers.rs
│
├── tests/
│   ├── integration/
│   └── fixtures/
│
└── packaging/
    ├── valet-manager.desktop
    ├── polkit/
    │   └── com.valetmanager.policy
    └── deb/
```

---

## 8. Core Domain Models

```rust
// src/valet/variant.rs

#[derive(Debug, Clone, PartialEq)]
pub enum ValetVariant {
    ValetLinux,          // cpriego/valet-linux
    ValetOfficial,       // laravel/valet (on Linux)
    ValetLinuxPlus,      // genesisweb/valet-linux-plus
}

#[derive(Debug, Clone)]
pub struct ValetPaths {
    pub config_root: PathBuf,    // ~/.valet/ or ~/.config/valet/
    pub nginx_dir: PathBuf,
    pub sites_dir: PathBuf,
    pub drivers_dir: PathBuf,
    pub log_dir: PathBuf,
    pub config_json: PathBuf,
    pub ca_dir: PathBuf,
}

impl ValetPaths {
    pub fn for_variant(variant: &ValetVariant) -> Self {
        let home = dirs::home_dir().unwrap();
        let root = match variant {
            ValetVariant::ValetLinux => home.join(".valet"),
            _ => home.join(".config/valet"),
        };
        ValetPaths {
            nginx_dir:   root.join("Nginx"),
            sites_dir:   root.join("Sites"),
            drivers_dir: root.join("Drivers"),
            log_dir:     root.join("Log"),
            config_json: root.join("config.json"),
            ca_dir:      root.join("CA"),
            config_root: root,
        }
    }
}

// src/state/valet_state.rs

#[derive(Debug, Clone)]
pub struct ValetConfig {
    pub tld: String,
    pub loopback: String,
    pub default_php: String,
    pub paths: Vec<PathBuf>,
    pub default_site: Option<PathBuf>,    // "default" key in config.json
    pub port: Option<u16>,                // valet-linux port override
    pub directory_listing: bool,
}

#[derive(Debug, Clone)]
pub struct ValetSite {
    pub name: String,
    pub domain: String,
    pub path: PathBuf,
    pub site_type: SiteType,
    pub php_version: Option<String>,    // From .valetrc or .valetphprc
    pub is_secured: bool,
    pub nginx_config: PathBuf,
    pub framework: Option<DetectedFramework>,
    pub env_vars_path: Option<PathBuf>, // .valet-env.php
}

#[derive(Debug, Clone, PartialEq)]
pub enum SiteType {
    Parked,
    Linked,
    Proxy(String),   // proxy target URL
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectedFramework {
    Laravel,
    WordPress,
    Bedrock,
    Symfony,
    CakePHP,
    Craft,
    Statamic,
    Jigsaw,
    Drupal,
    Joomla,
    Magento,
    Slim,
    Kirby,
    OctoberCms,
    ConcreteCms,
    Contao,
    ExpressionEngine,
    Katana,
    Sculpin,
    Zend,
    StaticHtml,
    Unknown,
}

// src/state/proxy_state.rs

#[derive(Debug, Clone)]
pub struct ValetProxy {
    pub domain: String,
    pub target: String,       // http://127.0.0.1:9200
    pub is_secured: bool,
    pub nginx_config: PathBuf,
}

// src/creator/project_types.rs

#[derive(Debug, Clone)]
pub struct ProjectType {
    pub id: &'static str,
    pub display_name: &'static str,
    pub group: ProjectGroup,
    pub required_tools: Vec<CliTool>,
    pub options: Vec<ProjectOption>,
    pub post_install_steps: Vec<PostInstallStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectGroup {
    LaravelEcosystem,
    WordPressEcosystem,
    OtherPhp,
    NodeFrontend,
    Static,
}

#[derive(Debug, Clone)]
pub enum CliTool {
    WpCli,
    WpCliValetCommand,
    LaravelInstaller,
    Composer,
    Npm,
    Npx,
    Pnpm,
    StatamicCli,
}

#[derive(Debug, Clone)]
pub struct ProjectOption {
    pub key: &'static str,
    pub label: &'static str,
    pub option_type: OptionType,
    pub default: OptionValue,
    pub choices: Option<Vec<(&'static str, &'static str)>>,
}

#[derive(Debug, Clone)]
pub enum OptionType {
    Text,
    Select,
    Toggle,
    Password,
    DirectoryPicker,
}

// src/commands.rs — full command enum

#[derive(Debug)]
pub enum AppCommand {
    // PHP
    SwitchGlobalPhp(String),
    SwitchSitePhp { site: String, version: String },
    UnswitchSitePhp(String),
    EnableExtension { php_version: String, extension: String },
    DisableExtension { php_version: String, extension: String },
    InstallExtension { php_version: String, extension: String },
    SavePhpIni { php_version: String, ini_type: IniType, content: String },
    RestartPhpFpm(String),
    InstallPhpVersion(String),

    // Sites
    ParkDirectory(PathBuf),
    ForgetDirectory(PathBuf),
    LinkSite { path: PathBuf, name: Option<String> },
    UnlinkSite(String),
    SecureSite(String),
    UnsecureSite(String),
    IsolateSite { site: String, php_version: String },
    UnisolateSite(String),
    OpenSiteInBrowser(String),
    OpenSiteInEditor(String),
    OpenSiteInFileManager(String),
    SaveSiteEnvVars { site: String, content: String },
    SetDefaultSite(PathBuf),

    // Proxy
    AddProxy { domain: String, target: String, secure: bool },
    RemoveProxy(String),

    // Nginx
    ReloadNginx,
    RestartNginx,
    SaveNginxSiteConfig { site: String, content: String },

    // dnsmasq
    SetTld(String),
    SetPort(u16),
    RestartDnsmasq,

    // Services
    StartAllServices,
    StopAllServices,
    RestartAllServices,
    StartService(String),
    StopService(String),
    RestartService(String),
    RefreshServiceStatus,

    // Sharing
    StartShare { site: String, tool: ShareTool },
    StopShare,
    SetShareTool(ShareTool),
    SetNgrokToken(String),

    // App Creator
    CreateApp(AppCreationRequest),
    DestroyWordPressSite { name: String, confirmed: bool },
    CancelCreation,

    // Diagnostics
    RunDiagnose,
    TrustValet,
    ViewLog(LogFile),
    SetDirectoryListing(bool),
    RefreshAll,
}

#[derive(Debug, Clone)]
pub enum ShareTool {
    Ngrok,
    Expose,
    Cloudflared,
}

#[derive(Debug, Clone)]
pub struct AppCreationRequest {
    pub project_type_id: String,
    pub name: String,
    pub parent_directory: PathBuf,
    pub php_version: Option<String>,
    pub options: HashMap<String, OptionValue>,
    pub post_install: PostInstallOptions,
}

#[derive(Debug, Clone)]
pub struct PostInstallOptions {
    pub link_site: bool,
    pub secure_site: bool,
    pub open_in_browser: bool,
    pub open_in_editor: bool,
}
```

---

## 9. Module Specifications

### 9.1 PHP Manager

**Detector** — scans `/usr/bin/php*`, `update-alternatives --list php`,
`~/.phpenv/versions/`, `/usr/bin/phpX.Y` probing. Per detected version,
resolves: binary path, ini paths (cli + fpm), conf.d path, fpm service name.

**Switcher** — priority chain:
1. `valet use php@X.Y` (delegates to Valet's switcher — preferred)
2. `update-alternatives --set php /usr/bin/phpX.Y` (Debian fallback)
3. Direct symlink at `/usr/local/bin/php`

**Per-site isolation** — `valet isolate php@X.Y [--site=name]` (valet v4 / plus).
For `cpriego/valet-linux`, writes `.valetphprc` to site root.

**Extension Manager** — parses `/etc/php/{ver}/{cli,fpm}/conf.d/*.ini`.
Enable: uncomment or recreate ini file. Disable: rename to `.disabled`.
Install: `apt install php{ver}-{ext}` via privilege helper.

**INI Manager** — reads both CLI and FPM ini files.
Section-aware parser groups settings into: Core, Date, Session, OPcache,
Xdebug, Mail, MySQL, Curl, etc. Validates values before write.

---

### 9.2 Nginx Manager

Reads configs from `{valet_paths.nginx_dir}/`. Each config file maps to one
`ValetSite`. Parser extracts: `server_name`, `root`, `fastcgi_pass` (FPM socket),
SSL certificate/key paths, `listen` directives.

Reload: `sudo systemctl reload nginx` via privilege helper.
Template rendering for new sites uses Handlebars templates from assets.

---

### 9.3 Valet Manager

**Config Reader** — deserializes `config.json`. Handles both schema variants
(valet-linux has `port` key; official Valet has `default` key).

**Site Scanner**:
1. Walk all dirs in `config.paths` → Parked sites
2. Walk `Sites/` symlinks → Linked sites
3. Walk `Nginx/` configs to find Proxy entries
4. For each site: detect framework, check `.valetrc`/`.valetphprc`, check CA dir for TLS

**Framework Detector** — lightweight heuristic checks (file/dir existence),
no PHP execution required:

```rust
pub fn detect_framework(site_path: &Path) -> DetectedFramework {
    if site_path.join("wp-admin").is_dir() {
        return DetectedFramework::WordPress;
    }
    if site_path.join("web/wp").is_dir() {
        return DetectedFramework::Bedrock;
    }
    if site_path.join("artisan").exists() {
        if site_path.join("modules/backend").is_dir() {
            return DetectedFramework::OctoberCms;
        }
        return DetectedFramework::Laravel;
    }
    if site_path.join("craft").exists() {
        return DetectedFramework::Craft;
    }
    // ... etc.
    DetectedFramework::Unknown
}
```

---

### 9.4 dnsmasq Manager

Config at `/etc/dnsmasq.d/valet`. Parses `address=/.{tld}/127.0.0.1` lines.
TLD change: runs `valet domain {tld}` which handles dnsmasq restart internally.
Custom entries added as additional `address=` lines via privilege helper.
DNS tester: `tokio::process::Command::new("dig").args([&domain, "@127.0.0.1"])`.

---

### 9.5 Service Monitor

Background tokio task polls every 5 seconds:

```rust
async fn query_service_status(service: &str) -> ServiceStatus {
    let out = Command::new("systemctl")
        .args(["is-active", "--quiet", service])
        .status().await;
    match out {
        Ok(s) if s.success() => ServiceStatus::Running,
        Ok(_)                 => ServiceStatus::Stopped,
        Err(_)                => ServiceStatus::Unknown,
    }
}
```

Services monitored: `nginx`, `php{X.Y}-fpm` (one per installed version), `dnsmasq`.
Failures trigger desktop notification via `notify-rust`.

---

### 9.6 App Creator Module

The App Creator is a multi-step wizard panel. It is the most distinctive feature of
this application beyond PHPMon's scope.

#### Wizard Step Machine

```
Step 1: Select Project Type
    ├── Group selector (Laravel / WordPress / Other PHP / Node / Static)
    └── Project type card grid within group

Step 2: Configure Project
    ├── Name input (→ auto-generates domain preview: name.{tld})
    ├── Parent directory picker (defaults to first parked path)
    ├── PHP version selector (filtered to compatible versions)
    └── Framework-specific options (dynamic, driven by ProjectType.options)

Step 3: Post-Install Options
    ├── [ ] Link site to Valet (valet link)
    ├── [ ] Secure with TLS (valet secure)
    ├── [ ] Isolate PHP version (valet isolate php@X.Y)
    ├── [ ] Open in browser after creation
    └── [ ] Open in editor after creation

Step 4: Creation Progress
    ├── Streaming terminal output widget
    ├── Step checklist (Download → Install deps → Configure DB → Link → Secure)
    └── Cancel button (sends SIGTERM to child process)

Step 5: Success / Error
    ├── Domain URL (clickable → opens browser)
    ├── Site path (clickable → opens file manager)
    └── Quick actions: Open in Editor, View in Browser, View Site Config
```

#### WordPress Creation Flow (wp-cli-valet-command)

```rust
pub async fn create_wordpress(req: &WordPressCreationRequest) -> anyhow::Result<()> {
    // 1. Verify wp-cli is installed
    // 2. Verify wp-cli-valet-command package is installed
    //    → if not, offer to install: `wp package install aaemnnosttv/wp-cli-valet-command:@stable`
    // 3. Build `wp valet new` command with all options from GUI
    // 4. Stream output to GUI terminal widget
    // 5. On success → run post_install steps (link, secure, isolate)
}
```

#### Laravel Creation Flow

```rust
pub async fn create_laravel(req: &LaravelCreationRequest) -> anyhow::Result<()> {
    // 1. Verify laravel installer: `laravel --version`
    //    → if not found, offer: `composer global require laravel/installer`
    // 2. Build command:
    //    `laravel new {name} [--breeze --stack=react] [--jet] [--api]`
    //    in parent_directory
    // 3. Stream output
    // 4. Post-install: link, secure, isolate
}
```

#### Node.js Creation Flow

```rust
pub async fn create_node(req: &NodeCreationRequest) -> anyhow::Result<()> {
    // 1. Run npm create / npx create-next-app / etc in parent_dir
    // 2. Detect dev server port from vite.config / next.config / etc
    // 3. Automatically run: `valet proxy {name} http://127.0.0.1:{port}`
    // 4. Optionally create ~/.config/systemd/user/{name}-dev.service
    //    to keep dev server alive
    // 5. Show the proxied domain in success screen
}
```

#### Output Streamer

```rust
pub struct OutputStreamer {
    tx: tokio::sync::mpsc::Sender<OutputLine>,
}

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub line: String,
    pub stream: StreamType,   // Stdout or Stderr
    pub timestamp: Instant,
}

pub async fn stream_command(
    mut cmd: tokio::process::Command,
    tx: tokio::sync::mpsc::Sender<OutputLine>,
) -> anyhow::Result<ExitStatus> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Spawn two reader tasks, send lines to tx
    // Await child exit
    // Return ExitStatus
}
```

---

### 9.7 CLI Tool Registry

At startup (and on refresh), the app probes for all tools and caches their status.
The App Creator uses this registry to determine which project types are available
and show install prompts for missing tools.

```rust
#[derive(Debug, Clone)]
pub struct CliToolStatus {
    pub tool: CliTool,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub install_command: &'static str,   // Shown in "install this tool" prompt
}

pub async fn probe_all_tools() -> Vec<CliToolStatus> {
    // Probe: wp, composer, laravel, php, node, npm, npx, pnpm, git
    // For wp-cli-valet-command: `wp package list --fields=name | grep aaemnnosttv`
}
```

| Tool | Detection | Install Suggestion |
|---|---|---|
| `wp` (WP-CLI) | `wp --version` | Download wp-cli.phar → install to `/usr/local/bin/wp` |
| `wp valet` (pkg) | `wp package list` | `wp package install aaemnnosttv/wp-cli-valet-command:@stable` |
| `laravel` installer | `laravel --version` | `composer global require laravel/installer` |
| `composer` | `composer --version` | Link to getcomposer.org |
| `node` / `npm` | `node --version` | Link to nodejs.org |
| `pnpm` | `pnpm --version` | `npm install -g pnpm` |
| `git` | `git --version` | `sudo apt install git` |
| `valet` | `valet --version` | Link to valet-linux docs |
| `ngrok` | `ngrok --version` | Download from ngrok.com |
| `expose` | `expose --version` | `composer global require beyondcode/expose` |
| `cloudflared` | `cloudflared --version` | Download from Cloudflare |

---

### 9.8 Proxy Manager

Wraps `valet proxy` / `valet unproxy` / `valet proxies`. Reads existing proxy
configs from Nginx directory (proxy configs have a `proxy_pass` directive instead
of `fastcgi_pass`). The panel shows:

- Domain → target URL mapping
- TLS status per proxy
- "Test Proxy" button (fires HTTP request to the local proxy URL)
- "Add Proxy" dialog with domain input + URL input + TLS toggle

Node.js projects created through the App Creator automatically appear here.

---

### 9.9 Site Environment Variables

`.valet-env.php` editor per site. The app:
1. Reads the file if it exists (or shows an empty template)
2. Presents a table editor: Site | Key | Value with add/remove rows
3. On save, renders the PHP array back to file
4. Falls back to raw PHP editor for advanced use

---

### 9.10 Custom Driver Manager

The Drivers panel lists all PHP files in `{valet_paths.drivers_dir}/`.
It shows: driver name, whether it extends a base driver, and which sites it
is currently serving (by running framework detection across sites).

Actions:
- "New Driver" → opens `SampleValetDriver.php` template in code editor
- "Edit" → opens driver in code editor
- "Delete" → confirms then removes file
- "Open Drivers Directory" → opens file manager

---

### 9.11 Sharing Manager

Wraps `valet share`, `valet share-tool`, `valet set-ngrok-token`.

The Sharing panel:
1. Shows configured share tool (ngrok / Expose / cloudflared)
2. Shows ngrok token input (stored in Valet config, not app config)
3. Per-site "Share" button → runs `valet share` in the site directory
4. Displays the public URL when sharing is active (streamed from ngrok output)
5. "Stop Sharing" button → sends SIGTERM to the share process

---

## 10. GUI Layout & Screens

### 10.1 Sidebar Navigation

```
MANAGEMENT
  Dashboard
  PHP Versions
  PHP Extensions
  PHP INI
  Sites
  Parks
  Nginx
  Proxies
  dnsmasq

CREATE
  App Creator        ← new section

TOOLS
  Sharing
  Drivers
  Logs
  Diagnostics

─────────────
SERVICES
  ● nginx
  ● php8.3-fpm
  ○ php8.2-fpm
  ○ php8.1-fpm
  ● dnsmasq

─────────────
Settings
```

### 10.2 App Creator Panel — Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│  Create New Application                              Step 2 of 5     │
│  ─────────────────────────────────────────────────────────────────   │
│                                                                       │
│  [← Back]                                          [Next: Options →] │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Project Type: WordPress (Standard)                             │ │
│  │                                                                 │ │
│  │  Name:         [mysite              ]  → mysite.test            │ │
│  │  Directory:    [~/Sites/            ]  [Browse...]              │ │
│  │  PHP Version:  [php8.3 ▼           ]                           │ │
│  │                                                                 │ │
│  │  ── WordPress Options ─────────────────────────────────────── │ │
│  │  Version:      [latest ▼           ]                           │ │
│  │  Locale:       [en_US              ]                           │ │
│  │  Database:     (● MySQL)  (○ SQLite)                           │ │
│  │  DB Name:      [wp_mysite          ]                           │ │
│  │  DB User:      [root               ]                           │ │
│  │  DB Password:  [**********         ]                           │ │
│  │  Admin User:   [admin              ]                           │ │
│  │  Admin Pass:   [admin              ]                           │ │
│  │  Admin Email:  [admin@example.com  ]                           │ │
│  │  Protocol:     (● HTTPS)  (○ HTTP only)                        │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

### 10.3 Sites Panel — Enhanced Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Sites                       [+ Create New App]  [🔗 Link]  [Refresh]       │
│  ──────────────────────────────────────────────────────────────────────────  │
│  Filter: [                 ]  Type: [All ▼]  PHP: [All ▼]  [☐ Isolated only]│
│                                                                              │
│  Domain              Path            PHP      Type      TLS  Actions        │
│  ─────────────────────────────────────────────────────────────────────────  │
│  mylaravel.test      ~/Sites/...     8.3      Parked    🔒   [⋮]            │
│  myblog.test         ~/Sites/...     8.2 *    Linked    🔓   [⋮]            │
│  myshop.test         ~/Sites/...     8.3      Parked    🔒   [⋮]            │
│  elasticsearch.test  → :9200         —        Proxy     🔓   [⋮]            │
│                                                                              │
│  * = isolated PHP version override                                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

Actions dropdown per site row:
- Open in Browser
- Open in Editor
- Open in File Manager
- Edit Nginx Config
- Edit PHP INI (per-site)
- Edit Environment Variables (.valet-env.php)
- Change PHP Version
- Secure / Unsecure
- Share via ngrok
- Destroy (WordPress only — runs `wp valet destroy`)
- Unlink

---

## 11. System Integration Layer

### Package Manager Abstraction

```rust
pub enum PackageManager { Apt, Dnf, Pacman }

// Auto-detected from /etc/os-release
// Used for: PHP version installation, PHP extension installation
// All installs run through privilege helper
```

### Privilege Helper Binary

The `valet-manager-helper` binary handles:
- Write `/etc/php/{ver}/cli/php.ini`, `/etc/php/{ver}/fpm/php.ini`
- Write `/etc/nginx/sites-available/`
- Write `/etc/dnsmasq.d/`
- `systemctl start|stop|restart|reload {service}`
- `apt|dnf install {package}`

Communication: Unix domain socket at `/run/valet-manager/helper.sock`.
Protocol: simple length-prefixed JSON messages.

---

## 12. State Management

```rust
#[derive(Debug, Default)]
pub struct AppState {
    pub valet_variant:  Option<ValetVariant>,
    pub valet_paths:    Option<ValetPaths>,
    pub php:            PhpState,
    pub nginx:          NginxState,
    pub valet:          ValetState,
    pub dnsmasq:        DnsmasqState,
    pub services:       ServiceState,
    pub proxies:        ProxyState,
    pub sharing:        ShareState,
    pub drivers:        DriverState,
    pub cli_tools:      Vec<CliToolStatus>,
    pub creator:        AppCreatorState,
    pub ui:             UiState,
}

#[derive(Debug, Default)]
pub struct AppCreatorState {
    pub step: CreatorStep,
    pub selected_type: Option<String>,
    pub form_values: HashMap<String, OptionValue>,
    pub output_lines: Vec<OutputLine>,
    pub creation_status: CreationStatus,
    pub active_child_pid: Option<u32>,  // For cancel support
}

#[derive(Debug, Default, PartialEq)]
pub enum CreatorStep {
    #[default] SelectType,
    Configure,
    PostInstall,
    Progress,
    Complete,
    Error(String),
}
```

---

## 13. Configuration & Persistence

`~/.config/valet-manager/config.toml`:

```toml
[editor]
command = "code"
terminal = "gnome-terminal"
file_manager = "nautilus"

[appearance]
theme = "dark"
font_size = 14
sidebar_width = 220

[php]
show_core_extensions = false
preferred_package_manager = "auto"

[notifications]
service_status_changes = true
php_switch_complete = true
app_creation_complete = true

[refresh]
service_poll_interval_secs = 5
site_scan_debounce_ms = 500

[creator]
default_parent_directory = "~/Sites"
default_db_user = "root"
default_admin_email = "admin@example.com"
auto_secure_on_create = true
auto_open_browser = true

[cli_tools]
wp_cli_path = ""           # override if not in PATH
composer_path = ""
laravel_path = ""
node_path = ""
npm_path = ""
```

---

## 14. Privilege Escalation Strategy

Polkit policy: `com.valetmanager.manage-system`
Rule: `auth_admin_keep` (once per session, not per action)
Helper binary: `/usr/lib/valet-manager/helper`
Socket: `/run/valet-manager/helper.sock`

The helper binary is small (< 500 LOC), handles only a strictly defined set of
operations, and validates all paths/service names before acting. No shell
interpolation — all subprocess calls use explicit argument arrays.

---

## 15. Packaging & Distribution

| Format | Tool | Notes |
|---|---|---|
| `.deb` | `cargo-deb` | Installs helper to `/usr/lib/valet-manager/`, Polkit policy, `.desktop` file |
| `.rpm` | `cargo-rpm` | Same as .deb |
| AppImage | `cargo-appimage` | Bundles helper binary, Polkit policy shipped separately |
| AUR | `PKGBUILD` | Arch Linux |
| GitHub Releases | GH Actions matrix | `x86_64` + `aarch64` for all formats |

---

## 16. Phased Development Roadmap

### Phase 1 — Foundation (Weeks 1–3)
- Cargo workspace (main app + helper binary)
- Valet variant auto-detection + path resolution
- App state + tokio command dispatcher
- egui window, sidebar, panel routing
- Distro + package manager detection
- PHP version detection
- Service status polling (systemd)
- Dashboard panel (read-only)
- CLI Tool Registry (probe + display)
- App config persistence

### Phase 2 — PHP Management (Weeks 4–6)
- PHP Versions panel + global switch (`valet use`)
- PHP-FPM service control
- PHP Extensions panel (list, enable, disable)
- PHP INI editor (section view + raw mode)
- INI validation
- Privilege helper binary + Polkit policy
- PHP installation via apt/dnf (through helper)

### Phase 3 — Sites & Nginx (Weeks 7–9)
- Valet config reader (both path variants)
- Site scanner (parked + linked + proxy)
- Framework detection heuristics
- Sites panel (full table, inline PHP switcher, actions)
- Per-site PHP isolation (`valet isolate` + `.valetrc` fallback)
- Parks panel
- Nginx site config viewer + editor
- inotify-based auto-refresh

### Phase 4 — App Creator — WordPress & Laravel (Weeks 10–12)
- App Creator wizard UI (5-step flow)
- CLI output streaming terminal widget
- WordPress creation (`wp valet new` full options surface)
- `wp valet destroy` with confirmation
- Laravel creation (blank + Breeze + Jetstream)
- Post-install automation (link, secure, isolate)
- Tool prerequisite checker + install prompts

### Phase 5 — App Creator — Full Framework Coverage (Weeks 13–15)
- Composer `create-project` runner (Symfony, CakePHP, Craft, Slim, etc.)
- Node.js project creation (Next.js, Nuxt, React/Vite, Astro)
- Auto-proxy creation for Node.js projects
- Optional systemd user service for Node dev servers

### Phase 6 — Advanced Features (Weeks 16–18)
- Proxies panel (add/remove/test)
- Site Environment Variables editor (.valet-env.php)
- Custom Drivers panel (viewer + editor)
- dnsmasq panel + TLD changer + DNS tester
- Sharing panel (ngrok / Expose / cloudflared)
- Logs panel (nginx-error, fpm-php, valet logs)
- Diagnostics panel (`valet diagnose` output)

### Phase 7 — Polish & Release (Weeks 19–21)
- System tray (quick PHP switcher, service status)
- Desktop notifications
- Dark / Light / System theme
- First-run setup wizard
- Fedora/RHEL (dnf) support
- AppImage + .deb + AUR packaging
- GitHub Actions release pipeline
- Documentation (mdBook)

---

## 17. Dependencies Reference

```toml
[workspace.dependencies]
# GUI
eframe        = { version = "0.31", features = ["default_fonts", "wgpu"] }
egui          = "0.31"
egui_extras   = { version = "0.31", features = ["all_loaders"] }

# Async
tokio         = { version = "1", features = ["full"] }

# System
sysinfo       = "0.33"
which         = "7"
nix           = { version = "0.29", features = ["process", "signal", "fs"] }

# File watching
notify        = "7"

# Serialization
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
toml          = "0.8"

# Templates
handlebars    = "6"

# Error handling
anyhow        = "1"
thiserror     = "2"

# Logging
tracing           = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Tray + notifications
tray-icon     = "0.21"
notify-rust   = "4"

# Paths
dirs          = "5"

# Utilities
regex         = "1"
chrono        = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio-test    = "0.4"
tempfile      = "3"
mockall       = "0.13"
```

---

## Appendix A — `wp valet new` vs Manual WP-CLI Comparison

The App Creator exposes both paths. The `wp valet new` path is the fast path (single command).
The manual WP-CLI path gives full control for advanced users.

| Step | `wp valet new` | Manual WP-CLI |
|---|---|---|
| Download WordPress | Automatic | `wp core download [--version=] [--locale=]` |
| Create config | Automatic | `wp config create --dbname= --dbuser= --dbpass=` |
| Create database | Automatic | `wp db create` |
| Install WordPress | Automatic | `wp core install --url= --title= --admin_user= --admin_email=` |
| Valet link | Automatic | `valet link {name}` |
| TLS cert | Automatic (default) | `valet secure {name}` |
| Destroy | `wp valet destroy {name}` | `wp db drop && rm -rf {dir} && valet unlink {name}` |

---

## Appendix B — Detecting Which `valet` is Installed

```rust
pub async fn detect_valet_variant() -> anyhow::Result<ValetVariant> {
    // 1. Check composer global installed packages
    let output = Command::new("composer")
        .args(["global", "show", "--format=json"])
        .output().await?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    if json["installed"].as_array().map(|a| {
        a.iter().any(|p| p["name"] == "genesisweb/valet-linux-plus")
    }).unwrap_or(false) {
        return Ok(ValetVariant::ValetLinuxPlus);
    }
    if json["installed"].as_array().map(|a| {
        a.iter().any(|p| p["name"] == "cpriego/valet-linux")
    }).unwrap_or(false) {
        return Ok(ValetVariant::ValetLinux);
    }

    // 2. Fallback: probe config paths
    let home = dirs::home_dir().unwrap();
    if home.join(".valet/config.json").exists() {
        return Ok(ValetVariant::ValetLinux);
    }
    if home.join(".config/valet/config.json").exists() {
        return Ok(ValetVariant::ValetOfficial);
    }

    Err(anyhow::anyhow!("Valet not found. Please install valet-linux first."))
}
```

---

## Appendix C — Post-Install Automation Sequence

After any App Creator run completes successfully:

```
1. cd {parent_directory}/{name}

2. If site is NOT in a parked directory:
   → valet link {name}
   (if the parent IS a parked dir, it auto-serves — skip link step)

3. If post_install.secure_site:
   → valet secure {name}
      or
   → valet isolate php@{version} --site={name}  (if php_version specified)

4. If post_install.open_in_browser:
   → xdg-open https://{name}.{tld}

5. If post_install.open_in_editor:
   → {config.editor.command} {parent_directory}/{name}
```

---

*Document Version: 2.0 — Author: Al Amin Ahamed (@mralaminahamed)*
