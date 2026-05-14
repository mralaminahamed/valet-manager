# Valet Manager — Claude Code Prompts
## All 14 Phases · Design-First · Copy-Paste Ready

---

## How to use these prompts

1. Open your project: `cd ~/projects/valet-manager && claude`
2. For **Phase 1**: paste Step 0 first, wait for Claude Code to fetch the design,
   then paste the full Phase 1 block.
3. For **Phases 2–14**: each phase already contains the design fetch at the top —
   paste the whole block at once.
4. After every phase: `cargo build --workspace && cargo clippy -- -D warnings`

**The design file is the visual specification.** Every panel Claude Code
implements must match it. The sections below describe the logic and
acceptance criteria — the design HTML describes exactly what it should look like.

---

## Design fetch command (run once at the start of Phase 1)

```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

Then paste the Phase 1 block below.

---

## Global design contract

Every `src/ui/` file must follow these rules. No exceptions.

```
COLOURS — use Colors::* from src/ui/theme.rs only. No from_rgb() in panels.
  Colors::DEEP_BG    #0E1613   sidebar, titlebar, terminal bg
  Colors::SURFACE    #162019   CentralPanel fill
  Colors::CARD       #1C2A26   cards, inputs, table bg
  Colors::CARD_HOVER #20302B   hover state
  Colors::BORDER     rgba(255,255,255,0.07)
  Colors::BORDER_MED rgba(255,255,255,0.11)
  Colors::TEXT_PRIMARY   #D8EDE6
  Colors::TEXT_SECONDARY #6E9488
  Colors::TEXT_TERTIARY  #3A5550
  Colors::ACCENT         #5DCAA5
  Colors::ACCENT_DARK    #0F6E56
  Colors::ACCENT_DEEP    #04342C
  Colors::DANGER         #E24B4A
  Colors::WARNING        #EF9F27
  Colors::INFO           #378ADD

LAYOUT
  Sidebar: 196px fixed, not resizable
  Panel header: 44px with border-bottom
  Table row: 32px
  Card padding: 12px 14px, rounding 8px
  Nav item: 28px tall, 16px icon, 8px gap
  Status dot: 7px circle
  All borders: Stroke::new(0.5, ...) — never 1px

VERIFICATION after every phase:
  grep -rn "from_rgb\|from_rgba" src/ui/panels/ src/ui/sidebar.rs  → 0 results
  grep -rn "Stroke::new(1\." src/ui/                               → 0 results
  cargo clippy --workspace -- -D warnings                           → 0 warnings
```

---

## Phase 1 — Foundation + Design System
**Weeks 1–3**

```
You are building Phase 1 of Valet Manager from scratch.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN (before writing any code)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: full app shell, sidebar navigation, dashboard panel.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART A — RUST FOUNDATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. CARGO WORKSPACE
   Root Cargo.toml with members: ["valet-manager", "valet-manager-helper"]
   valet-manager/Cargo.toml — use these exact versions:

   eframe      = { version = "0.34.2", features = ["default_fonts", "wgpu"] }
   egui        = "0.34.2"
   egui_extras = { version = "0.34.2", features = ["all_loaders"] }
   tokio       = { version = "1.44", features = ["full"] }
   serde       = { version = "1.219", features = ["derive"] }
   serde_json  = "1"
   toml        = "0.8"
   toml_edit   = "0.22"
   anyhow      = "1"
   thiserror   = "2"
   tracing     = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   dirs        = "6"
   which       = "7"
   chrono      = { version = "0.4.41", features = ["serde"] }
   notify      = "8"
   tray-icon   = "0.21.1"
   notify-rust = "4"
   nix         = { version = "0.29", features = ["process", "signal", "fs"] }
   sysinfo     = "0.33"
   regex       = "1"
   futures     = "0.3"
   fuzzy-matcher = "0.3.7"
   reqwest     = { version = "0.12.15", features = ["json"] }
   rusqlite    = { version = "0.34", features = ["bundled"] }
   handlebars  = "6.3"
   rand        = { version = "0.9", features = ["std"] }
   bcrypt      = "0.15.1"
   rfd         = "0.15"
   semver      = "1.0.25"
   flate2      = "1.1"
   tar         = "0.4.44"

   NOTE — egui 0.34 API: use App::ui(ui: &mut Ui) NOT the deprecated App::update().
   Ui now derefs to Context, so ui.input(|i| ...) replaces ui.ctx().input(|i| ...).

2. VALET VARIANT DETECTION — src/valet/variant.rs
   ValetVariant: ValetLinux | ValetOfficial | ValetLinuxPlus
   ValetPaths: config_root, nginx_dir, sites_dir, drivers_dir, log_dir, ca_dir (PathBuf)
   async fn detect_valet_variant() — probe composer global show then config file paths

3. DISTRO & PACKAGE MANAGER — src/system/distro.rs
   DistroKind: Ubuntu | Debian | Fedora | Arch | Unknown  (from /etc/os-release)
   PackageManager: Apt | Dnf | Pacman
   async fn list_installed_php_packages(pm) -> Vec<String>

4. PHP DETECTOR — src/php/detector.rs
   PhpVersion: version, full_version, binary_path, fpm_service, cli_ini_path,
               fpm_ini_path, conf_d_path, is_active, fpm_running
   async fn detect_installed_versions() -> Vec<PhpVersion>
     Scan /usr/bin/ for php\d+\.\d+ files, merge with update-alternatives --list php
   async fn detect_active_version() -> String   (php --version)

5. SERVICE MONITOR — src/services/monitor.rs
   ServiceStatus: Running | Stopped | Failed | Unknown
   ManagedService: name, display_name, status, pid: Option<u32>
   async fn query_service_status(name) via systemctl is-active
   async fn poll_services(services, tx: Sender<Vec<ManagedService>>)  loop every 5s

6. APP STATE — src/state/app_state.rs
   AppState { valet_variant, valet_paths, php_versions, active_php,
              services, ui: UiState }
   UiState { active_panel: Panel, loading, toast_queue, last_error }
   Panel enum (24 variants): Dashboard, PhpVersions, PhpExtensions, PhpIni, PhpInfo,
     PhpCompat, Sites, Parks, Nginx, Proxies, Dnsmasq, SslCerts, Database, EnvEditor,
     Artisan, QueueWorkers, Xdebug, MailCatcher, Sharing, Drivers, Logs, History,
     Diagnostics, Settings

7. CONFIG — src/config.rs
   AppConfig (serde, Default):
     editor_command: "code", terminal: "gnome-terminal", file_manager: "nautilus"
     theme: "dark", font_size: 14.0, service_poll_interval_secs: 5
     site_scan_debounce_ms: 500, default_parent_directory: "~/Sites"
     favorites: Vec<String>
     version_registry_ttl_hours: 24, version_registry_auto_refresh: true
   fn config_path() -> PathBuf: ~/.config/valet-manager/config.toml
   fn load() + fn save()

8. COMMANDS & EVENTS — src/commands.rs + src/events.rs
   AppCommand: RefreshAll, RefreshServiceStatus, SwitchGlobalPhp(String),
               OpenPanel(Panel), OpenCommandPalette
   AppEvent:   ServiceStatusUpdated(Vec<ManagedService>),
               PhpVersionsRefreshed(Vec<PhpVersion>),
               ValetDetected(ValetVariant, ValetPaths), Error(String)

9. DISPATCHER — spawn tokio task in src/main.rs
   async fn run_dispatcher(rx, tx, state: Arc<RwLock<AppState>>)
   Spawn poll_services as long-running background task

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART B — DESIGN IMPLEMENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Using the design file you fetched in Step 0, implement:

10. THEME ENGINE — src/ui/theme.rs
    All Colors::* constants as Color32 from the design file's colour palette.
    apply_dark(ctx) sets every egui::Visuals field to match the design.
    Helper fns: status_color, framework_badge_colors, framework_display_name,
    accent_button, ghost_button, danger_button, status_dot, framework_badge,
    section_label, divider, card_frame.

11. SIDEBAR — src/ui/sidebar.rs
    pub fn render(ui, state, cmd_tx)
    Match the sidebar exactly as shown in the design: logo area, nav sections
    (management, development, tools), service status at bottom.
    Active item: Colors::ACCENT_DARK fill, white text.
    Inactive: transparent, Colors::TEXT_SECONDARY.

12. WINDOW SETUP — src/main.rs + src/app.rs
    NativeOptions: inner_size [1280,800], min_size [900,560], dark theme, wgpu.
    ValetManagerApp::new: call theme::apply_dark(ctx).
    ValetManagerApp::ui (egui 0.34 API — not update):
      TopBottomPanel::top "titlebar" 36px: traffic-light dots, app name, PHP version
      SidePanel::left "sidebar" 196px fixed: call sidebar::render
      CentralPanel: Colors::SURFACE fill, route Panel enum to panel render fns
    Keyboard: Ctrl+K → OpenCommandPalette, Ctrl+R → RefreshAll, Ctrl+, → Settings

13. DASHBOARD PANEL — src/ui/panels/dashboard.rs
    Implement the dashboard exactly as shown in the design:
    header (44px), alert banners, stats row (4 cards), service grid (2 col).
    Alert banners: only show when condition is true. Types: Danger, Warning, Info.
    Stats: active PHP (Colors::ACCENT), sites count, services N/total, TLD.
    Service cards: status dot + name + pid/status, restart/open button.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ cargo build --workspace — zero errors
  □ cargo clippy -- -D warnings — zero warnings
  □ App opens: dark teal window matches the design
  □ Sidebar renders with V logo mark and all nav sections
  □ Service dots update every 5 seconds from real systemctl
  □ Active PHP shown in title bar
  □ No from_rgb() outside theme.rs
  □ All borders are 0.5px Stroke
```

---

## Phase 2 — PHP Management
**Weeks 4–6**

```
Phase 1 complete. Implement PHP management backend and PHP panels UI.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: PHP Versions panel, PHP Extensions panel.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART A — ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read: src/state/app_state.rs, src/commands.rs, src/php/detector.rs

1. PRIVILEGE HELPER — valet-manager-helper/src/main.rs
   Accept JSON ops via stdin: systemctl start/stop/restart/reload,
   write_file (Nginx configs), shell_script (apt installs).
   Validate all inputs strictly. Never run arbitrary commands.
   packaging/polkit/com.valetmanager.policy: auth_admin_keep for the helper binary.

2. PHP SWITCHER — src/php/switcher.rs
   async fn switch_global(version, valet_paths) — runs: valet use php@{ver}
     Fallback: update-alternatives --set php /usr/bin/php{ver}
     Then restart php{ver}-fpm via privilege helper.
   async fn isolate_site(site_path, version) — writes .valetrc + valet isolate
   async fn unisolate_site(site_path) — removes .valetrc + valet unisolate

3. PHP-FPM MANAGER — src/php/fpm_manager.rs
   async fn start/stop/restart(version) via privilege helper systemctl.
   async fn status(version) -> ServiceStatus via systemctl is-active.

4. EXTENSION MANAGER — src/php/extension_manager.rs
   PhpExtension: name, version, ext_type (Core|Bundled|Pecl), enabled, ini_path
   async fn list(version) -> Vec<PhpExtension>
     Parse ls /etc/php/{ver}/mods-available/*.ini
     Check /etc/php/{ver}/cli/conf.d/ for symlinks → enabled
   async fn enable/disable(version, name) via privilege helper
     (symlink/unlink in conf.d/, restart FPM)

5. INI MANAGER — src/php/ini_manager.rs
   IniSection: name, entries: Vec<IniEntry>
   IniEntry: key, value, raw_line, comment, validation_error: Option<String>
   fn parse(content) -> Vec<IniSection>
   fn render(sections) -> String
   fn validate_value(key, value) -> Option<String>
     memory_limit/upload: must match ^\d+(K|M|G)$ or -1
     max_execution_time/max_input_vars: non-negative integer

6. NEW AppCommand variants:
   SwitchGlobalPhp(String), IsolateSite{site,version}, UnisolateSite(String),
   RestartPhpFpm(String), EnableExtension{version,name},
   DisableExtension{version,name}, SavePhpIni{version,ini_type,content},
   InstallPhpVersion(String), RemovePhpVersion(String)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART B — DESIGN IMPLEMENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Using the fetched design, implement these panels:

7. PHP VERSIONS PANEL — src/ui/panels/php_versions.rs
   Implement exactly as shown in the design:
   2-column card grid. Active version card has 3px Colors::ACCENT left border.
   Each card: version number 19px, "active" badge if active, FPM status dot,
   version info 11px, action buttons (Set global / phpinfo / Restart FPM / trash).
   Install slot (dashed border, "+" icon) for next installable version.

8. PHP EXTENSIONS PANEL — src/ui/panels/php_extensions.rs
   TableBuilder: toggle col 24px | name flex | type 70px | version 80px | enabled 70px.
   Toggle: 10×10 Colors::ACCENT square if enabled else Colors::BORDER square.
   Row hover: Colors::CARD_HOVER background.
   Search TextEdit + PHP version ComboBox + "Show core" checkbox in toolbar.

9. PHP INI PANEL — src/ui/panels/php_ini.rs
   Left 160px: section list, SelectableLabel, Colors::ACCENT_DARK when active.
   Right: key-value table. TextEdit per value, Colors::CARD bg.
   Validation errors: RichText 11px Colors::DANGER below the field.
   Bottom bar: "Save" accent_button (disabled when unchanged), "Revert" ghost.
   Raw edit toggle: full-height TextEdit, monospace 13px, Colors::DEEP_BG bg.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ PHP Versions panel matches the design exactly
  □ Active version card has 3px left teal border
  □ "Set global" dispatches SwitchGlobalPhp and refreshes state
  □ Extension toggle enables/disables, state updates immediately
  □ INI save/revert work; validation errors shown in red
  □ zero from_rgb(), zero 1px Stroke
```

---

## Phase 3 — Sites & Nginx
**Weeks 7–9**

```
Phase 2 complete. Implement site scanning and sites/parks/nginx panels.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: Sites panel (table, framework badges, PHP dropdown, ⋮ menu).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART A — ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read: src/state/app_state.rs, src/commands.rs, src/valet/variant.rs

1. VALET CONFIG READER — src/valet/config_reader.rs
   ValetConfig: tld, loopback, paths (parked dirs), site_config (individual overrides)
   async fn load(valet_paths) — reads config.json at variant-specific path

2. SITE SCANNER — src/valet/site_scanner.rs
   ValetSite: name, domain, path, site_type (Parked|Linked|Proxy),
              framework: DetectedFramework, php_version: Option<String>,
              is_secured, ssl_expiry: Option<NaiveDate>, is_favorite
   DetectedFramework: all 21 variants from docs/specs/FRAMEWORKS.md
   async fn scan_all(valet_paths, valet_config) -> Vec<ValetSite>
   pub fn detect_framework(path: &Path) -> DetectedFramework
     Implement detection with EXACT priority order from docs/specs/FRAMEWORKS.md §1

3. NGINX MANAGER — src/nginx/site_manager.rs
   async fn read_site_config(site, valet_paths) -> String
   async fn write_site_config(site, content, valet_paths) via privilege helper
   async fn reload() via privilege helper systemctl reload nginx
   async fn inject_directives(site, block, valet_paths) — sentinel comment pattern

4. FILE WATCHER — src/valet/watcher.rs
   inotify on Sites/ and Nginx/ dirs → debounce 500ms → dispatch RefreshSites

5. NEW AppCommand variants:
   RefreshSites, ParkDirectory(PathBuf), ForgetDirectory(PathBuf),
   LinkSite{name,path}, UnlinkSite(String), SecureSite(String),
   UnsecureSite(String), IsolateSite{site,version}, UnisolateSite(String),
   ToggleFavoriteSite(String), OpenSiteInBrowser(String),
   OpenSiteInEditor(String), SaveNginxConfig{site,content}, ReloadNginx

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART B — DESIGN IMPLEMENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
6. SITES PANEL — src/ui/panels/sites.rs
   Implement the Sites panel exactly as shown in the design.
   TableBuilder columns: star 26px | domain 170px | path flex | PHP 78px |
     framework 96px | TLS 36px | menu 32px.
   Favorites sort to top. PHP ComboBox per row. framework_badge() per framework.
   TLS: lock icon in Colors::ACCENT (ok), Colors::WARNING (<30d), Colors::DANGER (<7d).
   ⋮ popup menu with all site actions. Row hover Colors::CARD_HOVER.
   Empty state when filtered: centered message + accent "Link a site" button.

7. PARKS PANEL — src/ui/panels/parks.rs
   Card list of parked directories. Add via rfd::FileDialog. Remove with confirm.

8. NGINX PANEL — src/ui/panels/nginx.rs
   Left 200px: site list SelectableLabel, Colors::ACCENT_DARK when selected.
   Right: raw config TextEdit monospace 13px Colors::DEEP_BG.
   Line numbers in 28px left gutter, Colors::TEXT_TERTIARY 11px.
   Bottom: "Save & Reload" accent + "Revert" ghost.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ Sites panel matches the design: framework badges, PHP dropdowns, TLS icons
  □ Favorites sort to top; star toggle persists
  □ Framework detection passes all 21 tests from docs/specs/FRAMEWORKS.md
  □ inotify triggers refresh when Sites/ changes
  □ ⋮ context menu dispatches correct commands
```

---

## Phase 4 — App Creator: WordPress & Laravel
**Weeks 10–12**

```
Phase 3 complete. Implement the App Creator wizard for WP and Laravel.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: App Creator wizard — all 5 steps (SelectType, Configure,
PostInstall, Progress with terminal output, Complete/Error screens).
The design shows Step 2 Configure (WordPress form) as the primary reference.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART A — ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read: src/state/app_state.rs, src/commands.rs

1. CLI TOOL REGISTRY — src/cli_tools/registry.rs
   CliTool: WpCli, WpCliValetCommand, LaravelInstaller, Composer, Npm, Git, Node
   async fn detect_all() -> HashMap<CliTool, ToolStatus>
   ToolStatus: Installed(path, version) | Missing

2. OUTPUT STREAMER — src/creator/output_streamer.rs
   OutputLine: { text, stream: Stdout|Stderr, timestamp }
   async fn stream_command(cmd, args, cwd, tx: Sender<OutputLine>) -> ExitStatus
   Sends SIGTERM on cancel signal from CancelCreation command.

3. PROJECT TYPES — src/creator/project_types.rs
   ProjectType: id, display_name, description, group, required_tools,
                install_command, options: Vec<ProjectOption>
   ProjectOption: key, label, option_type (Text|Password|Select|Toggle|Dir)
   all_project_types() — Phase 4: WordPress (5 variants) + Laravel (4 variants)

4. RUNNERS:
   src/creator/runners/wordpress.rs — wp valet new with all 14 options
   src/creator/runners/laravel.rs — blank/Breeze/Jetstream/API

5. POST-INSTALL — src/creator/post_install.rs
   Steps: valet link, valet secure, valet isolate, open browser.
   Each step sends OutputLine progress messages.

6. CREATOR STATE — src/state/creator_state.rs
   CreatorStep: SelectType | Configure | PostInstall | Progress | Complete | Error
   CreatorState: step, selected_type, form_values, output_lines, error

7. NEW AppCommand: CreateApp(AppCreationRequest), CancelCreation

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART B — DESIGN IMPLEMENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
8. TERMINAL OUTPUT COMPONENT — src/ui/components/terminal_output.rs
   Monospace 12px, Colors::DEEP_BG bg, auto-scroll, timestamps in TEXT_TERTIARY.
   Stderr lines in Colors::WARNING. Max 1000 lines displayed.

9. APP CREATOR PANEL — src/ui/panels/app_creator.rs
   Implement the full 5-step wizard from the design.

   Step indicator: 5 circles with lines between.
     Done: Colors::ACCENT_DEEP fill, ACCENT text "✓"
     Current: Colors::ACCENT fill, ACCENT_DEEP text
     Todo: Colors::CARD fill, BORDER stroke, TEXT_TERTIARY

   Step 1 SelectType: group tab row (Laravel/WordPress/PHP/Node/Static),
     3-column card grid, tool availability row at bottom.

   Step 2 Configure: 2-column form grid, domain preview below name field
     in Colors::ACCENT "→ name.test", DirectoryPicker with rfd.

   Step 3 PostInstall: checkbox list with inline PHP version ComboBox if isolated.

   Step 4 Progress: step checklist + terminal_output widget.
     Running step: Colors::ACCENT text. Cancel button top-right.

   Step 5a Complete: checkmark circle, domain URL Colors::ACCENT clickable.
   Step 5b Error: danger circle, error in Colors::WARNING monospace.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ All 5 steps render without overflow
  □ Step indicator shows correct state at each step
  □ WordPress form shows all 14 wp valet new options
  □ Domain preview updates live as name is typed
  □ Terminal streams in real-time with timestamps
  □ Cancel sends SIGTERM to child process
  □ Success screen shows clickable domain URL
```

---

## Phase 5 — App Creator: All 30+ Frameworks
**Weeks 13–15**

```
Phase 4 complete. Extend App Creator to all project types.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: App Creator Step 1 (all framework groups and cards),
Node.js project completion screen (two URLs shown), prerequisite installer modal.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART A — ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read: src/creator/project_types.rs, src/creator/runners/

1. Extend all_project_types() with ALL types from docs/specs/FRAMEWORKS.md §3:
   PHP (Symfony, CakePHP, ConcreteCMS, Contao, Craft, Drupal, Jigsaw, Joomla,
        Kirby, Magento, OctoberCMS, Sculpin, Slim, Laminas)
   Node.js (Next.js, Nuxt 4, React/Vite, Vue/Vite, SvelteKit, Astro)
   Static HTML (blank, Tailwind CDN, Bootstrap CDN — in-process, no subprocess)

2. Runners: src/creator/runners/composer.rs (generic composer create-project)
   src/creator/runners/nodejs.rs — creates project + valet proxy + optional systemd

3. ExpressionEngine: show_manual_download=true, no subprocess, display instructions

4. Katana: detection only — NOT in App Creator (project abandoned)

5. Prerequisite installer modal: ConfirmDialog with code block showing
   the exact install command. ghost cancel + accent install.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PART B — DESIGN IMPLEMENTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
6. App Creator Step 1: all framework cards visible in correct groups.
   Node.js cards: Colors::INFO left border strip, "Served via proxy → :port" note.
   Magento card: duration warning "10–20 min install" below description.
   ExpressionEngine: card shows download-required badge instead of install button.

7. Prerequisite modal (src/ui/components/confirm_dialog.rs):
   centered 360×160 Frame Colors::CARD bg Colors::BORDER_MED stroke.
   Title 16px TEXT_PRIMARY. Message 13px TEXT_SECONDARY.
   Code block: monospace Colors::DEEP_BG box with install command.
   cancel ghost | confirm accent/danger.

8. Node.js complete screen: two URL lines.
   https://{name}.test → Colors::ACCENT. http://localhost:{port} → Colors::INFO.
   "✓ Systemd service created" if enabled.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ All 30+ project types appear in correct groups
  □ Static HTML creates project in-process (no subprocess)
  □ Magento shows duration warning in Step 2
  □ ExpressionEngine shows download prompt, not a command
  □ Node.js success shows both proxy domain and localhost URL
```

---

## Phase 6 — Advanced Feature Panels
**Weeks 16–18**

```
Phase 5 complete. Implement remaining management panels.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: design system tokens (Colors::*, card_frame, section_label).
Apply the visual language consistently across all panels below — they are not
individually shown in the design but must match the established visual system.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE + UI (all in one — these panels are simpler)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Implement backend logic AND panel UI for each:

1. PROXIES (src/nginx/ + src/ui/panels/proxies.rs)
   valet proxy/unproxy commands. Table: domain | target | TLS | HTTP status | actions.
   "Test" fires async HTTP check: 200 → Colors::ACCENT, 4xx → WARNING, 5xx → DANGER.

2. DNSMASQ (src/ui/panels/dnsmasq.rs)
   TLD changer: TextEdit + "Apply" → runs valet domain {tld}.
   DNS tester: TextEdit + "Test →" → streams dig output in monospace box.

3. SHARING (src/ui/panels/sharing.rs)
   Three radio cards: ngrok | Expose | cloudflared. Token TextEdit per tool.
   Site ComboBox + "▶ Share" accent. When active: public URL Colors::ACCENT.

4. LOGS (src/ui/panels/logs.rs)
   Source tabs (pills): Nginx Error | PHP-FPM | Valet FPM | Access.
   Full-height Colors::DEEP_BG ScrollArea monospace. Lines containing "error"
   → Colors::DANGER, "warn" → Colors::WARNING, "notice" → Colors::INFO.

5. DIAGNOSTICS (src/ui/panels/diagnostics.rs)
   "Run Diagnostics" accent → streams valet diagnose.
   terminal_output widget. "Trust Valet" + "Restart All" ghost buttons below.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ Proxy HTTP status shows correct colors
  □ DNS tester shows dig output in monospace box
  □ Log panel color-codes lines correctly
  □ All panels use card_frame(), section_label(), Colors::* only
```

---

## Phase 7 — Polish & Release
**Weeks 19–21**

```
Phase 6 complete. System tray, notifications, settings, packaging.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: design system for Toast notifications, Settings panel layout,
and Onboarding wizard — match the same visual language as the rest of the app.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE + UI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. TOAST COMPONENT — src/ui/components/toast.rs
   Bottom-right stack, auto-dismiss 4s with fade (0.5s).
   3px left border in type color: ACCENT (success), DANGER (error),
   WARNING (warning), INFO (info).

2. SYSTEM TRAY — src/tray/mod.rs
   tray-icon crate. Menu: active PHP version (display only), quick PHP switcher,
   Restart services, Open Valet Manager, Quit.
   Tray icon color reflects service health (green/red).

3. DESKTOP NOTIFICATIONS — src/notifications/mod.rs
   notify-rust. Events (opt-in per type): PHP switched, service failed,
   creation complete, SSL expiry warning, update available.

4. SETTINGS PANEL — src/ui/panels/settings.rs
   sections via settings_section(ui, title) → card_frame() wrapper.
   Sections: appearance (theme, font size), editor/tools (3 TextEdit rows),
   notifications (checkbox list), app creator (defaults), danger zone.
   Save accent button, disabled when no changes.

5. ONBOARDING WIZARD — src/ui/panels/onboarding.rs
   Show when valet_variant is None. 5 steps: Welcome, PHP check, Valet check,
   Verify (valet diagnose stream), Ready. step_indicator at top.

6. RESPONSIVE SIDEBAR — update src/ui/sidebar.rs
   Width < 1000px: icon-only mode (32px wide). Tooltip on hover shows label.
   Width < 800px: sidebar hidden, hamburger in title bar.

7. PACKAGING:
   Add to Cargo.toml: [package.metadata.deb] section from docs/DEPENDENCIES.md.
   packaging/valet-manager.desktop with Exec, Icon, Categories=Development.
   packaging/polkit/com.valetmanager.policy.
   cargo deb produces installable .deb.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ Toast auto-dismisses in 4s with fade
  □ Settings save and reload config
  □ Onboarding shows on first run, hides after completion
  □ cargo deb produces installable package
  □ Responsive sidebar works below 1000px
```

---

## Phase 8 — PHPMon Gap-Closing
**Weeks 22–25**

```
Phase 7 complete. Close all PHPMon feature gaps.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: dashboard alert banners (all 4 types),
phpinfo panel, compatibility panel, history panel.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE + UI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. PHPINFO PANEL — src/ui/panels/phpinfo.rs
   Runs php{ver} -r "phpinfo();" parses HTML into sections + key-value map.
   Left 180px section list (SelectableLabel, ACCENT_DARK when active).
   Right: 3-col table (key | local value | master value). Search highlights matches.
   Copy button per row → clipboard.

2. COMPATIBILITY PANEL — src/ui/panels/compat.rs
   Check all sites: read composer.json require.php vs installed PHP.
   Status badges: Compatible (ACCENT tinted), Incompatible (DANGER tinted),
   NoRequirement (CARD, TEXT_TERTIARY).
   "Isolate PHP X.X" accent button on incompatible rows.
   Dashboard integration: alert_banner(WARNING, "⚠", "{N} sites have PHP compat issues")

3. HISTORY PANEL — src/ui/panels/history.rs
   SQLite (rusqlite). Every subprocess logged: command, args, duration, exit, source.
   Table: time-ago | command | duration | exit code.
   Exit 0 → Colors::ACCENT badge. Non-zero → Colors::DANGER.
   Row expand: show output preview. "Re-run" ghost button.

4. DEEP-LINK — src/deep_link/mod.rs
   Register valet-manager:// in .desktop file.
   Parse URL → dispatch AppCommand.
   Examples: valet-manager://open?site=myapp.test, valet-manager://php?switch=8.3

5. UPDATER — src/updater/mod.rs
   GET https://api.github.com/repos/{repo}/releases/latest → parse tag_name.
   Compare with env!("CARGO_PKG_VERSION") via semver.
   Dashboard alert: "Version {ver} available" with "Update" + "Skip" buttons.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ phpinfo sections navigate; search highlights matches
  □ Compat panel checks all sites; action dispatches IsolateSite
  □ Dashboard alert shows when incompatible sites exist
  □ History table shows all subprocesses with correct exit colors
  □ valet-manager:// URL opens app and dispatches action
```

---

## Phase 9 — DX Tier 1
**Weeks 26–30**

```
Phase 8 complete. Command palette, .env editor, Artisan, Database.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: Command Palette overlay, Database Manager panel.
These are the two most complex panels — implement them exactly as designed.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE + UI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. COMMAND PALETTE — src/ui/command_palette.rs
   Overlay: egui::Area order=Foreground, dim background rgba(0,0,0,100),
   520px card Colors::DEEP_BG Colors::BORDER_MED stroke rounding 10px.
   Search TextEdit 15px, frame=false. request_focus() every frame while open.
   8 results max. Category badges: PhpVersion(WARNING), Site(ACCENT),
   Service(INFO), Artisan(purple tinted), Panel/Action(CARD).
   Keyboard: ArrowDown/Up, Enter dispatches, Escape closes.
   Index: PHP versions, sites (open + open in editor), services, panels, artisan.
   Build index in build_index(state), fuzzy_match via fuzzy-matcher crate.
   Ctrl+K toggles open/close.

2. ENV EDITOR — src/ui/panels/env_editor.rs
   Group tabs by key prefix: APP | DB | MAIL | REDIS | Other.
   Key col 180px monospace, Value col flex TextEdit monospace.
   Secrets masked (password=true) unless "Show secrets" checked.
   Validation: compare against .env.example, banner for missing/extra keys.
   Save: atomic write (write .tmp then rename). Revert: reload from disk.

3. ARTISAN RUNNER — src/artisan/ + src/ui/panels/artisan.rs
   Discover via `php artisan list --format=json` for Laravel sites.
   Also supports: bin/console (Symfony), bin/magento (Magento), drush (Drupal).
   Autocomplete TextEdit + args TextEdit + "▶ Run" accent.
   Autocomplete dropdown: 8 rows, CARD_HOVER selected, Arrow keys navigate.
   Quick commands row: horizontal ghost buttons. terminal_output below.

4. DATABASE MANAGER — src/ui/panels/database.rs
   Implement exactly as shown in the design: split pane (168px left DB list,
   right table list). Selected DB: 2px ACCENT left border + ACCENT tinted bg.
   Row counts formatted with thousands separator (12,450 not 12450).
   Laravel actions bar (only for Laravel sites): Migrate | Migrate fresh | Seed | Rollback.
   Terminal output slides in when migration runs.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ Ctrl+K opens palette; result categories have correct badge colors
  □ Arrow keys navigate; Enter dispatches and closes
  □ .env editor groups keys; secrets masked by default
  □ Artisan autocomplete filters as you type
  □ Database split pane matches the design
  □ Laravel actions bar only shows for Laravel sites
  □ Migration output streams in real-time
```

---

## Phase 10 — DX Tier 2
**Weeks 31–34**

```
Phase 9 complete. SSL dashboard, Xdebug, Mail catcher, Queue workers.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: apply the established design system (card_frame, status dots,
Colors::*) to SSL, Xdebug, Mail catcher, and Queue panels consistently.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE + UI
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. SSL PANEL — src/ui/panels/ssl_certs.rs
   CA status badge header. Table: domain | issuer | expires | days | status | actions.
   Status sort: Critical(red) → Warning(amber) → OK(green) → Expired(red).
   Days column right-aligned in status color.

2. XDEBUG PANEL — src/ui/panels/xdebug.rs
   One card_frame per installed PHP version.
   If installed: mode selector (Off|Debug|Profile|Coverage|Trace) as radio pills
   (ACCENT_DARK active, CARD inactive). IDE key ComboBox + port TextEdit.
   If not installed: "Install Xdebug" accent button → streaming apt install.

3. MAIL CATCHER — src/ui/panels/mail_catcher.rs
   Status card: tool name + SMTP/HTTP port badges. status_dot + Running/Stopped.
   Start/Stop buttons. Unread count when running.
   SMTP config section: site ComboBox + "Apply to .env" accent.
   Preview monospace box showing the .env lines that will be written.

4. QUEUE WORKERS — src/ui/panels/queue.rs
   Table: site | queue | connection | status | jobs | failed | actions.
   Failed count in Colors::DANGER if > 0.
   "Add worker" modal: site, connection, queue TextEdits + "Start on boot" toggle.
   Workers persist as systemd user service files.

5. FINAL WIRING:
   All 26 panels routed in CentralPanel match block.
   Command palette index updated with all new panels and Xdebug actions.
   Dashboard SSL expiry alert wired to ssl_state data.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ SSL panel sorts correctly Critical → Warning → OK → Expired
  □ Xdebug cards show per-PHP-version state
  □ Mail catcher unread count updates
  □ Queue table shows live systemd status
  □ All 26 panels navigable from sidebar and command palette
  □ Zero from_rgb() in any panel — final check
  □ cargo test --workspace — all tests pass
```

---

## Phase 11 — Per-Site Config & HTTP Servers
**Weeks 35–38**

```
Phases 1–10 complete. Per-site TOML config backend + HTTP server abstraction.
The UI panels are already built (Phase 10). This phase wires the backend.
Full spec: source/spec-site-config-phase-11.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: Site configuration panel tabs (PHP, WordPress, Laravel,
Nginx/Server, Database, Development). Wire all form controls to the config state.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read source/spec-site-config-phase-11.md for complete data models and
module structure. Implement everything specified there:

- src/site_config/ (models, reader, writer, merger)
- src/php/user_ini.rs (.user.ini writer, validates values)
- src/wordpress/ (config_editor, multisite, wp_cli)
- src/laravel/ (octane, packages)
- src/http_servers/ (detector, nginx extend, frankenphp, caddy, apache, basic_auth)
- All AppCommand variants listed in the spec
- State fields: site_configs, installed_http_servers, octane_processes

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
UI WIRING (Phase 11b)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Wire all site config panel controls as specified in the spec §Phase 11b:
- PHP tab: version ComboBox → SetSitePhpVersion, INI override fields with
  inline validation errors in Colors::DANGER, Xdebug toggles
- WordPress tab: multisite toggle → streaming wp core multisite-install,
  network sites table, plugin list
- Laravel tab: package detection badges, Octane start/stop
- Server tab: server type selector (only installed servers shown), directive
  TextEdit, basic auth user management
- Dirty state: "● Unsaved changes" badge, confirm on navigate-away
- "Save to project (.valet-manager.toml)" ghost button with confirm dialog

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ cargo test --workspace — all new tests pass
  □ .user.ini written to correct path per framework
  □ wp-config.php backup created before any modification
  □ Multisite enable streams wp core multisite-install output
  □ Nginx directive injection uses sentinel comments (idempotent)
  □ Dirty badge shows and clears correctly
```

---

## Phase 12 — phpMyAdmin Per-Site
**Weeks 38–40**

```
Phase 11 complete. phpMyAdmin per-site service.
Full spec: source/spec-phpmyadmin-phase-12.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: phpMyAdmin tab in site config panel, install button with
terminal output, access URL display, DB scope selector.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read source/spec-phpmyadmin-phase-12.md for complete spec.
Implement everything: installer, config_generator, nginx_integration, global_site.
Credential resolution: .env first, wp-config.php second, AppConfig defaults.
blowfish_secret: generated once, stored in AppConfig, never logged.
config.inc.php permissions: 640.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
UI WIRING
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Wire the phpMyAdmin tab in site config panel:
- "Enable" toggle → ConfigurePhpMyAdminForSite (shows streaming terminal)
- Access mode selector (path alias / subdomain / global only)
- DB scope selector (site only / all databases)
- Access URL as Colors::ACCENT clickable link
- DB name display + "Detected from .env" caption
- "Open phpMyAdmin" accent + "Open (all DBs)" ghost
Sites panel ⋮ menu: add "Open phpMyAdmin" when site has pma enabled.
Dashboard quick actions: phpMyAdmin button when any site enabled.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ config.inc.php contains only_db for site-only mode
  □ config.inc.php has NO only_db for all-databases mode
  □ File permissions are 640
  □ Nginx sentinel block injected + idempotent on re-apply
  □ Global phpmyadmin.test site accessible
  □ DB credentials auto-detected from .env and wp-config.php
```

---

## Phase 13 — All 21 Frameworks
**Weeks 40–42**

```
Phase 12 complete. Full framework support for all 21 Valet-supported frameworks.
Full spec: docs/prompts/PHASE-13.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: framework badges in Sites panel (all 21 colors),
App Creator Step 1 (all framework cards grouped correctly).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TASKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read docs/prompts/PHASE-13.md and docs/specs/FRAMEWORKS.md.
Implement all 10 parts specified there:
  1. detect_framework() with correct priority ordering
  2. framework_badge_colors() and framework_display_name() for all 21
  3. user_ini_path() correct document root per framework
  4. recommended_php_version() per framework
  5. all_project_types() with all 30+ types
  6. Framework CLI runner (artisan/bin/console/bin/magento/drush)
  7. Compatibility checker updates for non-composer frameworks
  8. Framework-specific site config structs (Magento, Drupal, Craft, etc.)
  9. Sites panel badge tooltip with recommended PHP
  10. Command palette framework-specific quick actions

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ All 21 detection tests pass (including priority order tests)
  □ All 21 framework badges have non-empty color pairs
  □ App Creator shows 20 project types (Katana excluded)
  □ Artisan panel shows for Symfony/Magento/Drupal sites
  □ cargo test --workspace — all tests pass
```

---

## Phase 14 — Version Registry
**Weeks 42–44**

```
Phase 13 complete. Live version tracking with refresh button.
Full spec: docs/prompts/PHASE-14.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
STEP 0 — FETCH DESIGN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/349IlgTDPibEvDh1X1PltA?open_file=Valet+Manager.html
Implement: Valet Manager.html

Focus on for this phase: title bar refresh indicator, PHP version patch update
badges in PHP panel, Settings panel Version Registry section.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TASKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Read docs/prompts/PHASE-14.md and docs/specs/VERSION-REGISTRY.md.
Implement all 8 steps specified there:
  0. Update Cargo.toml to versions in docs/VERSIONS.md (egui→0.34.2 etc.)
  1. Data models (VersionRegistry, PhpVersionInfo, EolStatus, etc.)
  2. Cache module (~/.config/valet-manager/version-registry.json)
  3. Fetchers: php_fetcher (endoflife.date), github_fetcher, packagist_fetcher,
     wordpress_fetcher, nodejs_fetcher — all concurrent via tokio::join!
  4. Orchestrator refresh() with fallback to bundled defaults on network failure
  5. State + AppCommand: RefreshVersionRegistry
  6. Startup auto-refresh when cache is stale
  7. UI: PHP EOL banners, patch version badges, title bar button, Settings section
  8. App Creator: PHP version auto-selected from framework min_php requirement

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ACCEPTANCE CRITERIA
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  □ cargo build after Cargo.toml version bumps — zero errors
  □ egui 0.34 migration: App::ui() used, no App::update()
  □ PHP 8.1 EOL banner shows in dashboard (it is already EOL)
  □ PHP Versions panel shows "↑ X.Y.Z available" when patch update exists
  □ Refresh fetches all 14 sources concurrently in ~3s
  □ Network failure on one source falls back to bundled default
  □ cargo test --workspace — all tests pass
  □ cargo clippy -- -D warnings — zero warnings
```

---

## Final verification (run after Phase 14)

```bash
# Design contract
grep -rn "from_rgb\|from_rgba" src/ui/panels/ src/ui/sidebar.rs
# → must be zero

grep -rn "Stroke::new(1\." src/ui/
# → must be zero

# Quality
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release

# Package
cargo deb --package valet-manager
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
