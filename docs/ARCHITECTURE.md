# Architecture

This document describes the implementation as it exists in the source tree. The planning documents under `source/` (for example `source/architecture-v2.md`) describe the intended design and may differ from the code.

## Crate layout

The Cargo workspace (`Cargo.toml`) has two members.

| Crate | Kind | Role |
|---|---|---|
| `valet-manager` | binary | egui/eframe desktop application (`eframe` and `egui` 0.34.2, `valet-manager/Cargo.toml`). |
| `valet-manager-helper` | binary | Small program that performs the operations that need root. Reads one JSON request per line from stdin and writes one JSON response per line (`valet-manager-helper/src/main.rs`). |

Modules of `valet-manager/src`, grouped by purpose:

- Entry and wiring: `main.rs`, `app.rs` (eframe `App` and the command dispatcher), `commands.rs` (`AppCommand`), `events.rs` (`AppEvent`), `state/` (`AppState`).
- UI: `ui/` (`panels/`, `screens/`, `modals/`, `windows/`, `components/`, `theme.rs`, `command_palette.rs`).
- Valet and system: `valet/` (variant detection, `config.json` reader, site scanner, file watcher, sharing), `php/`, `nginx/`, `http_servers/`, `services/`, `ssl/`, `system/`.
- Features: `creator/` (App Creator), `artisan/`, `database/`, `env_file/`, `queue/`, `mail/`, `phpmyadmin/`, `wordpress/`, `laravel/`, `site_config/`, `history/`, `version_registry/`, `deep_link/`, `tray/`, `notifications/`, `updater/`.

`reqwest`, `rusqlite`, `regex`, `rand`, `bcrypt` and several other dependencies are declared under `[target.'cfg(target_os = "linux")'.dependencies]`, and `gtk` is used for the tray. The application therefore builds on Linux only.

## Request flow

1. `main.rs` creates a tokio runtime and two bounded channels (capacity 64): `AppCommand` (UI to dispatcher) and `AppEvent` (background tasks to UI). It spawns the dispatcher `app::run_dispatcher`, a service poller, a tray bridge, and queues startup commands (server detection, phpMyAdmin probe, version registry refresh). A `valet-manager://` argument is parsed by `deep_link::parse` and queued as a command.
2. A panel handles a user action in `ui/panels/*` by calling `cmd_tx.try_send(AppCommand::...)`. Some commands only change UI state and are applied directly in `app.rs`.
3. `run_dispatcher` matches the command and spawns a task. The task runs the work (a child process, a file read or write, or an HTTP request) and sends an `AppEvent` back.
4. `ValetManagerApp::ui` calls `drain_events` on each frame, copies event payloads into its `AppState`, and repaints at least once per second.

Example: switching the global PHP version. A panel sends `AppCommand::SwitchGlobalPhp(ver)`. The dispatcher calls `php::switcher::switch_global`, which runs `valet use php@<ver>`. If that fails, it calls the privileged helper with `SwitchPhp`. It then restarts `php<ver>-fpm` through the helper. The task replies with `AppEvent::PhpSwitched` or `AppEvent::Error`.

## Data stores and state files

- Valet files, read (and in some cases written): `~/.valet/` or `~/.config/valet/` depending on the detected variant (`valet/variant.rs`): `config.json`, `Nginx/`, `Sites/`, `Drivers/`, `Log/`, `CA/`. Variant detection uses `composer global show` and falls back to the presence of `config.json`.
- Application files under `~/.config/valet-manager/`: `config.toml` and the `.onboarded` marker (`config.rs`), `history.sqlite` (`history/mod.rs`), `version-registry.json` (`version_registry/cache.rs`), per-site configs `sites/<name>.toml` (`site_config/reader.rs`), `auth/<site>.htpasswd` (`http_servers/basic_auth.rs`), and phpMyAdmin configs (`phpmyadmin/config_generator.rs`).
- Per-site files: `<site>/.valet-manager.toml` (site-root config, overrides the central one), `.valetrc`, `.user.ini`, `.env`.
- Queue workers are user systemd units at `~/.config/systemd/user/valet-queue-<id>.service` (`queue/mod.rs`).
- In-memory state: `AppState` (`state/app_state.rs`) is rebuilt on each start. No other persistent store was found.

## Trust boundaries

- The GUI process runs as the logged-in user. It runs external programs without a shell, using `Command::new(program).args(...)`: `valet`, `systemctl` (system and `--user`), `php`, `composer`, `wp`, `mysql`, `openssl`, `xdg-open`, `sudo`, and the project creators' CLIs (`creator/`, `cli_tools/registry.rs`). Exceptions that use a shell: the shell window runs user-typed text through `bash -c` in the selected site directory (`app.rs`, `ShellOutput` handling), and `system/package_manager.rs` runs fixed `apt`, `dnf` or `pacman` listing pipelines through `bash -c`.
- `sudo` is used for `valet trust` and for restarting `dnsmasq` (`app.rs`, `TrustCa` and `FlushDns`).
- Root operations go through `pkexec /usr/local/bin/valet-manager-helper` (`php/privilege.rs`). The helper accepts seven actions: `SwitchPhp`, `StartFpm`, `StopFpm`, `RestartFpm`, `EnableExtension`, `DisableExtension`, `SaveIni`. It runs `update-alternatives`, `systemctl`, `phpenmod` and `phpdismod` with fixed program names, and `SaveIni` writes to a caller-supplied path. The polkit actions are defined in `policies/io.valet-manager.policy` (helper path, `auth_admin`) and `packaging/polkit/com.valetmanager.policy` (`/usr/bin/systemctl`).
- Input validation found in the code: PHP INI value checks (`php/ini_manager.rs`, `php/user_ini.rs`), and parsing of the helper protocol with `serde` (unknown actions are rejected with a parse error). The helper does not validate `version`, `extension` or the `SaveIni` path, and the GUI does not restrict which paths it sends. Apart from polkit authentication, no check in the code limits what the helper can be asked to do. The policy file for the helper (`policies/io.valet-manager.policy`) and the one referenced by the `.deb` assets (`packaging/polkit/com.valetmanager.policy`, `/usr/bin/systemctl` only) are different files with different action IDs.
- Network access: `reqwest` calls to `localhost:8025` (Mailpit), version sources (`version_registry/*_fetcher.rs`), and local HTTP probes. `nginx/proxy_manager.rs` disables certificate verification for probes of local sites.

## Failure behaviour

- Command failures are returned as `AppEvent::Error(String)` and shown as a toast. There is no retry logic.
- Missing or unparsable config files fall back to defaults (`valet/config_reader.rs`, `site_config/reader.rs`).
- If Valet is not detected, commands that need its paths report "Valet not detected".
- If `valet use` fails, the switcher falls back to the helper (`php/switcher.rs`). The results of `valet isolate` and `valet unisolate` are ignored.
- If neither `DISPLAY` nor `WAYLAND_DISPLAY` is set, or tray initialisation fails, the application continues without a tray icon (`main.rs`).
- A closed channel ends the corresponding background loop (`services/monitor.rs`).
