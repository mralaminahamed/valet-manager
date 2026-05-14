# Cargo Dependencies Reference
## valet-manager/Cargo.toml — all crates with rationale

```toml
[package]
name    = "valet-manager"
version = "0.1.0"
edition = "2021"
authors = ["Al Amin Ahamed <mralaminahamed@gmail.com>"]
license = "MIT"

[dependencies]

# ── GUI ──────────────────────────────────────────────────────────────
# Immediate-mode GUI — zero runtime dependencies, wgpu backend
eframe      = { version = "0.31", features = ["default_fonts", "wgpu"] }
egui        = "0.31"
egui_extras = { version = "0.31", features = ["all_loaders"] }
# Native file picker dialog (park directory, export PEM, etc.)
rfd         = "0.14"

# ── Async runtime ────────────────────────────────────────────────────
# Background tasks: subprocess, file watching, polling, HTTP
tokio       = { version = "1", features = ["full"] }
futures     = "0.3"

# ── System ───────────────────────────────────────────────────────────
# Locate binaries in PATH (which php, which caddy, etc.)
which       = "7"
# Process info, service PID lookup
sysinfo     = "0.33"
# POSIX signal sending (SIGTERM for octane, child process cancel)
nix         = { version = "0.29", features = ["process", "signal", "fs"] }
# File system change detection (inotify on Linux → auto-refresh sites)
notify      = "7"

# ── Serialization ────────────────────────────────────────────────────
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
# App config + SiteConfig storage
toml        = "0.8"
# Preserves formatting on partial config edits
toml_edit   = "0.22"

# ── HTTP client ──────────────────────────────────────────────────────
# GitHub releases API (updater), Mailpit unread count, proxy testing
reqwest     = { version = "0.12", features = ["json"] }

# ── Database (history) ───────────────────────────────────────────────
# Command history + artisan history (bundled SQLite = no system dep)
rusqlite    = { version = "0.32", features = ["bundled"] }

# ── Templates ────────────────────────────────────────────────────────
# Nginx / Caddy / Apache / FrankenPHP config templates
handlebars  = "6"

# ── Search / matching ────────────────────────────────────────────────
# Command palette fuzzy search
fuzzy-matcher = "0.3"
# Nginx config parsing, env file parsing
regex       = "1"

# ── Error handling ───────────────────────────────────────────────────
anyhow      = "1"
thiserror   = "2"

# ── Logging ──────────────────────────────────────────────────────────
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# ── Tray + desktop notifications ─────────────────────────────────────
# System packages required at link time:
#   Debian/Ubuntu: sudo apt install libxdo-dev libdbus-1-dev
#   tray-icon -> muda links against libxdo for global menu accessibility.
tray-icon   = "0.21"
notify-rust = "4"

# ── Path utilities ───────────────────────────────────────────────────
# ~/.config, ~/Sites, home dir resolution
dirs        = "5"

# ── Date / time ──────────────────────────────────────────────────────
chrono      = { version = "0.4", features = ["serde"] }

# ── Semver (compatibility checker + updater) ─────────────────────────
semver      = "1"

# ── Security ─────────────────────────────────────────────────────────
# phpMyAdmin blowfish_secret generation, basic auth htpasswd
rand        = { version = "0.8", features = ["std"] }
# htpasswd password hashing
bcrypt      = "0.15"

# ── Archives (phpMyAdmin manual download) ────────────────────────────
flate2      = "1.0"
tar         = "0.4"


# ──────────────────────────────────────────────────────────────────────
# valet-manager-helper/Cargo.toml (privilege helper binary)
# ──────────────────────────────────────────────────────────────────────

[dev-dependencies]
tokio-test  = "0.4"
tempfile    = "3"
mockall     = "0.13"

# ── Packaging ────────────────────────────────────────────────────────
[package.metadata.deb]
maintainer    = "Al Amin Ahamed <mralaminahamed@gmail.com>"
copyright     = "2025, Al Amin Ahamed"
license-file  = ["../LICENSE", "0"]
extended-description = "Native Linux desktop GUI for managing Laravel Valet PHP development environments."
depends       = "pkexec, libgtk-3-0"
section       = "devel"
priority      = "optional"
assets        = [
    ["target/release/valet-manager",         "usr/bin/",                       "755"],
    ["target/release/valet-manager-helper",  "usr/lib/valet-manager/helper",   "755"],
    ["../packaging/valet-manager.desktop",   "usr/share/applications/",        "644"],
    ["../packaging/polkit/com.valetmanager.policy",
                                             "usr/share/polkit-1/actions/",    "644"],
    ["../assets/icons/valet-manager-icon-512.svg",
                                             "usr/share/icons/hicolor/scalable/apps/valet-manager.svg", "644"],
]
maintainer-scripts = "../packaging/deb/"
```

---

## Version pins rationale

| Crate | Version | Why pinned |
|---|---|---|
| `eframe` | 0.31 | Stable egui API; upgrade = potential visual regression |
| `egui` | 0.31 | Must match eframe exactly |
| `rusqlite` | 0.32 + bundled | Bundled SQLite avoids libsqlite3-dev system dep |
| `reqwest` | 0.12 | tokio 1.x compatible, JSON feature needed for API calls |
| `notify` | 7 | inotify backend default on Linux |
| `bcrypt` | 0.15 | Stable; htpasswd format doesn't change |
| `rfd` | 0.14 | GTK3 file dialog on Linux |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
