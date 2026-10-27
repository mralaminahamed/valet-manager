# Valet Manager

Valet Manager is a native Linux desktop application for managing Laravel Valet environments. It is written in Rust and uses eframe and egui 0.34.2 for the interface. It calls the `valet` command and system tools such as `systemctl`, and reads and writes Valet's configuration files.

Status: the application is under development and is not released. The version in `Cargo.toml` is 0.1.0.

## What it manages

The following is implemented in the source code:

- Parked and linked sites, read from Valet's `config.json` and `Sites/` directory (`valet-manager/src/valet/site_scanner.rs`).
- Proxies, added and removed with `valet proxy` and `valet unproxy` (`valet-manager/src/nginx/proxy_manager.rs`).
- Per-site PHP isolation through `.valetrc` and `valet isolate` (`valet-manager/src/php/switcher.rs`), and a global PHP switch with `valet use`.
- PHP extensions and INI editing, including per-site `.user.ini`.
- TLS: `valet secure` and `valet unsecure` on sites, and reading certificate expiry dates with `openssl`.
- An artisan runner (also `bin/console`, `bin/magento` and `drush`).
- Queue workers, written as systemd user services.
- Mailpit message listing.
- `.env` file editing, with values of secret-looking keys masked.
- HTTP server modules for Nginx, Apache, Caddy and FrankenPHP (`valet-manager/src/http_servers/`).
- A per-site configuration file, `.valet-manager.toml`, or a central one under `~/.config/valet-manager/sites/`.

## Partial or not built

- Database manager: it lists databases and tables through the `mysql` client. Creating a database is not built. Migrate and seed buttons exist in the panel.
- PHP package install and remove: not built. The code only lists installed packages with `apt`, `dnf` or `pacman`.
- Xdebug refresh: uses fixed values (mode off, port 9003, first PHP version assumed installed) instead of reading the installation.
- PostgreSQL and SQLite appear as database engine choices, but no listing code for them exists.

## Platform

Linux only. Several dependencies are declared for Linux targets only but are used unconditionally, so the workspace does not build on macOS.

## Build and test

The commands below are taken from `docs/CONTRIBUTING.md`. On Debian or Ubuntu, that document lists `pkg-config`, `libssl-dev` and `libgtk-3-dev` as prerequisites.

```bash
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

These commands were not run on Linux while preparing this README. The formatting check passes on the current tree.

## Layout

- `valet-manager/`: the egui application.
- `valet-manager-helper/`: a small program that runs privileged operations through `pkexec`.
- `docs/`: documentation. `source/`: planning documents. `packaging/` and `policies/`: desktop entry and polkit files. `assets/`: icons.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): crate layout, request flow, state files, trust boundaries.
- [docs/README.md](docs/README.md): project overview. It was written during planning and describes some features that are not built.
- [docs/FEATURES.md](docs/FEATURES.md), [docs/ROADMAP.md](docs/ROADMAP.md), [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md), [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md).

## Known limitations

- There is no continuous integration. The repository has no workflow files, so formatting, lint, build and tests are not run automatically.
- The privileged helper does not validate its inputs; see the trust boundaries section of `docs/ARCHITECTURE.md`.

## License

See `LICENSE`.

Author: Al Amin Ahamed.
