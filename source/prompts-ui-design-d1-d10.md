> ⚠  **Use the live design file — not just this document.**
>
> Paste this command into Claude:
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```
>
> The prompts D1–D10 below are the offline fallback spec.

---

# Valet Manager — UI Design Prompts for Claude Code
## Complete Design System & Panel Implementation Guide

> These prompts implement the visual design layer in egui/eframe.
> Run from the project root after Phase 1 foundation is complete.
> Each prompt builds on the previous — run them in order.

---

## Design Brief (read before all prompts)

Valet Manager is a full-window Linux desktop application targeting developers.
The visual language is: **dark-first, teal-accented, information-dense but uncluttered**.

### Brand colours
```
Background deep:    #0E1613   (app bg, sidebar)
Background surface: #162019   (panel background)
Background card:    #1C2A26   (cards, inputs)
Border:             rgba(255,255,255,0.07)
Text primary:       #D8EDE6
Text secondary:     #6E9488
Text tertiary:      #3A5550
Accent (teal):      #5DCAA5   (active, links, success)
Accent dark:        #0F6E56   (active nav bg, buttons)
Accent darkest:     #04342C   (icon backgrounds)
Danger:             #E24B4A
Warning:            #EF9F27
Info:               #378ADD
```

### Typography
- Font: system-ui stack (egui default fonts)
- Sidebar labels: 12px regular
- Panel headers: 15px medium (weight 500)
- Body / table: 13px regular
- Captions / labels: 11px regular
- Monospace (INI, Nginx, terminal): 13px `Monospace`

### Spacing scale
- 4px micro, 8px small, 12px medium, 16px base, 24px large, 32px section

### Component rules
- Sidebar nav item height: 28px, icon 16px, gap 8px
- Panel header height: 44px
- Table row height: 32px
- Card padding: 12px 14px
- Border radius: 6px cards, 4px inputs, 8px large cards
- Active sidebar item: #0F6E56 fill, white text, no border radius
- All borders: 0.5px not 1px
- Status dots: 7px diameter circles

---

## Prompt D1 — Design System & Theme Engine
**Implement the complete egui visual theme**

```
You are implementing the design system for Valet Manager, a dark-themed
Linux desktop application built with egui/eframe.

Read src/ui/theme.rs (currently a stub) and replace it with a complete
implementation.

BRAND COLOURS (use these exact hex values as Color32 constants):
  DEEP_BG:      Color32::from_rgb(0x0E, 0x16, 0x13)   // sidebar, titlebar
  SURFACE:      Color32::from_rgb(0x16, 0x20, 0x19)   // panel content area
  CARD:         Color32::from_rgb(0x1C, 0x2A, 0x26)   // cards, inputs, table bg
  CARD_HOVER:   Color32::from_rgb(0x20, 0x30, 0x2B)   // card hover state
  BORDER:       Color32::from_rgba_unmultiplied(255, 255, 255, 18)  // 0.07 alpha
  BORDER_MED:   Color32::from_rgba_unmultiplied(255, 255, 255, 28)  // 0.11 alpha
  TEXT_PRIMARY:   Color32::from_rgb(0xD8, 0xED, 0xE6)
  TEXT_SECONDARY: Color32::from_rgb(0x6E, 0x94, 0x88)
  TEXT_TERTIARY:  Color32::from_rgb(0x3A, 0x55, 0x50)
  ACCENT:       Color32::from_rgb(0x5D, 0xCA, 0xA5)   // active states, links
  ACCENT_DARK:  Color32::from_rgb(0x0F, 0x6E, 0x56)   // active nav bg, primary btns
  ACCENT_DEEP:  Color32::from_rgb(0x04, 0x34, 0x2C)   // icon bg
  DANGER:       Color32::from_rgb(0xE2, 0x4B, 0x4A)
  WARNING:      Color32::from_rgb(0xEF, 0x9F, 0x27)
  INFO:         Color32::from_rgb(0x37, 0x8A, 0xDD)
  SUCCESS:      Color32::from_rgb(0x5D, 0xCA, 0xA5)   // same as ACCENT

TASK — implement these in src/ui/theme.rs:

1. pub fn apply(ctx: &egui::Context, theme: &str)
   Calls apply_dark() — dark is always the default; light is deferred to Phase 7.

2. pub fn apply_dark(ctx: &egui::Context)
   Set ctx.set_visuals() with a fully customised Visuals:
   visuals.dark_mode = true
   visuals.window_fill = SURFACE
   visuals.panel_fill = SURFACE
   visuals.faint_bg_color = CARD
   visuals.extreme_bg_color = DEEP_BG
   visuals.code_bg_color = CARD
   visuals.override_text_color = Some(TEXT_PRIMARY)
   visuals.window_stroke = Stroke::new(0.5, BORDER)
   visuals.widgets.noninteractive.bg_fill = CARD
   visuals.widgets.noninteractive.fg_stroke = Stroke::new(0.5, TEXT_TERTIARY)
   visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, BORDER)
   visuals.widgets.inactive.bg_fill = CARD
   visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY)
   visuals.widgets.inactive.bg_stroke = Stroke::new(0.5, BORDER_MED)
   visuals.widgets.hovered.bg_fill = CARD_HOVER
   visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY)
   visuals.widgets.hovered.bg_stroke = Stroke::new(0.5, BORDER_MED)
   visuals.widgets.active.bg_fill = ACCENT_DARK
   visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE)
   visuals.widgets.open.bg_fill = CARD_HOVER
   visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0x5D, 0xCA, 0xA5, 40)
   visuals.selection.stroke = Stroke::new(1.0, ACCENT)
   visuals.hyperlink_color = ACCENT
   visuals.warn_fg_color = WARNING
   visuals.error_fg_color = DANGER

3. pub struct Colors (a zero-size struct acting as namespace)
   Expose all brand Color32 constants as associated constants.

4. pub fn status_color(status: &ServiceStatus) -> egui::Color32
   Running → Colors::ACCENT, Stopped → Colors::DANGER, Failed → Colors::DANGER,
   Unknown → Colors::TEXT_TERTIARY

5. pub fn framework_color(fw: &DetectedFramework) -> (egui::Color32, egui::Color32)
   Returns (bg, text) pairs for framework badges:
   Laravel → (amber tinted), WordPress → (blue tinted), Symfony → (teal tinted),
   Proxy → (transparent, tertiary), Unknown → (transparent, tertiary)
   Use the brand palette mixing CARD with the semantic color at low opacity.

6. pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response
   Renders a styled accent-background button: ACCENT_DARK fill, white text.
   Add left padding of 12px, right 12px. Border radius 4px (rounding).

7. pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response
   CARD fill, BORDER_MED stroke, TEXT_SECONDARY text.

8. pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response
   Transparent fill, DANGER stroke and text.

9. pub fn status_dot(ui: &mut egui::Ui, status: &ServiceStatus)
   Paints a 7×7 circle with status_color(). Use ui.painter().circle_filled().

10. pub fn framework_badge(ui: &mut egui::Ui, framework: &DetectedFramework)
    Small pill with framework name. Uses framework_color().
    ui.painter().rect_filled() then ui.label() inside the rect.
    Text 11px, padding 2px 7px, radius 20px.

Call theme::apply(ctx, &config.theme) in ValetManagerApp::update() on every
frame (egui re-applies each frame; this is correct and efficient).
```

---

## Prompt D2 — Sidebar Navigation
**Implement the persistent left sidebar**

```
You are implementing the sidebar component in src/ui/sidebar.rs.
The theme is complete. Read src/ui/theme.rs and src/state/app_state.rs first.

The sidebar is 196px wide, rendered inside a side panel (egui::SidePanel::left).

IMPLEMENT pub fn render(
  ui: &mut egui::Ui,
  state: &AppState,
  cmd_tx: &tokio::sync::mpsc::Sender<AppCommand>
)

STRUCTURE (render in this order):

1. LOGO AREA (height 50px)
   Use ui.horizontal() with height 50px.
   Left: 28×28 rounded rect (ACCENT_DEEP fill, rounding 7.0) containing the
     V mark — paint with ui.painter():
       Outer triangle filled WHITE
       Three teal (ACCENT) horizontal lines below
   Right: "Valet Manager" in TEXT_PRIMARY 13px medium,
     below: valet variant + TLD in TEXT_TERTIARY 10px
   Bottom: horizontal rule (0.5px BORDER line)

2. NAV SECTIONS
   Sections: ["management", "development", "tools"]
   For management: [Dashboard, PHP versions, PHP extensions, PHP INI, phpinfo(),
     PHP compat, Sites, Parks, Nginx, Proxies, dnsmasq, SSL certs]
   For development: [Database, .env editor, Artisan, Queue workers, Xdebug, Mail catcher]
   For tools: [Sharing, Drivers, Logs, History, Diagnostics, Settings]

   SECTION HEADER: 10px TEXT_TERTIARY, letter_spacing 0.1, uppercase, left pad 8px,
     top margin 10px, bottom margin 4px.

   NAV ITEM: fn nav_item(ui, icon_name, label, panel, active_panel, cmd_tx)
     Height: 28px. Horizontal layout: 16px icon, 8px gap, label text.
     Active: ACCENT_DARK fill (full width, no border radius), WHITE text.
     Inactive: transparent fill, TEXT_SECONDARY text.
     Hover: Color32::from_rgba_unmultiplied(255,255,255,8) fill.
     On click: cmd_tx.try_send(AppCommand::OpenPanel(panel)).
     Icons (use egui_extras RichText with icon font, or draw with simple shapes):
       Dashboard → "⊞" | PhpVersions → "PHP" | Sites → "◈"
       Note: if icon font unavailable, use a 2-letter monogram in ACCENT color.

3. SERVICE STATUS (bottom, push with ui.with_layout(Layout::bottom_up()))
   Border top 0.5px BORDER, padding 10px 8px.
   Section label: "services" in 10px TEXT_TERTIARY.
   For each service in state.services:
     Row: 7px status dot, service name (12px TEXT_SECONDARY).
     For mailpit: show " · N unread" in WARNING color if unread > 0.

4. SPACING
   Add ui.add_space(2.0) between nav items (no gap between section label and first item).

ACCEPTANCE:
- Sidebar renders without panics on empty state
- Active panel highlights correctly
- Clicking any nav item dispatches OpenPanel command
- Service dots update as state.services changes
- Logo V mark is recognisable at 28px
```

---

## Prompt D3 — Dashboard Panel
**Implement the main dashboard with alert system**

```
You are implementing src/ui/panels/dashboard.rs.
Read theme.rs, app_state.rs, and the existing stub before writing.

IMPLEMENT pub fn render(
  ui: &mut egui::Ui,
  state: &AppState,
  cmd_tx: &tokio::sync::mpsc::Sender<AppCommand>
)

PANEL HEADER (44px, horizontal rule below):
  Left: "Dashboard" in 15px TEXT_PRIMARY medium
  Sub: "PHP {active_php} · Valet {variant} · {site_count} sites" in 12px TEXT_SECONDARY
  Right: ghost "Refresh" button with ↺ icon, accent "⌘K" button

ALERT BANNERS (render at top, before stats, variable height):
  Each banner: full-width horizontal strip, 3px left border, 8px padding, 12px font.
  Flex row: icon | message text | action button (right-aligned, margin-left: auto).
  Background: semantic color at 7% opacity (mix with SURFACE).
  Types in priority order:
    1. Danger (red): SSL expiry < 7 days → "{domain} SSL expires in {N} days"
       Action: "Regenerate" → SecureSite(domain)
    2. Warning (amber): PHP compat issues > 0 → "N sites have compatibility issues"
       Action: "View" → OpenPanel(Compatibility)
    3. Warning (amber): SSL expiry 7–30 days → "{N} certs expiring soon"
       Action: "View" → OpenPanel(SslCerts)
    4. Info (blue): update available → "Version {ver} available"
       Action: "Update" → CheckForUpdates | "Skip" → SkipVersion
  Only show banners when condition is true. No banner = no empty space.

STATS ROW (4 metric cards, equal width, 8px gap):
  Card style: CARD bg, rounding 8px, padding 10px 12px.
  Label: 10px TEXT_TERTIARY above. Value: 22px TEXT_PRIMARY medium below.
  Cards: Active PHP (value in ACCENT) | Sites count | Services (N/total, ACCENT) | TLD

SERVICE GRID (2 columns, 8px gap):
  Section label: 11px TEXT_TERTIARY medium, 12px top margin.
  Each service card: CARD bg, rounding 8px, padding 10px 14px.
  Left: status_dot + service name (13px) + status text (10px TEXT_TERTIARY below).
  Right: ghost icon button (restart/open depending on service).
  For mailpit: right button shows external-link icon → sends OpenMailCatcherUI.
  Card has 1px left border in status colour (Stroke on the left edge via painter).

QUICK ACTIONS ROW (below services, 12px top margin):
  Row of ghost buttons: [New App →], [phpinfo()], [Run migrations], [Restart all]
  Each sends the relevant AppCommand.

ACCEPTANCE:
- Alert banners appear/disappear based on state conditions
- Stats update live as AppState changes
- Service restart buttons dispatch commands
- "⌘K" opens command palette (OpenCommandPalette)
```

---

## Prompt D4 — Sites Panel
**Implement the full sites management table**

```
You are implementing src/ui/panels/sites.rs.
Read theme.rs, valet/site_scanner.rs, and app_state.rs first.

IMPLEMENT pub fn render(
  ui: &mut egui::Ui,
  state: &AppState,
  cmd_tx: &tokio::sync::mpsc::Sender<AppCommand>
)

TOOLBAR (horizontal, 44px, border-bottom):
  Left: "Sites" header.
  Right row (gap 6px): search TextEdit (140px wide), type ComboBox, PHP ComboBox,
    accent "Link site" button, ghost "Refresh" button.
  Search/filters stored in UiState.sites_search and UiState.sites_filter.

FILTER LOGIC (apply before rendering):
  Filter sites where:
    domain.contains(search_query) OR path.to_string().contains(search_query)
    AND (type_filter == All OR site.site_type matches filter)
    AND (php_filter == All OR site.php_version matches filter)
  Sort: is_favorite DESC (favorites first), then name ASC.

TABLE (egui::TableBuilder):
  Column widths: 26 | 170 | 180 | 80 | 96 | 36 | 36
  Headers: ★ | Domain | Path | PHP | Framework | TLS | ⋮
  Header row: 11px TEXT_TERTIARY, no border just a bottom line.

  ROW RENDERING (one row per visible ValetSite):
  ★ column: TEXT_WARNING if favorite, TEXT_TERTIARY otherwise.
    Clickable → ToggleFavoriteSite(site.name)
  Domain: ACCENT color (13px). Hovering shows underline. Click → OpenSiteInBrowser.
  Path: truncate to 28 chars + "…" from right. TEXT_TERTIARY 11px.
  PHP column:
    If php_version is None: show global active version in TEXT_TERTIARY.
    If Some: show version number in WARNING + "★" suffix (indicating isolated).
    Render as egui::ComboBox with all installed versions.
    On change: dispatch IsolateSite { site, version } or UnisolateSite if reset.
  Framework: call theme::framework_badge(ui, &site.framework).
  TLS column:
    Secured: lock icon in ACCENT.
    Expiry < 7 days: lock icon in DANGER + days remaining tooltip.
    Expiry 7–30 days: lock icon in WARNING.
    Not secured: open-lock icon in TEXT_TERTIARY.
  ⋮ column: ghost icon button. On click: show popup menu (egui::popup_below_widget):
    "Open in browser" | "Open in editor" | "Open in file manager"
    Separator
    "Edit Nginx config" | "Edit .env" | "Edit site env vars"
    Separator
    "Secure" / "Unsecure" (toggle based on is_secured)
    "Change PHP" (submenu showing versions)
    Separator
    "Unlink site" (red text)
    For WordPress sites: "Destroy site" (red text, runs wp valet destroy)

  ROW HOVER: CARD_HOVER background (set via painter rect before cells).

EMPTY STATE:
  If no sites after filtering: centered column with folder icon (32px TEXT_TERTIARY),
  "No sites found" in TEXT_SECONDARY, and "Link a site" accent button below.

ACCEPTANCE:
- Favorites sort to top and persist via ToggleFavoriteSite command
- PHP dropdown changes are dispatched correctly
- Context menu opens and each item dispatches the right command
- Empty state shows when search filters all results
- Framework badge colors match brand system
```

---

## Prompt D5 — PHP Versions & Extensions Panels
**Implement PHP version cards and extension manager**

```
You are implementing src/ui/panels/php_versions.rs and src/ui/panels/php_extensions.rs.

--- PHP VERSIONS PANEL ---
IMPLEMENT pub fn render(ui, state, cmd_tx)

HEADER: "PHP versions" + "Install version" accent button (right-aligned)
  Install button: opens a small modal with a TextEdit for version + confirm button.

VERSION CARD GRID (2 columns, 10px gap):
  Each card: CARD bg, rounding 8px, padding 12px 14px, min-height 96px.
  Active version card has a 3px left border in ACCENT (no border-radius on left side).

  CARD CONTENT:
  Top row:
    Left: "{PHP X.Y}" in 19px TEXT_PRIMARY medium.
      "active" badge (ACCENT_DARK fill, ACCENT light text, 10px) if is_active.
    Right: service status dot (7px).
  Middle: "{full_version} · FPM {status} · {extension_count} extensions"
    TEXT_TERTIARY 11px.
  Bottom row (8px top margin): action buttons.
    Inactive versions: accent "Set global" button + ghost "Start FPM" / "Restart FPM".
    Active version: ghost "phpinfo()" button + ghost "Restart FPM" button.
    All versions: ghost trash icon button (danger color).
    "Set global" dispatches SwitchGlobalPhp(version).

  INSTALL SLOT (if < 4 versions shown):
    Dashed-border card (BORDER color dashed stroke), centered content:
    Plus icon (TEXT_TERTIARY 20px) + "Install PHP 8.4" text (12px TEXT_TERTIARY).
    Clickable → pre-fills install modal with next logical version.

--- PHP EXTENSIONS PANEL ---
IMPLEMENT pub fn render(ui, state, cmd_tx)

TOOLBAR: PHP version ComboBox (left) | search TextEdit (flex) | "Show core" toggle.

TABLE (egui::TableBuilder, fixed columns):
  Widths: 24 | flex | 70 | 80 | 70
  Headers: (toggle) | Name | Type | Version | Enabled
  Toggle column: 10×10 colored square (ACCENT if enabled, BORDER if disabled).
    Clickable → Enable/DisableExtension command.
  Name: 13px TEXT_PRIMARY.
  Type badge: Core (gray), Bundled (teal dim), PECL (blue dim) using CARD bg variants.
  Version: TEXT_TERTIARY 11px.
  Enabled: same colored square as left column for consistency.

  ROW CLICK: Show detail panel below table (slide down, fixed 64px height).
    Detail: extension name bold + description (stub) + "Install via apt" ghost button.

SEARCH: filter by extension name contains query (case-insensitive).
"Show core" toggle: hides rows where extension_type == Core when off.

ACCEPTANCE:
- Active PHP card has left teal border accent
- Inactive cards have "Set global" button
- Extension enable/disable dispatches the right command
- Core extensions hidden by default, shown with toggle
```

---

## Prompt D6 — App Creator Wizard
**Implement the 5-step project creation wizard panel**

```
You are implementing src/ui/panels/app_creator.rs.
Read src/creator/project_types.rs and src/state/creator_state.rs first.

IMPLEMENT pub fn render(ui, state, cmd_tx)

STEP INDICATOR (render at top of every step except Progress/Complete):
  Horizontal row of 5 numbered circles connected by lines.
  Circle states: done (ACCENT_DEEP fill, ACCENT text ✓) |
    current (ACCENT fill, ACCENT_DEEP text) | todo (CARD fill, BORDER stroke, TEXT_TERTIARY)
  Lines: BORDER color, height 0.5px, flex-grow between circles.

STEP 1 — Select type
  Header: "Choose project type" in 16px.
  Group tab row: [Laravel] [WordPress] [PHP] [Node] [Static]
    Active tab: ACCENT_DARK fill, white text.
    Inactive: CARD fill, TEXT_SECONDARY.
    Clicking changes active group filter.
  Card grid (3 columns, 8px gap):
    Each ProjectType card: CARD bg, rounding 8px, padding 12px, cursor pointer.
    Hover: CARD_HOVER, 0.5px ACCENT border.
    Selected: ACCENT_DARK bg, white text.
    Card contains: type icon (placeholder 24px square ACCENT_DEEP bg) + name bold +
      description (11px TEXT_SECONDARY, 2 lines max).
  Tools status footer (bottom of panel):
    "✓ wp-cli  ✓ laravel  ✗ composer" — green for installed, red for missing.
    Red items are clickable and show "Install" tooltip.

STEP 2 — Configure
  Header: selected project type name + small framework badge.
  Form layout: 2-column grid (gap 10px).
  Each field:
    Label (11px TEXT_TERTIARY) + required asterisk if required.
    Input matching field type:
      Text: TextEdit, full width, CARD bg.
      Password: TextEdit with password mask toggle eye icon.
      Select: ComboBox.
      Toggle: Checkbox row.
      DirectoryPicker: TextEdit (flex) + "…" ghost button right.
  Domain preview (below name field): "→ {name}.{tld}" in ACCENT, 11px.
  Bottom row: ghost "← Back" | accent "Next →" (disabled if required fields empty).

STEP 3 — Post-install options
  Checklist of options with Checkboxes:
    ☑ Link site to Valet  ☑ Secure with TLS  ☐ Isolate PHP version
    ☐ Open in browser after  ☐ Open in editor after
  PHP isolation: if checked, show version ComboBox inline.
  Bottom row: "← Back" | "✓ Create" accent button (green bg).

STEP 4 — Progress
  Header: project type + name.
  Progress checklist (vertical, 6px gap):
    Each step: ○ pending | ⟳ running (animated) | ✓ done | ✕ failed
    Steps: Downloading | Installing dependencies | Configuring database | Linking | Securing
    Active step has ACCENT text; done has TEXT_TERTIARY strikethrough.
  Terminal output widget (below checklist):
    DEEP_BG bg, 8px rounding, monospace 12px font, auto-scroll.
    Max height 180px (scroll).
    Stderr lines in WARNING color.
  "✕ Cancel" ghost danger button (right-aligned, top of terminal).

STEP 5a — Complete
  Centered column: large ✓ checkmark (48px ACCENT) + "Site created!" header.
  Domain URL (ACCENT, clickable, 14px) + path (TEXT_TERTIARY).
  Action row: "Open in browser" accent btn | "Open in editor" ghost btn | "← New app" ghost btn.

STEP 5b — Error
  Centered: ✕ icon (DANGER) + "Creation failed" header.
  Error message (WARNING, monospace, scrollable box if long).
  Buttons: "← Try again" | "View logs".

ACCEPTANCE:
- All 5 steps render without overflow
- Step indicator reflects CreatorState.step correctly
- Form fields match ProjectType.options dynamically
- Terminal output auto-scrolls to bottom
- Cancel dispatches CancelCreation
```

---

## Prompt D7 — Database Manager Panel
**Implement the split-pane database management interface**

```
You are implementing src/ui/panels/database.rs.
Read src/database/ modules and src/state/database_state.rs first.

IMPLEMENT pub fn render(ui, state, cmd_tx)

TOOLBAR (44px, border-bottom):
  "Database manager" header (left)
  Engine ComboBox (MySQL | PostgreSQL | SQLite) + connection badge (green ● or red ●)
  Site ComboBox (only sites with detected database configs)
  "+" accent button → dispatch CreateDatabase dialog

SPLIT LAYOUT (use egui::SidePanel::left with 168px width for db list):

LEFT PANE — database list:
  Section label: "databases" (10px TEXT_TERTIARY)
  Each row: database name (12px) + table count (10px TEXT_TERTIARY, right-aligned)
  Selected row: 2px ACCENT left border (no radius), ACCENT tinted bg.
  Hover: CARD_HOVER bg.
  Right-click menu: "Drop database" (danger) | "Export SQL"

RIGHT PANE — table list + actions:
  Label: "tables · {selected_db}" (10px TEXT_TERTIARY)
  Table (3 cols: name | rows | engine):
    table-layout: fixed, widths flex | 80 | 70
    Rows right-aligned for numbers. TEXT_TERTIARY for engine.
    Row count formatted with thousands separator (12,450 not 12450).
  LARAVEL ACTIONS BAR (only when site is detected as Laravel):
    Show if site.framework == DetectedFramework::Laravel
    Horizontal row (gap 6px, top margin 10px, padding-top, border-top):
      accent "Migrate" | ghost "Migrate fresh" | ghost "Seed" | danger "Rollback"
      Each dispatches the relevant artisan migration command.
  MIGRATION OUTPUT (shows when running):
    Collapsible terminal output widget (same as app creator Step 4 terminal).
    DEEP_BG bg, monospace 12px, 120px max height.

EMPTY STATES:
  No engine connected: centered "Connect database" with instructions.
  No databases: centered "+" icon + "Create first database" button.
  No tables: centered "Empty database".

ACCEPTANCE:
- Split pane resizes correctly
- Laravel actions bar only shows for Laravel sites
- Migration output appears and streams correctly
- Database list highlights selected item
- Table row counts format correctly with thousands separators
```

---

## Prompt D8 — Command Palette Overlay
**Implement the Ctrl+K spotlight-style command palette**

```
You are implementing src/ui/command_palette.rs.
Read src/state/app_state.rs (CommandPaletteState) and src/commands.rs first.
Add fuzzy-matcher = "0.3" to Cargo.toml if not present.

IMPLEMENT pub fn render(ctx: &egui::Context, state: &mut AppState,
  cmd_tx: &tokio::sync::mpsc::Sender<AppCommand>)

Called from ValetManagerApp::update() every frame, before panel rendering.
Only renders when state.palette.visible == true.

OPEN/CLOSE:
  In update(), detect Ctrl+K: if ctx.input(|i| i.key_pressed(Key::K) && i.modifiers.ctrl)
    → state.palette.visible = !state.palette.visible
    → if opening: rebuild palette index from current state, clear query
  Escape key: → state.palette.visible = false
  Click outside window: → state.palette.visible = false

OVERLAY BACKGROUND:
  Use egui::Area::new("palette_overlay") with order TopMost.
  Fill entire viewport with Color32::from_rgba_unmultiplied(0,0,0,100) (dimmer layer).
  Center a 520×max(300,fit_content) egui::Frame within it:
    bg: DEEP_BG, rounding 10px, border: 0.5px BORDER_MED.

SEARCH INPUT:
  Full-width TextEdit, DEEP_BG background, 15px font, TEXT_PRIMARY.
  No border on the input itself (the frame provides the visual border).
  Placeholder: "Search commands, sites, PHP versions…" in TEXT_TERTIARY.
  Request focus every frame while palette is open (TextEdit::request_focus()).
  On text change: rebuild filtered results using fuzzy matching.

RESULTS LIST:
  Render up to 8 results. Each row 36px tall.
  Selected row (state.palette.selected_index): CARD_HOVER bg.
  Row layout:
    Left: category badge (28px wide, centered): colored pill, 10px text
      PhpVersion → amber | Site → teal | Service → blue | Artisan → purple
      Panel → gray | Action → gray
    Middle: label (13px TEXT_PRIMARY) + subtitle (11px TEXT_SECONDARY below)
    Right: shortcut hint if Some (11px TEXT_TERTIARY)
  Keyboard:
    ArrowDown → selected_index = (selected_index + 1) % results.len()
    ArrowUp → (selected_index + results.len() - 1) % results.len()
    Enter → dispatch results[selected_index].action; close palette

DIVIDER after search input: 0.5px BORDER line.
Footer: "↑↓ navigate · ↵ execute · esc dismiss" centered, 11px TEXT_TERTIARY.

INDEX BUILDING — fn build_index(state: &AppState) -> Vec<PaletteResult>:
  Sources in priority order:
  1. PHP versions: "Switch to PHP {ver}" (label), "{installed} · {status}" (sub)
     Action: SwitchGlobalPhp(ver). Category: PhpVersion.
  2. Sites (max 20): "Open {domain}" → OpenSiteInBrowser
     And: "Open {domain} in editor" → OpenSiteInEditor
  3. Services: "Restart {name}" → RestartService | "Stop {name}" → StopService
  4. Panels: "Go to {panel_name}" → OpenPanel(panel) for each Panel variant
  5. Common actions: "Create new app" | "Reload nginx" | "Check for updates" |
     "Run diagnostics" | "Open settings"
  6. Artisan (if selected_site is Laravel, discovered commands):
     First 8 commands by name relevance. "artisan {cmd}" label.

FUZZY FILTERING — fn filter(index: &[PaletteResult], query: &str) -> Vec<PaletteResult>:
  Use SkimMatcherV2::default().fuzzy_match(item.label, query).
  Score: score + (if category == PhpVersion { 100 } else { 0 })
  Return top 8, sorted by score descending.
  If query is empty: return first 8 from index (default suggestions).

ACCEPTANCE:
- Ctrl+K opens the palette over the entire window
- Typing filters results in real-time
- Arrow keys move selection
- Enter dispatches the action and closes palette
- Escape closes without action
- Palette index rebuilds on state changes (php versions, sites)
```

---

## Prompt D9 — Shared Components Library
**Implement all reusable UI components**

```
You are implementing the shared component library in src/ui/components/.
These components are used across multiple panels. Implement all of them fully.

--- src/ui/components/toast.rs ---
pub struct Toast { message: String, toast_type: ToastType, created_at: std::time::Instant }
pub enum ToastType { Success | Error | Info | Warning }

pub fn render_toasts(ui: &mut egui::Ui, toasts: &mut Vec<Toast>)
  Position: bottom-right of content area using ui.with_layout(Layout::bottom_up(right_to_left))
  Each toast: CARD bg, 3px left border in type color, padding 10px 14px.
  Layout: icon (16px) + message text (13px) + dismiss X button.
  Auto-dismiss: remove toasts where created_at.elapsed() > 4 seconds.
  Max 4 visible. New toasts push old ones up (bottom-up layout handles this).
  Animate: use opacity based on remaining time (1.0 → 0.0 in last 0.5s).

pub fn push_toast(toasts: &mut Vec<Toast>, msg: &str, toast_type: ToastType)
  Create Toast and push to vec.

--- src/ui/components/terminal_output.rs ---
pub struct TerminalOutput { lines: Vec<OutputLine>, scroll_to_bottom: bool }

pub fn render(ui: &mut egui::Ui, output: &TerminalOutput)
  egui::ScrollArea::vertical() with auto-shrink false.
  DEEP_BG background (painter rect fill).
  Monospace font (egui FontId::monospace(12.0)).
  Each line: Stdout → TEXT_PRIMARY, Stderr → WARNING color.
  Timestamp prefix: "[HH:MM:SS] " in TEXT_TERTIARY.
  If scroll_to_bottom: ui.scroll_to_cursor(Some(Align::BOTTOM)).
  Max 1000 lines displayed (trim from front).

--- src/ui/components/confirm_dialog.rs ---
pub struct ConfirmDialog {
  pub title: String, pub message: String,
  pub confirm_label: String, pub cancel_label: String,
  pub is_danger: bool
}

pub fn render(ui: &mut egui::Ui, dialog: &ConfirmDialog,
  confirmed_tx: &tokio::sync::mpsc::Sender<bool>)
  egui::Window: fixed size 360×160, no title bar, modal (darkens bg).
  Title (16px, danger color if is_danger), message (13px TEXT_SECONDARY).
  Button row (right-aligned): cancel ghost btn | confirm btn (accent or danger).

--- src/ui/components/service_badge.rs ---
pub fn render(ui: &mut egui::Ui, status: &ServiceStatus, label: &str)
  Horizontal: status_dot + label text + optional uptime/PID below.

--- src/ui/components/php_version_card.rs ---
pub fn render(ui: &mut egui::Ui, version: &PhpVersion, cmd_tx: ...) -> egui::Response
  Full card rendering as specified in Prompt D5. Returns the card's Response
  so the caller can detect hover/focus.

--- src/ui/components/site_row.rs ---
pub fn render_row(table_row: &mut egui::TableRow, site: &ValetSite,
  installed_versions: &[PhpVersion], cmd_tx: ..., show_context_menu: &mut bool)
  Full row as specified in Prompt D4. Sets *show_context_menu = true on ⋮ click.

--- src/ui/components/code_editor.rs ---
pub fn render(ui: &mut egui::Ui, content: &mut String, language: CodeLanguage,
  read_only: bool) -> bool (returns true if content changed)
  egui::TextEdit::multiline with:
    font: monospace 13px
    DEEP_BG background
    TEXT_PRIMARY foreground
    Minimal padding (4px)
    Horizontal scrolling enabled
  Basic syntax highlight: scan content line by line, colorize:
    PHP: comments (;...) → TEXT_TERTIARY, section headers ([Section]) → ACCENT_DARK text
    Nginx: directives → INFO, values → TEXT_PRIMARY, comments (#) → TEXT_TERTIARY
  language param: Php | Nginx | DotEnv | PlainText

ACCEPTANCE:
- Toasts auto-dismiss after 4 seconds
- Terminal output renders monospace with correct colors
- Confirm dialog blocks underlying UI (modal behaviour)
- Code editor renders syntax-colored text
- All components compile without warnings
```

---

## Prompt D10 — Responsive Layout & Window Setup
**Implement window configuration, font loading, and responsive panel layout**

```
You are implementing the final layout configuration in src/main.rs and src/app.rs.

TASKS:

1. WINDOW SETUP in src/main.rs:
   eframe::NativeOptions {
     viewport: ViewportBuilder::default()
       .with_title("Valet Manager")
       .with_inner_size([1280.0, 800.0])
       .with_min_inner_size([900.0, 560.0])
       .with_icon(/* embedded icon */)
       .with_decorations(true)
       .with_resizable(true),
     default_theme: eframe::Theme::Dark,
     renderer: eframe::Renderer::Wgpu,
     ..Default::default()
   }

2. FONT CONFIGURATION in ValetManagerApp::new():
   Load Inter (or system default) via ctx.set_fonts():
   Add Monospace font for code/terminal panels.
   Set default proportional: FontId::proportional(13.0)
   Set default monospace: FontId::monospace(13.0)
   Call theme::apply_dark(ctx) immediately.

3. ROOT LAYOUT in ValetManagerApp::update():
   STRUCTURE:
   a. egui::TopBottomPanel::top("titlebar") — 36px: app name, PHP version, window controls
   b. egui::SidePanel::left("sidebar") — exact 196px, no resize:
      Call sidebar::render(ui, state, cmd_tx)
   c. egui::CentralPanel::default():
      Fill with SURFACE color (painter.rect_filled)
      Match state.ui.active_panel → render corresponding panel
      Overlay: if state.palette.visible → call command_palette::render(ctx, state, cmd_tx)
      Overlay: render_toasts for state.ui.toast_queue

4. TITLE BAR (top panel, 36px):
   DEEP_BG background.
   Left: traffic-light dots (10px circles: red #E24B4A, amber #EF9F27, teal #5DCAA5), gap 14px.
   Center: "Valet Manager" 12px TEXT_TERTIARY.
   Right: "PHP {active_php} · valet-linux" 11px TEXT_TERTIARY.
   Note: egui doesn't control OS window decorations. This is an in-app title bar
   rendered inside the window, not replacing the OS title bar.

5. PANEL ROUTING (central panel match block):
   Panel::Dashboard     → dashboard::render
   Panel::PhpVersions   → php_versions::render
   Panel::PhpExtensions → php_extensions::render
   Panel::PhpIni        → php_ini::render
   Panel::PhpInfo       → phpinfo::render
   Panel::Compat        → compat::render
   Panel::Sites         → sites::render
   Panel::Parks         → parks::render
   Panel::Nginx         → nginx::render
   Panel::Proxies       → proxies::render
   Panel::Dnsmasq       → dnsmasq::render
   Panel::SslCerts      → ssl_certs::render
   Panel::Database      → database::render
   Panel::EnvEditor     → env_editor::render
   Panel::Artisan       → artisan::render
   Panel::QueueWorkers  → queue::render
   Panel::Xdebug        → xdebug::render
   Panel::MailCatcher   → mail_catcher::render
   Panel::Sharing       → sharing::render
   Panel::Drivers       → drivers::render
   Panel::Logs          → logs::render
   Panel::History       → history::render
   Panel::Diagnostics   → diagnostics::render
   Panel::Settings      → settings::render

6. RESPONSIVE BEHAVIOUR:
   If window width < 1000px: sidebar collapses to icon-only mode (32px wide).
     Icons only, no labels. Tooltip on hover shows label.
   If window width < 800px: hide sidebar entirely, show hamburger menu button in title bar.
   Track window size in UiState.window_size: Vec2. Update in update().

7. GLOBAL KEYBOARD SHORTCUTS (check in update() before rendering):
   Ctrl+K   → OpenCommandPalette
   Ctrl+R   → RefreshAll
   Ctrl+,   → OpenPanel(Settings)
   Ctrl+1…9 → navigate to first 9 sidebar items (common panels)

ACCEPTANCE:
- App launches at 1280×800 with correct dark theme
- Sidebar is exactly 196px and not resizable
- Title bar shows correct PHP version from state
- All 24 panels route correctly
- Keyboard shortcuts work globally
- Window below 1000px shows icon-only sidebar
- Command palette renders above everything else
```

---

## Design Prompt for External Tools (Figma / v0 / Lovable)

Use this prompt to generate the UI design in Figma, Framer, v0.dev, or any AI design tool.

```
Create a high-fidelity UI design for "Valet Manager", a native Linux desktop
application that manages Laravel Valet PHP development environments.

VISUAL STYLE:
- Dark theme, minimal, developer-tool aesthetic
- Inspired by: VS Code sidebar + TablePlus + Linear's information density
- NOT: rounded card-heavy "SaaS dashboard" style
- Flat, precise, information-dense without feeling crowded
- Custom dark teal colour palette (not generic gray/slate)

COLOUR PALETTE:
Background: #0E1613 (deepest), #162019 (panels), #1C2A26 (cards/inputs)
Text: #D8EDE6 (primary), #6E9488 (secondary), #3A5550 (tertiary/labels)
Accent: #5DCAA5 (teal, active/selected), #0F6E56 (dark teal, buttons/active nav)
Semantic: #E24B4A (danger), #EF9F27 (warning), #378ADD (info)
Borders: rgba(255,255,255,0.07)

TYPOGRAPHY:
System font stack (Inter or SF Pro equivalent), weights 400 and 500 only.
Sidebar: 12px. Panel headers: 15px medium. Body/tables: 13px. Captions: 11px.
Code/terminal: 13px monospace.

WINDOW LAYOUT (1280×800):
- 36px custom title bar: traffic light dots | app name (center) | PHP version (right)
- 196px left sidebar: logo area + navigation sections + service status at bottom
- Central content area: full-height panel content

SIDEBAR DESIGN:
Navigation sections: MANAGEMENT | DEVELOPMENT | TOOLS
Section labels: 10px uppercase letter-spaced tertiary text
Navigation items: 28px tall, 16px icons, 8px gap, 8px horizontal padding
Active item: #0F6E56 full-width fill, white text
Inactive: transparent, secondary text
Bottom: services list with 7px coloured status dots

PANELS TO DESIGN (one artboard each):

1. DASHBOARD:
   Alert banners (warning: SSL expiry, amber: compat issues) at top
   4 metric cards in a row (Active PHP, Sites, Services, TLD)
   2×2 service cards grid with status dots and restart buttons
   Quick actions row at bottom

2. SITES TABLE:
   Filter toolbar: search + type filter + PHP filter + "Link site" button
   Table columns: ★ | Domain | Path | PHP (dropdown) | Framework badge | TLS icon | ⋮
   5 rows showing: Laravel (orange badge), WordPress (blue badge), Symfony (teal badge),
     Proxy (gray badge), WooCommerce (pink badge)
   One row with yellow "★" (favorited, sorts to top)

3. PHP VERSIONS:
   2×2 card grid. Active card has 3px left teal border.
   Each card: PHP version number large, status dot, FPM status, action buttons
   4th card: dashed "Install PHP 8.4" placeholder

4. APP CREATOR (Step 2 — Configure WordPress):
   Step indicator (5 circles, step 2 active)
   2-column form grid: Name, Directory, PHP version, Database, Admin email, Protocol
   Domain preview below name field in teal: "myblog.test"

5. DATABASE MANAGER:
   Split pane: 168px left (database list with selected highlight)
   Right: table list with row counts + Laravel actions row (Migrate, Fresh, Seed)

COMPONENTS:
- Framework badges: small pill shapes, subtle colour fills matching framework
- Status dots: 7px circles, green=running, red=stopped, amber=warning
- Toast notifications: bottom-right stack, 3px left border
- Command palette overlay: centred 520px card with search + 8 results
- Terminal output widget: #090F0D bg, monospace text, auto-scroll

INTERACTIONS TO ANNOTATE:
- Sidebar hover state
- PHP dropdown in sites table
- ⋮ context menu structure
- Command palette (Ctrl+K) overlay effect
- App creator step progression
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
