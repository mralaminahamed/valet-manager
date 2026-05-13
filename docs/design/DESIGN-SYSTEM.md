> **Live design file** — paste this command into Claude to implement the UI:
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```
>
> The sections below are the offline specification for the same design.

---

# Valet Manager — Design System
## Colour tokens · Typography · Components · Layout rules

---

## Colour tokens (src/ui/theme.rs → Colors::*)

```rust
// Background hierarchy
Colors::DEEP_BG      #0E1613   sidebar, titlebar, terminal output bg
Colors::SURFACE      #162019   CentralPanel fill (main content area)
Colors::CARD         #1C2A26   cards, inputs, table rows
Colors::CARD_HOVER   #20302B   interactive card / row hover state

// Borders
Colors::BORDER       rgba(255,255,255, 0.07)   default — all dividers, outlines
Colors::BORDER_MED   rgba(255,255,255, 0.11)   hover / focus emphasis

// Text
Colors::TEXT_PRIMARY   #D8EDE6   body text, table values
Colors::TEXT_SECONDARY #6E9488   labels, secondary info
Colors::TEXT_TERTIARY  #3A5550   captions, section headers, placeholders

// Brand teal
Colors::ACCENT       #5DCAA5   active states, links, success, running dots
Colors::ACCENT_DARK  #0F6E56   active nav background, primary buttons
Colors::ACCENT_DEEP  #04342C   icon mark backgrounds

// Semantic
Colors::DANGER       #E24B4A   errors, destructive actions, stopped services
Colors::WARNING      #EF9F27   warnings, isolated PHP, expiry alerts
Colors::INFO         #378ADD   info banners, Node.js project cards
```

---

## Typography

| Use | Size | Weight | Color token |
|---|---|---|---|
| Panel header | 15px | 500 | TEXT_PRIMARY |
| Section label | 10px | 500 | TEXT_TERTIARY |
| Body / table | 13px | 400 | TEXT_PRIMARY |
| Captions / meta | 11px | 400 | TEXT_TERTIARY |
| Sidebar labels | 12px | 400 | TEXT_SECONDARY (inactive), WHITE (active) |
| Monospace code / terminal | 13px Monospace | 400 | TEXT_PRIMARY |
| Alert banners | 12px | 400 | TEXT_PRIMARY |

Two weights only: 400 regular, 500 medium. Never 600 or 700.

---

## Layout constants

| Token | Value | Usage |
|---|---|---|
| Sidebar width | 196px fixed | Not resizable |
| Panel header height | 44px | Border-bottom below |
| Table row height | 32px | |
| Card padding | 12px 14px | card_frame() |
| Card border-radius (large) | 8px | version cards, config cards |
| Card border-radius (normal) | 6px | compact cards |
| Input border-radius | 4px | TextEdit, ComboBox |
| Nav item height | 28px | 16px icon, 8px gap, 12px label |
| Status dot diameter | 7px | service indicator, extension toggle |
| All borders | 0.5px Stroke | Never 1px |

---

## Component API (src/ui/theme.rs)

### Buttons

```rust
// Primary action — ACCENT_DARK fill, white text
if theme::accent_button(ui, "+ Create site").clicked() { ... }

// Secondary — CARD fill, BORDER_MED stroke, TEXT_SECONDARY text
if theme::ghost_button(ui, "↺ Refresh").clicked() { ... }

// Destructive — transparent, DANGER stroke and text
if theme::danger_button(ui, "Delete site").clicked() { ... }
```

### Status dot

```rust
// 7px circle in status_color(status)
theme::status_dot(ui, &service.status);
```

### Framework badge

```rust
// Pill with correct bg/text for the framework type
theme::framework_badge(ui, &site.framework);
```

### Card frame

```rust
// CARD fill, 8px rounding, 12px×14px inner margin
theme::card_frame().show(ui, |ui| {
    // card content
});
```

### Section label

```rust
// 10px TEXT_TERTIARY, letter-spacing, top margin
theme::section_label(ui, "management");
```

### Divider

```rust
// Full-width 0.5px BORDER line
theme::divider(ui);
```

---

## Framework badge colours

| Framework | Background | Text |
|---|---|---|
| Laravel | amber 10% | amber 600 (#BA7517) |
| WordPress / Bedrock | blue 12% | blue 600 (#185FA5) |
| Symfony | teal 10% | teal 700 (#085041) |
| Proxy | CARD | TEXT_TERTIARY |
| Other / Unknown | CARD | TEXT_SECONDARY |

---

## Service status colours

| Status | Colour |
|---|---|
| Running | Colors::ACCENT (#5DCAA5) |
| Stopped | Colors::DANGER (#E24B4A) |
| Failed | Colors::DANGER |
| Unknown | Colors::TEXT_TERTIARY |

---

## Alert banner pattern

```rust
// Used in dashboard for SSL expiry, compat issues, updates
fn alert_banner(
    ui: &mut egui::Ui,
    border_color: Color32,   // Colors::DANGER / WARNING / INFO
    icon: &str,
    message: &str,
    action: Option<(&str, AppCommand)>,
)
// Renders: full-width strip, 3px left border, 7% opacity fill, 12px text
```

---

## Spacing scale

| Name | px | egui call |
|---|---|---|
| micro | 4 | `ui.add_space(4.0)` |
| small | 8 | `ui.add_space(8.0)` |
| medium | 12 | `ui.add_space(12.0)` |
| base | 16 | `ui.add_space(16.0)` |
| large | 24 | `ui.add_space(24.0)` |
| section | 32 | `ui.add_space(32.0)` |

---

## Sidebar navigation item (active vs inactive)

```rust
// Active:   ACCENT_DARK fill, full width, no border-radius, WHITE text, 16px icon
// Inactive: transparent fill, TEXT_SECONDARY text, hover CARD_HOVER
// Clicking: cmd_tx.try_send(AppCommand::OpenPanel(panel))
```

Icons use Unicode symbols (not emoji, not icon font in Rust MVP):
`⊞ ⌥ ◻ ≡ ⓘ ✓ ◈ ⌂ ⚙ ⇆ ⬡ 🔒 ⊟ ≣ ▶ ⏭ ⊗ ✉ ↗ ◇ ⌛ ⊕`

---

## egui quick patterns

```rust
// Panel background
Frame::none().fill(Colors::SURFACE)

// Divider line
ui.painter().line_segment(
    [r.left_top(), r.right_top()],
    Stroke::new(0.5, Colors::BORDER)
);

// Row hover highlight
if row_resp.hovered() {
    ui.painter().rect_filled(row_rect, 0.0, Colors::CARD_HOVER);
}

// Monospace text
ui.label(RichText::new(content)
    .family(FontFamily::Monospace)
    .size(13.0));

// Muted caption
ui.label(RichText::new(label)
    .size(11.0)
    .color(Colors::TEXT_TERTIARY));
```

---

## Zero-hardcode rule

Every `src/ui/panels/` and `src/ui/components/` file must use `Colors::*` only.
No `Color32::from_rgb(...)` outside `src/ui/theme.rs`.

Verification:
```bash
grep -rn "from_rgb\|from_rgba" src/ui/panels/ src/ui/sidebar.rs
# Must return zero results
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
