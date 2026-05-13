# Valet Manager — Linux Desktop Application
## Architecture & Implementation Plan (Rust)

> Inspired by PHPMon for macOS. A native Linux desktop GUI for managing Laravel Valet (valet-linux / valet-linux-plus), PHP versions, extensions, Nginx, dnsmasq, PHP INI configuration, and related services.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Technology Stack Decision](#2-technology-stack-decision)
3. [Application Architecture](#3-application-architecture)
4. [Crate & Module Structure](#4-crate--module-structure)
5. [Core Domain Models](#5-core-domain-models)
6. [Module Specifications](#6-module-specifications)
7. [GUI Layout & Screens](#7-gui-layout--screens)
8. [System Integration Layer](#8-system-integration-layer)
9. [State Management](#9-state-management)
10. [Configuration & Persistence](#10-configuration--persistence)
11. [Privilege Escalation Strategy](#11-privilege-escalation-strategy)
12. [Packaging & Distribution](#12-packaging--distribution)
13. [Phased Development Roadmap](#13-phased-development-roadmap)
14. [Dependencies Reference](#14-dependencies-reference)

---

## 1. Project Overview

**Application Name:** `valet-manager` (working title)
**Binary Name:** `valet-manager`
**Target Platform:** Linux (Ubuntu 20.04+, Debian 11+, Fedora 36+)
**Target Valet Forks:** `cpriego/valet-linux`, `genesisweb/valet-linux-plus`
**Language:** Rust (stable, edition 2021)

### Core Feature Set

| Feature Domain | Capabilities |
|---|---|
| **PHP Versions** | Detect installed versions, switch global/per-site, install/remove via apt/dnf |
| **PHP Extensions** | List, enable, disable per version; detect missing extensions |
| **PHP INI** | In-app editor for php.ini; per-site INI overrides; validation |
| **Nginx** | View/edit site configs; reload/restart service; upstream templates |
| **dnsmasq** | View TLD configuration; add/remove custom domains |
| **Valet Sites** | List parked/linked sites; add/remove links; open in browser/IDE |
| **Services** | Start/stop/restart nginx, php-fpm (per version), dnsmasq |
| **System Tray** | Quick-switch PHP version; service status indicators |
| **Notifications** | Desktop notifications for service state changes |

---

## 2. Technology Stack Decision

### GUI Framework Evaluation

| Framework | Language | Pros | Cons | Verdict |
|---|---|---|---|---|
| **egui / eframe** | Pure Rust | Zero dependencies, wgpu/OpenGL, easy state binding, immediate-mode | Non-native look, custom theming required | ✅ **Primary choice** |
| gtk4-rs | Rust + GTK4 | Native Linux look, GNOME HIG | Complex ownership model, heavy C bindings | Secondary option |
| Tauri v2 | Rust + Web frontend | Web tech for UI, React/TS | Requires Node.js build step; overkill for this scope | ❌ Rejected |
| iced | Pure Rust | Elm architecture, clean | Less mature ecosystem | Consider for v2 |

**Decision: `egui` (via `eframe`)** for the following reasons:
- Single-binary output, zero runtime dependencies
- Immediate-mode rendering suits live-updating service status
- Full control over theming to achieve a polished, custom aesthetic
- Easier async integration via `tokio` + `egui` channel patterns
- Custom fonts/icons via `egui_extras`

### Async Runtime

`tokio` — for spawning background tasks (service management, subprocess calls, file watching) without blocking the GUI thread.

### IPC Pattern

GUI thread ↔ Backend thread via `std::sync::mpsc` / `tokio::sync::mpsc` channels. All system operations run in background tasks; results are sent to the GUI via message passing.

---

## 3. Application Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          valet-manager binary                           │
│                                                                         │
│  ┌──────────────────────┐         ┌──────────────────────────────────┐  │
│  │     GUI Layer        │◄───────►│       App State (Arc<Mutex<>>)   │  │
│  │  (egui / eframe)     │  Msgs   │   PhpState, NginxState,          │  │
│  │                      │         │   ValetState, ServiceState        │  │
│  └──────────┬───────────┘         └──────────────────────────────────┘  │
│             │ Commands                                                   │
│             ▼                                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    Command Dispatcher (tokio)                    │   │
│  └────┬──────────┬──────────┬──────────┬──────────┬────────────────┘   │
│       │          │          │          │          │                     │
│       ▼          ▼          ▼          ▼          ▼                     │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐               │
│  │  PHP   │ │ Nginx  │ │ Valet  │ │dnsmasq │ │Service │               │
│  │Manager │ │Manager │ │Manager │ │Manager │ │Monitor │               │
│  └────┬───┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘               │
│       │         │          │          │           │                     │
│       └─────────┴──────────┴──────────┴───────────┘                    │
│                             │                                           │
│                    ┌────────▼────────┐                                  │
│                    │  System Bridge  │                                  │
│                    │  (subprocess,   │                                  │
│                    │  fs, pkexec)    │                                  │
│                    └─────────────────┘                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Crate & Module Structure

```
valet-manager/
├── Cargo.toml
├── Cargo.lock
├── build.rs                          # Asset embedding, icon compilation
├── assets/
│   ├── icons/                        # SVG icons (embedded at build time)
│   ├── fonts/                        # Custom font files
│   └── templates/
│       ├── nginx-site.conf.hbs       # Nginx site config template
│       └── php-fpm.conf.hbs          # PHP-FPM pool template
├── src/
│   ├── main.rs                       # Entry point, eframe::run_native
│   ├── app.rs                        # Root App struct, eframe::App impl
│   ├── state/
│   │   ├── mod.rs
│   │   ├── app_state.rs              # Global AppState struct
│   │   ├── php_state.rs              # PHP version/extension state
│   │   ├── nginx_state.rs            # Nginx sites/config state
│   │   ├── valet_state.rs            # Valet sites, config state
│   │   ├── service_state.rs          # Systemd service states
│   │   └── dnsmasq_state.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs            # Top-level window layout
│   │   ├── sidebar.rs                # Navigation sidebar
│   │   ├── panels/
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs          # Overview / service status
│   │   │   ├── php_versions.rs       # PHP version management panel
│   │   │   ├── php_extensions.rs     # Extension enable/disable panel
│   │   │   ├── php_ini.rs            # INI editor panel
│   │   │   ├── sites.rs              # Valet sites list panel
│   │   │   ├── nginx.rs              # Nginx config viewer/editor
│   │   │   └── dnsmasq.rs            # dnsmasq TLD/domain panel
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── service_badge.rs      # Running/stopped status badge
│   │   │   ├── php_version_card.rs   # PHP version card widget
│   │   │   ├── site_row.rs           # Site list row widget
│   │   │   ├── code_editor.rs        # Syntax-highlighted text editor
│   │   │   ├── confirm_dialog.rs     # Confirmation modal
│   │   │   └── toast.rs              # Toast notification component
│   │   └── theme.rs                  # Color palette, fonts, spacing
│   ├── managers/
│   │   ├── mod.rs
│   │   ├── php/
│   │   │   ├── mod.rs
│   │   │   ├── detector.rs           # Detect installed PHP versions
│   │   │   ├── switcher.rs           # Switch global/per-site PHP
│   │   │   ├── extension_manager.rs  # Enable/disable extensions
│   │   │   ├── ini_manager.rs        # Read/write php.ini
│   │   │   └── fpm_manager.rs        # PHP-FPM pool management
│   │   ├── nginx/
│   │   │   ├── mod.rs
│   │   │   ├── site_manager.rs       # Parse/write Nginx site configs
│   │   │   ├── service.rs            # nginx reload/restart
│   │   │   └── template.rs           # Config template rendering
│   │   ├── valet/
│   │   │   ├── mod.rs
│   │   │   ├── config_reader.rs      # Read ~/.valet/config.json
│   │   │   ├── site_scanner.rs       # Scan parked/linked sites
│   │   │   └── commands.rs           # Wrap `valet` CLI commands
│   │   ├── dnsmasq/
│   │   │   ├── mod.rs
│   │   │   ├── config_reader.rs
│   │   │   └── tld_manager.rs
│   │   └── service_monitor.rs        # Poll systemd service status
│   ├── system/
│   │   ├── mod.rs
│   │   ├── subprocess.rs             # tokio::process::Command wrappers
│   │   ├── privilege.rs              # pkexec / polkit integration
│   │   ├── package_manager.rs        # apt / dnf / pacman abstraction
│   │   ├── filesystem.rs             # File read/write helpers
│   │   └── distro.rs                 # Detect Linux distribution
│   ├── tray/
│   │   ├── mod.rs
│   │   └── tray_icon.rs              # System tray via tray-icon crate
│   ├── notifications.rs              # Desktop notifications (notify-rust)
│   ├── commands.rs                   # Enum of all dispatchable commands
│   ├── events.rs                     # Enum of all events/responses
│   └── config.rs                     # App-level config (~/.config/valet-manager/)
├── tests/
│   ├── integration/
│   │   ├── php_detection_test.rs
│   │   ├── nginx_parser_test.rs
│   │   └── valet_config_test.rs
│   └── fixtures/
│       ├── php_versions/
│       ├── nginx_configs/
│       └── valet_configs/
└── packaging/
    ├── valet-manager.desktop         # .desktop entry file
    ├── valet-manager.service         # Optional systemd user service
    ├── polkit/
    │   └── com.valetmanager.policy   # Polkit policy for privileged ops
    └── deb/                          # Debian package metadata
        ├── control
        ├── postinst
        └── postrm
```

---

## 5. Core Domain Models

```rust
// src/state/php_state.rs

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct PhpVersion {
    pub version: String,          // e.g. "8.3"
    pub full_version: String,     // e.g. "8.3.12"
    pub binary_path: PathBuf,     // /usr/bin/php8.3
    pub fpm_service: String,      // php8.3-fpm
    pub ini_path: PathBuf,        // /etc/php/8.3/cli/php.ini
    pub fpm_ini_path: PathBuf,    // /etc/php/8.3/fpm/php.ini
    pub conf_d_path: PathBuf,     // /etc/php/8.3/cli/conf.d/
    pub extensions: Vec<PhpExtension>,
    pub is_active: bool,          // Current global version
    pub fpm_running: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhpExtension {
    pub name: String,             // e.g. "xdebug"
    pub version: Option<String>,
    pub enabled: bool,
    pub ini_file: PathBuf,        // 20-xdebug.ini
    pub extension_type: ExtensionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionType {
    Core,       // Compiled-in, cannot disable
    Bundled,    // Ships with PHP
    Pecl,       // Installed via pecl/apt
}

// src/state/valet_state.rs

#[derive(Debug, Clone)]
pub struct ValetConfig {
    pub tld: String,              // "test" or "local"
    pub loopback: String,         // "127.0.0.1"
    pub default_php: String,      // "php8.2"
    pub paths: Vec<PathBuf>,      // Parked directory paths
}

#[derive(Debug, Clone)]
pub struct ValetSite {
    pub name: String,             // Site slug
    pub domain: String,           // site.test
    pub path: PathBuf,            // Absolute path on disk
    pub site_type: SiteType,
    pub php_version: Option<String>,  // Per-site override
    pub is_secured: bool,         // TLS secured
    pub nginx_config: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SiteType {
    Parked,   // Auto-discovered from parked path
    Linked,   // Manually linked via valet link
}

// src/state/service_state.rs

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ManagedService {
    pub name: String,             // "nginx", "php8.3-fpm", "dnsmasq"
    pub display_name: String,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub uptime: Option<String>,
}

// src/commands.rs

#[derive(Debug)]
pub enum AppCommand {
    // PHP
    SwitchGlobalPhp(String),
    SwitchSitePhp { site: String, version: String },
    EnableExtension { php_version: String, extension: String },
    DisableExtension { php_version: String, extension: String },
    SavePhpIni { php_version: String, ini_type: IniType, content: String },
    RestartPhpFpm(String),

    // Nginx
    ReloadNginx,
    RestartNginx,
    SaveNginxSiteConfig { site: String, content: String },

    // Valet
    RefreshSites,
    SecureSite(String),
    UnsecureSite(String),
    OpenSiteInBrowser(String),
    OpenSiteInEditor(String),
    UnlinkSite(String),

    // Services
    StartService(String),
    StopService(String),
    RestartService(String),
    RefreshServiceStatus,

    // dnsmasq
    SetTld(String),
    RestartDnsmasq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IniType {
    Cli,
    Fpm,
}
```

---

## 6. Module Specifications

### 6.1 PHP Manager

#### Version Detection (`managers/php/detector.rs`)

```rust
use tokio::process::Command;
use std::path::PathBuf;

/// Scan for all installed PHP versions on the system.
/// Supports Ondřej Surý PPA (Ubuntu/Debian) and Remi (Fedora/RHEL).
pub async fn detect_installed_versions() -> anyhow::Result<Vec<PhpVersion>> {
    let mut versions = Vec::new();

    // Strategy 1: Scan /usr/bin/php* symlinks (Debian/Ubuntu)
    // Strategy 2: /usr/bin/php{7.4,8.0,8.1,8.2,8.3,8.4} probing
    // Strategy 3: `update-alternatives --list php` output parsing
    // Strategy 4: phpenv / phpbrew shims (~/.phpenv/versions/)

    // For each found binary, derive full PhpVersion struct
    // including ini paths, fpm service name, extension list

    Ok(versions)
}

/// Detect the currently active global PHP version.
/// Checks: valet config → `php --version` → update-alternatives query.
pub async fn detect_active_version() -> anyhow::Result<String> {
    let output = Command::new("php")
        .arg("--version")
        .output()
        .await?;

    // Parse "PHP 8.3.12 (cli)" from output
    parse_php_version_string(&String::from_utf8_lossy(&output.stdout))
}
```

#### Version Switcher (`managers/php/switcher.rs`)

The switcher must handle three mechanisms depending on what is installed:

1. **`valet use php@8.3`** — preferred; delegates to Valet's own switcher
2. **`update-alternatives --set php /usr/bin/php8.3`** — Debian/Ubuntu fallback
3. **Direct symlink manipulation** — last resort

Per-site switching writes an `.valetphprc` file at the site root, which valet-linux-plus respects.

#### Extension Manager (`managers/php/extension_manager.rs`)

```rust
/// Parse all .ini files in /etc/php/{version}/cli/conf.d/
/// Extensions prefixed with `-` in the ini filename or with `; extension=`
/// are treated as disabled.
pub async fn list_extensions(php_version: &str) -> anyhow::Result<Vec<PhpExtension>>;

/// Enable: creates or uncomments the extension ini file.
/// May require pkexec for writes to /etc/php/.
pub async fn enable_extension(php_version: &str, extension: &str) -> anyhow::Result<()>;

/// Disable: renames ini file to <name>.ini.disabled or comments out.
pub async fn disable_extension(php_version: &str, extension: &str) -> anyhow::Result<()>;
```

#### INI Manager (`managers/php/ini_manager.rs`)

- Read `/etc/php/{version}/cli/php.ini` and `/etc/php/{version}/fpm/php.ini`
- Parse key-value pairs; present grouped by INI section
- Validate values before write (type checks for numeric settings, path checks for `extension_dir`)
- Write via pkexec-backed helper binary or polkit D-Bus call

---

### 6.2 Nginx Manager

#### Site Config Parser (`managers/nginx/site_manager.rs`)

Valet stores per-site Nginx configs at:
- `~/.valet/Nginx/{site.tld}` (valet-linux)
- `/etc/nginx/sites-available/valet-{site.tld}` (some variants)

```rust
#[derive(Debug, Clone)]
pub struct NginxSiteConfig {
    pub site_name: String,
    pub server_name: Vec<String>,
    pub root: PathBuf,
    pub php_fpm_socket: String,     // unix:/run/php/php8.3-fpm.sock
    pub ssl_enabled: bool,
    pub ssl_cert: Option<PathBuf>,
    pub ssl_key: Option<PathBuf>,
    pub raw_config: String,         // Full unparsed config for editor
    pub config_path: PathBuf,
}
```

The panel provides:
- Raw config view with syntax highlighting (`egui_extras` CodeEditor or custom tokenizer)
- Parsed field view for common directives (root, PHP version, SSL)
- "Reload Nginx" button after save

---

### 6.3 Valet Manager

#### Config Reader (`managers/valet/config_reader.rs`)

```rust
// ~/.valet/config.json — parsed structure
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ValetJsonConfig {
    pub tld: String,
    pub loopback: String,
    #[serde(rename = "defaultPhp")]
    pub default_php: String,
    pub paths: Vec<String>,
}
```

#### Site Scanner (`managers/valet/site_scanner.rs`)

- Scan all directories in `config.paths` → Parked sites
- Read symlinks in `~/.valet/Sites/` → Linked sites
- Cross-reference with `~/.valet/Nginx/` configs to determine PHP version
- Check for `.valetphprc` files for per-site PHP overrides
- Check `~/.valet/CA/` for secured domains

---

### 6.4 dnsmasq Manager

dnsmasq configuration for Valet lives at `/etc/dnsmasq.d/valet` (valet-linux-plus) or via a custom resolver file.

```rust
#[derive(Debug, Clone)]
pub struct DnsmasqConfig {
    pub tld: String,              // address=/.test/127.0.0.1
    pub custom_entries: Vec<DnsEntry>,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DnsEntry {
    pub domain: String,
    pub address: String,
}
```

Panel features:
- Display current TLD and loopback address
- Add/remove custom domain entries
- Trigger `valet tld <tld>` to change TLD (restarts dnsmasq automatically)
- "Test Resolution" button: runs `dig site.test @127.0.0.1` and shows result

---

### 6.5 Service Monitor

Polls systemd service status every 5 seconds using `systemctl is-active {service}` in a background tokio task. Results are sent to the GUI thread via an `mpsc` channel, updating `ServiceState`.

```rust
pub async fn poll_services(
    services: Vec<String>,
    tx: tokio::sync::mpsc::Sender<Vec<ManagedService>>,
) {
    loop {
        let mut results = Vec::new();
        for service in &services {
            let status = query_systemctl_status(service).await;
            results.push(ManagedService {
                name: service.clone(),
                status,
                ..Default::default()
            });
        }
        let _ = tx.send(results).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

---

## 7. GUI Layout & Screens

### 7.1 Main Window Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  ┌──────────────────┐ ┌──────────────────────────────────────────┐  │
│  │                  │ │  Panel Content Area                      │  │
│  │  SIDEBAR         │ │                                          │  │
│  │  ─────────────   │ │                                          │  │
│  │  [dashboard]     │ │                                          │  │
│  │  [php versions]  │ │                                          │  │
│  │  [extensions]    │ │                                          │  │
│  │  [php ini]       │ │                                          │  │
│  │  [sites]         │ │                                          │  │
│  │  [nginx]         │ │                                          │  │
│  │  [dnsmasq]       │ │                                          │  │
│  │  ─────────────   │ │                                          │  │
│  │  Services:       │ │                                          │  │
│  │  ● nginx         │ │                                          │  │
│  │  ● php8.3-fpm    │ │                                          │  │
│  │  ○ php8.2-fpm    │ │                                          │  │
│  │  ● dnsmasq       │ │                                          │  │
│  └──────────────────┘ └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 Dashboard Panel

- Active PHP version badge (large, prominent)
- Service status grid (nginx, active php-fpm, dnsmasq) with start/stop buttons
- Valet version string
- Site count summary
- Recent activity log (last 10 valet/nginx operations)
- Quick-action buttons: "Open Valet Dir", "Reload All Services"

### 7.3 PHP Versions Panel

- Card grid, one card per installed PHP version
- Each card: version number, active indicator, fpm status, "Set as Global" button, "Restart FPM" button
- "Install New Version" button → opens package manager dialog with available versions

### 7.4 PHP Extensions Panel

- PHP version selector dropdown at top
- Extensions listed in a table: Name | Type | Version | Enabled toggle
- Search/filter bar
- Grouped by: Core, Common, PECL
- Clicking an extension name shows description (sourced from `php -i` or a bundled extension DB)
- "Install Missing Extension" → generates the apt/dnf install command and confirms before running

### 7.5 PHP INI Editor Panel

- PHP version + INI type (CLI / FPM) selector
- Left column: INI section tree (Core, Date, Session, OPcache, etc.)
- Right column: key-value table for selected section with edit-in-place support
- "Raw Edit" toggle: full text editor with syntax highlighting
- Validation errors shown inline (red border + tooltip)
- "Save" requires pkexec; "Revert" restores from disk

### 7.6 Sites Panel

- Table: Domain | Path | PHP Version | Secured | Type | Actions
- Per-site PHP version dropdown (inline change)
- "Secure / Unsecure" toggle button per site
- "Open in Browser" | "Open in File Manager" | "Open in Editor" action buttons
- Refresh button + auto-refresh on file system changes (inotify via `notify` crate)
- "Link New Site" dialog: directory picker + domain override

### 7.7 Nginx Panel

- Left list: all valet site configs
- Right pane: syntax-highlighted config viewer/editor
- "Save & Reload" button
- Template variables highlighted (PHP socket, site root, domain)
- Diff view before saving

### 7.8 dnsmasq Panel

- Current TLD display with "Change TLD" button
- DNS entries table
- Resolution tester: input domain → runs dig → shows result inline
- "Restart dnsmasq" button

---

## 8. System Integration Layer

### 8.1 Package Manager Abstraction (`system/package_manager.rs`)

```rust
pub enum PackageManager {
    Apt,    // Debian, Ubuntu
    Dnf,    // Fedora, RHEL
    Pacman, // Arch Linux
}

pub trait PackageManagerOps {
    async fn install(&self, package: &str) -> anyhow::Result<()>;
    async fn remove(&self, package: &str) -> anyhow::Result<()>;
    async fn list_installed(&self, prefix: &str) -> anyhow::Result<Vec<String>>;
    async fn search(&self, query: &str) -> anyhow::Result<Vec<String>>;
}
```

Auto-detected at startup via `/etc/os-release` parsing.

### 8.2 Privilege Escalation (`system/privilege.rs`)

Two strategies depending on environment:

1. **pkexec** (PolicyKit): preferred on GNOME/KDE desktops. A Polkit `.policy` file registers a `com.valetmanager.manage-system` action with a friendly dialog.
2. **sudo with agent**: fallback for environments without a Polkit agent running.

A dedicated small helper binary (`valet-manager-helper`) is installed to `/usr/lib/valet-manager/helper` with `setuid` or Polkit action. The main process communicates with it via a Unix socket. This avoids running the entire GUI process as root.

```
valet-manager (user)  ──socket──►  valet-manager-helper (root/pkexec)
                                    - writes /etc/php/*/php.ini
                                    - writes /etc/nginx/sites-available/
                                    - systemctl start/stop/restart
                                    - apt install php8.4-{ext}
```

### 8.3 File System Watching

Use the `notify` crate (inotify backend on Linux) to watch:
- `~/.valet/Sites/` — triggers site list refresh
- `~/.valet/config.json` — triggers config reload
- `/etc/php/` — triggers extension/INI state refresh

---

## 9. State Management

```rust
// src/state/app_state.rs

use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct AppState {
    pub php: PhpState,
    pub nginx: NginxState,
    pub valet: ValetState,
    pub dnsmasq: DnsmasqState,
    pub services: ServiceState,
    pub ui: UiState,
}

#[derive(Debug, Default)]
pub struct UiState {
    pub active_panel: Panel,
    pub selected_php_version: Option<String>,
    pub selected_site: Option<String>,
    pub toast_queue: Vec<Toast>,
    pub loading: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub enum Panel {
    #[default]
    Dashboard,
    PhpVersions,
    PhpExtensions,
    PhpIni,
    Sites,
    Nginx,
    Dnsmasq,
}
```

The `AppState` is wrapped in `Arc<Mutex<AppState>>` and shared between the GUI thread and background task handlers. The GUI clones the relevant portion of state at the start of each frame (egui's `update()` call).

---

## 10. Configuration & Persistence

App-level configuration stored at `~/.config/valet-manager/config.toml`:

```toml
[editor]
command = "code"           # IDE to open sites with
terminal = "gnome-terminal"

[appearance]
theme = "dark"             # "dark" | "light" | "system"
font_size = 14

[php]
show_core_extensions = false
preferred_package_manager = "auto"  # "auto" | "apt" | "dnf" | "pacman"

[notifications]
service_status_changes = true
php_switch_complete = true

[refresh]
service_poll_interval_secs = 5
site_scan_debounce_ms = 500
```

Parsed via `serde` + `toml` crate. Config struct implements `Default` for first-run experience.

---

## 11. Privilege Escalation Strategy

### Polkit Policy (`packaging/polkit/com.valetmanager.policy`)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
  "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
  "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <action id="com.valetmanager.manage-system">
    <description>Manage Valet system services and configuration</description>
    <message>Authentication is required to modify PHP/Nginx configuration</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">
      /usr/lib/valet-manager/helper
    </annotate>
  </action>
</policyconfig>
```

The `auth_admin_keep` rule caches the authorization for the session duration so the user isn't prompted on every single action.

---

## 12. Packaging & Distribution

### `.desktop` Entry

```ini
[Desktop Entry]
Name=Valet Manager
Comment=Manage Laravel Valet for Linux
Exec=valet-manager
Icon=valet-manager
Terminal=false
Type=Application
Categories=Development;WebDevelopment;
Keywords=php;laravel;valet;nginx;
StartupWMClass=valet-manager
```

### Distribution Targets

| Format | Tool | Target |
|---|---|---|
| `.deb` | `cargo-deb` | Ubuntu 20.04+, Debian 11+ |
| `.rpm` | `cargo-rpm` | Fedora 36+, RHEL 9+ |
| AppImage | `cargo-appimage` | Universal Linux |
| GitHub Releases | GitHub Actions | All of the above |
| AUR | `PKGBUILD` | Arch Linux |

### Build Pipeline (GitHub Actions)

```yaml
jobs:
  build:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --target ${{ matrix.target }}
      - run: cargo deb --target ${{ matrix.target }}
      - run: cargo rpm build
      - uses: actions/upload-artifact@v4
```

---

## 13. Phased Development Roadmap

### Phase 1 — Foundation (Weeks 1–3)
- [x] Project scaffold, Cargo workspace setup
- [x] egui/eframe window with sidebar navigation shell
- [x] App state architecture + channel-based command dispatcher
- [x] Distro detection + package manager abstraction
- [x] PHP version detection (Debian/Ubuntu PPA variant)
- [x] Service status polling (systemd)
- [x] Dashboard panel (read-only, service status)
- [x] Config persistence (`~/.config/valet-manager/config.toml`)

### Phase 2 — PHP Management (Weeks 4–6)
- [ ] PHP Versions panel with active version indicator
- [ ] Global PHP version switching via `valet use`
- [ ] PHP-FPM start/stop/restart per version
- [ ] PHP Extensions panel — list, enable, disable
- [ ] PHP INI editor — section-based view + raw edit
- [ ] INI validation layer
- [ ] Polkit helper binary for privileged file writes

### Phase 3 — Sites & Nginx (Weeks 7–9)
- [ ] Valet config reader + site scanner
- [ ] Sites panel with full table and actions
- [ ] Per-site PHP version override (`.valetphprc`)
- [ ] Nginx site config viewer and editor
- [ ] Nginx reload on config save
- [ ] Site secure / unsecure via `valet secure`
- [ ] inotify-based site list auto-refresh

### Phase 4 — dnsmasq & Polish (Weeks 10–12)
- [ ] dnsmasq config reader and TLD management
- [ ] DNS resolution tester
- [ ] System tray icon with quick PHP switcher
- [ ] Desktop notifications (notify-rust)
- [ ] Dark / Light / System theme support
- [ ] Toast notification component
- [ ] Error handling and user-facing error messages
- [ ] First-run setup wizard (valet not found, suggest install)

### Phase 5 — Extended Features & Release (Weeks 13–16)
- [ ] PHP version installation from package manager (with terminal output stream)
- [ ] PHP extension installation
- [ ] Nginx upstream template editor
- [ ] Log viewer (nginx error.log, php-fpm error.log)
- [ ] Fedora/RHEL (dnf) support
- [ ] AppImage + .deb + AUR packaging
- [ ] GitHub Actions release pipeline
- [ ] Documentation site (mdBook)

---

## 14. Dependencies Reference

```toml
[dependencies]
# GUI
eframe = { version = "0.31", features = ["default_fonts", "wgpu"] }
egui = "0.31"
egui_extras = { version = "0.31", features = ["all_loaders"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# System
sysinfo = "0.33"          # Process info, memory/cpu
which = "7"               # Locate binaries in PATH
nix = { version = "0.29", features = ["process", "signal", "fs"] }

# File system watching
notify = "7"              # inotify / cross-platform watcher

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Template rendering (nginx configs)
handlebars = "6"

# Error handling
anyhow = "1"
thiserror = "2"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# System tray
tray-icon = "0.21"

# Desktop notifications
notify-rust = "4"

# Config paths
dirs = "5"

# String utilities
regex = "1"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
mockall = "0.13"
```

---

## Appendix A — valet-linux-plus Compatibility Notes

`genesisweb/valet-linux-plus` differs from the original `cpriego/valet-linux` in several areas that affect this application:

| Aspect | cpriego/valet-linux | genesisweb/valet-linux-plus |
|---|---|---|
| PHP switching | `valet use php@X.X` | Same + supports `--update-cli` flag |
| Config location | `~/.valet/` | `~/.config/valet/` on some versions |
| Per-site PHP | `.valetphprc` | Same |
| Nginx configs | `~/.valet/Nginx/` | Same |
| dnsmasq config | `/etc/dnsmasq.d/valet` | Same |
| Isolated drivers | No | Yes (`~/.valet/Drivers/`) |

The detector module must probe both config paths and pick the correct one at startup.

---

## Appendix B — Recommended IDE Integration Points

The "Open in Editor" action reads `config.editor.command` and supports:

| Editor | Command Template |
|---|---|
| VS Code | `code {path}` |
| Cursor | `cursor {path}` |
| Zed | `zed {path}` |
| PhpStorm | `phpstorm {path}` |
| Neovim | `gnome-terminal -- nvim {path}` |
| Custom | User-configurable in settings |

---

*Document Version: 1.0 — Author: Al Amin Ahamed (@mralaminahamed)*
