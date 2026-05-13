# Valet Manager — Unified Claude Code Prompts
## Architecture + Design Integration · All 10 Phases

> Each prompt produces a complete implementation: Rust backend logic **and**
> egui visual rendering in a single Claude Code session.
>
> **How to run:**
> ```bash
> cd ~/projects/valet-manager
> claude   # interactive session — paste the phase prompt below
> ```
> Run phases in order. Each phase reads files written by the previous phase.
>
> **Prerequisites:** Rust stable 1.78+, Cargo, Linux dev machine, valet-linux installed.
>
> **Design reference files** (produced alongside these prompts):
> - Claude Design file: https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
> - Local copy: `docs/design/UI-PROMPTS.md`  (offline fallback)
> - `docs/design/comparison.md`         — PHPMon gap analysis
> - `docs/design/architecture-v3.md`    — full module spec

---

## DESIGN CONTRACT (applies to every phase)

Before implementing any `src/ui/` file, Claude Code must read `src/ui/theme.rs`
and use only the `Colors::*` constants defined there. Never hardcode hex or RGB
in panel/component files. The design system is the single source of truth.

```
COLOUR TOKENS (defined in src/ui/theme.rs after Phase 1):
  Colors::DEEP_BG      #0E1613   sidebar, titlebar, terminal output
  Colors::SURFACE      #162019   panel background (CentralPanel fill)
  Colors::CARD         #1C2A26   cards, inputs, table backgrounds
  Colors::CARD_HOVER   #20302B   hover state for interactive cards/rows
  Colors::BORDER       rgba(255,255,255,0.07)   all borders, dividers
  Colors::BORDER_MED   rgba(255,255,255,0.11)   hover/focus borders
  Colors::TEXT_PRIMARY   #D8EDE6
  Colors::TEXT_SECONDARY #6E9488
  Colors::TEXT_TERTIARY  #3A5550   labels, captions, section headers
  Colors::ACCENT         #5DCAA5   active, links, success, running
  Colors::ACCENT_DARK    #0F6E56   nav active bg, primary buttons
  Colors::ACCENT_DEEP    #04342C   icon backgrounds, deepest teal
  Colors::DANGER         #E24B4A
  Colors::WARNING        #EF9F27
  Colors::INFO           #378ADD

LAYOUT TOKENS:
  Sidebar width:        196px (fixed, not resizable)
  Panel header height:  44px (border-bottom below)
  Table row height:     32px
  Card padding:         12px 14px
  Card border-radius:   8px (large), 6px (normal), 4px (inputs)
  Sidebar nav item:     28px tall, 16px icon, 8px icon-label gap
  Status dot:           7px diameter circle
  All borders:          0.5px Stroke (never 1px)

TYPOGRAPHY:
  Panel headers:  15px, weight 500, Colors::TEXT_PRIMARY
  Body/tables:    13px, weight 400, Colors::TEXT_PRIMARY
  Captions:       11px, weight 400, Colors::TEXT_TERTIARY
  Section labels: 10px, letter-spacing 0.1em, Colors::TEXT_TERTIARY
  Monospace:      FontId::monospace(13.0) for code/terminal/INI/Nginx
```

---

## Phase 1 — Foundation + Design System
**Weeks 1–3 · Workspace, state, PHP detection, service polling, theme engine, sidebar, window setup**

```
You are building Phase 1 of Valet Manager from scratch.

This phase produces BOTH the complete Rust foundation AND the full egui design
system. When this phase is done, the app must open with a correctly themed dark
window, a branded sidebar, and a functional dashboard panel.

══════════════════════════════════════════════════════════════
PART A — RUST FOUNDATION
══════════════════════════════════════════════════════════════

1. CARGO WORKSPACE
   Root Cargo.toml with members: ["valet-manager", "valet-manager-helper"]
   valet-manager/Cargo.toml dependencies:
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
     fuzzy-matcher = "0.3"

2. VALET VARIANT DETECTION — src/valet/variant.rs
   ValetVariant enum: ValetLinux | ValetOfficial | ValetLinuxPlus
   ValetPaths struct: config_root, nginx_dir, sites_dir, drivers_dir,
     log_dir, config_json, ca_dir (all PathBuf)
   fn for_variant(v: &ValetVariant) -> ValetPaths using dirs::home_dir()
   async fn detect_valet_variant() -> anyhow::Result<ValetVariant>:
     1. Run "composer global show --format=json", parse "installed" array
        checking for "genesisweb/valet-linux-plus" then "cpriego/valet-linux"
     2. Fallback: probe ~/.valet/config.json and ~/.config/valet/config.json

3. DISTRO & PACKAGE MANAGER — src/system/distro.rs + package_manager.rs
   Parse /etc/os-release → DistroKind enum: Ubuntu | Debian | Fedora | Arch | Unknown
   PackageManager enum: Apt | Dnf | Pacman
   fn detect_package_manager() -> PackageManager from distro
   async fn list_installed_php_packages(pm) -> Vec<String>:
     apt: "apt list --installed 2>/dev/null | grep php"
     dnf: "dnf list installed | grep php"
     pacman: "pacman -Q | grep php"

4. PHP VERSION DETECTION — src/php/detector.rs
   PhpVersion struct: version, full_version, binary_path, fpm_service,
     cli_ini_path, fpm_ini_path, conf_d_path, is_active, fpm_running
   async fn detect_installed_versions() -> Vec<PhpVersion>:
     Scan /usr/bin/ for files matching `^php\d+\.\d+$`
     Also run "update-alternatives --list php" and merge results
     For each binary: run "{binary} --version" to get full_version
     Derive all paths from debian conventions: /etc/php/{ver}/cli/php.ini etc.
   async fn detect_active_version() -> String: run "php --version", parse first line

5. SERVICE MONITOR — src/services/monitor.rs
   ServiceStatus enum: Running | Stopped | Failed | Unknown
   ManagedService struct: name, display_name, status, pid: Option<u32>
   async fn query_service_status(name: &str) -> ServiceStatus:
     "systemctl is-active --quiet {name}" → Running if exit 0, else Stopped
   async fn poll_services(services: Vec<String>,
     tx: mpsc::Sender<Vec<ManagedService>>): loop every 5s

6. APP STATE — src/state/app_state.rs
   AppState struct (Default): valet_variant, valet_paths, php_versions,
     active_php, services, cli_tools (stub Vec), ui: UiState
   UiState struct (Default): active_panel: Panel, loading, toast_queue, last_error
   Panel enum (Default = Dashboard): Dashboard, PhpVersions, PhpExtensions,
     PhpIni, PhpInfo, PhpCompat, Sites, Parks, Nginx, Proxies, Dnsmasq,
     SslCerts, Database, EnvEditor, Artisan, QueueWorkers, Xdebug, MailCatcher,
     Sharing, Drivers, Logs, History, Diagnostics, Settings

7. CONFIG — src/config.rs
   AppConfig (serde, Default): editor_command="code", terminal="gnome-terminal",
     file_manager="nautilus", theme="dark", font_size=14.0,
     service_poll_interval_secs=5, site_scan_debounce_ms=500,
     default_parent_directory="~/Sites", favorites: Vec<String>
   fn config_path() -> PathBuf: ~/.config/valet-manager/config.toml
   fn load() -> AppConfig  |  fn save(cfg: &AppConfig) -> anyhow::Result<()>

8. COMMANDS & EVENTS — src/commands.rs + events.rs
   AppCommand (Phase 1): RefreshAll, RefreshServiceStatus, SwitchGlobalPhp(String),
     OpenPanel(Panel), OpenCommandPalette
   AppEvent: ServiceStatusUpdated(Vec<ManagedService>),
     PhpVersionsRefreshed(Vec<PhpVersion>), ValetDetected(ValetVariant, ValetPaths),
     Error(String)

9. COMMAND DISPATCHER — spawn tokio task in src/main.rs that runs
   async fn run_dispatcher(rx: mpsc::Receiver<AppCommand>,
     tx: mpsc::Sender<AppEvent>, state: Arc<RwLock<AppState>>)
   Spawn poll_services as separate long-running task.

══════════════════════════════════════════════════════════════
PART B — DESIGN SYSTEM (must be done before any UI code)
══════════════════════════════════════════════════════════════

10. THEME ENGINE — src/ui/theme.rs
    Create a Colors struct with ALL of the following as pub const:
      DEEP_BG:        Color32::from_rgb(0x0E, 0x16, 0x13)
      SURFACE:        Color32::from_rgb(0x16, 0x20, 0x19)
      CARD:           Color32::from_rgb(0x1C, 0x2A, 0x26)
      CARD_HOVER:     Color32::from_rgb(0x20, 0x30, 0x2B)
      BORDER:         Color32::from_rgba_unmultiplied(255, 255, 255, 18)
      BORDER_MED:     Color32::from_rgba_unmultiplied(255, 255, 255, 28)
      TEXT_PRIMARY:   Color32::from_rgb(0xD8, 0xED, 0xE6)
      TEXT_SECONDARY: Color32::from_rgb(0x6E, 0x94, 0x88)
      TEXT_TERTIARY:  Color32::from_rgb(0x3A, 0x55, 0x50)
      ACCENT:         Color32::from_rgb(0x5D, 0xCA, 0xA5)
      ACCENT_DARK:    Color32::from_rgb(0x0F, 0x6E, 0x56)
      ACCENT_DEEP:    Color32::from_rgb(0x04, 0x34, 0x2C)
      DANGER:         Color32::from_rgb(0xE2, 0x4B, 0x4A)
      WARNING:        Color32::from_rgb(0xEF, 0x9F, 0x27)
      INFO:           Color32::from_rgb(0x37, 0x8A, 0xDD)

    pub fn apply_dark(ctx: &egui::Context)
      Sets ctx.set_visuals() — customise every widget state:
        panel_fill = Colors::SURFACE
        window_fill = Colors::SURFACE
        extreme_bg_color = Colors::DEEP_BG
        faint_bg_color = Colors::CARD
        widgets.noninteractive: bg=CARD, fg=TEXT_TERTIARY, stroke=BORDER
        widgets.inactive:       bg=CARD, fg=TEXT_SECONDARY, stroke=BORDER_MED
        widgets.hovered:        bg=CARD_HOVER, fg=TEXT_PRIMARY, stroke=BORDER_MED
        widgets.active:         bg=ACCENT_DARK, fg=WHITE
        selection.bg_fill = ACCENT at 16% opacity
        hyperlink_color = ACCENT
        warn_fg_color = WARNING
        error_fg_color = DANGER

    pub fn status_color(status: &ServiceStatus) -> Color32
      Running=ACCENT, Stopped=DANGER, Failed=DANGER, Unknown=TEXT_TERTIARY

    pub fn framework_badge_colors(fw: &DetectedFramework) -> (Color32, Color32)
      (bg, text):
        Laravel    → (Color32::from_rgba(0xEF,0x9F,0x27,28), Color32::from_rgb(0xBA,0x75,0x17))
        WordPress  → (Color32::from_rgba(0x37,0x8A,0xDD,30), Color32::from_rgb(0x18,0x5F,0xA5))
        Symfony    → (Color32::from_rgba(0x5D,0xCA,0xA5,25), Color32::from_rgb(0x08,0x50,0x41))
        Bedrock    → same as WordPress
        Proxy/None → (CARD, TEXT_TERTIARY)
        _          → (CARD, TEXT_SECONDARY)

    pub fn accent_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response
      Frame with ACCENT_DARK fill, rounding 4px, no outer border.
      TEXT_PRIMARY text inside. Hover: slightly lighter fill.

    pub fn ghost_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response
      Frame with CARD fill, BORDER_MED stroke, TEXT_SECONDARY text.

    pub fn danger_button(ui: &mut egui::Ui, label: impl Into<String>) -> Response
      Transparent fill, DANGER stroke and text.

    pub fn status_dot(ui: &mut egui::Ui, status: &ServiceStatus)
      Paints 7×7 circle: ui.painter().circle_filled(pos, 3.5, status_color(status))

    pub fn framework_badge(ui: &mut egui::Ui, fw: &DetectedFramework)
      Pill with 2px 7px padding, 20px radius.
      fn name_for(fw) -> &str: "Laravel"/"WordPress"/"Symfony" etc.

    pub fn section_label(ui: &mut egui::Ui, text: &str)
      Adds 10px top margin, renders text at 10px TEXT_TERTIARY.

    pub fn divider(ui: &mut egui::Ui)
      Full-width 0.5px BORDER line.

    pub fn card_frame() -> egui::Frame
      Returns Frame::default().fill(Colors::CARD).rounding(8.0).inner_margin(Margin::symmetric(14, 12))

11. SIDEBAR — src/ui/sidebar.rs
    pub fn render(ui: &mut egui::Ui, state: &AppState, cmd_tx: &Sender<AppCommand>)
    
    LOGO AREA (50px height):
      Left: 28×28 rounded rect (Colors::ACCENT_DEEP fill, rounding 7.0).
        Inside: paint the V mark using ui.painter():
          Outer triangle: painter.add(Shape::convex_polygon([
            (4,6.4),(22.4,6.4),(13.2,17.1)], Colors::TEXT_PRIMARY, Stroke::NONE))
          Inner cutout: same shape slightly smaller, Colors::ACCENT_DEEP fill
          Three lines below tip: 3 rects in Colors::ACCENT (opacity 1.0, 0.75, 0.5)
            sizes 4.6×0.64px, 3.6×0.64px, 2.6×0.64px centered at x=13.2
      Right: "Valet Manager" at 13px Colors::TEXT_PRIMARY weight 500
        Below: "{variant_display} · .{tld}" at 10px Colors::TEXT_TERTIARY
      Bottom: divider()

    NAV SECTIONS — render in order:
      section_label(ui, "management")
      nav_item for: Dashboard, PHP versions, PHP extensions, PHP INI,
        phpinfo(), PHP compat, Sites, Parks, Nginx, Proxies, dnsmasq, SSL certs
      section_label(ui, "development")
      nav_item for: Database, .env editor, Artisan, Queue workers, Xdebug, Mail catcher
      section_label(ui, "tools")
      nav_item for: Sharing, Drivers, Logs, History, Diagnostics
      nav_item for: Settings (at absolute bottom)

    fn nav_item(ui, icon_char: &str, label: &str, panel: Panel,
      active: &Panel, cmd_tx): 
      ui.add_sized([196.0, 28.0], |ui| {
        let active = *active == panel;
        let bg = if active { Colors::ACCENT_DARK } else { Color32::TRANSPARENT };
        let fg = if active { Color32::WHITE } else { Colors::TEXT_SECONDARY };
        Frame::none().fill(bg).show(ui, |ui| {
          ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(RichText::new(icon_char).size(15.0).color(fg)); // icon
            ui.add_space(4.0);
            ui.label(RichText::new(label).size(12.0).color(fg));
          })
        })
      })
      On click anywhere in the row: cmd_tx.try_send(OpenPanel(panel))
      Use emoji/unicode for icons:
        Dashboard=⊞ PhpVersions=⌥ PhpExtensions=◻ PhpIni=≡
        PhpInfo=ⓘ PhpCompat=✓ Sites=◈ Parks=⌂ Nginx=⚙
        Proxies=⇆ Dnsmasq=⬡ SslCerts=🔒 Database=⊟
        EnvEditor=≣ Artisan=▶ QueueWorkers=⏭ Xdebug=⊗
        MailCatcher=✉ Sharing=↗ Drivers=◇ Logs=≡ History=⌛
        Diagnostics=⊕ Settings=⚙

    SERVICE STATUS (bottom, layout bottom_up):
      1px BORDER top edge
      section_label(ui, "services")
      For each service in state.services:
        ui.horizontal(|ui|: status_dot + label 12px TEXT_SECONDARY
        For mailpit with unread > 0: append " · {n}" in WARNING

12. WINDOW & MAIN ENTRY POINT — src/app.rs + src/main.rs
    NativeOptions: inner_size=[1280,800], min_size=[900,560],
      default_theme=Dark, renderer=Wgpu, embedded icon from assets/icons/

    ValetManagerApp::new(cc): call theme::apply_dark(&cc.egui_ctx)

    ValetManagerApp::update(ctx, frame):
      theme::apply_dark(ctx)  // re-apply each frame (correct for egui)
      
      KEYBOARD: check ctx.input(|i| ...):
        Ctrl+K → cmd_tx.try_send(OpenCommandPalette)
        Ctrl+R → cmd_tx.try_send(RefreshAll)
        Ctrl+, → cmd_tx.try_send(OpenPanel(Settings))

      egui::TopBottomPanel::top("titlebar")
        .exact_height(36.0)
        .frame(Frame::none().fill(Colors::DEEP_BG)):
        LEFT: three dots (paint circles R=5: DANGER, WARNING, ACCENT at x=16,28,40)
        CENTER: "Valet Manager" 12px TEXT_TERTIARY centered
        RIGHT: "PHP {active_php} · {variant}" 11px TEXT_TERTIARY

      egui::SidePanel::left("sidebar")
        .exact_width(196.0)
        .resizable(false)
        .frame(Frame::none().fill(Colors::DEEP_BG)):
        call sidebar::render(ui, &state, &cmd_tx)

      egui::CentralPanel::default()
        .frame(Frame::none().fill(Colors::SURFACE)):
        match state.ui.active_panel → route to panel render function
        For Phase 1: only panels::dashboard::render is wired
        Others: stub showing "Panel coming in Phase N" message

13. DASHBOARD PANEL — src/ui/panels/dashboard.rs
    pub fn render(ui, state, cmd_tx):

    PANEL HEADER:
      ui.horizontal_centered(|ui| { ui.set_height(44.0);
        LEFT: "Dashboard" RichText 15px TEXT_PRIMARY weight 500
        Sub (below): "{PHP ver} · Valet {variant} · {n} sites" 12px TEXT_SECONDARY
        RIGHT (push via ui.with_layout(right_to_left)):
          ghost_button "↺ Refresh" → RefreshAll
          accent_button "⌘K" → OpenCommandPalette
      })
      divider(ui)

    ALERT BANNERS (before stats, only if condition true):
      fn alert_banner(ui, color: Color32, icon: &str, msg: &str, btn: Option<(&str, AppCommand)>)
        Frame: fill=color at 7% opacity, left border=3px solid color (no radius)
        Horizontal: icon 15px + msg text 12px TEXT_PRIMARY + button right-aligned if Some

    STATS GRID (4 cards, equal width):
      ui.columns(4, |cols| for each: card_frame().show(col, |ui| label + value))
      Values: active_php (ACCENT color), site count, services N/total (ACCENT), TLD

    SERVICE SECTION LABEL: section_label(ui, "services")

    SERVICE GRID (2 columns):
      ui.columns(2, |cols| for each service: card_frame().show(col, |ui|
        LEFT: status_dot + name 13px + status_text 10px TEXT_TERTIARY below
        RIGHT: ghost icon button (↺ or ↗ for mailpit)
        Card has 2px left border in status_color painted via painter.line_segment
      ))

══════════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 1
══════════════════════════════════════════════════════════════
  □ cargo build --workspace compiles with zero warnings
  □ cargo clippy -- -D warnings passes
  □ App opens: dark teal theme, sidebar left, dashboard right
  □ Logo V mark visible in sidebar at 28px
  □ Service dots update every 5 seconds
  □ Active PHP shows in titlebar and dashboard stats
  □ Colors::* constants used — zero hardcoded RGB in panel files
  □ All borders are 0.5px Stroke (grep for Stroke::new(1. in panel files → 0)
```

---

## Phase 2 — PHP Management + PHP Panels UI
**Weeks 4–6 · Version switch, extension manager, INI editor, privilege helper, PHP UI panels**

```
Phase 1 is complete. The app opens with a themed window and working dashboard.

This phase implements PHP management logic AND the three PHP panels with full
design fidelity. After this phase, PHP Versions, PHP Extensions, and PHP INI
panels must render exactly as designed.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → PHP Versions, PHP Extensions, and PHP INI panels.
  Also read src/ui/theme.rs for Colors::* constants.

══════════════════════════════════════════════════════════════
PART A — ARCHITECTURE
══════════════════════════════════════════════════════════════

[Include all architecture tasks from the original Phase 2 prompt:]
- Privilege helper binary (valet-manager-helper/src/main.rs)
- Polkit policy (packaging/polkit/com.valetmanager.policy)
- PHP version switcher (src/php/switcher.rs)
- PHP-FPM manager (src/php/fpm_manager.rs)
- PHP extension manager (src/php/extension_manager.rs)
- PHP INI manager (src/php/ini_manager.rs)
- New AppState fields and AppCommand variants
- Wire up dispatcher

══════════════════════════════════════════════════════════════
PART B — DESIGN: PHP VERSIONS PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/php_versions.rs
pub fn render(ui, state, cmd_tx):

HEADER (44px, divider below):
  "PHP versions" RichText 15px TEXT_PRIMARY weight 500
  RIGHT: accent_button("+ Install version") → opens install_modal state

VERSION CARD GRID:
  Use ui.columns(2, ...) with 10px gap. Each card: card_frame() + 8px rounding.
  Active version card: paint 3px ACCENT left border via:
    let r = card.response.rect;
    ui.painter().line_segment([r.left_top(), r.left_bottom()],
      Stroke::new(3.0, Colors::ACCENT));

  CARD CONTENT for PhpVersion v:
    ROW 1: ui.horizontal(|ui| {
      RichText::new(format!("PHP {}", v.version)).size(19.0).color(TEXT_PRIMARY).strong()
      if v.is_active: small pill label "active" in ACCENT_DARK bg
      ui.with_layout(right_to_left): status_dot(ui, fpm_status)
    })
    ROW 2: format!("{} · FPM {} · {} extensions", v.full_version, fpm_str, ext_count)
      RichText 11px TEXT_TERTIARY
    ROW 3 (8px top margin): action buttons
      if v.is_active:
        ghost_button("ⓘ phpinfo()") → OpenPanel(PhpInfo) + set selected version
        ghost_button("↺ Restart FPM") → RestartPhpFpm(v.version)
      else:
        accent_button("Set global") → SwitchGlobalPhp(v.version)
        ghost_button("▶ Start FPM") → ... or "↺ Restart FPM" if running
        danger_button("🗑") → with confirm dialog

  INSTALL SLOT (if versions.len() < 4):
    Dashed-border card:
    painter.rect_stroke(rect, 8.0, Stroke::new(0.5, Colors::BORDER_MED))
    Centered: "+" 20px TEXT_TERTIARY + "Install PHP X.X" 12px TEXT_TERTIARY
    Clicking: send InstallPhpVersion with next logical version

══════════════════════════════════════════════════════════════
PART C — DESIGN: PHP EXTENSIONS PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/php_extensions.rs
pub fn render(ui, state, cmd_tx):

TOOLBAR: php version ComboBox | search TextEdit flex | "Show core" Checkbox
  ComboBox: Colors::CARD bg, 0.5px BORDER_MED stroke, 13px TEXT_SECONDARY
  TextEdit: Colors::CARD bg, placeholder "Search extensions…"

TABLE (egui TableBuilder):
  .column(Column::exact(24))   // toggle square
  .column(Column::remainder()) // name
  .column(Column::exact(70))   // type badge
  .column(Column::exact(80))   // version
  .column(Column::exact(70))   // enabled square

  TABLE HEADER: 11px TEXT_TERTIARY, bottom border 0.5px BORDER
  BODY ROWS (row height 32px):
    Toggle cell: 10×10 rect, fill=ACCENT if enabled else BORDER, rounding=2px
      Clickable → Enable/DisableExtension
    Name: RichText 13px TEXT_PRIMARY
    Type badge: tiny pill using framework_badge_colors logic but for types:
      Core=(CARD,TEXT_TERTIARY), Bundled=(CARD teal tinted, ACCENT), PECL=(CARD blue tinted, INFO)
    Version: 11px TEXT_TERTIARY
    Enabled column: same square as left toggle (visual consistency)

  ROW HOVER: fill entire row background with CARD_HOVER via painter.rect_filled
  ROW CLICK: expand detail below table (extra 48px row):
    Extension name bold + description stub + "Install via apt" ghost_button

══════════════════════════════════════════════════════════════
PART D — DESIGN: PHP INI PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/php_ini.rs
pub fn render(ui, state, cmd_tx):

TOOLBAR: version ComboBox | IniType radio (CLI / FPM) | "Raw edit" toggle
  All using Colors::CARD bg and 0.5px borders.

SECTION VIEW (default):
  Left 160px: section list (SidePanel or manual columns)
    Each section: Selectable label 13px. Active: ACCENT_DARK bg, white text.
  Right: ScrollArea containing key-value table
    Key col: 200px, RichText 13px TEXT_PRIMARY
    Value col: flex, TextEdit Colors::CARD bg, 0.5px BORDER
    Below value: if validation error → RichText "⚠ {msg}" 11px DANGER

RAW EDIT MODE: full-height TextEdit with monospace 13px
  Colors::DEEP_BG background, Colors::TEXT_PRIMARY text, 4px inner margin

BOTTOM BAR (44px, divider above):
  "Save" accent_button (disabled when no changes, grayed: CARD fill, TEXT_TERTIARY text)
  "Revert" ghost_button
  "Last saved: {time}" 11px TEXT_TERTIARY right-aligned

══════════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 2
══════════════════════════════════════════════════════════════
  □ PHP Versions panel shows cards with left accent border on active version
  □ "Set global" dispatches SwitchGlobalPhp and refreshes active version
  □ Extensions table toggle enables/disables with visual state change
  □ INI section tree navigable; selected section highlights in ACCENT_DARK
  □ INI validation errors show in DANGER color below affected fields
  □ All panel backgrounds use Colors::SURFACE, cards use Colors::CARD
  □ Zero hardcoded colors in any of the three panel files
```

---

## Phase 3 — Sites & Nginx + Sites/Parks/Nginx UI
**Weeks 7–9 · Valet config reader, site scanner, framework detection, Sites/Parks/Nginx panels**

```
Phases 1–2 complete. Implement site management logic and the three site panels.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → Sites panel, Parks panel, Nginx panel.
  Also read src/ui/theme.rs Colors::*.
                  The interactive mockup shows Sites as the reference visual.

══════════════════════════════════════════════════════════════
PART A — ARCHITECTURE
══════════════════════════════════════════════════════════════

[Include all architecture tasks from original Phase 3 prompt:]
- Valet config reader (src/valet/config_reader.rs)
- Site scanner (src/valet/site_scanner.rs) with DetectedFramework enum
- Framework detector (22 frameworks via file probes)
- Nginx site config (src/nginx/site_manager.rs)
- File system watcher (src/valet/watcher.rs) with inotify
- New AppState fields: valet_config, sites, nginx_configs, selected_site
- AppCommand variants: RefreshSites, ParkDirectory, ForgetDirectory, LinkSite,
  UnlinkSite, SecureSite, UnsecureSite, IsolateSite, UnisolateSite,
  OpenSiteInBrowser, OpenSiteInEditor, OpenSiteInFileManager,
  SaveNginxConfig, ReloadNginx, ToggleFavoriteSite

══════════════════════════════════════════════════════════════
PART B — DESIGN: SITES PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/sites.rs
pub fn render(ui, state, cmd_tx):

TOOLBAR (44px):
  LEFT: "Sites" 15px TEXT_PRIMARY weight 500
  RIGHT (gap 6px):
    TextEdit 140px, Colors::CARD bg, placeholder "Search sites…"
    ComboBox "All types" / "Parked" / "Linked" / "Proxy"
    ComboBox "All PHP" / one per installed version
    accent_button("+ Link site") → triggers link site modal
    ghost_button("↺") → RefreshSites

FILTER + SORT LOGIC (before rendering):
  Filter: domain/path contains search AND type/php matches filters
  Sort: is_favorite DESC then name ASC (favorites always first)

TABLE (egui TableBuilder with BODY rows at 32px height):
  Columns: exact(26) | exact(170) | remainder | exact(78) | exact(96) | exact(36) | exact(32)
  HEADER: each 11px TEXT_TERTIARY, bottom 0.5px BORDER

  ROW (per ValetSite s):
    Col 1 — star:
      Label "★" in WARNING if s.is_favorite else "☆" in TEXT_TERTIARY
      Clickable → ToggleFavoriteSite(s.name)
    Col 2 — domain:
      RichText s.domain 13px Colors::ACCENT underline on hover
      Clicking → OpenSiteInBrowser
    Col 3 — path:
      Truncate to 28 chars + "…", 11px TEXT_TERTIARY
    Col 4 — PHP:
      If None: global active version in TEXT_TERTIARY
      If Some: version + " ★" in WARNING color (isolated)
      Rendered as ComboBox (installed versions) → IsolateSite on change
    Col 5 — framework:
      framework_badge(ui, &s.framework)
    Col 6 — TLS:
      if s.is_secured:
        Check SslCertInfo for this domain:
          days > 30: 🔒 icon Colors::ACCENT
          7-30 days: 🔒 icon Colors::WARNING + tooltip "Expires in Nd"
          < 7 days: 🔒 icon Colors::DANGER
      else: 🔓 icon Colors::TEXT_TERTIARY
    Col 7 — menu:
      ghost_button("⋮") → popup below:
        "Open in browser" → OpenSiteInBrowser
        "Open in editor" → OpenSiteInEditor
        "Open in file manager" → OpenSiteInFileManager
        separator (0.5px BORDER line)
        "Edit Nginx config" → OpenPanel(Nginx) + select this site
        "Edit .env" → OpenPanel(EnvEditor) + select this site
        "Edit env vars (.valet-env.php)" → ...
        separator
        if s.is_secured: "Unsecure" else "Secure" → SecureSite/UnsecureSite
        separator
        "Unlink site" in DANGER color → UnlinkSite
        For WordPress/Bedrock: "Destroy site" in DANGER → wp valet destroy

  ROW HOVER: painter.rect_filled for entire row with Colors::CARD_HOVER
  EMPTY STATE: if no visible sites:
    Centered column: 📁 icon 32px TEXT_TERTIARY, "No sites" TEXT_SECONDARY,
    accent_button("Link a site")

══════════════════════════════════════════════════════════════
PART C — DESIGN: PARKS PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/parks.rs:
  Header: "Parked directories" + accent "Add directory" button using rfd::FileDialog
  Add rfd = "0.14" to Cargo.toml.
  Each park entry: card_frame() containing:
    LEFT: 🗀 icon 14px TEXT_TERTIARY + path string 13px TEXT_PRIMARY
    RIGHT: ghost "📂 Open" + ghost "✕ Remove" (danger color on hover)
    Remove → ForgetDirectory(path) with confirm dialog

══════════════════════════════════════════════════════════════
PART D — DESIGN: NGINX PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/nginx.rs:
  SidePanel::left 200px: site list (SelectableLabel per site, ACCENT_DARK when selected)
  RIGHT: two tabs "Config" and "Info"
    Config tab: TextEdit monospace 13px, Colors::DEEP_BG bg, full height
      Paint line numbers in left gutter: 28px wide, TEXT_TERTIARY 11px
    Info tab: two-column table: server_name, root, FPM socket, SSL cert paths
  BOTTOM: "Save & Reload" accent_button + "Revert" ghost_button
  TOP RIGHT: ghost "↺ Reload Nginx" button (always visible)

══════════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 3
══════════════════════════════════════════════════════════════
  □ Sites table renders with correct column widths
  □ Favorites sort to top; star toggle persists in config
  □ Framework badges use the correct bg/text color pairs
  □ PHP dropdown per row changes PHP version
  □ TLS icons change color based on SSL cert expiry
  □ ⋮ context menu opens below the button
  □ Parks panel shows real parked paths from valet config
  □ Nginx editor shows raw config in monospace
  □ inotify triggers site list refresh on file system changes
```

---

## Phase 4 — App Creator: WordPress & Laravel + Wizard UI
**Weeks 10–12 · CLI tool registry, output streaming, WP-CLI, Laravel CLI, wizard panel**

```
Phases 1–3 complete. Implement the App Creator wizard.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → App Creator wizard (Steps 1–5).
  The design shows Step 2 Configure (WordPress form) as the reference visual.

══════════════════════════════════════════════════════════════
PART A — ARCHITECTURE
══════════════════════════════════════════════════════════════

[Include all architecture tasks from original Phase 4 prompt:]
- CLI tool registry (src/cli_tools/registry.rs)
- Output streamer (src/creator/output_streamer.rs)
- Project type definitions (src/creator/project_types.rs) — Phase 4 types only
- Creator state (src/state/creator_state.rs) with CreatorStep enum
- WordPress runner (src/creator/runners/wordpress.rs) — all 14 wp valet new options
- Laravel runner (src/creator/runners/laravel.rs)
- Post-install handler (src/creator/post_install.rs)
- Terminal output component (src/ui/components/terminal_output.rs)
- AppCommand: CreateApp(AppCreationRequest), CancelCreation

══════════════════════════════════════════════════════════════
PART B — DESIGN: TERMINAL OUTPUT COMPONENT
══════════════════════════════════════════════════════════════

src/ui/components/terminal_output.rs:
pub fn render(ui: &mut egui::Ui, lines: &[OutputLine], auto_scroll: bool):
  Frame::none().fill(Colors::DEEP_BG).rounding(6.0).show(ui, |ui|
    ui.set_min_height(120.0)
    ScrollArea::vertical().show(ui, |ui|
      for line in lines.iter().take(1000):
        let ts = format_hms(line.ts)
        let ts_text = RichText::new(format!("[{}] ", ts)).size(11.0).color(TEXT_TERTIARY)
          .family(FontFamily::Monospace)
        let msg_color = if line.stream == Stderr { WARNING } else { TEXT_PRIMARY }
        let msg_text = RichText::new(&line.line).size(12.0).color(msg_color)
          .family(FontFamily::Monospace)
        ui.horizontal(|ui| { ui.label(ts_text); ui.label(msg_text); })
      if auto_scroll: scroll to cursor
    )
  )

══════════════════════════════════════════════════════════════
PART C — DESIGN: APP CREATOR PANEL
══════════════════════════════════════════════════════════════

src/ui/panels/app_creator.rs:
pub fn render(ui, state, cmd_tx):

STEP INDICATOR (render for steps 1–3):
  fn step_indicator(ui, current: u8):
    ui.horizontal(|ui| {
      for i in 1..=5:
        let (bg, fg, text) = match (i.cmp(&current)) {
          Less    => (ACCENT_DEEP, ACCENT, "✓")
          Equal   => (ACCENT, ACCENT_DEEP, i.to_string())
          Greater => (CARD, TEXT_TERTIARY, i.to_string())
        };
        painter.circle_filled(center, 10.0, bg);
        painter.text(center, Align2::CENTER_CENTER, text, 10px, fg);
        if i < 5: painter.line_segment(left_right, Stroke::new(0.5, BORDER));
    })

STEP 1 — SELECT TYPE:
  step_indicator(ui, 1)
  Group tab row: ["Laravel", "WordPress", "PHP", "Node", "Static"]
    Active tab: ACCENT_DARK bg, WHITE text, rounding 4px
    Inactive: CARD, TEXT_SECONDARY, rounding 4px
    8px gap between tabs. Store active_group in creator state.
  Card grid (3 per row, 8px gap):
    Each card: Frame::default().fill(CARD).rounding(8.0).stroke(BORDER_MED).show
      Icon area: 36×36 ACCENT_DEEP bg rounding 6px + monogram or symbol
      Name: RichText 13px TEXT_PRIMARY weight 500
      Description: RichText 11px TEXT_SECONDARY (2 line clamp with truncation)
    Selected card: stroke = Stroke::new(1.5, Colors::ACCENT), bg = ACCENT_DARK
    Hover: CARD_HOVER bg
  Tools status row (bottom):
    For each required tool: "✓ {name}" in TEXT_SECONDARY (ACCENT if installed)
      or "✗ {name} — Install" in DANGER (clickable)

STEP 2 — CONFIGURE:
  step_indicator(ui, 2)
  Header row: type name weight 500 + framework_badge(ui, type.framework)
  FORM GRID (2 columns, 10px gap):
    fn form_field(ui, label: &str, required: bool, content: impl FnOnce(&mut Ui)):
      ui.vertical(|ui|
        label row: RichText 11px TEXT_TERTIARY + if required: " *" in DANGER
        content(ui)  // renders the input
        ui.add_space(2.0)
      )
    TextEdit fields: TextEdit::singleline().desired_width(f32::INFINITY)
      .frame(true) with Colors::CARD bg
    Domain preview below name: RichText "→ {name}.{tld}" 11px Colors::ACCENT
    ComboBox fields: Colors::CARD bg, TEXT_SECONDARY placeholder
    Radio groups: horizontal layout with ACCENT accent-color on selected
    DirectoryPicker: TextEdit + ghost_button("…") → rfd::FileDialog::new().pick_folder()
  BOTTOM ROW (44px, divider above):
    ghost_button("← Back") | accent_button("Next: Post-install →")

STEP 3 — POST-INSTALL:
  step_indicator(ui, 3)
  Checklist with Checkboxes and accent color:
    ☑ Link site to Valet   ☑ Secure with TLS
    ☐ Isolate PHP version → if checked: ComboBox of installed versions inline
    ☐ Open in browser      ☐ Open in editor
  BOTTOM: ghost "← Back" | accent_button("✓ Create") in Colors::ACCENT_DARK

STEP 4 — PROGRESS:
  Header: type name + site name in TEXT_SECONDARY
  CHECKLIST (vertical, 8px gap):
    Steps: Downloading | Installing deps | Configuring | Linking | Securing
    fn progress_step(label, step_status):
      icon: ○ (BORDER) pending | ⟳ (ACCENT spinning) running | ✓ (ACCENT) done | ✕ (DANGER) error
      label: TEXT_SECONDARY if pending/done, TEXT_PRIMARY if running, DANGER if error
  TERMINAL OUTPUT:
    terminal_output::render(ui, &state.creator.output_lines, true)
  TOP RIGHT: danger_button("✕ Cancel") → CancelCreation

STEP 5a — COMPLETE:
  Centered column:
    painter.circle_filled(center, 24.0, Colors::ACCENT_DARK)
    painter.text(center, "✓", 20px, Colors::ACCENT)
    "Site created!" RichText 16px TEXT_PRIMARY weight 500
    domain URL RichText 14px Colors::ACCENT (clickable → xdg-open)
    path RichText 12px TEXT_TERTIARY
    Row: accent_button("↗ Open in browser") + ghost_button("Open in editor")
      + ghost_button("← New app") → reset creator state

STEP 5b — ERROR:
  ✕ circle in DANGER, "Creation failed" 16px, error message in WARNING monospace
  ghost_button("← Try again") | ghost_button("View logs → Logs panel")

══════════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 4
══════════════════════════════════════════════════════════════
  □ All 5 steps render without overflow in 1280×800 window
  □ Step indicator shows correct state (done/current/todo) at each step
  □ WordPress form shows all 14 wp valet new options with correct input types
  □ Domain preview updates live as name is typed
  □ Terminal output streams in real-time with timestamps
  □ Cancel sends SIGTERM to child process
  □ Success screen shows clickable domain link
  □ Tool status row shows red for missing tools
```

---

## Phase 5 — App Creator: All Frameworks + Node Proxy
**Weeks 13–15 · Composer runners, Node.js creation, auto-proxy, systemd dev services**

```
Phase 4 complete. Extend App Creator to all 30+ project types.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → App Creator extended panels: Node.js dev server step, completion screen variants.

[All architecture tasks from original Phase 5 prompt — Composer runner,
static HTML runner, Node.js runner with port detection, systemd dev service,
expanded project types, post-install extension for Node, prerequisite installer dialog]

══════════════════════════════════════════════════════════════
DESIGN ADDITIONS
══════════════════════════════════════════════════════════════

1. PREREQUISITE INSTALLER MODAL — src/ui/components/confirm_dialog.rs
   Create ConfirmDialog struct and render fn (from design prompt D9).
   Use for "Install missing tool?" prompts.
   Frame: centered 360×160 modal
     Title: RichText 16px weight 500 (DANGER color if is_danger)
     Message: 13px TEXT_SECONDARY
     Code block: if show_command — monospace DEEP_BG box with command text
     Buttons: cancel ghost | confirm accent/danger

2. NODE.JS PROJECT CARDS (in creator SelectType step):
   Cards for Node projects add a small info row:
     "Served via Valet proxy → localhost:{port}"
     11px TEXT_TERTIARY italic
   Uses same card_frame() but with an INFO color left border strip.

3. POST-INSTALL SUCCESS SCREEN for Node:
   Show TWO URL lines:
     "https://{name}.test" → ACCENT (the proxied domain)
     "http://localhost:{port}" → INFO (the dev server directly)
   Also show: ✓ "Systemd service created (auto-start)" if option was enabled.

ACCEPTANCE:
  □ All 30+ project types appear in correct groups
  □ Selecting a Composer framework shows minimal form (name + dir only)
  □ Node.js project creation auto-creates Valet proxy
  □ Prerequisite modal shows install command in monospace code block
  □ Node success screen shows both proxy domain and localhost URL
```

---

## Phase 6 — Advanced Features Panels
**Weeks 16–18 · Proxies, .valet-env.php, Custom Drivers, dnsmasq, Sharing, Logs, Diagnostics**

```
Phases 1–5 complete. Implement the remaining management panels.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → Proxies, dnsmasq, Sharing, Logs, Diagnostics panels.
  Read src/ui/theme.rs Colors::* for all panels.

[All architecture tasks from original Phase 6 prompt]

══════════════════════════════════════════════════════════════
DESIGN SPECIFICATIONS FOR EACH PANEL
══════════════════════════════════════════════════════════════

PROXIES PANEL (src/ui/panels/proxies.rs):
  Standard header + "Add Proxy" accent button.
  TABLE: Domain | Target URL | TLS | Status | Actions
    Status col: "Testing…" TEXT_TERTIARY → once tested: HTTP code
      200 → "200 OK" in ACCENT, 4xx → WARNING, 5xx → DANGER, fail → DANGER
    TLS col: 🔒 ACCENT if secured, "—" TEXT_TERTIARY if not
    Delete: danger icon button with confirm dialog
  "Test all" ghost button in toolbar fires all tests concurrently.

DNSMASQ PANEL (src/ui/panels/dnsmasq.rs):
  Two cards stacked:
  CARD 1 "TLD configuration":
    "Current TLD: .{tld}" 14px TEXT_PRIMARY
    TextEdit + accent_button("Apply") → runs valet domain
    (only show port section if variant == ValetLinux)
  CARD 2 "DNS resolution tester":
    TextEdit placeholder "domain.test" + ghost_button("Test →")
    Output: monospace DEEP_BG box showing dig output (answer section only)

SHARING PANEL (src/ui/panels/sharing.rs):
  Tool selector: three radio cards (ngrok | Expose | cloudflared)
    Each card 80×60px: logo placeholder + name + "Configured"/"Not set" badge
  ngrok token: Password TextEdit + ghost "Save"
  Site + Share: ComboBox + accent "▶ Share" button + danger "Stop" (when active)
  When sharing active: public URL in ACCENT clickable, animated ● dot in ACCENT

LOGS PANEL (src/ui/panels/logs.rs):
  Source tabs (horizontal pills): Nginx Error | PHP-FPM | Valet FPM | Access
  Each pill: CARD bg inactive, ACCENT_DARK bg active, 20px radius
  Log content: full-height DEEP_BG ScrollArea, monospace 12px
    Lines with "error": DANGER color
    Lines with "warn": WARNING color
    Lines with "notice": INFO color
    Other: TEXT_SECONDARY
  Top right: ghost "↺ Refresh" + ghost "Clear display"

DIAGNOSTICS PANEL (src/ui/panels/diagnostics.rs):
  "Run Diagnostics" accent button → streams valet diagnose output
  terminal_output::render() for output
  Separate section: "Trust Valet" ghost + "Restart All Services" ghost

ACCEPTANCE:
  □ Proxies panel tests proxies concurrently and shows correct HTTP status colors
  □ dnsmasq DNS tester shows dig output in monospace box
  □ Sharing panel shows public URL when valet share is running
  □ Log panel color-codes lines correctly
  □ All panels use consistent Colors::* tokens
```

---

## Phase 7 — Polish & Release
**Weeks 19–21 · System tray, notifications, toast, onboarding, components library, packaging**

```
Phases 1–6 complete. Polish the UX and prepare for distribution.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → Toast component, Settings panel, Onboarding wizard, responsive sidebar.
  Focus on §D9 (shared components) and §D10 (window + layout).

[All architecture tasks from original Phase 7 prompt:
- System tray icon
- Desktop notifications
- Dark/Light/System theme
- Onboarding wizard
- Settings panel
- Keyboard shortcuts
- Debian package
- GitHub Actions]

══════════════════════════════════════════════════════════════
DESIGN: TOAST COMPONENT — src/ui/components/toast.rs
══════════════════════════════════════════════════════════════

Toast { message: String, toast_type: ToastType, created_at: Instant }
ToastType: Success | Error | Info | Warning

pub fn render_toasts(ui: &mut egui::Ui, toasts: &mut Vec<Toast>):
  ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
    toasts.retain(|t| t.created_at.elapsed() < Duration::from_secs(4));
    for toast in toasts.iter().rev().take(4):
      let (border_color, icon) = match toast.toast_type {
        Success => (Colors::ACCENT, "✓"),
        Error   => (Colors::DANGER, "✕"),
        Warning => (Colors::WARNING, "⚠"),
        Info    => (Colors::INFO, "ⓘ"),
      };
      // Fade out in last 0.5s
      let alpha = ((4.0 - elapsed.as_secs_f32()) / 0.5).min(1.0).max(0.0);
      Frame::none()
        .fill(Colors::CARD.linear_multiply(alpha))
        .rounding(6.0)
        .inner_margin(Margin { left: 16, right: 14, top: 10, bottom: 10 })
        .show(ui, |ui| {
          ui.horizontal(|ui|
            painter.line_segment left 3px solid border_color
            label icon 14px border_color
            label message 13px TEXT_PRIMARY.linear_multiply(alpha)
            ghost X button dismiss
          )
        })
  })

══════════════════════════════════════════════════════════════
DESIGN: SETTINGS PANEL — src/ui/panels/settings.rs
══════════════════════════════════════════════════════════════

fn settings_section(ui, title: &str, content: impl FnOnce(&mut Ui)):
  section_label(ui, title)
  Frame::none().fill(Colors::CARD).rounding(8.0).inner_margin(12px).show(ui, content)
  ui.add_space(12.0)

Sections:
  "appearance": Theme radio (Dark/Light/System) + font size slider 11–20 with live preview
  "editor & tools": Three TextEdit rows (editor, terminal, file manager)
  "notifications": Checkbox list, each with 11px description below
  "app creator": Default dir picker + DB user + admin email inputs
  "danger zone": rounding=0 red-border card:
    "Uninstall Valet" danger_button (confirm required)
    "Reset settings" danger_button

BOTTOM: accent_button("Save changes") right-aligned, disabled when no changes

══════════════════════════════════════════════════════════════
DESIGN: ONBOARDING WIZARD — src/ui/panels/onboarding.rs
══════════════════════════════════════════════════════════════

Show instead of Dashboard when valet_variant is None.
Centered card (500px max-width) with step_indicator top.

Step 1 "Welcome":
  V logo mark 48px + "Welcome to Valet Manager" 18px TEXT_PRIMARY + subtitle
  accent_button("Get started →")

Step 2 "PHP check":
  List installed PHP versions with status. If none: accent "Install PHP 8.3"
  Shows apt install output in terminal widget

Step 3 "Valet check":
  "valet --version" output shown. If error: installation instructions card.
  ghost_button("I installed Valet, check again") → re-detect

Step 4 "Verify":
  Streams valet diagnose. "Continue" if clean, "Fix issues" if errors.

Step 5 "Ready":
  Summary card: variant, PHP versions, TLD. accent_button("Open dashboard")
  On click: save onboarding_complete=true, OpenPanel(Dashboard)

ACCEPTANCE:
  □ Toasts appear bottom-right and auto-dismiss in 4s with fade
  □ Toast border color matches type
  □ Settings save and apply theme immediately
  □ Onboarding shows on first run (no valet) and hides once complete
  □ cargo deb produces installable package
```

---

## Phase 8 — PHPMon Gap-Closing
**Weeks 22–25 · phpinfo(), compat checker, history, favorites, deep-link, updater**

```
Phases 1–7 complete. Close all PHPMon feature gaps.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → phpinfo panel, Compatibility panel, History panel, dashboard alert banners.
  Focus on the Dashboard panel alert banner design (§D3).

[All architecture tasks from original Phase 8 prompt]

══════════════════════════════════════════════════════════════
DESIGN: PHPINFO PANEL — src/ui/panels/phpinfo.rs
══════════════════════════════════════════════════════════════

TOOLBAR: PHP version ComboBox | TextEdit search | ghost "↺ Refresh"

SPLIT LAYOUT (SidePanel 180px left):
  LEFT — section list:
    SelectableLabel per section, ACCENT_DARK bg when active, TEXT_SECONDARY text
    On click: set selected_section
  RIGHT — filtered table:
    TableBuilder 3 cols: flex | 200px | 140px (Key | Local Value | Master Value)
    Rows where key or value contains search query (highlight match in ACCENT)
    Each row: ghost "⎘ Copy" icon at far right → clipboard

══════════════════════════════════════════════════════════════
DESIGN: COMPATIBILITY PANEL — src/ui/panels/compat.rs
══════════════════════════════════════════════════════════════

TOOLBAR: "PHP compatibility" header + accent "Check all" + last-checked timestamp

TABLE: Site | Required PHP | Active PHP | Status | Action
  Status badges:
    Compatible    → ACCENT bg, ACCENT_DEEP text, "✓ OK"
    Incompatible  → DANGER-tinted bg, DANGER text, "✕ Mismatch"
    NoRequirement → CARD, TEXT_TERTIARY, "No constraint"
    NoComposer    → CARD, TEXT_TERTIARY, "Not Composer"
  Action col (Incompatible rows only):
    accent_button("Isolate PHP X.X") → IsolateSite with suggestion version

DASHBOARD INTEGRATION:
  In dashboard.rs render_alert_banners():
    Count incompatible sites from state.compat.results
    if count > 0: alert_banner(WARNING, "⚠", "{count} sites have PHP compat issues",
      Some(("View", OpenPanel(PhpCompat))))

══════════════════════════════════════════════════════════════
DESIGN: HISTORY PANEL — src/ui/panels/history.rs
══════════════════════════════════════════════════════════════

TOOLBAR: TextEdit search + "Export CSV" ghost + "Clear all" danger

TABLE: Time | Command | Duration | Exit | Source
  Time: "2 min ago" style (use chrono humanize or manual format)
    Text: TEXT_TERTIARY 11px
  Command + Args: monospace 12px TEXT_PRIMARY, ellipsis if long
  Duration: right-aligned "1.2s" TEXT_SECONDARY
  Exit: "0" in ACCENT, non-zero in DANGER
  Source: small pill badge in CARD

ROW EXPAND (click): 8px extra height, monospace output_preview in DEEP_BG box
  "Re-run" ghost button right-aligned

ACCEPTANCE:
  □ phpinfo() sections navigate correctly; search highlights matches
  □ Compat panel shows all sites; action buttons dispatch IsolateSite
  □ Dashboard alert appears when incompatible sites exist
  □ History table shows all commands with correct timing and exit codes
  □ valet-manager:// URL opens app and dispatches action
```

---

## Phase 9 — DX Tier 1
**Weeks 26–30 · Command palette, .env editor, Artisan runner, Database manager, Dashboard alerts**

```
Phases 1–8 complete. Implement the highest-impact developer workflow features.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → Command Palette overlay, .env editor, Artisan runner, Database manager.
  Focus on §D7 (Database) and §D8 (Command Palette) — the two most complex panels.

[All architecture tasks from original Phase 9 prompt]

══════════════════════════════════════════════════════════════
DESIGN: COMMAND PALETTE — src/ui/command_palette.rs
══════════════════════════════════════════════════════════════

See design prompt D8 for full specification.
Key implementation notes for egui:

Overlay rendering:
  Use egui::Area::new(Id::new("palette_overlay"))
    .order(Order::Foreground)
    .anchor(Align2::CENTER_TOP, [0.0, 0.0])
    .fixed_pos(viewport_center):
    // Dim background behind palette
    painter.rect_filled(viewport_rect, 0.0,
      Color32::from_rgba_unmultiplied(0,0,0,100));
    // Palette card
    Frame::default()
      .fill(Colors::DEEP_BG)
      .rounding(10.0)
      .stroke(Stroke::new(0.5, Colors::BORDER_MED))
      .shadow(...)
      .show(ui, palette_content)

Search input focus:
  let resp = ui.add(TextEdit::singleline(&mut state.palette.query)
    .hint_text("Search commands, sites, PHP versions…")
    .frame(false)
    .font(FontId::proportional(15.0)));
  if state.palette.visible { resp.request_focus(); }

Category badge colors (result rows):
  PhpVersion → pill: WARNING bg tinted, WARNING text
  Site       → pill: ACCENT bg tinted, ACCENT_DEEP text
  Service    → pill: INFO bg tinted, INFO dark text
  Artisan    → CARD with left 2px ACCENT_DARK border
  Panel/Action → pill: CARD, TEXT_TERTIARY

Keyboard navigation:
  ctx.input(|i| {
    if i.key_pressed(Key::ArrowDown): selected = (selected+1).min(results.len()-1)
    if i.key_pressed(Key::ArrowUp):   selected = selected.saturating_sub(1)
    if i.key_pressed(Key::Enter) && !results.is_empty():
      dispatch results[selected].action; close palette
    if i.key_pressed(Key::Escape): close palette
  })

══════════════════════════════════════════════════════════════
DESIGN: ENV EDITOR PANEL — src/ui/panels/env_editor.rs
══════════════════════════════════════════════════════════════

TOOLBAR: site ComboBox | group tabs (APP|DB|MAIL|REDIS|Other) |
  "Show secrets" Checkbox | ghost "Raw edit" toggle

TABLE: Key | Value | ⋮
  Key col: 180px, monospace 13px TEXT_PRIMARY
  Value col: flex, TextEdit monospace 13px
    if is_secret && !show_secrets: password=true (shows ●●●●●)
    CARD bg, 0.5px BORDER
  ⋮ col: popup: "Copy value" | "Reset to example"

VALIDATION BANNERS (above table, if issues exist):
  MissingFromEnv:    red banner   "Key present in .env.example but missing here"
  MissingFromExample: amber banner "Key not in .env.example — may be unused"
  Show only unique issues, collapsed if > 3

BOTTOM: "Save" accent (disabled if unchanged) | "Revert" ghost | "Modified" badge if dirty

══════════════════════════════════════════════════════════════
DESIGN: ARTISAN PANEL — src/ui/panels/artisan.rs
══════════════════════════════════════════════════════════════

SITE SELECTOR: ComboBox (only Laravel sites) + framework_badge inline

COMMAND INPUT ROW:
  Autocomplete TextEdit (flex) + args TextEdit (200px) + accent "▶ Run"
  Autocomplete dropdown (below input, DEEP_BG, BORDER_MED):
    8 rows max, each 28px. Selected row: CARD_HOVER bg.
    Filtered via str::contains on command name.
    Arrow keys navigate, Enter selects, Escape closes.

QUICK COMMANDS ROW: scrollable horizontal row of ghost buttons
  Each: "{command}" ghost_button → fills command input and runs
  "+" ghost button → saves current command to quick list

TERMINAL OUTPUT: terminal_output::render(ui, &artisan_output, true)

HISTORY (below output):
  Compact table: command | time-ago | exit badge | duration
  Row hover: ghost "Re-run" button appears right-aligned

══════════════════════════════════════════════════════════════
DESIGN: DATABASE PANEL — src/ui/panels/database.rs
══════════════════════════════════════════════════════════════

See design prompt D7 for full specification.
Key implementation notes:

ENGINE STATUS BADGE:
  "● Connected" in ACCENT | "✕ Error: {msg}" in DANGER
  Rendered as small pill next to engine ComboBox

SPLIT LAYOUT:
  SidePanel::left(168.0).frame(Frame::none().fill(Colors::DEEP_BG)):
    DB list items: 32px tall, highlight with 2px ACCENT left border when selected
      and ACCENT-tinted background fill

TABLE ROW COUNTS:
  Use Intl-style formatting: 12_450 → "12,450"
  fn format_count(n: u64) -> String — use thousands separator

LARAVEL ACTIONS BAR (only when site.framework == Laravel):
  Horizontal row, 44px, border-top 0.5px BORDER, 10px vertical margin
  Buttons: accent "Migrate" | ghost "Migrate fresh" | ghost "Seed" | danger "Rollback"
  When running: show terminal_output widget below the bar (sliding in, min 80px max 200px)
    Cancel X button top-right of terminal

ACCEPTANCE:
  □ Command palette fuzzy-searches and shows results in all categories
  □ Ctrl+K opens palette; Escape closes; Enter dispatches action
  □ .env editor groups keys correctly and masks secrets
  □ Artisan autocomplete filters live as you type
  □ Database manager connects and lists databases for configured site
  □ Migration output streams in real-time
  □ Dashboard shows all four alert banner types when conditions are met
```

---

## Phase 10 — DX Tier 2
**Weeks 31–34 · SSL dashboard, Xdebug toggle, Mail catcher, Queue workers**

```
Phases 1–9 complete. Implement the final developer tooling panels.

DESIGN REFERENCE: Run this Claude Design command before implementing panels in this phase:
  Fetch this design file, read its readme, and implement the relevant aspects of the design.
  https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
  Implement: Valet Manager.html
  → SSL dashboard, Xdebug panel, Mail catcher, Queue workers panels.
  All follow card_frame + Colors::* — no new design tokens needed.

[All architecture tasks from original Phase 10 prompt]

══════════════════════════════════════════════════════════════
DESIGN: SSL CERTIFICATES PANEL — src/ui/panels/ssl_certs.rs
══════════════════════════════════════════════════════════════

HEADER ROW: "SSL certificates" + CA status badge right-aligned:
  "Valet CA: Installed" in ACCENT-tinted pill | "Not installed" in DANGER pill

TABLE: Domain | Issuer | Expires | Days | Status | Actions
  Status badge:
    Ok       → ACCENT bg, "✓ OK"
    Warning  → WARNING bg-tinted, "⚠ {N}d"
    Critical → DANGER bg-tinted, "✕ {N}d"
    Expired  → DANGER, "Expired"
  Sort: Critical first, Warning, Ok, Expired.
  Days col: right-aligned number in status color
  Actions: ghost "↺ Regenerate" | ghost "⬇ Export PEM"

══════════════════════════════════════════════════════════════
DESIGN: XDEBUG PANEL — src/ui/panels/xdebug.rs
══════════════════════════════════════════════════════════════

One card_frame() per installed PHP version. Stacked vertically.

CARD HEADER: "php{ver}" weight 500 + status badge:
  "Xdebug X.X installed" ACCENT-tinted | "Not installed" CARD+TEXT_TERTIARY

If installed:
  MODE SELECTOR (horizontal radio buttons):
    ["Off", "Debug", "Profile", "Coverage", "Trace"]
    Active: ACCENT_DARK bg, WHITE text, 4px rounding
    Inactive: CARD bg, TEXT_SECONDARY

  CONFIG ROW: IDE key ComboBox (VSCODE/PHPSTORM/NETBEANS/Custom) + port TextEdit 60px
  ACTION: if disabled → accent "Enable" | if enabled → danger "Disable"

If not installed:
  ghost_button("Install Xdebug") → apt install php{ver}-xdebug streaming

══════════════════════════════════════════════════════════════
DESIGN: MAIL CATCHER PANEL — src/ui/panels/mail_catcher.rs
══════════════════════════════════════════════════════════════

STATUS CARD (card_frame):
  Left: tool name + version 12px TEXT_SECONDARY
        SMTP port badge + HTTP port badge (CARD pills)
  Right: "● Running" ACCENT | "○ Stopped" TEXT_TERTIARY
         "▶ Start" accent button | "■ Stop" danger button

UNREAD COUNT (if running):
  Large: "14" in 28px TEXT_PRIMARY + "unread messages" 12px TEXT_TERTIARY
  ghost "Open Web UI" → xdg-open | danger "Clear Messages"

SMTP CONFIG SECTION (border-top card):
  "Configure SMTP for site:" label + site ComboBox
  "Apply to .env" accent button + ghost preview of what will be written:
    monospace DEEP_BG small box:
      MAIL_MAILER=smtp
      MAIL_HOST=localhost
      MAIL_PORT=1025

══════════════════════════════════════════════════════════════
DESIGN: QUEUE WORKERS PANEL — src/ui/panels/queue.rs
══════════════════════════════════════════════════════════════

SITE SELECTOR (only Laravel sites) + "Add worker" accent button opens modal:
  Modal: site ComboBox + connection ComboBox (redis/database) + queue TextEdit
         "Start on boot" Checkbox + "Create" button

WORKERS TABLE: Site | Queue | Connection | Status | Jobs | Failed | Actions
  Status: status_dot() + "Running"/"Stopped" TEXT_SECONDARY 12px
  Jobs/Failed: right-aligned numbers, failed in DANGER if > 0
  Actions: Start/Stop toggle + trash danger button (removes service file)

EMPTY STATE: centered "No queue workers running" + "Add worker" accent button

══════════════════════════════════════════════════════════════
FINAL INTEGRATION PASS
══════════════════════════════════════════════════════════════

After implementing all panels above:

1. Update src/ui/sidebar.rs nav_item list to include ALL 26 panels in correct sections.

2. Update Panel routing in src/app.rs CentralPanel to cover all 26 panels.

3. Update command palette index in src/ui/command_palette.rs to include entries
   for all new panels and the Xdebug enable/disable quick actions.

4. Update dashboard alert banners to include SSL expiry check from ssl_state.

5. Run final verification:
   grep -r "from_rgb(" src/ui/panels/ → should return 0 results (all via Colors::*)
   grep -r "Stroke::new(1\." src/ui/ → should return 0 (all 0.5px borders)
   cargo test --workspace → all tests pass
   cargo clippy -- -D warnings → zero warnings

══════════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 10 (Final)
══════════════════════════════════════════════════════════════
  □ All 26 panels navigate from sidebar and command palette
  □ SSL panel sorts correctly: Critical → Warning → OK → Expired
  □ Xdebug cards show correct enabled/disabled state per PHP version
  □ Mail catcher start/stop works and unread count updates every 10s
  □ Queue workers table shows live status from systemd
  □ Zero hardcoded colors in any panel file (only Colors::* constants)
  □ All borders are 0.5px Stroke
  □ cargo test --workspace passes all tests
  □ cargo clippy -- -D warnings produces zero warnings
  □ App runs on Ubuntu 22.04 LTS and matches the interactive mockup design
```

---

## Design Validation Checklist

Run this after each phase to verify design consistency:

```bash
# Verify no hardcoded colors in panel files
echo "=== Hardcoded colors check ==="
grep -rn "from_rgb\|from_rgba\|Color32::from_gray" src/ui/panels/ src/ui/sidebar.rs

# Verify no 1px borders (should all be 0.5px)
echo "=== Border width check ==="
grep -rn "Stroke::new(1\." src/ui/

# Verify all panels referenced in routing
echo "=== Panel routing check ==="
grep -n "Panel::" src/app.rs | grep -v "//\|Panel::{" | wc -l
grep -n "pub enum Panel" src/state/app_state.rs | head -1

# Verify font sizes (no sizes below 11)
echo "=== Font size check ==="
grep -rn "\.size([0-9]\." src/ui/ | grep -E "\.size\([0-9]\."

# Build + lint
cargo build --workspace 2>&1 | grep "^error"
cargo clippy --workspace -- -D warnings 2>&1 | grep "^error"
```

---

## Quick Reference: Design Token Usage in egui

```rust
// Panel background (set in CentralPanel frame)
Frame::none().fill(Colors::SURFACE)

// Card
theme::card_frame().show(ui, |ui| { ... })

// Divider line
let r = ui.available_rect_before_wrap();
ui.painter().line_segment(
    [r.left_top(), r.right_top()],
    Stroke::new(0.5, Colors::BORDER)
);
ui.add_space(0.5);

// Section label
theme::section_label(ui, "management");

// Status dot
theme::status_dot(ui, &service.status);

// Framework badge
theme::framework_badge(ui, &site.framework);

// Accent button
if theme::accent_button(ui, "+ Add site").clicked() { ... }

// Ghost button
if theme::ghost_button(ui, "↺ Refresh").clicked() { ... }

// Danger button
if theme::danger_button(ui, "✕ Remove").clicked() { ... }

// Table row hover highlight
if row_response.hovered() {
    ui.painter().rect_filled(row_rect, 0.0, Colors::CARD_HOVER);
}

// Monospace text
ui.label(RichText::new(content).family(FontFamily::Monospace).size(13.0));

// Muted caption
ui.label(RichText::new(label).size(11.0).color(Colors::TEXT_TERTIARY));
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
