# Valet Manager — Architecture Additions v3.0
## Gap-Closing & New DX Features

> This document extends v2.0. All sections below describe net-new modules, state,
> commands, panel specifications, and roadmap changes. Cross-references to v2 sections
> are noted inline.

---

## Table of Contents

1. [New Feature Summary](#1-new-feature-summary)
2. [Updated Application Architecture Diagram](#2-updated-application-architecture-diagram)
3. [Updated Crate & Module Structure (delta)](#3-updated-crate--module-structure-delta)
4. [New Domain Models & State](#4-new-domain-models--state)
5. [New AppCommand Variants](#5-new-appcommand-variants)
6. [New Module Specifications](#6-new-module-specifications)
   - [6.1 phpinfo() Viewer](#61-phpinfo-viewer)
   - [6.2 PHP Compatibility Checker](#62-php-compatibility-checker)
   - [6.3 Command History & Audit Log](#63-command-history--audit-log)
   - [6.4 Favorite Domains](#64-favorite-domains)
   - [6.5 Database Manager](#65-database-manager)
   - [6.6 Artisan Command Runner](#66-artisan-command-runner)
   - [6.7 .env File Editor](#67-env-file-editor)
   - [6.8 Queue Worker Manager](#68-queue-worker-manager)
   - [6.9 Xdebug Quick Toggle](#69-xdebug-quick-toggle)
   - [6.10 Mail Catcher Control](#610-mail-catcher-control)
   - [6.11 SSL Certificate Dashboard](#611-ssl-certificate-dashboard)
   - [6.12 Command Palette (Ctrl+K)](#612-command-palette-ctrlk)
   - [6.13 Deep-Link Protocol Handler](#613-deep-link-protocol-handler)
   - [6.14 Built-in App Updater](#614-built-in-app-updater)
7. [Updated Sidebar Navigation](#7-updated-sidebar-navigation)
8. [New Panel Layouts](#8-new-panel-layouts)
9. [Updated AppState](#9-updated-appstate)
10. [Updated Configuration Schema](#10-updated-configuration-schema)
11. [Updated Dependencies](#11-updated-dependencies)
12. [Updated Phased Roadmap](#12-updated-phased-roadmap)

---

## 1. New Feature Summary

### Gap-closing features (PHPMon parity)

| # | Feature | Priority | Gap Source |
|---|---|---|---|
| G1 | phpinfo() in-app viewer | P2 | PHPMon has it, we didn't |
| G2 | PHP compatibility checker | P2 | PHPMon has it, we didn't |
| G3 | Command history & audit log | P2 | PHPMon has it, we didn't |
| G4 | Favorite / bookmark domains | P2 | PHPMon has it, we didn't |
| G5 | Built-in app updater | P3 | PHPMon has it, we didn't |
| G6 | `valet-manager://` deep-link protocol | P2 | PHPMon has `phpmon://`, we need ours |

### New DX features (neither app has)

| # | Feature | Priority | Developer Impact |
|---|---|---|---|
| D1 | Command palette (Ctrl+K) | P1 | Keyboard-first access to every action |
| D2 | Database manager | P1 | Create/drop/migrate/seed without terminal |
| D3 | `.env` file editor | P1 | Per-site env with grouping and validation |
| D4 | Artisan command runner | P1 | Run + stream artisan from GUI |
| D5 | Queue worker manager | P3 | Systemd user service control per site |
| D6 | Xdebug quick toggle | P3 | One-click debug mode + IDE key preset |
| D7 | Mail catcher control | P3 | Mailpit/MailHog start + SMTP auto-config |
| D8 | SSL certificate dashboard | P3 | Expiry warnings + one-click regenerate |

---

## 2. Updated Application Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              valet-manager binary                               │
│                                                                                 │
│  ┌─────────────────────────┐     ┌────────────────────────────────────────────┐ │
│  │       GUI Thread         │     │              App State                     │ │
│  │    (egui / eframe)       │◄───►│  Arc<RwLock<AppState>>                     │ │
│  │                          │msgs │  php, nginx, valet, services, dnsmasq,     │ │
│  │  Panels / Command Palette│     │  proxy, share, drivers, creator,           │ │
│  │  Theme / Layout          │     │  phpinfo, compat, history, database,       │ │
│  └────────────┬─────────────┘     │  artisan, env, queue, xdebug, mail, ssl    │ │
│               │ AppCommand enum   └────────────────────────────────────────────┘ │
│               ▼                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                     Command Dispatcher (tokio runtime)                   │   │
│  └─┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬──────┘   │
│    │    │    │    │    │    │    │    │    │    │    │    │    │    │           │
│  PHP  NGX  VAL DNS  SVC  PRX  SHR  CRT  PHPi CMP  DB  ART  ENV  QUE           │
│  Mgr  Mgr  Mgr Mgr  Mon  Mgr  Mgr  Mgr  Mgr  Chk  Mgr  Run  Ed   Wkr          │
│    │    │    │    │    │    │    │    │    │    │    │    │    │    │           │
│    └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘           │
│                                         │                                       │
│                    ┌────────────────────┴───────────────────┐                  │
│                    │         Shared Infrastructure           │                  │
│                    │  CLI Registry · Audit Log · Deep-link   │                  │
│                    │  Updater · History DB · System Bridge   │                  │
│                    └─────────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Updated Crate & Module Structure (delta)

Only additions relative to v2. Insert these alongside the existing modules in `src/`.

```
src/
│
├── phpinfo/
│   ├── mod.rs
│   ├── runner.rs               # Run `php -r "phpinfo();"`, capture output
│   └── parser.rs               # Parse phpinfo HTML → structured sections
│
├── compat/
│   ├── mod.rs
│   └── checker.rs              # Parse composer.json require.php, semver compare
│
├── database/
│   ├── mod.rs
│   ├── detector.rs             # Detect MySQL/PostgreSQL/SQLite per site
│   ├── mysql.rs                # mysql CLI wrapper (create, drop, list, import)
│   ├── postgres.rs             # psql CLI wrapper
│   ├── sqlite.rs               # sqlite3 CLI wrapper
│   └── migrator.rs             # php artisan migrate / db:seed orchestration
│
├── artisan/
│   ├── mod.rs
│   ├── runner.rs               # Run artisan with streamed output
│   ├── discover.rs             # `php artisan list --format=json` parser
│   └── history.rs              # Per-site artisan command history (SQLite)
│
├── env_editor/
│   ├── mod.rs
│   ├── parser.rs               # Parse .env KEY=VALUE, preserve comments/blanks
│   ├── validator.rs            # Compare against .env.example
│   └── writer.rs               # Atomic write (.env.tmp → rename)
│
├── queue/
│   ├── mod.rs
│   ├── worker_manager.rs       # Create/control systemd user services
│   └── stats.rs                # Poll Redis / DB for job counts + failed
│
├── xdebug/
│   ├── mod.rs
│   ├── toggle.rs               # Enable/disable Xdebug per PHP version
│   └── config.rs               # Write xdebug.mode, client_host, idekey
│
├── mail_catcher/
│   ├── mod.rs
│   ├── mailpit.rs              # Mailpit install, start, stop, HTTP API
│   ├── mailhog.rs              # MailHog fallback
│   └── smtp_config.rs          # Auto-write MAIL_* vars to site .env
│
├── ssl/
│   ├── mod.rs
│   ├── cert_reader.rs          # openssl x509 parser → SslCertInfo
│   └── expiry_monitor.rs       # Background poll, notify on near-expiry
│
├── history/
│   ├── mod.rs
│   ├── audit_log.rs            # Intercept every subprocess, write to SQLite
│   └── export.rs               # CSV export
│
├── protocol/
│   ├── mod.rs
│   └── handler.rs              # valet-manager:// URL scheme dispatch
│
└── updater/
    ├── mod.rs
    ├── checker.rs              # GitHub releases API, semver compare
    └── installer.rs            # Download + install .deb/.rpm/AppImage

src/ui/panels/ (additions):
│
├── phpinfo.rs                  # phpinfo() searchable table panel
├── compat.rs                   # Compatibility checker panel
├── database.rs                 # Database manager panel
├── artisan.rs                  # Artisan runner panel
├── env_editor.rs               # .env editor panel
├── queue.rs                    # Queue worker manager panel
├── xdebug.rs                   # Xdebug toggle panel
├── mail_catcher.rs             # Mail catcher panel
├── ssl_certs.rs                # SSL certificate dashboard
└── history.rs                  # Command history panel

src/ui/ (new top-level):
└── command_palette.rs          # Ctrl+K overlay, rendered over all panels

assets/templates/ (additions):
├── systemd-queue-worker.service.hbs
└── mailpit-env.snippet.hbs
```

---

## 4. New Domain Models & State

```rust
// src/state/phpinfo_state.rs

#[derive(Debug, Default, Clone)]
pub struct PhpInfoState {
    pub current_version: Option<String>,
    pub sections: Vec<PhpInfoSection>,
    pub search_query: String,
    pub loading: bool,
}

#[derive(Debug, Clone)]
pub struct PhpInfoSection {
    pub name: String,
    pub entries: Vec<PhpInfoEntry>,
}

#[derive(Debug, Clone)]
pub struct PhpInfoEntry {
    pub key: String,
    pub local_value: String,
    pub master_value: Option<String>,
}

// src/state/compat_state.rs

#[derive(Debug, Default, Clone)]
pub struct CompatibilityState {
    pub results: Vec<CompatibilityResult>,
    pub loading: bool,
    pub last_checked: Option<chrono::DateTime<chrono::Local>>,
}

#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub site: String,
    pub domain: String,
    pub required_php: Option<String>,     // from composer.json
    pub active_php: String,               // resolved for this site
    pub status: CompatStatus,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompatStatus {
    Compatible,
    Incompatible,
    NoRequirement,    // no require.php in composer.json
    NoComposer,       // not a Composer project
}

// src/state/history_state.rs

#[derive(Debug, Default, Clone)]
pub struct HistoryState {
    pub entries: Vec<CommandHistoryEntry>,
    pub filter: String,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub struct CommandHistoryEntry {
    pub id: u64,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub started_at: chrono::DateTime<chrono::Local>,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub triggered_by: String,    // "php_switcher", "app_creator", "artisan", etc.
    pub output_preview: String,  // first 200 chars of stdout
}

// src/state/database_state.rs

#[derive(Debug, Default, Clone)]
pub struct DatabaseState {
    pub detected_engines: Vec<DatabaseEngine>,
    pub site_databases: HashMap<String, Vec<DatabaseInfo>>,
    pub selected_site: Option<String>,
    pub selected_db: Option<String>,
    pub tables: Vec<TableInfo>,
    pub loading: bool,
    pub operation_output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseEngine {
    Mysql { host: String, port: u16, user: String },
    Postgres { host: String, port: u16, user: String },
    Sqlite,
}

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub name: String,
    pub engine: DatabaseEngine,
    pub size_bytes: Option<u64>,
    pub table_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub row_count: Option<u64>,
    pub engine: Option<String>,   // InnoDB, MyISAM, etc.
}

// src/state/artisan_state.rs

#[derive(Debug, Default, Clone)]
pub struct ArtisanState {
    pub selected_site: Option<String>,
    pub available_commands: Vec<ArtisanCommand>,
    pub filtered_commands: Vec<ArtisanCommand>,
    pub command_input: String,
    pub args_input: String,
    pub output_lines: Vec<OutputLine>,
    pub running: bool,
    pub history: Vec<ArtisanHistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct ArtisanCommand {
    pub name: String,              // e.g. "migrate:fresh"
    pub description: String,
    pub group: String,             // e.g. "migrate"
}

#[derive(Debug, Clone)]
pub struct ArtisanHistoryEntry {
    pub command: String,
    pub ran_at: chrono::DateTime<chrono::Local>,
    pub exit_code: i32,
    pub duration_ms: u64,
}

// src/state/env_state.rs

#[derive(Debug, Default, Clone)]
pub struct EnvEditorState {
    pub selected_site: Option<String>,
    pub entries: Vec<EnvEntry>,
    pub groups: Vec<EnvGroup>,
    pub validation_issues: Vec<EnvValidationIssue>,
    pub modified: bool,
    pub show_secrets: bool,
}

#[derive(Debug, Clone)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,   // inline comment
    pub is_secret: bool,           // heuristic: _KEY, _SECRET, _PASSWORD, TOKEN
    pub in_example: bool,          // key exists in .env.example
}

#[derive(Debug, Clone)]
pub struct EnvGroup {
    pub prefix: String,            // "APP", "DB", "MAIL", "REDIS", etc.
    pub keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnvValidationIssue {
    pub key: String,
    pub issue: EnvIssueType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvIssueType {
    MissingFromExample,   // in .env but not .env.example
    MissingFromEnv,       // in .env.example but not .env
    EmptyRequired,        // key with no value, present in example with a value
}

// src/state/queue_state.rs

#[derive(Debug, Default, Clone)]
pub struct QueueState {
    pub workers: Vec<QueueWorker>,
}

#[derive(Debug, Clone)]
pub struct QueueWorker {
    pub site: String,
    pub connection: String,        // "redis", "database", "sqs"
    pub queue: String,             // "default", "high", etc.
    pub service_name: String,      // valet-queue-{site}-{queue}.service
    pub status: ServiceStatus,
    pub processed_jobs: Option<u64>,
    pub failed_jobs: Option<u64>,
}

// src/state/xdebug_state.rs

#[derive(Debug, Default, Clone)]
pub struct XdebugState {
    pub php_versions: Vec<XdebugPhpStatus>,
}

#[derive(Debug, Clone)]
pub struct XdebugPhpStatus {
    pub php_version: String,
    pub installed: bool,
    pub enabled: bool,
    pub mode: XdebugMode,
    pub ide_key: String,
    pub client_host: String,
    pub client_port: u16,
    pub ini_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum XdebugMode {
    #[default]
    Off,
    Debug,
    Profile,
    Coverage,
    Trace,
    DevelopDebug,   // develop,debug
}

// src/state/mail_state.rs

#[derive(Debug, Default, Clone)]
pub struct MailCatcherState {
    pub installed_tool: Option<MailCatcherTool>,
    pub running: bool,
    pub port: u16,
    pub smtp_port: u16,
    pub unread_count: Option<u32>,
    pub web_ui_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MailCatcherTool {
    Mailpit,
    MailHog,
}

// src/state/ssl_state.rs

#[derive(Debug, Default, Clone)]
pub struct SslState {
    pub certificates: Vec<SslCertInfo>,
    pub ca_cert_path: Option<PathBuf>,
    pub ca_installed_in_system: bool,
}

#[derive(Debug, Clone)]
pub struct SslCertInfo {
    pub domain: String,
    pub cert_path: PathBuf,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
    pub days_remaining: i64,
    pub issuer: String,
    pub subject_alt_names: Vec<String>,
    pub expiry_status: ExpiryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpiryStatus {
    Ok,           // > 30 days
    Warning,      // 7–30 days
    Critical,     // < 7 days
    Expired,
}

// src/state/updater_state.rs

#[derive(Debug, Default, Clone)]
pub struct UpdaterState {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub status: UpdateStatus,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    Downloading,
    Installing,
    UpToDate,
    Error(String),
}

// src/ui/command_palette.rs — state

#[derive(Debug, Default, Clone)]
pub struct CommandPaletteState {
    pub visible: bool,
    pub query: String,
    pub results: Vec<PaletteResult>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
pub struct PaletteResult {
    pub label: String,
    pub subtitle: String,
    pub category: PaletteCategory,
    pub action: AppCommand,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteCategory {
    PhpVersion,
    Site,
    Service,
    ArtisanCommand,
    Panel,
    Action,
}
```

---

## 5. New AppCommand Variants

Add these to the `AppCommand` enum in `src/commands.rs`:

```rust
// phpinfo
ViewPhpInfo { php_version: String },
RefreshPhpInfo,

// Compatibility
CheckSiteCompatibility(String),
CheckAllCompatibility,

// Favorites
ToggleFavoriteSite(String),

// History
ClearCommandHistory,
ExportCommandHistory(PathBuf),

// Database
DetectDatabases,
CreateDatabase { name: String, engine: DatabaseEngine },
DropDatabase { name: String, engine: DatabaseEngine, confirmed: bool },
RunMigrations { site: String, fresh: bool, seed: bool },
RunSeeders(String),
RollbackMigrations { site: String, steps: Option<u32> },
RefreshTableList { site: String, db_name: String },

// Artisan
DiscoverArtisanCommands(String),
RunArtisanCommand { site: String, command: String, args: Vec<String> },
CancelArtisanCommand,

// .env Editor
LoadDotEnv(String),
SaveDotEnv { site: String, entries: Vec<EnvEntry> },
ValidateDotEnv(String),
CopyEnvKey { key: String },
ToggleEnvSecretVisibility,

// Queue Workers
StartQueueWorker { site: String, connection: String, queue: String },
StopQueueWorker { site: String, queue: String },
RestartQueueWorker { site: String, queue: String },
RefreshQueueStats,

// Xdebug
EnableXdebug { php_version: String, mode: XdebugMode },
DisableXdebug(String),
SetXdebugMode { php_version: String, mode: XdebugMode },
SetXdebugIdeKey { php_version: String, ide_key: String },

// Mail Catcher
InstallMailCatcher(MailCatcherTool),
StartMailCatcher,
StopMailCatcher,
OpenMailCatcherUI,
ClearMailCatcherMessages,
ConfigureSiteSmtp(String),

// SSL
RegenerateSslCert(String),
ExportSslCert { domain: String, dest: PathBuf },
InstallCaInSystem,
RefreshSslCerts,

// Updater
CheckForUpdates,
DownloadUpdate,
InstallUpdate,
SkipVersion(String),

// Deep-link / protocol
HandleProtocolUrl(String),

// Command Palette
OpenCommandPalette,
CloseCommandPalette,
ExecutePaletteAction(usize),
```

---

## 6. New Module Specifications

### 6.1 phpinfo() Viewer

**File:** `src/phpinfo/runner.rs`

Run `php{ver} -r "phpinfo();"` which outputs HTML. The runner captures stdout,
strips HTML tags, and passes the plain text to the parser.

```rust
pub async fn get_phpinfo(php_binary: &str) -> anyhow::Result<Vec<PhpInfoSection>> {
    let output = Command::new(php_binary)
        .args(["-r", "phpinfo();"])
        .output().await?;
    let raw = String::from_utf8_lossy(&output.stdout);
    // Strip HTML, split by section headers (lines matching /^[A-Z]/)
    parse_phpinfo_text(&raw)
}
```

**Parser strategy:** The plain-text phpinfo output uses a predictable structure:
section headers followed by `key => local_value => master_value` rows.
Sections are separated by blank lines. The parser groups rows per section.

**Panel features:**
- Version selector dropdown at top
- Section tree (left) + key-value table (right)
- Search bar: highlights rows where key or value matches query
- "Copy Value" button per row
- Refresh button

---

### 6.2 PHP Compatibility Checker

**File:** `src/compat/checker.rs`

```rust
pub async fn check_site(site: &ValetSite) -> CompatibilityResult {
    // 1. Read composer.json from site.path
    // 2. Extract require.php constraint (e.g. "^8.1", ">=8.0 <9.0")
    // 3. Read site's active PHP version (from .valetrc or global)
    // 4. Use semver crate to test version against constraint
    // 5. Return CompatibilityResult with suggestion if incompatible
}
```

Batch check: calls `check_site` concurrently for all sites using
`futures::future::join_all`. Results update the `CompatibilityState`.

**Panel layout:**
- Table: Site | Required PHP | Active PHP | Status | Action
- Status badges: Compatible (green) / Incompatible (red) / No requirement (gray) / No Composer (gray)
- Action: "Isolate to php@X.X" button shown when incompatible
- "Check all" button at top
- Auto-runs on panel open, result cached for 30 seconds

---

### 6.3 Command History & Audit Log

**Storage:** SQLite at `~/.config/valet-manager/history.db` via `rusqlite` crate.

**Schema:**
```sql
CREATE TABLE command_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    command     TEXT NOT NULL,
    args        TEXT NOT NULL,          -- JSON array
    working_dir TEXT,
    started_at  TEXT NOT NULL,          -- ISO 8601
    duration_ms INTEGER NOT NULL,
    exit_code   INTEGER,
    triggered_by TEXT NOT NULL,
    output_preview TEXT
);
```

**Interceptor:** The `subprocess.rs` `run_command()` function wraps every
`tokio::process::Command` call and writes an entry to `history.db` before and
after execution. This is transparent to all callers.

```rust
pub async fn run_command(
    cmd: &str,
    args: &[&str],
    cwd: Option<&Path>,
    triggered_by: &str,
) -> anyhow::Result<CommandOutput> {
    let started = Instant::now();
    // ... execute ...
    let entry = CommandHistoryEntry { ... };
    history::audit_log::insert(&entry).await?;
    Ok(output)
}
```

**Panel features:**
- Search/filter by command name, triggered_by, date range
- Table: Time | Command | Args | Duration | Exit Code | Source
- Row click: expand to show full output_preview + re-run button
- "Export CSV" button
- "Clear history" with confirmation
- Max 5000 entries (rolling, oldest auto-deleted)

---

### 6.4 Favorite Domains

Stored in `~/.config/valet-manager/config.toml` under:
```toml
[sites]
favorites = ["mylaravel", "myblog", "myshop"]
```

**Toggle command:** `AppCommand::ToggleFavoriteSite(String)` adds/removes the
site name from the `favorites` list and saves config.

**Sites panel behavior:**
- Favorited rows show a filled star icon in the first column
- Favorites always sort to the top regardless of other sort order
- Right-click context menu includes "Add to favorites" / "Remove from favorites"
- Tray quick-menu shows favorites section above full site list

---

### 6.5 Database Manager

The database manager is a GUI frontend for existing CLI tools (`mysql`, `psql`,
`sqlite3`). It does not implement a custom query engine. Focus is on the most
common Laravel/WordPress developer workflows.

**Detection (`src/database/detector.rs`):**
```rust
pub async fn detect_engines() -> Vec<DatabaseEngine> {
    let mut engines = Vec::new();

    // MySQL / MariaDB — try connecting with root/empty pass
    if which("mysql").is_ok() {
        if test_mysql_connection("root", "", "127.0.0.1", 3306).await.is_ok() {
            engines.push(DatabaseEngine::Mysql { ... });
        }
    }

    // PostgreSQL
    if which("psql").is_ok() {
        engines.push(DatabaseEngine::Postgres { ... });
    }

    // SQLite — always available if sqlite3 is installed
    if which("sqlite3").is_ok() {
        engines.push(DatabaseEngine::Sqlite);
    }

    engines
}
```

**Operations per engine:**

| Action | MySQL | PostgreSQL | SQLite |
|---|---|---|---|
| List databases | `SHOW DATABASES` | `\l` | Scan `*.sqlite` in site root |
| Create | `CREATE DATABASE` | `createdb` | File creation |
| Drop | `DROP DATABASE` | `dropdb` | File deletion |
| List tables | `SHOW TABLES` | `\dt` | `.tables` |
| Row count | `SELECT COUNT(*)` | Same | Same |
| Import `.sql` | `mysql < file.sql` | `psql < file.sql` | `sqlite3 < file.sql` |
| Export | `mysqldump` | `pg_dump` | `.dump` |

**Laravel integration:**
- Detect site as Laravel (has `artisan`)
- Read `DB_CONNECTION`, `DB_DATABASE`, `DB_USERNAME`, `DB_PASSWORD` from `.env`
- Surface "Run Migrations", "Run Seeders", "Fresh + Seed", "Rollback N"
- These call `php artisan migrate [--fresh] [--seed]` in the site directory
- Output is streamed to the terminal widget

**Panel layout:**
```
┌──────────────────────────────────────────────────────────────────────┐
│  Database Manager                                                     │
│  Engine: [MySQL ▼]  Site: [mylaravel.test ▼]   [+ Create DB]        │
│  ───────────────────────────────────────────────────────────────────  │
│  Databases                    Tables (mylaravel_db)                  │
│  ┌────────────────────┐       ┌──────────────────────────────────┐   │
│  │ ● mylaravel_db     │       │  users              12,450 rows  │   │
│  │ ● wp_blog          │       │  posts               3,210 rows  │   │
│  │ ● test_db          │       │  categories             24 rows  │   │
│  └────────────────────┘       └──────────────────────────────────┘   │
│                                                                       │
│  Laravel Actions:  [Migrate]  [Migrate Fresh]  [Seed]  [Rollback]   │
│  ───────────────────────────────────────────────────────────────────  │
│  Output:                                                              │
│  > php artisan migrate                                               │
│  Migrating: 2024_01_create_users_table                               │
│  Migrated:  2024_01_create_users_table (12ms)                        │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 6.6 Artisan Command Runner

**Command discovery (`src/artisan/discover.rs`):**
```rust
pub async fn discover_commands(site_path: &Path, php_bin: &str) -> Vec<ArtisanCommand> {
    let output = Command::new(php_bin)
        .args(["artisan", "list", "--format=json"])
        .current_dir(site_path)
        .output().await?;

    // Parse JSON: commands[].name, commands[].description
    // Group by first segment of name (before ":")
}
```

**Panel layout:**
```
┌──────────────────────────────────────────────────────────────────────┐
│  Artisan Runner                         Site: [mylaravel.test ▼]    │
│  ───────────────────────────────────────────────────────────────────  │
│  Command: [migrate:fresh                          ]  [▶ Run]        │
│  Options: [--seed                                 ]                  │
│                                                                       │
│  Quick commands:  [migrate]  [migrate:fresh --seed]  [route:list]   │
│                   [cache:clear]  [config:clear]  [queue:work]        │
│                                                                       │
│  ──── Output ──────────────────────────────────────── [✕ Cancel]    │
│  Dropped all tables successfully.                                    │
│  Running migrations.                                                  │
│  ✓  2024_01_01_create_users_table     (8ms)                         │
│  ✓  2024_01_02_create_posts_table     (6ms)                         │
│                                                                       │
│  ──── History ─────────────────────────────────────────────────────  │
│  migrate:fresh --seed     2min ago    exit 0    1.2s                │
│  route:list               5min ago    exit 0    0.3s                │
└──────────────────────────────────────────────────────────────────────┘
```

**Quick commands:** A configurable list of frequently-used commands shown as
one-click buttons. Stored in `config.toml` per site:
```toml
[artisan.quick_commands.mylaravel]
commands = ["migrate:fresh --seed", "route:list", "cache:clear"]
```

---

### 6.7 .env File Editor

**Parser (`src/env_editor/parser.rs`):**

The parser preserves the original file structure (comments, blank lines,
ordering). The goal is `parse → display → edit → write` with no unintended diffs.

```rust
pub fn parse_env_file(content: &str) -> Vec<EnvEntry> {
    content.lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .filter_map(|line| {
            let (key, rest) = line.split_once('=')?;
            let (value, comment) = extract_inline_comment(rest);
            let is_secret = is_secret_key(key.trim());
            Some(EnvEntry { key: key.trim().to_string(), value, comment, is_secret, .. })
        })
        .collect()
}

fn is_secret_key(key: &str) -> bool {
    let k = key.to_uppercase();
    k.ends_with("_KEY") || k.ends_with("_SECRET") || k.ends_with("_PASSWORD")
        || k.ends_with("_TOKEN") || k.ends_with("_PASS") || k.contains("PRIVATE")
}
```

**Validation:** Loads `.env.example` alongside `.env`. Keys in `.env.example`
but absent from `.env` are flagged `MissingFromEnv`. Keys in `.env` but absent
from `.env.example` are flagged `MissingFromExample` (notice, not error).

**Panel layout:**
```
┌──────────────────────────────────────────────────────────────────────┐
│  .env Editor               Site: [mylaravel.test ▼]  [Show secrets] │
│  2 issues  ·  [Raw Edit]  ·  [Save]  ·  [Revert]                    │
│  ───────────────────────────────────────────────────────────────────  │
│  Group: [APP ▼]  [DB ▼]  [MAIL ▼]  [REDIS ▼]  [All]               │
│                                                                       │
│  APP_NAME             MyLaravel                                      │
│  APP_ENV              local                                           │
│  APP_KEY              ●●●●●●●●●●●●●●●●●●●●   [Copy]                │
│  APP_DEBUG            true                                           │
│                                                                       │
│  DB_CONNECTION        mysql                                           │
│  DB_HOST              127.0.0.1                                      │
│  DB_DATABASE          mylaravel_db                                   │
│  DB_USERNAME          root                                           │
│  DB_PASSWORD          ●●●●●●   [Copy]                               │
│                                                                       │
│  ⚠  PUSHER_APP_KEY — present in .env.example but missing here       │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 6.8 Queue Worker Manager

Each queue worker is a systemd user service to avoid requiring root.

**Service template (`assets/templates/systemd-queue-worker.service.hbs`):**
```ini
[Unit]
Description=Laravel Queue Worker — {{site}} ({{queue}})
After=network.target

[Service]
Type=simple
WorkingDirectory={{site_path}}
ExecStart={{php_bin}} artisan queue:work {{connection}} --queue={{queue}} --sleep=3 --tries=3
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
```

**Manager:**
```rust
pub async fn start_worker(site: &ValetSite, connection: &str, queue: &str) -> anyhow::Result<()> {
    let service_name = format!("valet-queue-{}-{}.service", site.name, queue);
    let service_path = dirs::config_dir().unwrap()
        .join("systemd/user").join(&service_name);

    // Render template → write to service_path
    // systemctl --user daemon-reload
    // systemctl --user enable --now {service_name}
}
```

**Stats:** For Redis-backed queues, poll `redis-cli LLEN {queue_name}` every 10s.
For database-backed queues, `SELECT COUNT(*) FROM jobs` and
`SELECT COUNT(*) FROM failed_jobs`.

---

### 6.9 Xdebug Quick Toggle

```rust
pub async fn enable(php_version: &str, mode: XdebugMode) -> anyhow::Result<()> {
    let ini_content = format!(
        "zend_extension=xdebug\n\
         xdebug.mode={mode}\n\
         xdebug.client_host=127.0.0.1\n\
         xdebug.client_port=9003\n\
         xdebug.idekey=VSCODE\n\
         xdebug.start_with_request=yes\n",
        mode = mode.as_str()
    );
    // Write via privilege helper → /etc/php/{ver}/cli/conf.d/99-xdebug-valet.ini
    // Restart php-fpm for that version
}
```

**IDE key presets:** VSCODE, PHPSTORM, NETBEANS, user-configurable.

**Panel layout — compact card per PHP version:**
```
┌─────────────────────────────────────────────────────┐
│  php8.3   Xdebug 3.3.2 installed                   │
│  Mode:  [Off]  [Debug]  [Profile]  [Coverage]       │
│  IDE Key: [VSCODE ▼]   Port: [9003]   [Enable]     │
├─────────────────────────────────────────────────────┤
│  php8.2   Xdebug 3.2.1 installed   ● Enabled       │
│  Mode: Debug   IDE Key: PHPSTORM   Port: 9003       │
│  [Disable]                                          │
└─────────────────────────────────────────────────────┘
```

---

### 6.10 Mail Catcher Control

**Mailpit** is the preferred tool. MailHog is the fallback.

**Start Mailpit:**
```rust
pub async fn start(smtp_port: u16, http_port: u16) -> anyhow::Result<()> {
    Command::new("mailpit")
        .args(["--smtp", &format!("127.0.0.1:{smtp_port}"),
               "--listen", &format!("127.0.0.1:{http_port}")])
        .spawn()?;
    // Write PID to /tmp/valet-manager-mailpit.pid
}

pub async fn get_unread_count(http_port: u16) -> anyhow::Result<u32> {
    // GET http://localhost:{http_port}/api/v1/messages → parse total
}
```

**Site SMTP auto-config:** Updates site `.env` with:
```
MAIL_MAILER=smtp
MAIL_HOST=localhost
MAIL_PORT=1025
MAIL_ENCRYPTION=null
```

**Tray integration:** Shows "Mail: N unread" in tray menu when running.

**Panel layout:**
```
┌──────────────────────────────────────────────────────────────────────┐
│  Mail Catcher                                        ● Running       │
│  Tool: Mailpit 1.21.0    SMTP: :1025    Web UI: :8025               │
│  [Stop]  [Open Web UI]  [Clear Messages]                            │
│  Unread: 14 messages                                                 │
│  ─────────────────────────────────────────────────────────────────   │
│  Configure SMTP:  [mylaravel.test ▼]  [Apply to .env]              │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 6.11 SSL Certificate Dashboard

```rust
pub async fn read_cert(cert_path: &Path) -> anyhow::Result<SslCertInfo> {
    let output = Command::new("openssl")
        .args(["x509", "-in", cert_path.to_str().unwrap(),
               "-noout", "-text", "-subject", "-dates"])
        .output().await?;
    parse_openssl_output(&output.stdout)
}
```

Background expiry monitor checks every 6 hours. Sends notification when
`days_remaining < 14`. Color coding: green (>30d), amber (7–30d), red (<7d).

**Panel layout:**
```
┌──────────────────────────────────────────────────────────────────────┐
│  SSL Certificates                              Valet CA: Installed  │
│  ─────────────────────────────────────────────────────────────────   │
│  Domain              Issuer       Expires       Status              │
│  mylaravel.test      Valet CA     312 days      ✓ Ok               │
│  myblog.test         Valet CA      18 days      ⚠ Warning          │
│  myshop.test         Valet CA       3 days      ✕ Critical         │
│  ─────────────────────────────────────────────────────────────────   │
│  [Regenerate Certificate]  [Export PEM]  [Re-install Valet CA]     │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 6.12 Command Palette (Ctrl+K)

Rendered as an egui `Window` with `order: TopMost`, triggered by `Ctrl+K` and
dismissed by `Escape`. Keyboard events are intercepted at the App `update()` level.

**Search index** is rebuilt on every state refresh. Sources:
- All installed PHP versions → `Switch to PHP X.X`
- All Valet sites → `Open {domain}`, `Open {domain} in editor`
- All services → `Restart {service}`
- All artisan commands (selected site) → `artisan {command}`
- All panels → `Go to {panel}`
- Common actions → `Create new app`, `Reload nginx`, `Check for updates`

**Fuzzy matching:** Prefix match → contains match → initials match.
Max 8 results shown. Arrow keys + Enter to execute.

**Palette appearance:**
```
╔══════════════════════════════════════════════════════╗
║  🔍 [Switch to PHP 8.3                        ]      ║
║  ────────────────────────────────────────────────    ║
║  ▶ Switch to PHP 8.3                  PHP version    ║
║    Switch to PHP 8.2                  PHP version    ║
║    artisan migrate:fresh              Artisan         ║
║    Restart nginx                      Service        ║
║  ────────────────────────────────────────────────    ║
║    ↑↓ navigate    ↵ execute    esc dismiss           ║
╚══════════════════════════════════════════════════════╝
```

---

### 6.13 Deep-Link Protocol Handler

**Registered in `.desktop` file:**
```ini
MimeType=x-scheme-handler/valet-manager;
```

**Supported URL actions:**

| URL | Command dispatched |
|---|---|
| `valet-manager://switch-php/8.3` | `SwitchGlobalPhp("8.3")` |
| `valet-manager://open-site/myapp` | `OpenSiteInBrowser("myapp")` |
| `valet-manager://open-site/myapp/editor` | `OpenSiteInEditor("myapp")` |
| `valet-manager://restart-service/nginx` | `RestartService("nginx")` |
| `valet-manager://run-artisan/myapp/migrate` | `RunArtisanCommand { site: "myapp", .. }` |
| `valet-manager://toggle-xdebug/8.3/debug` | `EnableXdebug { version: "8.3", mode: Debug }` |
| `valet-manager://navigate/database` | Navigate to database panel |
| `valet-manager://open-palette` | `OpenCommandPalette` |

A companion Raycast extension spec ships in `integrations/raycast/` for one-click
`open valet-manager://...` actions from the macOS-style launcher.

---

### 6.14 Built-in App Updater

```rust
const GITHUB_API: &str =
    "https://api.github.com/repos/mralaminahamed/valet-manager/releases/latest";

pub async fn check_for_updates() -> anyhow::Result<UpdateInfo> {
    let resp: serde_json::Value = reqwest::get(GITHUB_API).await?.json().await?;
    let latest = resp["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");
    Ok(UpdateInfo {
        update_available: semver::Version::parse(latest)? > semver::Version::parse(current)?,
        latest_version: latest.to_string(),
        release_notes: resp["body"].as_str().unwrap_or("").to_string(),
        download_url: pick_asset_url(&resp["assets"]),
        ..Default::default()
    })
}
```

- Checked every 24h, cached. Checked at startup if cache is stale.
- Non-intrusive banner on dashboard when update is available (no modal, no focus steal).
- Install path: `.deb` via privilege helper (`dpkg -i`), AppImage via file replace + restart.

---

## 7. Updated Sidebar Navigation

```
────────────────────────────────
  MANAGEMENT
  Dashboard
  PHP Versions
  PHP Extensions
  PHP INI
  phpinfo()                ← G1
  PHP Compatibility        ← G2
  ─────────────
  Sites                    (with ★ favorites)
  Parks
  Nginx
  Proxies
  dnsmasq
  SSL Certificates         ← D8

────────────────────────────────
  DEVELOPMENT              ← new section
  Database Manager         ← D2
  .env Editor              ← D3
  Artisan Runner           ← D4
  Queue Workers            ← D5
  Xdebug                   ← D6
  Mail Catcher             ← D7

────────────────────────────────
  CREATE
  App Creator

────────────────────────────────
  TOOLS
  Sharing
  Drivers
  Logs
  Command History          ← G3
  Diagnostics

────────────────────────────────
  SERVICES
  ● nginx
  ● php8.3-fpm
  ○ php8.2-fpm
  ● dnsmasq
  ● mailpit (if running)   ← D7

  Settings
```

**Total panels: 26** (was 18 in v2)

---

## 8. New Panel Layouts

### Dashboard — updated with alert banners

```
┌─────────────────────────────────────────────────────────────────────┐
│  Dashboard                            PHP 8.3.12 · Valet 4.8.2     │
│  ─────────────────────────────────────────────────────────────────  │
│  ⚠ 2 sites have PHP compatibility issues          [View →]         │
│  ✕ myshop.test SSL cert expires in 3 days          [Regenerate]    │
│  ↑ Valet Manager v1.2.0 available                  [Update]        │
│  ─────────────────────────────────────────────────────────────────  │
│  Services:  ● nginx  ● php8.3-fpm  ● dnsmasq  ● mailpit (14)      │
│  ─────────────────────────────────────────────────────────────────  │
│  Quick actions:                                                      │
│  [Ctrl+K]  [New App]  [Open Mail UI]  [Restart All]  [Diagnostics] │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 9. Updated AppState

```rust
pub struct AppState {
    // ── v2 fields (unchanged) ──────────────────────────────────
    pub valet_variant:  Option<ValetVariant>,
    pub valet_paths:    Option<ValetPaths>,
    pub php:            PhpState,
    pub nginx:          NginxState,
    pub valet:          ValetState,
    pub dnsmasq:        DnsmasqState,
    pub services:       ServiceState,
    pub proxies:        ProxyState,
    pub sharing:        ShareState,
    pub drivers:        DriverState,
    pub cli_tools:      Vec<CliToolStatus>,
    pub creator:        AppCreatorState,
    pub ui:             UiState,

    // ── v3 additions ───────────────────────────────────────────
    pub phpinfo:        PhpInfoState,
    pub compat:         CompatibilityState,
    pub history:        HistoryState,
    pub updater:        UpdaterState,
    pub palette:        CommandPaletteState,
    pub database:       DatabaseState,
    pub artisan:        ArtisanState,
    pub env_editor:     EnvEditorState,
    pub queue:          QueueState,
    pub xdebug:         XdebugState,
    pub mail:           MailCatcherState,
    pub ssl:            SslState,
}
```

---

## 10. Updated Configuration Schema

New sections added to `~/.config/valet-manager/config.toml`:

```toml
[sites]
favorites = []

[database]
mysql_user = "root"
mysql_pass = ""
mysql_host = "127.0.0.1"
mysql_port = 3306
postgres_user = "postgres"
postgres_host = "127.0.0.1"
postgres_port = 5432
preferred_engine = "auto"

[artisan]
default_quick_commands = [
    "migrate",
    "migrate:fresh --seed",
    "route:list",
    "cache:clear",
    "config:clear",
    "queue:work",
]

[xdebug]
default_mode = "debug"
ide_key = "VSCODE"
client_host = "127.0.0.1"
client_port = 9003

[mail_catcher]
tool = "mailpit"
smtp_port = 1025
http_port = 8025
auto_start = false

[ssl]
expiry_warn_days = 14
check_interval_hours = 6

[updater]
enabled = true
check_interval_hours = 24
last_checked = ""
skipped_version = ""

[protocol]
enabled = true

[notifications]
service_status_changes = true
php_switch_complete = true
app_creation_complete = true
ssl_expiry_warning = true
queue_worker_failed = true
update_available = true
```

---

## 11. Updated Dependencies

```toml
# HTTP client (updater + Mailpit API)
reqwest     = { version = "0.12", features = ["json"] }

# SQLite (command history + artisan history)
rusqlite    = { version = "0.32", features = ["bundled"] }

# Semver (compat checker + updater)
semver      = "1"

# Fuzzy search (command palette)
fuzzy-matcher = "0.3"

# TOML editing that preserves formatting (favorites, config writes)
toml_edit   = "0.22"

# futures (concurrent compat check)
futures     = "0.3"
```

---

## 12. Updated Phased Roadmap

Phases 1–7 (weeks 1–21) remain as specified in v2. New phases below:

### Phase 8 — Gap-Closing: PHPMon Parity (Weeks 22–25)

- phpinfo() viewer (runner + parser + searchable table panel)
- PHP compatibility checker (batch concurrent check + inline in Sites panel)
- Command history & audit log (SQLite intercept + panel + CSV export)
- Favorite domains (toggle + sorting + tray section)
- `valet-manager://` deep-link registration + handler + supported action set
- Built-in app updater (GitHub API + banner on dashboard + .deb/AppImage install)
- Onboarding wizard (guided first-run: detect valet → verify PHP → test nginx)

### Phase 9 — DX Tier 1: Core Developer Workflows (Weeks 26–30)

- Command palette Ctrl+K (search index + overlay widget + keyboard navigation)
- .env file editor (parser preserving structure + validator + atomic write + panel)
- Artisan command runner (discover via `list --format=json` + stream + history)
- Database manager — Laravel integration (migrate/seed/rollback via artisan)
- Database manager — raw operations (create/drop, list tables, row counts)
- Dashboard alert banners (compatibility, SSL expiry, update badge)

### Phase 10 — DX Tier 2: Dev Environment Extensions (Weeks 31–34)

- SSL certificate dashboard (cert reader + expiry monitor + notification)
- Xdebug quick toggle (per-version cards + mode preset + IDE key + FPM restart)
- Mail catcher control (Mailpit install/start/stop + unread count + tray badge)
- Mail catcher SMTP auto-config (one-click site .env update)
- Queue worker manager (systemd user service templates + stats polling)
- Queue worker Redis integration (LLEN polling + failed_jobs display)

---

## Appendix D — Complete Feature Inventory v3.0

**40 features across 10 phases, 34 weeks.**

| # | Feature | Priority | Phase |
|---|---|---|---|
| 1 | PHP version detection | P1 | 1 |
| 2 | Global PHP switch | P1 | 2 |
| 3 | Per-site PHP isolation | P1 | 3 |
| 4 | PHP version install/remove | P1 | 2 |
| 5 | PHP extension manager | P1 | 2 |
| 6 | PHP INI editor (section + raw) | P1 | 2 |
| 7 | phpinfo() viewer | P2 | 8 |
| 8 | PHP compatibility checker | P2 | 8 |
| 9 | Sites panel (table + actions) | P1 | 3 |
| 10 | Parks management | P1 | 3 |
| 11 | TLS secure/unsecure | P1 | 3 |
| 12 | Favorite/bookmark domains | P2 | 8 |
| 13 | Site env vars (.valet-env.php) | P2 | 6 |
| 14 | .env file editor | P1 | 9 |
| 15 | Nginx site config editor | P1 | 3 |
| 16 | Proxy manager | P1 | 6 |
| 17 | dnsmasq + DNS tester | P2 | 6 |
| 18 | SSL certificate dashboard | P3 | 10 |
| 19 | Service start/stop/restart | P1 | 1 |
| 20 | Service status monitoring | P1 | 1 |
| 21 | Diagnostics panel | P2 | 6 |
| 22 | Command history & audit log | P2 | 8 |
| 23 | Log viewer | P2 | 6 |
| 24 | App Creator wizard (30+ types) | P1 | 4–5 |
| 25 | WP-CLI integration | P1 | 4 |
| 26 | Laravel CLI integration | P1 | 4 |
| 27 | Artisan command runner | P1 | 9 |
| 28 | Database manager | P1 | 9 |
| 29 | Queue worker manager | P3 | 10 |
| 30 | Xdebug quick toggle | P3 | 10 |
| 31 | Mail catcher control | P3 | 10 |
| 32 | System tray | P2 | 7 |
| 33 | Desktop notifications | P2 | 7 |
| 34 | Custom driver manager | P2 | 6 |
| 35 | Sharing (ngrok/Expose/cloudflared) | P2 | 6 |
| 36 | Multi-valet-fork support | P1 | 1 |
| 37 | Command palette (Ctrl+K) | P1 | 9 |
| 38 | Deep-link protocol | P2 | 8 |
| 39 | Built-in updater | P3 | 8 |
| 40 | First-run onboarding wizard | P2 | 8 |

---

*Document Version: 3.0 (additions to v2.0) — Author: Al Amin Ahamed (@mralaminahamed)*
