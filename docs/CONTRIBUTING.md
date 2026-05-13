# Contributing to Valet Manager

---

## Development prerequisites

```bash
# Rust stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add clippy rustfmt

# Additional tools
cargo install cargo-deb cargo-watch
sudo apt install pkg-config libssl-dev libgtk-3-dev   # Ubuntu / Debian
sudo dnf install openssl-devel gtk3-devel              # Fedora

# Optional: for building .deb packages
cargo install cargo-deb
```

---

## Workspace layout

```
valet-manager/                  root workspace
├── Cargo.toml                  workspace definition
├── valet-manager/              main GUI binary
│   ├── Cargo.toml
│   ├── src/
│   └── tests/
├── valet-manager-helper/       privilege helper binary
│   ├── Cargo.toml
│   └── src/
├── assets/
│   ├── icons/                  SVG icons
│   ├── fonts/
│   └── templates/servers/      Handlebars templates
└── packaging/
    ├── valet-manager.desktop
    ├── polkit/com.valetmanager.policy
    └── deb/
```

---

## Build commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run with live reload (requires cargo-watch)
cargo watch -x run

# Tests
cargo test --workspace

# Lint (zero warnings enforced)
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Build .deb
cargo deb --package valet-manager
```

---

## Code standards

### Rust
- `declare(strict_types=1)` equivalent: explicit types everywhere
- No `unwrap()` in non-test code — use `?` or `expect("meaningful message")`
- All async functions that call subprocesses must write to `history::audit_log`
- Never run the GUI process as root — use privilege helper via Polkit

### UI
- All color references must use `Colors::*` from `src/ui/theme.rs`
- No `Color32::from_rgb(...)` outside `theme.rs`
- All borders are `0.5px Stroke` — never `1px`
- Status dots are always `7px` diameter circles
- Panel headers always `44px` height with `divider()` below

### Testing
- Unit tests for all parsers (INI, env, wp-config, Nginx sentinel)
- Integration tests for all file I/O (use `tempfile::tempdir()`)
- No tests that require a live Valet installation (mock subprocess output)

---

## Commit message format

```
feat(phase-N): short description

- Implements: list key modules
- Tests: what is covered
- Breaking: API changes (omit if none)
```

Examples:
```
feat(phase-2): implement PHP INI section parser and validator
feat(phase-11): add WordPressConfig multisite enablement flow
fix(nginx): sentinel injection regex handles multi-line blocks
```

---

## Running the app against a real Valet installation

Requires `cpriego/valet-linux` or `genesisweb/valet-linux-plus` installed:
```bash
composer global require cpriego/valet-linux
valet install
valet park ~/Sites
```

Then:
```bash
cargo run
```

The app auto-detects the Valet fork and config path at startup.

---

## Architecture decisions

| Decision | Record |
|---|---|
| egui over gtk4-rs | Zero runtime deps, immediate-mode suits live service status |
| tokio mpsc for GUI↔backend | egui is single-threaded; background tasks send events via channels |
| SQLite for history | rusqlite (bundled feature) avoids system dependency |
| Polkit + helper binary | Never run GUI as root; helper validates all inputs strictly |
| Dual config files | Site root = portable/git; centralized = machine-local overrides |
| Sentinel comments in Nginx | Idempotent inject/remove without corrupting existing config |

---

## License

MIT. See [LICENSE](../LICENSE).

---

*Author: Al Amin Ahamed (@mralaminahamed)*
