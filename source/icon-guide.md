# Valet Manager — Icon Asset Inventory

## Delivered files

| File | Size | Format | Use |
|---|---|---|---|
| `valet-manager-icon-512.svg` | 512×512 | SVG | App icon (primary), XDG hicolor theme |
| `valet-manager-wordmark.svg` | 540×80 | SVG | Horizontal lockup for splash / about dialog |
| `valet-manager-favicon.svg` | 32×32 | SVG | Browser tab / documentation site favicon |
| `valet-manager-tray.svg` | 22×22 | SVG | System tray (GTK symbolic, uses currentColor) |

---

## XDG icon installation

Place the 512×512 SVG into the hicolor theme directory so the `.desktop` entry
resolves it automatically at any resolution:

```bash
# System-wide (requires sudo)
sudo install -Dm644 valet-manager-icon-512.svg \
  /usr/share/icons/hicolor/scalable/apps/valet-manager.svg

# Per-user
install -Dm644 valet-manager-icon-512.svg \
  ~/.local/share/icons/hicolor/scalable/apps/valet-manager.svg

# Refresh icon cache
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor
```

For PNG exports at standard sizes (128, 64, 48, 32, 16), use `rsvg-convert`:

```bash
for SIZE in 256 128 64 48 32 16; do
  rsvg-convert -w $SIZE -h $SIZE valet-manager-icon-512.svg \
    -o valet-manager-${SIZE}x${SIZE}.png
  install -Dm644 valet-manager-${SIZE}x${SIZE}.png \
    /usr/share/icons/hicolor/${SIZE}x${SIZE}/apps/valet-manager.png
done
```

---

## Embedding in the Rust app (egui)

The icon is embedded at build time via `build.rs` using the `image` crate:

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=assets/icons/valet-manager-icon-512.svg");
}
```

```rust
// src/main.rs
fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/icons/valet-manager-icon-512.png");
    let icon_image = image::load_from_memory(icon_bytes).unwrap();
    let (w, h) = icon_image.dimensions();
    let rgba = icon_image.into_rgba8();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Valet Manager")
            .with_icon(Arc::new(egui::viewport::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }))
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native("Valet Manager", native_options, Box::new(|cc| {
        Ok(Box::new(ValetManagerApp::new(cc)))
    }))
}
```

---

## Tray icon (tray-icon crate)

The system tray icon uses the `tray-icon` crate. For GTK symbolic icons that
adapt to light/dark tray themes, the SVG uses `fill="currentColor"`:

```rust
// src/tray/tray_icon.rs
use tray_icon::{TrayIconBuilder, menu::Menu};

pub fn create_tray_icon() -> tray_icon::TrayIcon {
    let icon_bytes = include_bytes!("../../assets/icons/valet-manager-32.png");
    let icon_image = image::load_from_memory(icon_bytes).unwrap().into_rgba8();
    let (w, h) = icon_image.dimensions();
    let tray_icon_data = tray_icon::Icon::from_rgba(icon_image.into_raw(), w, h).unwrap();

    TrayIconBuilder::new()
        .with_menu(Box::new(build_tray_menu()))
        .with_tooltip("Valet Manager")
        .with_icon(tray_icon_data)
        .build()
        .unwrap()
}
```

---

## Design tokens (Rust constants)

```rust
// src/ui/theme.rs

pub mod brand {
    /// Icon background / darkest surface
    pub const DARK:    egui::Color32 = egui::Color32::from_rgb(0x04, 0x34, 0x2C);
    /// Primary brand teal
    pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(0x0F, 0x6E, 0x56);
    /// Accent / stack lines
    pub const ACCENT:  egui::Color32 = egui::Color32::from_rgb(0x5D, 0xCA, 0xA5);
    /// Light background tint
    pub const LIGHT:   egui::Color32 = egui::Color32::from_rgb(0xE1, 0xF5, 0xEE);
    /// Neutral dark (panels, dark variant strip)
    pub const NEUTRAL: egui::Color32 = egui::Color32::from_rgb(0x2C, 0x2C, 0x2A);
}
```

---

## Design concept

The **Valet Manager** mark is a geometric **V** letterform drawn as a triangle
with a triangular cutout (even-odd fill). The three stacked accent lines below
the V tip represent the layered services under management: **PHP-FPM**, **Nginx**,
and **dnsmasq** — each line shorter than the last, suggesting a converging,
controlled stack.

The V reads simultaneously as:
- **V**alet — the application name
- A **chevron** — forward motion, speed, CLI prompt direction
- A **parking valet stand** — the hospitality metaphor Valet was named for
- A **funnel** — all web requests flowing through a single managed entry point

The deep forest teal palette references the terminal/server aesthetic without
defaulting to the overused blue or purple developer tool conventions.

---

*Author: Al Amin Ahamed (@mralaminahamed)*
