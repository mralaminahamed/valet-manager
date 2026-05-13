# Valet Manager — Claude Code Prompts
## Phase-by-Phase Implementation Guide

> Each section contains a complete Claude Code prompt for one development phase.
> Copy the prompt block into your terminal: `claude "..."` or paste into an
> interactive Claude Code session. Run each phase prompt from the project root.
>
> **Prerequisites:** Rust stable (1.78+), Cargo, git, Linux development machine
> with a valet-linux variant installed for integration testing.

---

## Phase 1 — Foundation
**Weeks 1–3 · Cargo workspace, egui shell, state architecture, PHP detection, service polling**

```
You are implementing Phase 1 of Valet Manager, a native Linux desktop application
built with Rust and egui that manages Laravel Valet environments. This is a
greenfield project. Your task is to scaffold the complete foundation.

PROJECT CONTEXT:
- App name: valet-manager
- Language: Rust 2021 edition, stable toolchain
- GUI: egui 0.31 + eframe 0.31 (wgpu backend)
- Async: tokio (full features)
- Target: Linux desktop (Ubuntu 20.04+, Debian 11+, Fedora 36+)
- Architecture: GUI thread ↔ tokio background tasks via mpsc channels
- The app manages Laravel Valet (cpriego/valet-linux, laravel/valet on Linux,
  genesisweb/valet-linux-plus) with auto-detection of which fork is installed.

TASK — create the following structure and implement each module fully:

1. CARGO WORKSPACE
   Create Cargo.toml as a workspace with two members:
   - valet-manager (the main GUI binary)
   - valet-manager-helper (a small privilege helper binary, stub only in Phase 1)

   In valet-manager/Cargo.toml add these dependencies:
   eframe = { version = "0.31", features = ["default_fonts", "wgpu"] }
   egui = "0.31"
   egui_extras = { version = "0.31", features = ["all_loaders"] }
   tokio = { version = "1", features = ["full"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   toml = "0.8"
   toml_edit = "0.22"
   anyhow = "1"
   thiserror = "2"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   dirs = "5"
   which = "7"
   chrono = { version = "0.4", features = ["serde"] }
   notify = "7"
   tray-icon = "0.21"
   notify-rust = "4"
   nix = { version = "0.29", features = ["process", "signal", "fs"] }
   sysinfo = "0.33"
   regex = "1"
   futures = "0.3"

2. VALET VARIANT DETECTION — src/valet/variant.rs
   Implement ValetVariant enum: ValetLinux | ValetOfficial | ValetLinuxPlus
   Implement ValetPaths struct with fields:
     config_root, nginx_dir, sites_dir, drivers_dir, log_dir, config_json, ca_dir
   Implement for_variant(v: &ValetVariant) -> ValetPaths using dirs::home_dir()
   Implement async fn detect_valet_variant() -> anyhow::Result<ValetVariant>:
     1. Run `composer global show --format=json`, parse JSON, check "installed" array
        for "genesisweb/valet-linux-plus" or "cpriego/valet-linux"
     2. Fallback: probe ~/.valet/config.json and ~/.config/valet/config.json
     3. Return Err if neither found

3. LINUX DISTRO & PACKAGE MANAGER — src/system/distro.rs + src/system/package_manager.rs
   Parse /etc/os-release to detect: Ubuntu, Debian, Fedora, RHEL, Arch, Unknown
   Implement PackageManager enum: Apt | Dnf | Pacman
   Implement fn detect_package_manager() -> PackageManager based on distro
   Implement async fn list_installed_php_packages(pm: PackageManager) -> Vec<String>
     For apt: run `apt list --installed 2>/dev/null | grep php` and parse output
     For dnf: run `dnf list installed | grep php` and parse
     For pacman: run `pacman -Q | grep php` and parse

4. PHP VERSION DETECTION — src/php/detector.rs
   Implement PhpVersion struct:
     version: String,           // "8.3"
     full_version: String,      // "8.3.12"
     binary_path: PathBuf,
     fpm_service: String,       // "php8.3-fpm"
     cli_ini_path: PathBuf,
     fpm_ini_path: PathBuf,
     conf_d_path: PathBuf,
     is_active: bool,
     fpm_running: bool
   Implement async fn detect_installed_versions() -> anyhow::Result<Vec<PhpVersion>>:
     Strategy 1: Scan /usr/bin/ for files matching regex `^php\d+\.\d+$`
     Strategy 2: Run `update-alternatives --list php` and parse output
     For each found binary, run `{binary} --version` to get full version string
     Derive all paths from the version string (debian path conventions)
   Implement async fn detect_active_version() -> anyhow::Result<String>:
     Run `php --version` and parse "PHP X.Y.Z" from first line

5. SERVICE MONITOR — src/services/monitor.rs
   Implement ServiceStatus enum: Running | Stopped | Failed | Unknown
   Implement ManagedService struct: name, display_name, status, pid (Option<u32>)
   Implement async fn query_service_status(name: &str) -> ServiceStatus:
     Run `systemctl is-active --quiet {name}` → Running if exit 0, Stopped otherwise
   Implement async fn poll_services(
     services: Vec<String>,
     tx: tokio::sync::mpsc::Sender<Vec<ManagedService>>
   ) that loops every 5 seconds, queries all services, sends results to tx

6. APP STATE — src/state/app_state.rs
   Implement AppState struct (derive Default) with fields:
     valet_variant: Option<ValetVariant>
     valet_paths: Option<ValetPaths>
     php_versions: Vec<PhpVersion>
     active_php: Option<String>
     services: Vec<ManagedService>
     cli_tools: Vec<CliToolStatus>       // stub Vec for Phase 1
     ui: UiState
   Implement UiState struct:
     active_panel: Panel
     loading: bool
     toast_queue: Vec<Toast>
     last_error: Option<String>
   Implement Panel enum (derive Default):
     Dashboard (default), PhpVersions, PhpExtensions, PhpIni, Sites, Parks,
     Nginx, Proxies, Dnsmasq, Logs, Diagnostics, Settings

7. APP CONFIG — src/config.rs
   Implement AppConfig struct (serde Deserialize + Serialize + Default):
     editor_command: String = "code"
     terminal_command: String = "gnome-terminal"
     file_manager: String = "nautilus"
     theme: String = "system"
     font_size: f32 = 14.0
     service_poll_interval_secs: u64 = 5
     site_scan_debounce_ms: u64 = 500
     default_parent_directory: String = "~/Sites"
   Implement fn config_path() -> PathBuf: ~/.config/valet-manager/config.toml
   Implement fn load() -> AppConfig: read+parse or return Default
   Implement fn save(cfg: &AppConfig) -> anyhow::Result<()>: serialize + write

8. COMMAND & EVENT ENUMS — src/commands.rs + src/events.rs
   In commands.rs implement AppCommand enum. Phase 1 variants only:
     RefreshAll
     RefreshServiceStatus
     SwitchGlobalPhp(String)
     OpenPanel(Panel)
   In events.rs implement AppEvent enum:
     ServiceStatusUpdated(Vec<ManagedService>)
     PhpVersionsRefreshed(Vec<PhpVersion>)
     ValetDetected(ValetVariant, ValetPaths)
     Error(String)

9. COMMAND DISPATCHER — src/app.rs (tokio side)
   Implement async fn run_dispatcher(
     mut rx: tokio::sync::mpsc::Receiver<AppCommand>,
     tx: tokio::sync::mpsc::Sender<AppEvent>,
     state: Arc<tokio::sync::RwLock<AppState>>
   ) that matches on AppCommand and calls the appropriate manager functions.
   Spawn poll_services as a separate tokio task that runs continuously.

10. GUI SHELL — src/ui/main_window.rs + src/ui/sidebar.rs
    Implement the root egui app struct ValetManagerApp implementing eframe::App.
    The update() method renders:
      Left sidebar (220px wide): navigation items grouped into MANAGEMENT, TOOLS
        Each item: icon area (placeholder) + label + active indicator
        Clicking an item sends AppCommand::OpenPanel(panel) through cmd_tx
      Right content area: match on AppState.ui.active_panel and render the panel
    In sidebar.rs implement fn render_sidebar(ui: &mut egui::Ui, state: &AppState,
      cmd_tx: &mpsc::Sender<AppCommand>) as a free function.

11. DASHBOARD PANEL — src/ui/panels/dashboard.rs
    Implement fn render(ui: &mut egui::Ui, state: &AppState) showing:
      - Active PHP version badge (large, 20px font)
      - Valet variant display ("valet-linux · TLD: .test")
      - Services status grid: 2-column grid of service cards
        Each card shows: name, colored status dot, start/stop button (sends command)
      - "Refresh" button at top-right

12. MAIN ENTRY POINT — src/main.rs
    Set up tokio runtime.
    Create two mpsc channels: (cmd_tx, cmd_rx) and (event_tx, event_rx).
    Create Arc<RwLock<AppState>>.
    Spawn tokio task: run_dispatcher(cmd_rx, event_tx, state.clone()).
    At startup, immediately send RefreshAll and RefreshServiceStatus to cmd_tx.
    Run eframe::run_native with ValetManagerApp.

ACCEPTANCE CRITERIA:
- `cargo build` compiles with zero warnings on stable Rust
- `cargo clippy -- -D warnings` passes
- App window opens: sidebar visible on left, dashboard panel on right
- Dashboard shows detected PHP versions (if any installed)
- Service status grid updates every 5 seconds
- Config loads from disk or defaults silently
- Valet variant is detected and displayed in dashboard header
- All code uses `declare(strict_types)` equivalent: explicit types everywhere,
  no unwrap() in non-test code (use ? or expect() with meaningful messages)
```

---

## Phase 2 — PHP Management
**Weeks 4–6 · PHP version switch, extension manager, INI editor, privilege helper**

```
You are implementing Phase 2 of Valet Manager. Phase 1 is complete: the app
builds, the sidebar and dashboard render, and service status polling works.

Now implement the full PHP management stack.

CONTEXT: Read src/php/detector.rs, src/state/app_state.rs, and src/commands.rs
from Phase 1 before writing any code. All new code extends the existing structure.

TASKS:

1. PRIVILEGE HELPER BINARY — valet-manager-helper/src/main.rs
   This is a small binary that receives JSON requests over stdin and executes
   privileged operations. It will be called via pkexec.
   Implement a JSON protocol:
     Request: { "op": "write_file", "path": "/etc/...", "content": "..." }
     Request: { "op": "systemctl", "action": "restart", "service": "php8.3-fpm" }
     Request: { "op": "apt_install", "package": "php8.3-xdebug" }
     Response: { "ok": true } or { "ok": false, "error": "..." }
   Use a whitelist: only allow paths under /etc/php/ and /etc/nginx/,
   only allow known service names matching `php\d+\.\d+-fpm|nginx|dnsmasq`,
   only allow package names matching `php\d+\.\d+-\w+`.
   The binary reads one JSON line from stdin, processes it, writes one JSON line
   to stdout, then exits. Never loop or persist.

   Implement fn call_helper(request: &HelperRequest) -> anyhow::Result<()> in
   src/system/privilege.rs that runs:
     pkexec /usr/lib/valet-manager/helper
   and sends the JSON request via stdin, reads the response from stdout.

2. POLKIT POLICY — packaging/polkit/com.valetmanager.policy
   Write the XML policy file with:
     action id: com.valetmanager.manage-system
     allow_active: auth_admin_keep
     exec.path: /usr/lib/valet-manager/helper

3. PHP VERSION SWITCHER — src/php/switcher.rs
   Implement async fn switch_global(version: &str) -> anyhow::Result<()>:
     Strategy 1: Run `valet use php@{version}` — check exit code
     Strategy 2 (fallback): update-alternatives --set php /usr/bin/php{version}
   Implement async fn switch_site(site: &str, version: &str, valet_paths: &ValetPaths)
     -> anyhow::Result<()>:
     For valet v4+ (laravel/valet, valet-linux-plus): run `valet isolate php@{version} --site={site}`
     For cpriego/valet-linux: write `php@{version}` to {site_root}/.valetphprc
   Implement async fn unswitch_site(site: &str) -> anyhow::Result<()>:
     Run `valet unisolate --site={site}` or remove .valetphprc

4. PHP-FPM MANAGER — src/php/fpm_manager.rs
   Implement async fn restart(version: &str) -> anyhow::Result<()>
   Implement async fn start(version: &str) -> anyhow::Result<()>
   Implement async fn stop(version: &str) -> anyhow::Result<()>
   All call privilege helper with systemctl action on "php{version}-fpm" service.

5. PHP EXTENSION MANAGER — src/php/extension_manager.rs
   Implement PhpExtension struct:
     name: String, version: Option<String>, enabled: bool,
     ini_file: PathBuf, extension_type: ExtensionType
   Implement ExtensionType enum: Core | Bundled | Pecl
   Implement async fn list_extensions(php_version: &str) -> anyhow::Result<Vec<PhpExtension>>:
     Read all *.ini files from /etc/php/{version}/cli/conf.d/
     Parse each: extension=name or zend_extension=name in file content
     Check if file has .disabled suffix → enabled = false
     Detect if extension is core by checking against a known core list
   Implement async fn enable(php_version: &str, ext_name: &str) -> anyhow::Result<()>:
     If .disabled suffix: call privilege helper to rename without suffix
   Implement async fn disable(php_version: &str, ext_name: &str) -> anyhow::Result<()>:
     Call privilege helper to rename ini file to add .disabled suffix
   Implement async fn install(php_version: &str, ext_name: &str, pm: PackageManager)
     -> anyhow::Result<()>:
     Build package name: php{version}-{ext_name}
     Call privilege helper with apt_install/dnf_install operation

6. PHP INI MANAGER — src/php/ini_manager.rs
   Implement IniType enum: Cli | Fpm
   Implement IniSection struct: name: String, entries: Vec<IniEntry>
   Implement IniEntry struct: key: String, value: String, comment: Option<String>
   Implement fn parse_ini(content: &str) -> Vec<IniSection>:
     Parse PHP ini format: [Section] headers and key = value lines
     Preserve inline comments (text after ;)
   Implement fn render_ini(sections: &[IniSection]) -> String:
     Render back to valid PHP ini format
   Implement async fn read(php_version: &str, ini_type: IniType) -> anyhow::Result<Vec<IniSection>>
   Implement async fn write(php_version: &str, ini_type: IniType, sections: &[IniSection])
     -> anyhow::Result<()>: call privilege helper to write the file
   Implement fn validate_entry(key: &str, value: &str) -> Option<String>:
     Return Some(error_msg) for invalid values:
       memory_limit, upload_max_filesize, post_max_size: validate byte notation (128M, 1G)
       max_execution_time, max_input_time: validate integer
       extension_dir: validate path existence

7. NEW STATE — src/state/php_state.rs
   Add to AppState: php_extensions: HashMap<String, Vec<PhpExtension>>
   Add to AppState: php_ini: HashMap<(String, IniType), Vec<IniSection>>
   Add AppCommand variants:
     RefreshExtensions(String)              // php version
     EnableExtension { php_version, ext }
     DisableExtension { php_version, ext }
     InstallExtension { php_version, ext }
     SavePhpIni { php_version, ini_type: IniType, sections: Vec<IniSection> }
     RestartPhpFpm(String)
     InstallPhpVersion(String)

8. PHP VERSIONS PANEL — src/ui/panels/php_versions.rs
   Implement fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>)
   Layout: vertical list of version cards
   Each card (egui Frame with border) shows:
     Left: version number (large, "PHP 8.3"), full version small below
     Middle: FPM status dot (green/red) + "FPM: running/stopped"
     Right: "Set Global" button (only on inactive versions), "Restart FPM" button
   At the bottom: "Install New Version" button that opens a simple text input dialog
   Active version card has a teal left border accent (use Stroke)

9. PHP EXTENSIONS PANEL — src/ui/panels/php_extensions.rs
   Top: PHP version selector (egui ComboBox), search input, "Show Core" toggle
   Body: egui TableBody with columns: Name | Type | Version | Enabled (toggle)
   Enabled column: egui Checkbox that sends Enable/DisableExtension command on change
   Selected extension info panel (below table): show extension name, a description
     stub ("Extension description not available"), and "Install via apt" button
   Refresh button top-right

10. PHP INI PANEL — src/ui/panels/php_ini.rs
    Top: PHP version ComboBox, INI type selector (CLI / FPM radio), "Raw Edit" toggle
    Section view (default): left egui SelectableLabel list of sections,
      right egui ScrollArea with table of key/value rows
      Value cells: egui TextEdit, inline validation error shown in red below if invalid
    Raw edit mode: full egui TextEdit multiline with the raw INI content
    Bottom: "Save" button (disabled when no changes), "Revert" button
    Save calls SavePhpIni command, which goes through privilege helper

11. WIRE UP DISPATCHER
    In run_dispatcher(), add match arms for all new AppCommand variants.
    After each operation, refresh the relevant state slice and send an AppEvent.

12. TESTS — tests/integration/php_detection_test.rs
    Write integration tests (use #[cfg(test)]) for:
    - parse_ini round-trips: parse then render should produce equivalent output
    - validate_entry: test valid and invalid values for each validated key
    - detect_installed_versions: mock /usr/bin/ using tempdir

ACCEPTANCE CRITERIA:
- `cargo test` passes all unit and integration tests
- PHP Versions panel renders all installed versions correctly
- "Set Global" triggers valet use and refreshes active version
- Extensions panel lists all extensions for selected PHP version
- Enable/Disable toggles work and persist on disk
- INI section editor shows sections and allows editing with inline validation
- "Save" writes via privilege helper (test by temporarily mocking the helper)
- All async operations show a loading indicator while running
```

---

## Phase 3 — Sites & Nginx
**Weeks 7–9 · Valet config reader, site scanner, framework detection, Sites/Parks panels, Nginx editor**

```
You are implementing Phase 3 of Valet Manager. Phases 1 and 2 are complete.

Now implement everything related to site and Nginx management.

CONTEXT: The app already detects valet variant and paths. Read src/valet/variant.rs
and study ValetPaths before starting. The site scanner must handle all three
valet forks.

TASKS:

1. VALET CONFIG READER — src/valet/config_reader.rs
   Implement ValetJsonConfig (serde Deserialize):
     tld: String, loopback: String, default_php: String,
     paths: Vec<String>, default: Option<String>, port: Option<u16>
   Implement async fn read_config(paths: &ValetPaths) -> anyhow::Result<ValetJsonConfig>:
     Read and parse config_json from ValetPaths. Normalize ~ in path strings.
   Implement async fn write_config(paths: &ValetPaths, cfg: &ValetJsonConfig)
     -> anyhow::Result<()>

2. SITE SCANNER — src/valet/site_scanner.rs
   Implement ValetSite struct:
     name: String, domain: String, path: PathBuf,
     site_type: SiteType,          // Parked | Linked | Proxy(String)
     php_version: Option<String>,  // From .valetrc / .valetphprc
     is_secured: bool,
     nginx_config: PathBuf,
     framework: DetectedFramework,
     is_favorite: bool
   Implement DetectedFramework enum with all 22 Valet-supported frameworks + Unknown
   Implement async fn scan_sites(paths: &ValetPaths, config: &ValetJsonConfig,
     favorites: &[String]) -> anyhow::Result<Vec<ValetSite>>:
     Step 1: For each path in config.paths, scan directory entries → Parked sites
     Step 2: Read symlinks in paths.sites_dir → Linked sites
     Step 3: Scan paths.nginx_dir for proxy configs (contains `proxy_pass`) → Proxy sites
     Step 4: For each site, call detect_framework(&site_path)
     Step 5: For each site, check .valetrc (php=php@X.X) or .valetphprc (php@X.X)
     Step 6: Check paths.ca_dir for {domain}.crt to determine is_secured

3. FRAMEWORK DETECTOR — src/valet/site_scanner.rs (within same file)
   Implement fn detect_framework(path: &Path) -> DetectedFramework using file probes:
     wp-admin/ dir → WordPress
     web/wp/ dir → Bedrock
     artisan file + modules/backend/ → OctoberCms
     artisan file + artisan content contains "statamic" → Statamic
     artisan file → Laravel
     craft file → Craft
     config/ + public/index.php + composer.json contains "symfony" → Symfony
     index.html (no index.php) → StaticHtml
     config.php + source/ dir → Jigsaw
     etc. for all 22 frameworks
     Default → Unknown

4. NGINX SITE CONFIG — src/nginx/site_manager.rs
   Implement NginxSiteConfig struct:
     site_name: String, server_names: Vec<String>, root: PathBuf,
     php_fpm_socket: String, ssl_enabled: bool,
     ssl_cert: Option<PathBuf>, ssl_key: Option<PathBuf>,
     raw_config: String, config_path: PathBuf
   Implement async fn read_site_config(path: &Path) -> anyhow::Result<NginxSiteConfig>:
     Read raw content, extract fields using regex patterns:
       server_name: `server_name\s+([^;]+);`
       root: `root\s+([^;]+);`
       fastcgi_pass: `fastcgi_pass\s+([^;]+);`
       ssl_certificate: `ssl_certificate\s+([^;]+);`
   Implement async fn write_site_config(config: &NginxSiteConfig) -> anyhow::Result<()>:
     Write raw_config to config_path via privilege helper
   Implement async fn reload_nginx() -> anyhow::Result<()>: call privilege helper

5. FILE SYSTEM WATCHER — src/valet/watcher.rs
   Implement fn start_site_watcher(
     sites_dir: PathBuf, nginx_dir: PathBuf,
     tx: tokio::sync::mpsc::Sender<AppCommand>
   ) using the notify crate (inotify backend):
     Watch sites_dir and nginx_dir recursively
     On any Create/Remove/Modify event, debounce 500ms, send AppCommand::RefreshSites

6. NEW STATE FIELDS (add to AppState):
   valet_config: Option<ValetJsonConfig>
   sites: Vec<ValetSite>
   nginx_configs: HashMap<String, NginxSiteConfig>  // keyed by site name
   selected_site: Option<String>
   Add AppCommand variants:
     RefreshSites
     ParkDirectory(PathBuf)
     ForgetDirectory(PathBuf)
     LinkSite { path: PathBuf, name: Option<String> }
     UnlinkSite(String)
     SecureSite(String)
     UnsecureSite(String)
     IsolateSite { site: String, php_version: String }
     UnisolateSite(String)
     OpenSiteInBrowser(String)
     OpenSiteInEditor(String)
     OpenSiteInFileManager(String)
     SaveNginxConfig { site: String, content: String }
     ReloadNginx

7. SITES PANEL — src/ui/panels/sites.rs
   Top bar:
     Search input (filter by domain/path)
     Type filter ComboBox: All | Parked | Linked | Proxy
     PHP filter ComboBox: All | 8.3 | 8.2 | etc.
     "Link Site" button | "Refresh" button
   Table (egui TableBuilder with fixed columns):
     ★ (favorite toggle) | Domain | Path (truncated) | PHP | Type badge | TLS icon | ⋮ menu
   Column widths: 24 | 180 | 200 | 70 | 80 | 36 | 32
   ⋮ context menu (egui popup): Open in Browser, Open in Editor, Open in File Manager,
     Edit Nginx Config, Change PHP Version (submenu), Secure/Unsecure, Unlink
   PHP version cell: egui ComboBox showing all installed versions, sends IsolateSite on change
   Clicking domain text: sends OpenSiteInBrowser
   Favorite star: filled style when is_favorite=true, sends ToggleFavoriteSite on click
   Favorites always appear at the top of the list regardless of sort order

8. PARKS PANEL — src/ui/panels/parks.rs
   Simple list of parked directory paths from valet_config.paths
   Each row: path string, "Open in File Manager" icon, "Remove" button (sends ForgetDirectory)
   Bottom: "Add Directory" button → opens egui file dialog (use rfd crate for native dialog)
     On confirm, sends ParkDirectory(selected_path)
   Add rfd = "0.14" to dependencies for native file picker

9. NGINX PANEL — src/ui/panels/nginx.rs
   Left pane (200px): scrollable list of site names, click to select
   Right pane: shows NginxSiteConfig for selected site
     Two tabs: "Config" (raw text editor) and "Info" (parsed fields table)
     Config tab: egui TextEdit with monospace font (size 13), full height
       "Save & Reload" button at bottom: sends SaveNginxConfig then ReloadNginx
       "Revert" button: restores to last saved content
     Info tab: simple two-column table of parsed fields (server_name, root, socket, ssl)
   "Reload Nginx" button always visible at top-right

10. INOTIFY WATCHER STARTUP
    In src/main.rs, after spawning the dispatcher, start the file system watcher:
      start_site_watcher(valet_paths.sites_dir.clone(), valet_paths.nginx_dir.clone(),
        cmd_tx.clone())

ACCEPTANCE CRITERIA:
- `cargo test` passes (add unit tests for detect_framework with fixture directories)
- Sites panel shows all parked and linked sites with correct framework badges
- Favorite sites sort to the top
- PHP version dropdown in sites panel changes and sends isolate command
- Nginx panel shows raw config with editable text area
- "Save & Reload" writes config and runs nginx reload via helper
- Parks panel shows all parked paths, "Remove" calls valet forget
- Adding/removing a site in the filesystem triggers auto-refresh (test with inotify)
- Search filter correctly reduces visible sites
```

---

## Phase 4 — App Creator: WordPress & Laravel
**Weeks 10–12 · 5-step wizard, CLI streaming, WP-CLI integration, Laravel CLI, post-install**

```
You are implementing Phase 4 of Valet Manager. Phases 1–3 are complete.

Implement the App Creator wizard — starting with WordPress (via wp-cli-valet-command)
and Laravel (via laravel/installer). This is the most complex panel in the app.

CONTEXT: Read src/cli_tools/ (Phase 1 stubs), src/valet/site_scanner.rs, and
src/state/app_state.rs before starting.

TASKS:

1. CLI TOOL REGISTRY — src/cli_tools/registry.rs
   Implement CliTool enum:
     WpCli, WpCliValetCommand, LaravelInstaller, Composer,
     Npm, Npx, Git, Valet
   Implement CliToolStatus struct:
     tool: CliTool, installed: bool, version: Option<String>,
     path: Option<PathBuf>, install_hint: String
   Implement async fn probe_all() -> Vec<CliToolStatus>:
     For each tool, use `which` to find binary, then run --version to get string
     For WpCliValetCommand: run `wp package list --fields=name 2>/dev/null | grep aaemnnosttv`
     Set install_hint per tool:
       WpCli: "curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar"
       WpCliValetCommand: "wp package install aaemnnosttv/wp-cli-valet-command:@stable"
       LaravelInstaller: "composer global require laravel/installer"
       Composer: "Link: https://getcomposer.org/download/"

2. OUTPUT STREAMER — src/creator/output_streamer.rs
   Implement OutputLine struct: line: String, stream: StreamType, ts: Instant
   Implement StreamType enum: Stdout | Stderr
   Implement async fn stream_command(
     cmd: &str, args: &[&str], cwd: Option<&Path>,
     tx: tokio::sync::mpsc::Sender<OutputLine>
   ) -> anyhow::Result<std::process::ExitStatus>:
     Spawn child process with piped stdout + stderr
     Spawn two reader tasks (one per stream) using tokio::io::BufReader + lines()
     Each line read → send OutputLine to tx
     Await child process exit
     Return ExitStatus
   Implement fn is_success(status: &ExitStatus) -> bool

3. PROJECT TYPE DEFINITIONS — src/creator/project_types.rs
   Implement ProjectGroup enum:
     LaravelEcosystem | WordPressEcosystem | OtherPhp | NodeFrontend | Static
   Implement OptionType enum: Text | Select | Toggle | Password | DirectoryPicker
   Implement OptionValue enum: Text(String) | Bool(bool)
   Implement ProjectOption struct:
     key: &'static str, label: &'static str, option_type: OptionType,
     default: OptionValue, choices: Option<Vec<(&'static str, &'static str)>>,
     required: bool, placeholder: &'static str
   Implement ProjectType struct:
     id: &'static str, display_name: &'static str, description: &'static str,
     group: ProjectGroup, icon_name: &'static str,
     required_tools: Vec<CliTool>, options: Vec<ProjectOption>
   Implement fn all_project_types() -> Vec<ProjectType> returning these types:
     Laravel (blank), Laravel + Breeze, Laravel + Jetstream, Laravel API
     WordPress (standard), WordPress (SQLite/portable), Bedrock
     Static HTML
   (Other types added in Phase 5)

4. CREATOR STATE — src/state/creator_state.rs
   Implement CreatorStep enum (derive Default):
     SelectType (default), Configure, PostInstall, Progress, Complete, Error(String)
   Implement PostInstallOptions struct:
     link_site: bool = true, secure_site: bool = true,
     isolate_php: bool = false, preferred_php: Option<String>,
     open_browser: bool = true, open_editor: bool = false
   Implement AppCreatorState struct:
     step: CreatorStep
     selected_type_id: Option<String>
     name: String
     parent_directory: PathBuf
     form_values: HashMap<String, OptionValue>
     post_install: PostInstallOptions
     output_lines: Vec<OutputLine>
     running: bool
     child_pid: Option<u32>
     completed_domain: Option<String>
   Add to AppCommand: CreateApp(AppCreationRequest), CancelCreation

5. WORDPRESS RUNNER — src/creator/runners/wordpress.rs
   Implement async fn create(req: &AppCreationRequest,
     tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
     Step 1: Check WpCli and WpCliValetCommand are installed
       If missing WpCliValetCommand → stream install: `wp package install aaemnnosttv/wp-cli-valet-command:@stable`
     Step 2: Build `wp valet new {name}` command from req.options:
       --project: "wp" or "bedrock"
       --in: req.parent_directory
       --version, --locale, --db, --dbname, --dbuser, --dbpass, --dbhost, --dbprefix
       --admin_user, --admin_password, --admin_email
       Add --unsecure if protocol = "http"
       Add --portable if db = "sqlite" AND protocol = "http"
     Step 3: stream_command("wp", &args, Some(&req.parent_directory), tx.clone())
     Step 4: On success, run post_install steps

   Implement async fn destroy(name: &str, tx: mpsc::Sender<OutputLine>)
     -> anyhow::Result<()>:
     stream_command("wp", &["valet", "destroy", name, "--yes"], None, tx)

6. LARAVEL RUNNER — src/creator/runners/laravel.rs
   Implement async fn create(req: &AppCreationRequest,
     tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
     Step 1: Verify laravel installer. If missing, stream:
       `composer global require laravel/installer`
     Step 2: Build `laravel new {name}` command:
       If type = "Laravel + Breeze": add --breeze --stack={stack} --dark (if toggled)
       If type = "Laravel + Jetstream": add --jet --stack={stack} --teams (if toggled)
       If type = "Laravel API": add --api
       Always add --git if git is installed
     Step 3: stream_command in req.parent_directory
     Step 4: post_install steps

7. POST-INSTALL HANDLER — src/creator/post_install.rs
   Implement async fn run(
     site_name: &str, site_path: &Path, opts: &PostInstallOptions,
     valet_config: &ValetJsonConfig, valet_paths: &ValetPaths,
     tx: mpsc::Sender<OutputLine>
   ) -> anyhow::Result<String> (returns the final domain URL):
     Step 1: If site_path.parent() not in valet_config.paths:
       stream "valet link {site_name}"
     Step 2: If opts.secure_site:
       stream "valet secure {site_name}"
     Step 3: If opts.isolate_php && opts.preferred_php.is_some():
       stream "valet isolate php@{ver} --site={site_name}"
     Step 4: Return format!("http{}://{}.{}", if secure {"s"} else {""}, site_name, tld)

8. TERMINAL OUTPUT WIDGET — src/ui/components/terminal_output.rs
   Implement fn render(ui: &mut egui::Ui, lines: &[OutputLine], scroll_to_bottom: bool)
   Uses egui ScrollArea with vertical scroll
   Each line: monospace font (13px), Stderr lines shown in amber color
   If scroll_to_bottom: call ui.scroll_to_cursor(Some(Align::BOTTOM)) each frame
   Max display: last 1000 lines (trim from front if over limit)

9. APP CREATOR PANEL — src/ui/panels/app_creator.rs
   Implement fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>)
   Render the active CreatorStep:

   STEP 1 - SelectType:
     Header: "Choose project type"
     Five group tabs: Laravel | WordPress | PHP | Node | Static
     Grid of ProjectType cards (3 columns) showing icon placeholder, name, description
     Clicking a card: sets selected_type_id and advances to Configure
     Available tools shown bottom: "✓ wp-cli  ✓ laravel  ✗ composer → Install"

   STEP 2 - Configure:
     Header: selected project type name
     "← Back" button (returns to SelectType)
     Form fields driven by ProjectType.options:
       Text / Password: egui TextEdit
       Select: egui ComboBox with choices
       Toggle: egui Checkbox
       DirectoryPicker: TextEdit + "Browse" button using rfd::FileDialog
     Domain preview: "{name}.{tld}" shown as user types name
     "Next →" button (disabled if required fields empty)

   STEP 3 - PostInstall:
     Checkboxes: Link site, Secure with TLS, Isolate PHP (+ version selector), Open in browser, Open in editor
     "← Back" | "Create →" buttons

   STEP 4 - Progress:
     Project type + name in header
     Progress checklist: Downloading | Installing dependencies | Configuring | Linking | Securing
     terminal_output::render() showing real-time output
     "✕ Cancel" button: sends CancelCreation

   STEP 5 - Complete / Error:
     Complete: success icon, domain URL as clickable link, "Open in Editor" button
     Error: red error message, "← Try Again" button (resets to Configure)

10. WIRE UP DISPATCHER
    Add match arms for CreateApp and CancelCreation in run_dispatcher.
    CreateApp: spawn tokio task, route to wordpress/laravel runner, pipe output via mpsc
    CancelCreation: send SIGTERM to child_pid if Some

ACCEPTANCE CRITERIA:
- All CLI tool statuses display correctly on the SelectType step
- WordPress form shows all 14 wp valet new options with correct input types
- Laravel form shows starter kit options (Breeze stacks, Jetstream options)
- Terminal output streams in real-time during creation
- Cancel stops the child process
- Post-install runs valet link, valet secure, valet isolate in sequence
- Completed domain appears as a clickable link on the success screen
- wp valet destroy works from the Sites panel context menu for WordPress sites
```

---

## Phase 5 — App Creator: Full Framework Coverage
**Weeks 13–15 · 20+ frameworks via Composer, Node.js + auto-proxy, systemd dev services**

```
You are implementing Phase 5 of Valet Manager. Phase 4 is complete with WordPress
and Laravel creation working.

Extend the App Creator to cover all remaining project types.

TASKS:

1. COMPOSER RUNNER — src/creator/runners/composer.rs
   Implement async fn create_project(
     package: &str, name: &str, parent_dir: &Path,
     extra_args: &[&str], tx: mpsc::Sender<OutputLine>
   ) -> anyhow::Result<()>:
     stream_command("composer", &["create-project", package, name, "--prefer-dist",
       "--no-interaction", ...extra_args], Some(parent_dir), tx)
   No additional logic needed — Composer handles everything.

2. STATIC HTML RUNNER — src/creator/runners/static_html.rs
   Create directory at parent_dir/name
   Write index.html with a minimal HTML5 template including TailwindCSS CDN link
   Send OutputLine messages simulating output ("Creating directory...", "Done")
   No subprocess needed

3. NODE.JS RUNNER — src/creator/runners/node.rs
   Implement ProjectNodeConfig struct: framework, command, port: u16, package_manager
   Implement async fn create(req: &AppCreationRequest, tx: mpsc::Sender<OutputLine>)
     -> anyhow::Result<u16> (returns detected dev server port):
     Match on project type to get command and default port:
       Next.js: `npx create-next-app@latest {name} --ts --tailwind --no-git` → port 3000
       Nuxt 4: `npx nuxi@latest init {name}` → port 3000
       React/Vite: `npm create vite@latest {name} -- --template react-ts` → port 5173
       Vue/Vite: `npm create vite@latest {name} -- --template vue-ts` → port 5173
       SvelteKit: `npx sv create {name}` → port 5173
       Astro: `npm create astro@latest {name} -- --template minimal --typescript strict` → 4321
     Run the creation command via stream_command
     Install dependencies: `npm install` in the created directory
     Return the default port for the framework

4. NODE.JS PROXY CREATION — src/creator/runners/node.rs (continued)
   After creation, automatically create a Valet proxy:
   Implement async fn create_proxy_for_node(
     site_name: &str, port: u16, secure: bool, tx: mpsc::Sender<OutputLine>
   ) -> anyhow::Result<()>:
     Run: `valet proxy {site_name} http://127.0.0.1:{port}` [--secure if secure]
     Stream the output
   This is called from post_install automatically for Node.js project types

5. SYSTEMD DEV SERVICE — src/creator/runners/node.rs (continued)
   Implement async fn create_dev_service(
     site_name: &str, site_path: &Path, start_command: &str, port: u16
   ) -> anyhow::Result<()>:
     Render systemd user service template:
       [Unit] Description=Valet Manager Dev Server — {site_name}
       [Service] WorkingDirectory={site_path}, ExecStart={start_command},
         Environment=PORT={port}, Restart=on-failure
       [Install] WantedBy=default.target
     Write to: ~/.config/systemd/user/valet-dev-{site_name}.service
     Run: systemctl --user daemon-reload
     Ask user (via post_install option): enable service? → systemctl --user enable --now

6. EXPAND PROJECT TYPES — src/creator/project_types.rs
   Add all remaining project types to all_project_types():

   PHP frameworks (Composer):
     Symfony Full:    symfony/website-skeleton
     Symfony Micro:   symfony/skeleton
     CakePHP 4:       cakephp/app
     Slim 4:          slim/slim-skeleton
     Craft CMS 5:     craftcms/craft
     Kirby 4:         getkirby/starterkit
     OctoberCMS:      october/october
     Drupal:          drupal/recommended-project
     Joomla:          joomla/joomla-cms
     Magento 2:       magento/project-community-edition
     Statamic:        statamic/statamic

   Node.js (from step 3):
     Next.js, Nuxt 4, React/Vite, Vue 3/Vite, SvelteKit, Astro

   Static: Static HTML (from step 2)

   For Node.js types, add option: "Keep dev server running (systemd service)" toggle

7. POST-INSTALL EXTENSION for Node.js
   Modify src/creator/post_install.rs to detect if project_type is Node:
     Instead of valet link, call create_proxy_for_node
     Optionally call create_dev_service if option is enabled

8. EXPAND APP CREATOR UI — src/ui/panels/app_creator.rs
   Step 1 now shows all 5 groups with correct counts
   Node.js group cards show additional info: "Served via Valet proxy"
   PHP framework cards show: "via Composer · {package_name}"
   Laravel and WordPress cards have special visual treatment (logo placeholder)

9. PREREQUISITE INSTALLER DIALOG — src/ui/components/confirm_dialog.rs
   Implement a general-purpose confirm dialog using egui Window:
     title, message, confirm_label, cancel_label, on_confirm: AppCommand
   Use this for "Install missing tool?" prompts before creation starts
   Show the exact install command that will run so the user can verify

10. TESTS
    Add tests/integration/creator_test.rs:
    - Test static HTML runner creates correct directory structure
    - Test composer runner builds correct argument list for each framework
    - Test node runner returns correct port for each framework
    - Test post_install logic for node (proxy) vs PHP (link) projects

ACCEPTANCE CRITERIA:
- All 30+ project types appear in the creator with correct group assignment
- Selecting a Composer-based PHP framework shows a simple name+directory form
- Node.js creation creates the project AND sets up a Valet proxy automatically
- "Keep dev server running" creates a systemd user service file
- Missing tool prompts show the exact install command before running it
- Static HTML creates a valid index.html with Tailwind CDN
- All runners use stream_command so output appears in real-time
```

---

## Phase 6 — Advanced Features
**Weeks 16–18 · Proxies, .valet-env.php, Custom Drivers, dnsmasq, Sharing, Logs, Diagnostics**

```
You are implementing Phase 6 of Valet Manager. Phases 1–5 are complete.

Implement the remaining management panels.

TASKS:

1. PROXY MANAGER — src/proxy/proxy_manager.rs
   Implement ValetProxy struct:
     domain: String, target: String, is_secured: bool, nginx_config: PathBuf
   Implement async fn list_proxies(paths: &ValetPaths) -> anyhow::Result<Vec<ValetProxy>>:
     Read all nginx configs in paths.nginx_dir that contain `proxy_pass`
     Parse domain from filename, target from proxy_pass directive
     Check for ssl_certificate to determine is_secured
   Implement async fn add(domain: &str, target: &str, secure: bool) -> anyhow::Result<()>:
     Run `valet proxy {domain} {target}` [--secure]
   Implement async fn remove(domain: &str) -> anyhow::Result<()>:
     Run `valet unproxy {domain}`
   Implement async fn test_proxy(proxy: &ValetProxy) -> anyhow::Result<u16>:
     Make HTTP GET to proxy.target using reqwest, return status code
     Add reqwest = { version = "0.12", features = ["json"] } to Cargo.toml

2. PROXIES PANEL — src/ui/panels/proxies.rs
   Table: Domain | Target | TLS | Status | Actions
   Status column: "Testing..." initially, then shows HTTP status code (200 green, others amber/red)
   Test all button: fires test_proxy for each proxy concurrently
   "Add Proxy" button: opens a modal with domain + target + TLS toggle inputs
   Delete button per row: calls remove proxy with confirmation dialog
   Right-click on proxy domain: "Copy URL" option

3. SITE ENV VARS EDITOR — src/valet/env_vars.rs
   Implement SiteEnvVar struct: site: String, key: String, value: String
   Implement async fn read_valet_env(site_path: &Path) -> anyhow::Result<Vec<SiteEnvVar>>:
     Read site_path/.valet-env.php, parse PHP array content
     Use regex to extract 'key' => 'value' pairs for the site name and '*'
   Implement async fn write_valet_env(site_path: &Path, vars: &[SiteEnvVar])
     -> anyhow::Result<()>:
     Render a valid PHP file: <?php return ['sitename' => ['KEY' => 'VALUE'], ...];
     Write the file
   Add to Sites panel context menu: "Edit env vars" → opens a panel/modal with
     a table of key/value pairs (TextEdit per cell), Add/Remove row buttons, Save button

4. CUSTOM DRIVER MANAGER — src/valet/driver_manager.rs
   Implement ValetDriver struct: filename: String, path: PathBuf, content: String,
     base_driver: Option<String>, serving_sites: Vec<String>
   Implement async fn list_drivers(paths: &ValetPaths) -> anyhow::Result<Vec<ValetDriver>>:
     Read PHP files from paths.drivers_dir
     Parse `extends {ClassName}` to extract base_driver
     Cross-reference with scanned sites to find which sites use each driver
   Implement fn create_sample() -> String: return the standard SampleValetDriver.php content

5. DRIVERS PANEL — src/ui/panels/drivers.rs
   Left list: driver names with base class shown below
   Right pane: egui TextEdit showing PHP content (monospace)
   "New Driver" button: creates new file from sample template, opens in editor
   "Delete" button: confirm dialog then fs::remove_file
   "Open in File Manager": xdg-open drivers_dir

6. DNSMASQ PANEL — src/ui/panels/dnsmasq.rs
   Read: current TLD from valet_config.tld
   Show: "address=/.{tld}/127.0.0.1" as the primary entry
   TLD change: text input + "Apply" button → runs `valet domain {new_tld}`
   DNS Tester:
     Input field: domain to test (pre-populated with first site domain)
     "Test" button: spawns `dig {domain} @127.0.0.1` via Command::new
     Output: show the ANSWER section from dig output in a monospace box
   Port override (valet-linux only): number input + "Apply" → `valet port {n}`
     Only show this section if ValetVariant == ValetLinux

7. SHARING PANEL — src/ui/panels/sharing.rs
   Sharing tool selector: radio buttons ngrok | Expose | Cloudflared
   "Set as default" button: runs `valet share-tool {tool}`
   ngrok section: token text input (masked), "Save Token" button: `valet set-ngrok-token {token}`
   Site selector: ComboBox from sites list
   "Share" button: spawn `valet share` in site directory, stream output to terminal widget
     Show the public URL when it appears in output (parse ngrok URL from stdout)
   "Stop Sharing" button (SIGTERM to share process)

8. LOGS PANEL — src/ui/panels/logs.rs
   Log source selector (tabs or ComboBox):
     Nginx Error, Nginx Access, PHP-FPM Error, Valet FPM log, valet log output
   For each source, define the file path:
     Nginx error: /var/log/nginx/error.log
     PHP-FPM: /var/log/php{active_ver}-fpm.log or /var/log/php-fpm.log
     Valet: {valet_paths.log_dir}/fpm-php.www.log
   Read last N lines: use `tail -n 500 {file}` via Command::new
   Display in scrollable monospace text area, auto-scroll to bottom
   "Refresh" button | "Clear display" button (clears the UI buffer, not the file)
   Color-code: lines containing "error" → red, "warning" → amber, "notice" → blue

9. DIAGNOSTICS PANEL — src/ui/panels/diagnostics.rs
   "Run Diagnostics" button: streams `valet diagnose` output
   Display output in terminal widget
   "Trust Valet" button: runs `valet trust`
   "Restart All Services" button: runs `valet restart` and streams output

10. WIRE UP ALL NEW COMMANDS in dispatcher

ACCEPTANCE CRITERIA:
- Proxies panel lists all proxy sites, test button shows HTTP status
- "Add Proxy" creates proxy and it appears immediately after refresh
- Site env vars dialog shows correct PHP key/value pairs and saves correctly
- dnsmasq TLD changer runs valet domain and refreshes the TLD in config
- DNS tester shows dig output inline
- Sharing panel spawns valet share and displays the ngrok URL when available
- Logs panel correctly identifies log file paths for the active PHP version
- All panels handle the case where the relevant file/directory does not exist
```

---

## Phase 7 — Polish & Release
**Weeks 19–21 · System tray, notifications, themes, onboarding, packaging, GitHub Actions**

```
You are implementing Phase 7 of Valet Manager. Phases 1–6 are complete.

This phase focuses on UX polish, packaging, and release infrastructure.

TASKS:

1. SYSTEM TRAY ICON — src/tray/tray_icon.rs
   Use the tray-icon crate to create a system tray icon.
   Load the 32×32 PNG icon from embedded bytes: include_bytes!("../../assets/icons/valet-manager-32.png")
   Build the tray menu dynamically from AppState:
     "PHP {active_version}" header (disabled item)
     Separator
     For each installed PHP version: "Switch to PHP X.X" → sends SwitchGlobalPhp command
     Separator
     "★ Favorites" submenu: list favorites, clicking opens browser
     Separator
     Services submenu: "Restart nginx", "Restart php-fpm", "Restart all"
     Separator
     "Open Valet Manager" → brings main window to front
     "Quit"
   Update tray tooltip to show active PHP version
   Rebuild menu on AppEvent::ServiceStatusUpdated and AppEvent::PhpVersionsRefreshed

2. DESKTOP NOTIFICATIONS — src/notifications.rs
   Use the notify-rust crate.
   Implement fn notify_php_switched(version: &str):
     notify-rust Notification with summary="PHP switched to {version}", timeout 4s
   Implement fn notify_service_failed(service: &str):
     Notification with summary="Service failed: {service}", urgency critical, timeout 10s
   Implement fn notify_app_creation_complete(domain: &str):
     Notification with summary="Site created: {domain}", action "Open in browser"
   Implement fn notify_update_available(version: &str):
     Notification with summary="Valet Manager {version} available"
   All notifications check config.notifications.* flags before firing

3. THEME SYSTEM — src/ui/theme.rs
   Implement ValetTheme struct with egui::Style fields
   Implement fn apply_dark(ctx: &egui::Context) → customize egui's dark visuals:
     Panel background: Color32::from_rgb(0x1E, 0x1E, 0x1E)
     Sidebar: Color32::from_rgb(0x16, 0x16, 0x16)
     Accent: Color32::from_rgb(0x5D, 0xCA, 0xA5)     // brand teal
     Widget bg: Color32::from_rgb(0x2A, 0x2A, 0x2A)
   Implement fn apply_light(ctx: &egui::Context):
     Panel bg: Color32::WHITE, Sidebar: Color32::from_rgb(0xF5, 0xF5, 0xF5)
     Accent: Color32::from_rgb(0x0F, 0x6E, 0x56)
   Implement fn apply_system(ctx: &egui::Context):
     Check GTK theme via gsettings or COLORFGBG env var → apply_dark or apply_light
   Apply selected theme in ValetManagerApp::update() on every frame if theme changed

4. FIRST-RUN ONBOARDING WIZARD — src/ui/panels/onboarding.rs
   Show this panel instead of Dashboard if valet_variant is None (Valet not found)
   Steps:
     Step 1: "Welcome" — app logo, description, "Let's get started" button
     Step 2: "Check PHP" — show installed PHP versions, prompt to install if none found
       Show: "Install PHP 8.3" button → runs `sudo apt install php8.3`
     Step 3: "Check Valet" — probe for valet binary
       Show install instructions based on distro if not found
       "I have installed Valet" → re-run detection
     Step 4: "Verify installation" — run `valet diagnose`, stream output
       "Continue" if exit 0, "Try again" if errors
     Step 5: "Ready!" — show detected Valet variant, PHP versions, TLD
       "Open Dashboard" → complete onboarding, save onboarding_complete=true to config
   Store onboarding_complete: bool in config. Skip wizard if true.

5. SETTINGS PANEL — src/ui/panels/settings.rs
   Section: Appearance
     Theme selector (Dark / Light / System)
     Font size slider (11–20)
   Section: Editor & Tools
     Editor command text input
     Terminal command text input
     File manager command text input
   Section: Notifications checkboxes (one per notification type)
   Section: App Creator defaults
     Default parent directory (path picker)
     Default DB user, default admin email
   Section: Danger Zone
     "Uninstall Valet" button → runs `valet uninstall`, confirm dialog
     "Reset All Settings" → delete config.toml, restart with defaults
   Save button bottom-right: writes config and applies theme immediately

6. GLOBAL KEYBOARD SHORTCUTS — src/app.rs
   In ValetManagerApp::update(), before rendering panels, check:
     egui::Key::K pressed with Ctrl modifier → send OpenCommandPalette (stub for Phase 9)
     egui::Key::R pressed with Ctrl modifier → send RefreshAll
     egui::Key::Comma pressed with Ctrl modifier → send OpenPanel(Settings)
   Show shortcuts in tooltips on relevant buttons

7. TOAST NOTIFICATION WIDGET — src/ui/components/toast.rs
   Implement Toast struct: message: String, toast_type: ToastType, created_at: Instant
   Implement ToastType enum: Success | Error | Info | Warning
   Implement fn render_toasts(ui: &mut egui::Ui, toasts: &mut Vec<Toast>):
     Render in bottom-right corner using egui Area with fixed anchor
     Each toast: colored left border + icon + message
     Auto-dismiss after 4s (remove from Vec in the same render call)
     Max 4 visible at once
   Show toasts on: PHP switch complete, nginx reload, site link/unlink, errors

8. DEBIAN PACKAGE — packaging/deb/
   Create packaging/deb/control:
     Package: valet-manager, Architecture: amd64, Depends: pkexec
   Create packaging/deb/postinst:
     mkdir -p /usr/lib/valet-manager
     cp {binary_dir}/valet-manager-helper /usr/lib/valet-manager/helper
     chmod 755 /usr/lib/valet-manager/helper
     cp packaging/polkit/com.valetmanager.policy /usr/share/polkit-1/actions/
     gtk-update-icon-cache (if gtk-update-icon-cache exists)
   Create packaging/deb/postrm: cleanup
   Add to Cargo.toml: [package.metadata.deb] section for cargo-deb
   Add to Cargo.toml: assets for icon, desktop file, polkit policy

9. DESKTOP FILE — packaging/valet-manager.desktop
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
   MimeType=x-scheme-handler/valet-manager;

10. GITHUB ACTIONS CI/CD — .github/workflows/release.yml
    Create workflow triggered on push to tags matching v*.*.*:
    jobs:
      build:
        strategy: matrix: [x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu]
        steps:
          - actions/checkout@v4
          - dtolnay/rust-toolchain@stable with target
          - cargo test --workspace
          - cargo clippy -- -D warnings
          - cargo build --release --target {matrix.target}
          - cargo install cargo-deb && cargo deb --target {matrix.target}
          - actions/upload-artifact@v4 with .deb file
      release:
        needs: build
        runs-on: ubuntu-latest
        steps:
          - Download all artifacts
          - Create GitHub Release using softprops/action-gh-release@v1
          - Upload .deb files as release assets

ACCEPTANCE CRITERIA:
- System tray icon appears after startup with correct active PHP version in tooltip
- PHP switch from tray updates main window immediately
- Desktop notifications fire correctly for each event type (test on real desktop)
- Theme switching applies immediately without restart
- Onboarding wizard shows if valet is not detected
- Toast messages appear and auto-dismiss after 4 seconds
- `cargo deb` produces a valid .deb package that installs correctly with dpkg -i
- .desktop file appears in application launcher after dpkg install
- GitHub Actions workflow runs on tag push and produces release artifacts
```

---

## Phase 8 — PHPMon Gap-Closing
**Weeks 22–25 · phpinfo() viewer, compat checker, command history, favorites, deep-link, updater, onboarding**

```
You are implementing Phase 8 of Valet Manager. Phases 1–7 are complete.

This phase closes all remaining gaps versus PHPMon.

TASKS:

1. PHPINFO() VIEWER — src/phpinfo/
   Create src/phpinfo/runner.rs:
     Implement async fn fetch(php_binary: &str) -> anyhow::Result<String>:
       Run `{php_binary} -r "phpinfo();"` → captures stdout as plain text
       phpinfo() in CLI mode outputs plain text, not HTML
   Create src/phpinfo/parser.rs:
     Implement PhpInfoSection struct: name: String, entries: Vec<PhpInfoEntry>
     Implement PhpInfoEntry struct: key: String, local_value: String, master_value: Option<String>
     Implement fn parse(output: &str) -> Vec<PhpInfoSection>:
       Split by blank lines to find sections
       First non-empty line of each section is the section name
       Subsequent lines: split on " => " → key, local_value, master_value
       Lines with only two parts have no master_value
   Create src/ui/panels/phpinfo.rs:
     Version selector at top (ComboBox of installed PHP versions)
     Search input: filter entries where key.contains(query) || local_value.contains(query)
     Left: section list (egui SelectableLabel), selected section highlighted
     Right: filtered table with columns Key | Local Value | Master Value
       "Copy" icon button on each row (sends value to clipboard)
     "Refresh" button: re-runs fetch for selected version
   Add to AppCommand: ViewPhpInfo(String), RefreshPhpInfo

2. PHP COMPATIBILITY CHECKER — src/compat/checker.rs
   Add semver = "1" to Cargo.toml
   Add futures = "0.3" if not already present
   Implement async fn check_site(site: &ValetSite) -> CompatibilityResult:
     Read {site.path}/composer.json if exists
     Parse require.php field (e.g. "^8.1", ">=8.0,<9.0", "~8.2")
     Convert to semver::VersionReq
     Get active PHP version for site (from php_version field or global active)
     Parse as semver::Version and test compatibility
     Return CompatibilityResult with status + suggestion if incompatible
   Implement async fn check_all(sites: &[ValetSite]) -> Vec<CompatibilityResult>:
     futures::future::join_all(sites.iter().map(check_site)).await
   Create src/ui/panels/compat.rs:
     "Check All" button at top: runs check_all, shows loading spinner during check
     Table: Site | Required | Active | Status | Action
     Status badges: Compatible (green) / Incompatible (red) / No Requirement (gray)
     Action column: "Isolate to PHP X.X" button when incompatible
       (determines suggestion from CompatibilityResult.suggestion)
     Last-checked timestamp shown bottom-right
   Dashboard integration: show "N sites have compatibility issues" alert banner
     when any site has Incompatible status

3. COMMAND HISTORY — src/history/
   Add rusqlite = { version = "0.32", features = ["bundled"] } to Cargo.toml
   Create src/history/audit_log.rs:
     Implement fn db_path() -> PathBuf: ~/.config/valet-manager/history.db
     Implement fn init_db() -> anyhow::Result<rusqlite::Connection>:
       CREATE TABLE IF NOT EXISTS command_history (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         command TEXT NOT NULL, args TEXT NOT NULL,
         working_dir TEXT, started_at TEXT NOT NULL,
         duration_ms INTEGER NOT NULL, exit_code INTEGER,
         triggered_by TEXT NOT NULL, output_preview TEXT
       )
     Implement fn insert(entry: &CommandHistoryEntry) -> anyhow::Result<()>
     Implement fn query(filter: &str, limit: usize) -> anyhow::Result<Vec<CommandHistoryEntry>>:
       SELECT ... WHERE command LIKE '%{filter}%' OR triggered_by LIKE '%{filter}%'
       ORDER BY started_at DESC LIMIT {limit}
     Implement fn clear() -> anyhow::Result<()>: DELETE FROM command_history
   Modify src/system/subprocess.rs run_command() to call audit_log::insert after every command
   Create src/ui/panels/history.rs:
     Search input at top
     Table: Time | Command + Args | Duration | Exit | Source
     Time: human-friendly "2 minutes ago" using chrono
     Row click: expand to show output_preview in a collapsible section
     "Re-run" button per row: dispatches the same command again
     "Export CSV" button: writes all entries to ~/Desktop/valet-manager-history.csv
     "Clear All" button with confirmation dialog

4. FAVORITE DOMAINS — update Sites panel and config
   In AppConfig add: favorites: Vec<String> = vec![]
   In config save/load: serialize/deserialize favorites
   In src/ui/panels/sites.rs:
     Sort sites: favorites first (stable sort: is_favorite DESC, then name ASC)
     Star column: egui Button with ★ label, colored gold when is_favorite=true
     Button click: sends AppCommand::ToggleFavoriteSite(site.name.clone())
   In dispatcher: ToggleFavoriteSite → add/remove from config.favorites, save config,
     re-sort sites state, send event to refresh UI
   In tray menu: add "Favorites" submenu section listing favorite domains

5. DEEP-LINK PROTOCOL — src/protocol/handler.rs
   Register in .desktop file: MimeType=x-scheme-handler/valet-manager;
   In postinst script: xdg-mime default valet-manager.desktop x-scheme-handler/valet-manager
   Implement fn parse_url(url: &str) -> Option<AppCommand>:
     Parse valet-manager://{action}/{params...}
     "switch-php/{version}" → SwitchGlobalPhp(version)
     "open-site/{name}" → OpenSiteInBrowser(name)
     "open-site/{name}/editor" → OpenSiteInEditor(name)
     "restart-service/{service}" → RestartService(service)
     "run-artisan/{site}/{command}" → RunArtisanCommand { site, command, args: vec![] }
     "toggle-xdebug/{ver}/{mode}" → EnableXdebug { php_version, mode }
     "navigate/{panel}" → OpenPanel(parse_panel(panel))
     "open-palette" → OpenCommandPalette
   In src/main.rs: check command-line args on startup:
     If args[1] starts with "valet-manager://", call parse_url and dispatch command
   Create integrations/raycast/README.md documenting the URL scheme for Raycast users

6. BUILT-IN UPDATER — src/updater/
   Create src/updater/checker.rs:
     Implement async fn check() -> anyhow::Result<UpdateInfo>:
       GET https://api.github.com/repos/mralaminahamed/valet-manager/releases/latest
         with header User-Agent: valet-manager/{version}
       Parse tag_name, body (release notes), assets array
       Compare semver: current = env!("CARGO_PKG_VERSION"), latest = tag_name
       Return UpdateInfo { update_available, latest_version, release_notes, download_url }
     Implement fn pick_asset_url(assets: &serde_json::Value) -> Option<String>:
       Find asset ending in .deb for x86_64 target, or .AppImage
   Create src/updater/installer.rs:
     Implement async fn download_and_install(url: &str, tx: mpsc::Sender<OutputLine>)
       Download to /tmp/valet-manager-update.deb via reqwest with progress streaming
       Call privilege helper with { "op": "dpkg_install", "path": "/tmp/valet-manager-update.deb" }
       On success: show "Restart to apply update" toast
   Dashboard banner: if update_available, show non-modal banner:
     "Valet Manager v{latest} is available  [Update Now]  [Skip]"
     "Skip" saves skipped_version to config and hides banner
   Check for updates: on startup if last_checked > 24h ago, and on Settings page manually

ACCEPTANCE CRITERIA:
- phpinfo() panel shows all sections for the selected PHP version, searchable
- Compatibility panel shows results for all sites after "Check All"
- Incompatible sites show a suggestion and an "Isolate" button that works
- Command history shows all commands run during the session with timing
- "Re-run" button dispatches the command again correctly
- Favorite toggle persists across restarts
- Favorites appear at top of sites list
- valet-manager:// URL opens the app and dispatches the correct command
- Update check makes the API call and shows the banner if a new version exists
```

---

## Phase 9 — DX Tier 1
**Weeks 26–30 · Command palette, .env editor, Artisan runner, Database manager, Dashboard alerts**

```
You are implementing Phase 9 of Valet Manager. Phases 1–8 are complete.

This phase adds the most impactful developer workflow features.

TASKS:

1. COMMAND PALETTE — src/ui/command_palette.rs
   Add fuzzy-matcher = "0.3" to Cargo.toml
   Implement CommandPaletteState (add to AppState):
     visible: bool, query: String,
     results: Vec<PaletteResult>, selected_index: usize
   Implement PaletteResult: label, subtitle, category, action: AppCommand, shortcut: Option<String>
   Implement PaletteCategory enum: PhpVersion | Site | Service | Artisan | Panel | Action
   Implement fn build_index(state: &AppState) -> Vec<PaletteResult>:
     PHP versions: "Switch to PHP {ver}" → SwitchGlobalPhp
     Sites (first 20): "Open {domain}", "Open {domain} in editor"
     Services: "Restart {service}", "Stop {service}"
     Panels: "Go to {panel_name}" → OpenPanel
     Actions: "Create new app", "Reload nginx", "Run diagnostics", "Check for updates"
     Artisan (if selected site is Laravel): top 10 common commands
   Implement fn filter_results(index: &[PaletteResult], query: &str) -> Vec<PaletteResult>:
     Use fuzzy_matcher::SkimMatcherV2::fuzzy_match to score each result
     Sort by score descending, take first 8
   Render in update() as egui::Window with:
     id: egui::Id::new("command_palette"), title_bar: false
     Fixed width 520px, centered in viewport
     Search input (always focused when visible)
     Results list: up to 8 rows, each showing category badge + label + subtitle + shortcut
     Selected row highlighted (egui::Stroke on the row background)
     Keyboard: ArrowUp/Down moves selection, Enter executes, Escape closes
   Open/close: Ctrl+K in update() toggles palette.visible
   On Enter: dispatch palette.results[palette.selected_index].action.clone()

2. ENV FILE EDITOR — src/env_editor/
   Create src/env_editor/parser.rs:
     Implement fn parse(content: &str) -> Vec<EnvEntry>:
       Process line by line, preserving order
       Skip blank lines and full-line comments
       Split on first `=`: key is left, value is right (handle quoted values)
       Detect is_secret: key ends with _KEY, _SECRET, _PASSWORD, _TOKEN, _PASS,
         or contains PRIVATE, CERTIFICATE
     Implement fn render(entries: &[EnvEntry]) -> String:
       Reconstruct file preserving blank lines between groups
       For comments: include as-is
   Create src/env_editor/validator.rs:
     Implement fn validate(entries: &[EnvEntry], example_path: &Path)
       -> Vec<EnvValidationIssue>:
       Read .env.example if it exists, parse keys
       Missing from .env: keys in example not in entries
       Extra in .env: keys in entries not in example (warn only)
   Create src/env_editor/writer.rs:
     Implement async fn save_atomic(path: &Path, content: &str) -> anyhow::Result<()>:
       Write to {path}.tmp then fs::rename to path
   Create src/ui/panels/env_editor.rs:
     Site selector ComboBox at top
     "Show secrets" toggle
     Group tabs: APP | DB | MAIL | REDIS | Other (group by key prefix)
     Table: Key | Value | ⋮
     Value column: TextEdit (shows ●●●●●● if is_secret and !show_secrets)
     ⋮ context menu: "Copy value", "Reset to example default"
     Validation issues shown as colored banners above affected keys:
       Red: MissingFromEnv, Amber: MissingFromExample
     "Save" button (disabled when no changes), "Revert" button
     Raw edit toggle: shows full TextEdit multiline of the file

3. ARTISAN RUNNER — src/artisan/
   Create src/artisan/discover.rs:
     Implement async fn discover(site_path: &Path, php_bin: &str)
       -> anyhow::Result<Vec<ArtisanCommand>>:
       Run `{php_bin} artisan list --format=json` in site_path
       Parse JSON: commands[].name, commands[].description, commands[].synopsis
       Group by first segment of name before ":"
   Create src/artisan/runner.rs:
     Implement async fn run(
       site_path: &Path, php_bin: &str, command: &str, args: &[&str],
       tx: mpsc::Sender<OutputLine>
     ) -> anyhow::Result<ExitStatus>:
       stream_command(php_bin, &["artisan", command, ...args], Some(site_path), tx)
   Create src/artisan/history.rs:
     SQLite table artisan_history: id, site, command, args, ran_at, exit_code, duration_ms
     fn insert(site: &str, entry: &ArtisanHistoryEntry) -> anyhow::Result<()>
     fn query_site(site: &str, limit: usize) -> anyhow::Result<Vec<ArtisanHistoryEntry>>
   Create src/ui/panels/artisan.rs:
     Site selector at top with framework badge (only show artisan for Laravel sites)
     Command input: TextEdit with autocomplete dropdown from discovered commands
       Filter commands as user types using contains() match
       Arrow keys + Enter to select from dropdown
     Args input: TextEdit for additional arguments
     Quick commands row: configurable buttons from config.artisan.quick_commands per site
       "+" button to add current command to quick list
     "▶ Run" button: disabled while running
     Output: terminal_output::render()
     History section below output: last 10 commands for this site as clickable rows
     "Re-run" on history rows: fills in the command+args inputs

4. DATABASE MANAGER — src/database/
   Create src/database/detector.rs:
     Implement async fn detect_mysql(host: &str, port: u16, user: &str, pass: &str)
       -> anyhow::Result<()>: run `mysql -h{host} -P{port} -u{user} -p{pass} -e "SELECT 1"`
     Implement async fn list_databases_mysql(creds: &MysqlCreds) -> anyhow::Result<Vec<String>>:
       run mysql -e "SHOW DATABASES" and parse output lines (skip system databases)
     Implement async fn list_tables_mysql(creds: &MysqlCreds, db: &str)
       -> anyhow::Result<Vec<TableInfo>>:
       run mysql -e "SELECT TABLE_NAME, TABLE_ROWS FROM information_schema.TABLES
         WHERE TABLE_SCHEMA='{db}'" and parse
   Create src/database/migrator.rs:
     Implement async fn migrate(site_path: &Path, php_bin: &str, fresh: bool, seed: bool,
       tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
       Build artisan command: migrate [--fresh] [--seed] --force
       stream_command(php_bin, &["artisan", ...args], Some(site_path), tx)
     Implement async fn rollback(site_path: &Path, php_bin: &str, steps: Option<u32>,
       tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
       stream_command with migrate:rollback [--step=N]
     Implement async fn seed(site_path: &Path, php_bin: &str,
       tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
       stream_command with db:seed --force
   Create src/ui/panels/database.rs:
     Top: Engine selector (MySQL / PostgreSQL / SQLite), connection status badge
     Site selector: ComboBox — read DB_* vars from site .env on selection
     Left panel: database list. "+" button to create, trash to drop (confirm)
     Right panel: table list for selected database with row counts
     Laravel actions bar (only shown for Laravel sites):
       [Migrate]  [Migrate Fresh]  [Migrate Fresh + Seed]  [Seed]  [Rollback]
     Output area: terminal_output::render() for migration output

5. DASHBOARD ALERT BANNERS — update src/ui/panels/dashboard.rs
   Add fn render_alert_banners(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>)
   called at the top of dashboard render, before service grid.
   Banner types (each is a horizontal strip with left colored border):
     SSL expiry (red): "{N} sites have SSL certs expiring soon"  [View →]
       Condition: any SslCertInfo with days_remaining < 14
     Compat issues (amber): "{N} sites have PHP compatibility issues"  [View →]
       Condition: any CompatibilityResult with status == Incompatible
     Update available (blue): "Version {ver} available"  [Update]  [Skip]
       Condition: updater.update_available
     Diagnostics error (red): "Valet issue detected: run diagnostics"  [Fix →]
       Condition: last_diagnose_had_errors (bool in state)
   [View →] buttons send OpenPanel to the relevant panel

ACCEPTANCE CRITERIA:
- Command palette opens with Ctrl+K, closes with Escape
- Fuzzy search works: typing "migr" shows "Run migrations" and artisan migrate commands
- Arrow keys navigate results, Enter dispatches the selected action
- .env editor loads and saves correctly with atomic write
- Secrets masked by default, toggle shows them
- Validation issues highlighted correctly by comparing with .env.example
- Artisan runner discovers commands for a real Laravel site
- Autocomplete dropdown appears and is navigable by keyboard
- Database manager connects to MySQL and lists databases correctly
- Migration output streams in real-time in the output widget
- Dashboard banners appear for relevant conditions and link to correct panels
```

---

## Phase 10 — DX Tier 2
**Weeks 31–34 · SSL dashboard, Xdebug toggle, Mail catcher, Queue workers**

```
You are implementing Phase 10 of Valet Manager. Phases 1–9 are complete.

This is the final development phase.

TASKS:

1. SSL CERTIFICATE DASHBOARD — src/ssl/
   Create src/ssl/cert_reader.rs:
     Implement async fn read_cert(cert_path: &Path) -> anyhow::Result<SslCertInfo>:
       Run `openssl x509 -in {cert_path} -noout -text -dates -subject`
       Parse output:
         Not Before: ... → parse with chrono
         Not After: ...  → parse with chrono
         Subject: CN=domain.test → extract domain
         Subject Alternative Name block → extract SANs
       Calculate days_remaining = (not_after - Utc::now()).num_days()
       Assign expiry_status:
         > 30 days → Ok
         7-30 days → Warning
         0-7 days → Critical
         < 0 → Expired
   Create src/ssl/expiry_monitor.rs:
     Implement async fn scan_all_certs(paths: &ValetPaths)
       -> anyhow::Result<Vec<SslCertInfo>>:
       List all .crt files in paths.ca_dir/
       Concurrently read each with read_cert
   Background task: spawn at startup, poll every 6 hours, send notification
     if any cert goes from Ok to Warning or Critical since last check
   Create src/ui/panels/ssl_certs.rs:
     Header: "Valet CA: {installed/not installed}" badge with "Install CA" button
     Table: Domain | Issuer | Expires | Days | Status
     Status badge: green/amber/red background based on ExpiryStatus
     Row actions: "Regenerate" → runs `valet secure {domain}`
                  "Export PEM" → copies cert file to ~/Downloads/{domain}.crt
     "Re-check all" button at top-right
     Certs sorted: Critical first, then Warning, then Ok, then Expired

2. XDEBUG QUICK TOGGLE — src/xdebug/
   Create src/xdebug/toggle.rs:
     Implement fn xdebug_ini_path(php_version: &str) -> PathBuf:
       /etc/php/{php_version}/cli/conf.d/99-xdebug-valet-manager.ini
     Implement async fn get_status(php_version: &str) -> XdebugPhpStatus:
       Check if xdebug extension .so exists in php extension dir
       Check if 99-xdebug-valet-manager.ini exists and contains zend_extension
       Parse xdebug.mode from the ini file
     Implement async fn enable(php_version: &str, mode: XdebugMode, ide_key: &str,
       client_host: &str, client_port: u16) -> anyhow::Result<()>:
       Render ini content:
         zend_extension=xdebug
         xdebug.mode={mode}
         xdebug.client_host={client_host}
         xdebug.client_port={client_port}
         xdebug.idekey={ide_key}
         xdebug.start_with_request=yes
       Write via privilege helper to xdebug_ini_path
       Restart php{version}-fpm via privilege helper
     Implement async fn disable(php_version: &str) -> anyhow::Result<()>:
       Delete 99-xdebug-valet-manager.ini via privilege helper (if exists)
       Restart php{version}-fpm
   Implement XdebugMode enum: Off | Debug | Profile | Coverage | Trace | DevelopDebug
     Implement fn as_str(&self) → &str for each variant
   Create src/ui/panels/xdebug.rs:
     One card per installed PHP version:
       Version + "Xdebug X.X.X installed" or "Not installed" badge
       Mode radio buttons: Off | Debug | Profile | Coverage
       IDE Key ComboBox: VSCODE | PHPSTORM | NETBEANS | Custom
         Custom: text input appears
       Client host + port inputs (default 127.0.0.1:9003)
       Enable/Disable button based on current status
     "Refresh" button top-right to re-check all versions

3. MAIL CATCHER CONTROL — src/mail_catcher/
   Create src/mail_catcher/mailpit.rs:
     Implement async fn is_installed() -> bool: which("mailpit").is_ok()
     Implement async fn install(tx: mpsc::Sender<OutputLine>) -> anyhow::Result<()>:
       Download from GitHub releases using reqwest:
         GET https://api.github.com/repos/axllent/mailpit/releases/latest → asset URL
       Stream download to /tmp/mailpit, chmod +x, move to ~/.local/bin/mailpit
       Or: stream_command("bash", &["-c",
         "curl -sL https://raw.githubusercontent.com/axllent/mailpit/develop/install.sh | sudo bash"],
         None, tx)
     Implement async fn start(smtp_port: u16, http_port: u16) -> anyhow::Result<u32>:
       Spawn mailpit process: --smtp 127.0.0.1:{smtp_port} --listen 127.0.0.1:{http_port}
       Write PID to /tmp/valet-manager-mailpit.pid
       Return PID
     Implement async fn stop(pid: u32) -> anyhow::Result<()>:
       nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
       Remove /tmp/valet-manager-mailpit.pid
     Implement async fn get_unread(http_port: u16) -> anyhow::Result<u32>:
       GET http://localhost:{http_port}/api/v1/messages
       Parse JSON: total field
     Implement async fn clear_messages(http_port: u16) -> anyhow::Result<()>:
       DELETE http://localhost:{http_port}/api/v1/messages
   Create src/mail_catcher/smtp_config.rs:
     Implement async fn configure_site(site_path: &Path, smtp_port: u16)
       -> anyhow::Result<()>:
       Load site .env, update or add these keys:
         MAIL_MAILER=smtp, MAIL_HOST=localhost,
         MAIL_PORT={smtp_port}, MAIL_ENCRYPTION=null,
         MAIL_FROM_ADDRESS=local@localhost, MAIL_FROM_NAME="${APP_NAME}"
       Save with atomic write
   Create src/ui/panels/mail_catcher.rs:
     Status section: tool name, version, SMTP port, HTTP port, running status
     If not installed: "Install Mailpit" button with streaming install output
     Control buttons: "Start" / "Stop", "Open Web UI" (xdg-open)
     Unread count: shown prominently, "Clear Messages" button
     Auto-refresh unread count every 10 seconds when running
     SMTP config section: site ComboBox + "Apply to .env" button
   Tray integration: if mail running, add "Mail: {N} unread" to tray menu
   Start mailpit poll task in main.rs if auto_start = true in config

4. QUEUE WORKER MANAGER — src/queue/
   Create src/queue/worker_manager.rs:
     Implement fn service_name(site: &str, queue: &str) -> String:
       format!("valet-queue-{site}-{queue}.service")
     Implement fn service_path(site: &str, queue: &str) -> PathBuf:
       dirs::config_dir().unwrap().join("systemd/user").join(service_name(site, queue))
     Implement async fn create_service(site: &ValetSite, connection: &str, queue: &str,
       php_bin: &str) -> anyhow::Result<()>:
       Render template (hardcoded string, no external file needed):
         [Unit] Description=Queue Worker — {site} ({queue})
         After=network.target
         [Service] Type=simple
         WorkingDirectory={site.path}
         ExecStart={php_bin} artisan queue:work {connection} --queue={queue} --sleep=3 --tries=3 --max-time=3600
         Restart=on-failure, RestartSec=5s
         StandardOutput=journal, StandardError=journal
         [Install] WantedBy=default.target
       Write to service_path
       Run: systemctl --user daemon-reload
     Implement async fn start(site: &str, queue: &str) -> anyhow::Result<()>:
       systemctl --user start {service_name}
     Implement async fn stop(site: &str, queue: &str) -> anyhow::Result<()>:
       systemctl --user stop {service_name}
     Implement async fn enable(site: &str, queue: &str) -> anyhow::Result<()>:
       systemctl --user enable {service_name}
     Implement async fn list_workers() -> anyhow::Result<Vec<QueueWorker>>:
       systemctl --user list-units "valet-queue-*.service" --no-legend --output json
       Parse to get service names and states
   Create src/queue/stats.rs:
     Implement async fn redis_queue_length(queue: &str) -> Option<u64>:
       Run `redis-cli LLEN queues:{queue}` if redis-cli exists, parse output
     Implement async fn db_failed_jobs(site_path: &Path, php_bin: &str) -> Option<u64>:
       Run `{php_bin} artisan queue:failed --count` and parse output
   Create src/ui/panels/queue.rs:
     Site selector (only Laravel sites shown)
     "Add Worker" button: opens form with connection, queue name, "Start on boot" toggle
     Worker list:
       Each row: site, queue name, connection, status dot, processed/failed counts
       Actions: Start/Stop toggle, Enable/Disable on boot, Delete service file
     Poll worker status every 10 seconds
     Stats: show Redis LLEN and failed_jobs count when available

5. FINAL INTEGRATION PASS
   Ensure all 10 panels added in Phases 8–10 appear correctly in the sidebar:
   - phpinfo() → MANAGEMENT section, below PHP INI
   - PHP Compatibility → MANAGEMENT section, below phpinfo()
   - SSL Certificates → MANAGEMENT section, below dnsmasq
   - Database Manager → DEVELOPMENT section (new section in sidebar)
   - .env Editor → DEVELOPMENT section
   - Artisan Runner → DEVELOPMENT section
   - Queue Workers → DEVELOPMENT section
   - Xdebug → DEVELOPMENT section
   - Mail Catcher → DEVELOPMENT section
   - Command History → TOOLS section
   Verify that Ctrl+K command palette includes entries for all new panels and actions.
   Verify that dashboard banners include SSL expiry and compatibility checks.

6. COMPREHENSIVE TESTS — tests/
   Write integration tests covering:
   - ssl cert reader: mock openssl output, verify parsed fields and days_remaining
   - xdebug toggle: verify ini file content rendered correctly for each mode
   - mail_catcher::smtp_config: test that correct .env keys are written
   - queue service template: verify rendered systemd unit file is valid
   - command palette filter: verify fuzzy matching ranks correct results first
   - env parser: round-trip test (parse then render produces equivalent output)

ACCEPTANCE CRITERIA:
- SSL panel shows all secured sites with correct expiry status and color coding
- "Regenerate" runs valet secure and refreshes the cert info
- Xdebug cards appear for all installed PHP versions
- Enable Xdebug writes the correct ini file and restarts FPM
- Mode change updates the ini file and restarts FPM
- Mail catcher start/stop works, unread count updates every 10 seconds
- "Apply to .env" writes correct MAIL_* keys to the selected site
- Queue worker service file is valid systemd unit (validate with systemd-analyze verify)
- Workers start/stop correctly via systemctl --user
- All 26 panels navigate correctly from sidebar and command palette
- cargo test passes all 40+ integration tests
- cargo clippy -- -D warnings produces zero warnings
- The complete app builds and runs on Ubuntu 22.04 LTS
```

---

## Phase Completion Checklist

After each phase, run these commands before committing:

```bash
# 1. Build verification
cargo build --workspace 2>&1 | grep -E "^error"

# 2. Lint (zero warnings policy)
cargo clippy --workspace -- -D warnings

# 3. Tests
cargo test --workspace -- --test-output immediate

# 4. Format check
cargo fmt --all -- --check

# 5. Integration smoke test (if Valet installed)
cargo run -- --test-mode 2>&1 | head -20
```

Commit message convention per phase:
```
feat(phase-N): <description>

- Implements: <list key modules>
- Tests: <what is covered>
- Breaking: <any API changes> (none if not applicable)
```

---

## Notes for Claude Code

- Always read existing files before editing them — never overwrite without reading first.
- All `unwrap()` calls in non-test code must be replaced with `?` or `expect("reason")`.
- Every `async fn` that calls subprocess must add an entry to the command history.
- The privilege helper binary must validate all inputs before acting — treat it as a security boundary.
- egui is immediate-mode: do not store `egui::Response` or `egui::Ui` across frames.
- Use `Arc<tokio::sync::RwLock<AppState>>` everywhere state is shared between threads.
- When in doubt about a Valet command, check the variant first — not all commands exist in all forks.

*Author: Al Amin Ahamed (@mralaminahamed)*
