> **Claude Design command** (for Phase 12 UI wiring):
>
> ```
Fetch this design file, read its readme, and implement the relevant aspects of the design.
https://api.anthropic.com/v1/design/h/n7aIciwuf7VybL2mSu03Tg?open_file=Valet+Manager.html
Implement: Valet Manager.html
```

---

# Valet Manager — phpMyAdmin Per-Site Service
## Spec Addendum + Claude Code Prompt Update

> Extends per-site-config-spec-and-prompts.md.
> Add this to Phase 11 before running any Claude Code prompts.

---

## 1. What We're Building

| Mode | URL | Shows | Auth |
|---|---|---|---|
| **Per-site** | `https://mylaravel.test/_pma/` | Only `mylaravel_db` | Auto-login (config auth) |
| **All databases** | `https://phpmyadmin.test/` | All accessible DBs | Login prompt |
| **Per-site subdomain** | `https://pma.mylaravel.test/` | Only `mylaravel_db` | Auto-login |

User chooses preferred access mode per site in the site config panel.
A global phpMyAdmin valet site (`phpmyadmin.test`) is also created for full access.

---

## 2. How phpMyAdmin Config Restriction Works

phpMyAdmin loads a `config.inc.php` from a directory set by the
`PMA_CONFIG_DIR` environment variable (passed via `fastcgi_param` in Nginx).

Each site gets its own config directory:
```
~/.config/valet-manager/phpmyadmin/sites/{site-name}/config.inc.php
```

The config file sets `$cfg['Servers'][1]['only_db']` to restrict the
visible database. For "all databases" mode the field is omitted.

phpMyAdmin's `blowfish_secret` is generated once at install time and
shared across all site configs (stored in global app config, not per-site).

---

## 3. Data Models (add to src/site_config/models.rs)

```rust
// Add to SiteConfig struct:
#[serde(default)]
pub phpmyadmin: PhpMyAdminSiteConfig,

// New struct:
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhpMyAdminSiteConfig {
    /// Whether phpMyAdmin is enabled for this site
    #[serde(default)]
    pub enabled: bool,

    /// How to serve phpMyAdmin for this site
    #[serde(default)]
    pub access_mode: PmaAccessMode,

    /// DB scope — this site's DB only, or all accessible DBs
    #[serde(default)]
    pub db_scope: PmaDbScope,

    /// Override the DB name detected from .env (leave None to auto-detect)
    pub db_name_override: Option<String>,

    /// Custom path when mode = PathAlias (default "_pma")
    #[serde(default = "default_pma_path")]
    pub path_alias: String,
}

fn default_pma_path() -> String { "_pma".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PmaAccessMode {
    /// https://{site.domain}/{path_alias}/
    #[default]
    PathAlias,
    /// https://pma.{site.domain}/
    Subdomain,
    /// Use the global phpmyadmin.{tld} site only
    GlobalOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PmaDbScope {
    /// Show only this site's database (auto-detected or overridden)
    #[default]
    SiteOnly,
    /// Show all databases the DB user can access
    AllDatabases,
}
```

---

## 4. Global phpMyAdmin State (add to AppState)

```rust
// src/state/phpmyadmin_state.rs

#[derive(Debug, Default, Clone)]
pub struct PhpMyAdminState {
    /// Installation status
    pub installed: bool,
    pub install_path: Option<PathBuf>,  // /usr/share/phpmyadmin or custom
    pub version: Option<String>,

    /// The global valet site for all-DB access
    pub global_site_configured: bool,
    pub global_site_domain: String,     // "phpmyadmin.test"

    /// Per-site status: site_name → PmaSiteStatus
    pub sites: HashMap<String, PmaSiteStatus>,
}

#[derive(Debug, Clone, Default)]
pub struct PmaSiteStatus {
    pub enabled: bool,
    pub access_url: String,       // the full URL to open
    pub config_path: PathBuf,     // the generated config.inc.php path
    pub db_name: String,          // resolved DB name being shown
    pub access_mode: PmaAccessMode,
    pub last_configured: Option<chrono::DateTime<chrono::Local>>,
}
```

---

## 5. New AppCommand Variants (add to src/commands.rs)

```rust
// phpMyAdmin
CheckPhpMyAdminInstalled,
InstallPhpMyAdmin,                                  // apt install / manual download
UninstallPhpMyAdmin,
ConfigurePhpMyAdminForSite(String),                 // site name → generate config + inject nginx
RemovePhpMyAdminFromSite(String),                   // remove config + nginx block
OpenPhpMyAdmin { site: String, scope: PmaDbScope }, // xdg-open the URL
SetupGlobalPhpMyAdminSite,                          // create phpmyadmin.test valet site
SetPmaAccessMode { site: String, mode: PmaAccessMode },
SetPmaDbScope { site: String, scope: PmaDbScope },
RefreshPhpMyAdminStatus,
```

---

## 6. New Module: src/phpmyadmin/

### 6.1 Installer — src/phpmyadmin/installer.rs

```rust
use std::path::{Path, PathBuf};

/// Standard installation paths to probe (in order)
const PROBE_PATHS: &[&str] = &[
    "/usr/share/phpmyadmin",
    "/usr/local/share/phpmyadmin",
    "/var/www/phpmyadmin",
    "/opt/phpmyadmin",
];

pub async fn detect() -> Option<(PathBuf, String)>
    // Returns (install_path, version_string) if found
    // 1. Probe PROBE_PATHS for index.php existence
    // 2. If found, read {path}/libraries/classes/DatabaseInterface.php
    //    or {path}/ChangeLog line 1 for version string
    // 3. Return Some((path, version)) or None

pub async fn is_installed() -> bool
    detect().await.is_some()

pub async fn install_via_apt(tx: mpsc::Sender<OutputLine>) -> anyhow::Result<PathBuf>
    // stream_command: "sudo apt install phpmyadmin -y"
    // phpMyAdmin apt install is interactive (asks for web server config).
    // Use DEBIAN_FRONTEND=noninteractive with pre-seeded answers:
    //   echo "phpmyadmin phpmyadmin/dbconfig-install boolean true" | debconf-set-selections
    //   echo "phpmyadmin phpmyadmin/app-password-confirm password" | debconf-set-selections
    //   echo "phpmyadmin phpmyadmin/mysql/admin-pass password" | debconf-set-selections
    //   echo "phpmyadmin phpmyadmin/mysql/app-pass password" | debconf-set-selections
    //   echo "phpmyadmin phpmyadmin/reconfigure-webserver multiselect" | debconf-set-selections
    // Then: DEBIAN_FRONTEND=noninteractive apt-get install -y phpmyadmin
    // Via privilege helper.
    // Return /usr/share/phpmyadmin path.

pub async fn install_manual(tx: mpsc::Sender<OutputLine>) -> anyhow::Result<PathBuf>
    // Download latest phpMyAdmin from GitHub releases API
    // GET https://api.github.com/repos/phpmyadmin/phpmyadmin/releases/latest
    // Download phpMyAdmin-{ver}-all-languages.tar.gz
    // Extract to ~/.config/valet-manager/phpmyadmin/app/
    // Return that path.
    // Use reqwest for download, flate2 + tar for extraction.

pub async fn get_latest_version() -> anyhow::Result<String>
    // GET https://api.github.com/repos/phpmyadmin/phpmyadmin/releases/latest
    // Parse tag_name
```

### 6.2 Config Generator — src/phpmyadmin/config_generator.rs

```rust
/// Directory where per-site phpMyAdmin configs live
pub fn site_config_dir(site_name: &str) -> PathBuf
    dirs::config_dir().unwrap()
        .join("valet-manager/phpmyadmin/sites")
        .join(site_name)

pub fn site_config_path(site_name: &str) -> PathBuf
    site_config_dir(site_name).join("config.inc.php")

pub fn global_config_path() -> PathBuf
    dirs::config_dir().unwrap()
        .join("valet-manager/phpmyadmin/global/config.inc.php")

/// Generate the blowfish_secret once, store in AppConfig
pub fn generate_blowfish_secret() -> String
    use rand::Rng;
    (0..32)
        .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric) as char)
        .collect()

/// Generate per-site config.inc.php
pub fn render_site_config(
    db_name: Option<&str>,     // None = all databases
    db_user: &str,
    db_pass: &str,
    db_host: &str,
    db_port: u16,
    blowfish_secret: &str,
    auth_type: PmaAuthType,
) -> String
    let only_db_line = match db_name {
        Some(name) => format!("$cfg['Servers'][$i]['only_db'] = '{name}';"),
        None       => "// All databases accessible".to_string(),
    };
    let (auth_line, user_line, pass_line) = match auth_type {
        PmaAuthType::Config =>
            ("config", format!("$cfg['Servers'][$i]['user'] = '{db_user}';"),
             format!("$cfg['Servers'][$i]['password'] = '{db_pass}';")),
        PmaAuthType::Cookie =>
            ("cookie", String::new(), String::new()),
    };
    format!(r#"<?php
/**
 * phpMyAdmin config — generated by Valet Manager
 * Site: {site}  |  DB scope: {scope}
 * DO NOT EDIT — changes will be overwritten on next save.
 */
declare(strict_types=1);

$cfg['blowfish_secret'] = '{blowfish_secret}';

$i = 0;
$i++;

$cfg['Servers'][$i]['auth_type']    = '{auth_type}';
$cfg['Servers'][$i]['host']         = '{db_host}';
$cfg['Servers'][$i]['port']         = '{db_port}';
$cfg['Servers'][$i]['compress']     = false;
$cfg['Servers'][$i]['AllowNoPassword'] = false;
{user_line}
{pass_line}
{only_db_line}

$cfg['UploadDir']  = '';
$cfg['SaveDir']    = '';
$cfg['SendErrorReports'] = 'never';
"#)

#[derive(Debug, Clone, PartialEq)]
pub enum PmaAuthType {
    Config,  // auto-login with stored credentials
    Cookie,  // login prompt
}

pub async fn write_site_config(
    site_name: &str,
    config_content: &str,
) -> anyhow::Result<()>
    // Create site_config_dir(site_name) if not exists
    // Write config_content to site_config_path(site_name)
    // Permissions: 640 (readable by web server, not world-readable)
    //   use nix::sys::stat::chmod

pub async fn remove_site_config(site_name: &str) -> anyhow::Result<()>
    // Remove site_config_dir(site_name) recursively if exists
```

### 6.3 Nginx Integration — src/phpmyadmin/nginx_integration.rs

```rust
/// Nginx location block for path-alias mode (e.g. /_pma/)
pub fn render_path_alias_block(
    path_alias: &str,              // "_pma"
    pma_install_path: &Path,       // /usr/share/phpmyadmin
    config_dir: &Path,             // ~/.config/valet-manager/phpmyadmin/sites/{name}
    php_fpm_socket: &str,          // unix:/run/php/php8.3-fpm.sock
) -> String
    Returns a ready-to-inject Nginx location block:
    ```nginx
    # BEGIN valet-manager phpmyadmin
    location /{path_alias} {{
        return 301 /{path_alias}/;
    }}
    location /{path_alias}/ {{
        alias {pma_install_path}/;
        index index.php;

        location ~ ^/{path_alias}/(.+\.php)$ {{
            fastcgi_split_path_info ^(.+\.php)(/.+)$;
            fastcgi_pass {php_fpm_socket};
            fastcgi_index index.php;
            fastcgi_param SCRIPT_FILENAME {pma_install_path}/$1;
            fastcgi_param PMA_CONFIG_DIR {config_dir}/;
            fastcgi_param SCRIPT_NAME /{path_alias}/$1;
            include fastcgi_params;
        }}

        location ~* ^/{path_alias}/(.+\.(jpg|jpeg|gif|css|png|js|ico|html|xml|txt))$ {{
            alias {pma_install_path}/$1;
            access_log off;
            expires 1d;
        }}
    }}
    # END valet-manager phpmyadmin
    ```

/// Inject the Nginx block into the site's config file
pub async fn inject_into_site(
    site: &ValetSite,
    block: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()>
    // Read the site's Nginx config
    // Find "# BEGIN valet-manager phpmyadmin" ... "# END valet-manager phpmyadmin"
    //   → if found: replace the block between sentinels
    //   → if not found: insert before the last closing brace of server{}
    // Write via privilege helper → reload nginx

/// Remove the phpMyAdmin block from the site's Nginx config
pub async fn remove_from_site(
    site: &ValetSite,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()>
    // Read Nginx config
    // Remove lines from "# BEGIN valet-manager phpmyadmin" to "# END valet-manager phpmyadmin"
    // Write back → reload nginx

/// For subdomain mode: create a new valet-linked site for pma.{site.domain}
pub async fn create_subdomain_site(
    site: &ValetSite,
    pma_install_path: &Path,
    config_dir: &Path,
    php_fpm_socket: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()>
    // Write Nginx config for pma.{site.domain}
    // Call `valet link pma.{site.name}` pointing to pma_install_path
    // Create a minimal index.php at pma_install_path that loads the right config
    // (Actually: create a wrapper PHP file that sets PMA_CONFIG_DIR then includes
    //  the real phpMyAdmin index.php from the install path)
```

### 6.4 Global Site Setup — src/phpmyadmin/global_site.rs

```rust
/// Create the phpmyadmin.{tld} valet site for all-database access
pub async fn setup_global_site(
    pma_install_path: &Path,
    global_config_path: &Path,
    tld: &str,
    php_fpm_socket: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<String>  // returns the URL
    // 1. Verify phpMyAdmin is installed at pma_install_path
    // 2. Write global config (no only_db restriction, cookie auth for security)
    // 3. Create Nginx config for phpmyadmin.{tld}:
    //      server_name phpmyadmin.{tld};
    //      root {pma_install_path};
    //      fastcgi_param PMA_CONFIG_DIR {global_config_path.parent()}/;
    //      (Standard PHP-FPM location block)
    // 4. valet link phpmyadmin in the pma_install_path (or use valet park equivalent)
    //    Actually: write nginx config directly to valet_paths.nginx_dir/phpmyadmin.{tld}
    //    and add a symlink in valet_paths.sites_dir/phpmyadmin → pma_install_path
    // 5. valet secure phpmyadmin (so it's HTTPS)
    // 6. Return "https://phpmyadmin.{tld}"

pub async fn is_global_site_configured(tld: &str, valet_paths: &ValetPaths) -> bool
    // Check if phpmyadmin.{tld} nginx config exists
```

### 6.5 Orchestrator — src/phpmyadmin/mod.rs

```rust
/// Main entry point called by the command dispatcher.
/// Wires installer + config_generator + nginx_integration together.
pub async fn configure_for_site(
    site: &ValetSite,
    site_config: &SiteConfig,
    installed_path: &Path,
    blowfish_secret: &str,
    valet_paths: &ValetPaths,
    tx: mpsc::Sender<OutputLine>,
) -> anyhow::Result<PmaSiteStatus>

    // 1. Resolve DB credentials:
    //    Read site's .env for DB_DATABASE, DB_USERNAME, DB_PASSWORD, DB_HOST, DB_PORT
    //    If env_vars not found, try reading wp-config.php for WordPress sites
    //    Apply db_name_override if set in site_config.phpmyadmin
    let (db_name, db_user, db_pass, db_host, db_port) = resolve_db_credentials(site, site_config)?;

    // 2. Determine auth type:
    //    SiteOnly scope → Config auth (auto-login, single DB visible)
    //    AllDatabases scope → Cookie auth (login prompt, all DBs visible)
    let auth_type = if site_config.phpmyadmin.db_scope == PmaDbScope::SiteOnly {
        PmaAuthType::Config
    } else {
        PmaAuthType::Cookie
    };

    // 3. Determine db_name to restrict (None = all DBs)
    let restrict_db = if site_config.phpmyadmin.db_scope == PmaDbScope::SiteOnly {
        Some(db_name.as_str())
    } else {
        None
    };

    // 4. Generate and write config.inc.php
    let config_content = config_generator::render_site_config(
        restrict_db, &db_user, &db_pass, &db_host, db_port,
        blowfish_secret, auth_type,
    );
    config_generator::write_site_config(&site.name, &config_content).await?;
    tx.send(OutputLine::stdout("phpMyAdmin config generated")).await?;

    // 5. Configure Nginx based on access mode
    let php_fpm_socket = format!("unix:/run/php/php{}-fpm.sock",
        site.php_version.as_deref().unwrap_or("8.3"));
    let config_dir = config_generator::site_config_dir(&site.name);

    let access_url = match site_config.phpmyadmin.access_mode {
        PmaAccessMode::PathAlias => {
            let block = nginx_integration::render_path_alias_block(
                &site_config.phpmyadmin.path_alias,
                installed_path,
                &config_dir,
                &php_fpm_socket,
            );
            nginx_integration::inject_into_site(site, &block, valet_paths).await?;
            tx.send(OutputLine::stdout("Nginx config updated")).await?;
            format!("https://{}/{}/" , site.domain, site_config.phpmyadmin.path_alias)
        }
        PmaAccessMode::Subdomain => {
            nginx_integration::create_subdomain_site(
                site, installed_path, &config_dir, &php_fpm_socket, valet_paths
            ).await?;
            format!("https://pma.{}/", site.domain)
        }
        PmaAccessMode::GlobalOnly => {
            format!("https://phpmyadmin.{}/", valet_tld)
        }
    };

    tx.send(OutputLine::stdout(format!("phpMyAdmin ready at: {access_url}"))).await?;

    Ok(PmaSiteStatus {
        enabled: true,
        access_url,
        config_path: config_generator::site_config_path(&site.name),
        db_name: db_name.clone(),
        access_mode: site_config.phpmyadmin.access_mode.clone(),
        last_configured: Some(chrono::Local::now()),
    })

/// Resolve database credentials from the site's .env or wp-config.php
fn resolve_db_credentials(
    site: &ValetSite,
    site_config: &SiteConfig,
) -> anyhow::Result<(String, String, String, String, u16)>
    // Try .env first (covers Laravel, Symfony, generic PHP)
    let env_path = site.path.join(".env");
    if env_path.exists() {
        let content = std::fs::read_to_string(&env_path)?;
        let db_name = site_config.phpmyadmin.db_name_override.clone()
            .or_else(|| parse_env_key(&content, "DB_DATABASE"))
            .unwrap_or_else(|| format!("{}_db", site.name));
        let db_user = parse_env_key(&content, "DB_USERNAME").unwrap_or("root".into());
        let db_pass = parse_env_key(&content, "DB_PASSWORD").unwrap_or_default();
        let db_host = parse_env_key(&content, "DB_HOST").unwrap_or("127.0.0.1".into());
        let db_port = parse_env_key(&content, "DB_PORT")
            .and_then(|p| p.parse().ok()).unwrap_or(3306u16);
        return Ok((db_name, db_user, db_pass, db_host, db_port));
    }
    // WordPress: try wp-config.php
    let wp_config = site.path.join("wp-config.php");
    if wp_config.exists() {
        let content = std::fs::read_to_string(&wp_config)?;
        let db_name = site_config.phpmyadmin.db_name_override.clone()
            .or_else(|| parse_wp_constant(&content, "DB_NAME"))
            .unwrap_or_else(|| format!("wp_{}", site.name));
        let db_user = parse_wp_constant(&content, "DB_USER").unwrap_or("root".into());
        let db_pass = parse_wp_constant(&content, "DB_PASSWORD").unwrap_or_default();
        let db_host = parse_wp_constant(&content, "DB_HOST").unwrap_or("127.0.0.1".into());
        return Ok((db_name, db_user, db_pass, db_host, 3306));
    }
    // Fallback: use AppConfig defaults
    Ok((
        format!("{}_db", site.name),
        app_config.database.mysql_user.clone(),
        app_config.database.mysql_pass.clone(),
        "127.0.0.1".into(),
        3306,
    ))
```

---

## 7. Updated Module Structure

```
src/
└── phpmyadmin/
    ├── mod.rs                  ← orchestrator (configure_for_site)
    ├── installer.rs            ← detect/install phpMyAdmin
    ├── config_generator.rs     ← render + write config.inc.php
    ├── nginx_integration.rs    ← inject/remove Nginx blocks
    └── global_site.rs          ← setup phpmyadmin.{tld} valet site
```

---

## 8. Updated SiteConfig TOML Example

```toml
# .valet-manager.toml — per-site config (committed to git)

[php]
version = "8.3"

[php.ini_overrides]
memory_limit = "512M"
upload_max_filesize = "64M"

[wordpress]
multisite_enabled = true
multisite_type = "subdomain"
wp_debug = true

# phpMyAdmin — per-site
[phpmyadmin]
enabled = true
access_mode = "path_alias"   # path_alias | subdomain | global_only
db_scope = "site_only"       # site_only | all_databases
path_alias = "_pma"
# db_name_override = "custom_db_name"  # optional override
```

---

## 9. New Dependencies (add to Cargo.toml)

```toml
rand    = { version = "0.8", features = ["std"] }       # blowfish_secret generation
flate2  = "1.0"                                          # tar.gz extraction for manual install
tar     = "0.4"                                          # tar extraction
```

---

## 10. Claude Code Prompt — Phase 11c: phpMyAdmin Service

```
You are adding phpMyAdmin per-site service support to Valet Manager.
Phase 11 (per-site config backend) is complete.
Phase 11b (UI wiring) is complete.
The UI panel for phpMyAdmin already exists — you are adding the backend logic only.

Read these files before writing anything:
  src/site_config/models.rs          — add PhpMyAdminSiteConfig here
  src/state/app_state.rs             — add PhpMyAdminState here
  src/commands.rs                    — add phpMyAdmin AppCommand variants
  src/phpmyadmin/                    — create this entire module from scratch

══════════════════════════════════════════════════════════
STEP 1 — UPDATE DATA MODELS
══════════════════════════════════════════════════════════

In src/site_config/models.rs:

Add these enums and structs exactly as specified in the architecture spec above
(PhpMyAdminSiteConfig, PmaAccessMode, PmaDbScope).

Add the phpmyadmin field to SiteConfig:
  #[serde(default)]
  pub phpmyadmin: PhpMyAdminSiteConfig,

In src/state/app_state.rs:

Create src/state/phpmyadmin_state.rs with PhpMyAdminState and PmaSiteStatus structs.
Add to AppState:
  pub phpmyadmin: PhpMyAdminState,

In src/commands.rs:
Add all AppCommand variants from section 5 of this document.

══════════════════════════════════════════════════════════
STEP 2 — INSTALLER MODULE
══════════════════════════════════════════════════════════

Create src/phpmyadmin/installer.rs implementing:

  pub async fn detect() -> Option<(PathBuf, String)>
    Probe these paths in order:
      /usr/share/phpmyadmin
      /usr/local/share/phpmyadmin
      /var/www/phpmyadmin
      ~/.config/valet-manager/phpmyadmin/app
    For each: check if "index.php" and "libraries/" exist.
    Version: try reading from ChangeLog or VERSION file at the root.
    Return Some((path, version)) for first match, None if none found.

  pub async fn is_installed() -> bool
    detect().await.is_some()

  pub async fn install_via_apt(tx: mpsc::Sender<OutputLine>) -> anyhow::Result<PathBuf>
    Pre-seed debconf to avoid interactive prompts:
      echo "phpmyadmin phpmyadmin/dbconfig-install boolean true"
      echo "phpmyadmin phpmyadmin/app-password-confirm password "
      echo "phpmyadmin phpmyadmin/mysql/admin-pass password "
      echo "phpmyadmin phpmyadmin/mysql/app-pass password "
      echo "phpmyadmin phpmyadmin/reconfigure-webserver multiselect "
    Stream these via privilege helper (op: "shell_script").
    Then: DEBIAN_FRONTEND=noninteractive apt-get install -y phpmyadmin
    Also via privilege helper.
    Return /usr/share/phpmyadmin.

  pub async fn install_manual(tx: mpsc::Sender<OutputLine>) -> anyhow::Result<PathBuf>
    1. GET https://api.github.com/repos/phpmyadmin/phpmyadmin/releases/latest
       Parse assets for file ending in "-all-languages.tar.gz"
    2. Stream download to /tmp/phpmyadmin-latest.tar.gz using reqwest with
       Content-Length progress (send OutputLine "Downloading X%" every 10%)
    3. Extract: use flate2::read::GzDecoder + tar::Archive
       to ~/.config/valet-manager/phpmyadmin/app/
    4. Return the extracted path.
    Add rand, flate2, tar to Cargo.toml if not present.

  pub async fn get_latest_release_url() -> anyhow::Result<(String, String)>
    Returns (download_url, version_string).

══════════════════════════════════════════════════════════
STEP 3 — CONFIG GENERATOR MODULE
══════════════════════════════════════════════════════════

Create src/phpmyadmin/config_generator.rs implementing all functions from
section 6.2 of this document.

Key implementation notes:
- generate_blowfish_secret(): use rand::distributions::Alphanumeric, 32 chars
- render_site_config(): produce the PHP config string EXACTLY as specified,
  with all fields present even if empty (use empty string not missing)
- write_site_config(): create directories with fs::create_dir_all, write file,
  then chmod 640 using nix::sys::stat::fchmodat or std::fs::set_permissions
- The config dir is USER-OWNED (~/.config/valet-manager/) so no privilege helper needed

Security notes:
- Never log db_pass values (use "●●●●" in OutputLine messages)
- chmod 640 on config files (owner read/write, group read, world none)
- For cookie auth configs: omit user/password from config entirely

══════════════════════════════════════════════════════════
STEP 4 — NGINX INTEGRATION MODULE
══════════════════════════════════════════════════════════

Create src/phpmyadmin/nginx_integration.rs.

render_path_alias_block(path_alias, pma_install_path, config_dir, php_fpm_socket)
  Returns the Nginx location block string EXACTLY as in section 6.3.
  Use format!() with proper escaping of braces (double them: {{ and }}).
  The fastcgi_param PMA_CONFIG_DIR line must end with "/" (phpMyAdmin requires it).

inject_into_site(site, block, valet_paths) -> anyhow::Result<()>
  1. Read nginx config: fs::read_to_string(nginx_config_path_for_site)
  2. Find sentinel: if "# BEGIN valet-manager phpmyadmin" exists in content:
       Replace everything from BEGIN to END sentinel with the new block.
       Use a regex or split_once approach, not substring index arithmetic.
  3. If no sentinel: find the last `}` in the file (closing server brace).
       Insert the block BEFORE the last `}`.
  4. Write back via privilege helper (op: "write_file", path, content).
  5. Send privilege helper reload: (op: "systemctl", action: "reload", service: "nginx")

remove_from_site(site, valet_paths) -> anyhow::Result<()>
  1. Read nginx config.
  2. If sentinels exist: remove everything from BEGIN to END sentinel line (inclusive).
  3. Write back → reload nginx.

create_subdomain_site(site, pma_install_path, config_dir, php_fpm_socket, valet_paths)
  -> anyhow::Result<()>
  1. The "site" for pma.{site.name} needs an Nginx config AND a symlink in Sites dir.
  2. Write Nginx config: render a standard PHP-FPM nginx config template but with:
       server_name pma.{site.name}.{tld};
       root {pma_install_path};
       fastcgi_param PMA_CONFIG_DIR {config_dir}/;
  3. Create symlink: valet_paths.sites_dir.join("pma.{site.name}") → pma_install_path
     use std::os::unix::fs::symlink (no privilege needed if Sites dir is user-owned)
  4. Write nginx config → reload nginx.

══════════════════════════════════════════════════════════
STEP 5 — GLOBAL SITE MODULE
══════════════════════════════════════════════════════════

Create src/phpmyadmin/global_site.rs implementing setup_global_site and
is_global_site_configured from section 6.4.

For setup_global_site:
  The global config uses PmaAuthType::Cookie (login prompt) and no only_db restriction.
  This is for security — the global site shows all DBs but requires login.
  Generate the config via config_generator::render_site_config(None, "", "", ...)
    with auth_type=Cookie.
  Write Nginx config to: valet_paths.nginx_dir.join("phpmyadmin.{tld}")
  Create Sites symlink: valet_paths.sites_dir.join("phpmyadmin") → pma_install_path

══════════════════════════════════════════════════════════
STEP 6 — MAIN ORCHESTRATOR
══════════════════════════════════════════════════════════

Create src/phpmyadmin/mod.rs implementing:
  - configure_for_site() — full flow as specified in section 6.5
  - resolve_db_credentials() — try .env then wp-config.php then defaults
  - parse_env_key(content, key) → Option<String>
      regex: `^{key}=(.+)$` multiline
  - parse_wp_constant(content, name) → Option<String>
      regex: `define\s*\(\s*'{name}'\s*,\s*'([^']+)'\s*\)`

Also implement:
  pub async fn remove_from_site(site: &ValetSite, valet_paths: &ValetPaths)
    -> anyhow::Result<()>
    config_generator::remove_site_config(&site.name).await?
    nginx_integration::remove_from_site(site, valet_paths).await?

══════════════════════════════════════════════════════════
STEP 7 — COMMAND DISPATCHER WIRING
══════════════════════════════════════════════════════════

Add match arms to run_dispatcher() in src/app.rs:

  CheckPhpMyAdminInstalled → {
    let result = phpmyadmin::installer::detect().await;
    let (installed, path, version) = match result {
      Some((p, v)) => (true, Some(p), Some(v)),
      None         => (false, None, None),
    };
    state.write().await.phpmyadmin.installed = installed;
    state.write().await.phpmyadmin.install_path = path;
    state.write().await.phpmyadmin.version = version;
    event_tx.send(AppEvent::PhpMyAdminStatusUpdated).await?;
  }

  InstallPhpMyAdmin → {
    let (otx, orx) = mpsc::channel(256);
    // Check if apt is available → install_via_apt else install_manual
    let pm = detect_package_manager();
    let path = if pm == PackageManager::Apt {
      installer::install_via_apt(otx).await?
    } else {
      installer::install_manual(otx).await?
    };
    // Stream output lines to GUI
    dispatch!(CheckPhpMyAdminInstalled);
    dispatch!(SetupGlobalPhpMyAdminSite);
  }

  ConfigurePhpMyAdminForSite(site_name) → {
    let state_r = state.read().await;
    let site = find_site(&state_r.sites, &site_name)?;
    let site_config = state_r.site_configs.get(&site_name).cloned().unwrap_or_default();
    let pma_path = state_r.phpmyadmin.install_path.clone()
      .ok_or(anyhow!("phpMyAdmin not installed"))?;
    let blowfish_secret = state_r.config.phpmyadmin_blowfish_secret.clone();
    drop(state_r);

    let (tx, _rx) = mpsc::channel(256);
    let status = phpmyadmin::configure_for_site(
      &site, &site_config, &pma_path, &blowfish_secret, &valet_paths, tx
    ).await?;

    state.write().await.phpmyadmin.sites.insert(site_name, status);
  }

  RemovePhpMyAdminFromSite(site_name) → {
    let site = find_site_by_name(&site_name)?;
    phpmyadmin::remove_from_site(&site, &valet_paths).await?;
    state.write().await.phpmyadmin.sites.remove(&site_name);
  }

  OpenPhpMyAdmin { site, scope } → {
    // If scope == AllDatabases, use global URL
    // Otherwise use per-site URL from state.phpmyadmin.sites[site].access_url
    let url = if scope == PmaDbScope::AllDatabases {
      state.read().await.phpmyadmin.global_site_domain.clone()
    } else {
      state.read().await.phpmyadmin.sites.get(&site)
        .map(|s| s.access_url.clone())
        .unwrap_or_else(|| format!("https://phpmyadmin.test/"))
    };
    tokio::process::Command::new("xdg-open").arg(&url).spawn()?;
  }

  SetupGlobalPhpMyAdminSite → {
    let pma_path = get_pma_install_path(&state).await?;
    let global_config = global_config_path();
    let blowfish = get_blowfish(&state).await;
    let config = config_generator::render_site_config(
      None, "", "", "127.0.0.1", 3306, &blowfish, PmaAuthType::Cookie
    );
    config_generator::write_site_config("global", &config).await?;
    let url = global_site::setup_global_site(
      &pma_path, &global_config, &tld, &php_fpm_socket, &valet_paths
    ).await?;
    state.write().await.phpmyadmin.global_site_configured = true;
    state.write().await.phpmyadmin.global_site_domain = url;
  }

  SetPmaAccessMode { site, mode } → {
    let mut cfg = get_site_config_mut(&site, &state).await;
    cfg.phpmyadmin.access_mode = mode;
    save_site_config(&site, &cfg).await?;
    dispatch!(ConfigurePhpMyAdminForSite(site));
  }

  SetPmaDbScope { site, scope } → {
    let mut cfg = get_site_config_mut(&site, &state).await;
    cfg.phpmyadmin.db_scope = scope;
    save_site_config(&site, &cfg).await?;
    dispatch!(ConfigurePhpMyAdminForSite(site));
  }

══════════════════════════════════════════════════════════
STEP 8 — STARTUP INTEGRATION
══════════════════════════════════════════════════════════

In src/main.rs after app state is initialised:
  1. dispatch!(CheckPhpMyAdminInstalled) immediately at startup.
  2. If phpmyadmin.installed:
       For each site in state.sites where site_config.phpmyadmin.enabled:
         dispatch!(ConfigurePhpMyAdminForSite(site.name))

In src/config.rs, add to AppConfig:
  /// Generated once, stored in config.toml, used for all phpMyAdmin cookie encryption
  #[serde(default)]
  pub phpmyadmin_blowfish_secret: String,

In config::load(): if phpmyadmin_blowfish_secret is empty, generate one:
  config.phpmyadmin_blowfish_secret = phpmyadmin::config_generator::generate_blowfish_secret();
  config::save(&config)?;  // persist immediately

══════════════════════════════════════════════════════════
STEP 9 — UI WIRING (phpMyAdmin tab in site config panel)
══════════════════════════════════════════════════════════

The phpMyAdmin tab already exists from Claude Design. Wire these controls:

  "Enable phpMyAdmin" toggle →
    site_config.phpmyadmin.enabled = !enabled
    if enabling: dispatch!(ConfigurePhpMyAdminForSite(site))
    else: dispatch!(RemovePhpMyAdminFromSite(site))

  Install button (shows when phpmyadmin.installed = false):
    accent_button("Install phpMyAdmin") → dispatch!(InstallPhpMyAdmin)
    Show streaming terminal output below

  Access mode selector (Path alias / Subdomain / Global only):
    onChange → dispatch!(SetPmaAccessMode { site, mode })

  DB scope selector (Site's DB only / All databases):
    onChange → dispatch!(SetPmaDbScope { site, scope })

  DB name override TextEdit:
    onChange → update site_config.phpmyadmin.db_name_override in state
    Save button → dispatch!(SaveSiteConfig + ConfigurePhpMyAdminForSite)

  "Open phpMyAdmin" accent button:
    → dispatch!(OpenPhpMyAdmin { site, scope: site_config.phpmyadmin.db_scope })

  "Open (all databases)" ghost button:
    → dispatch!(OpenPhpMyAdmin { site, scope: PmaDbScope::AllDatabases })

  Current access URL display:
    Show state.phpmyadmin.sites[site].access_url in ACCENT color as clickable link

  DB name display (read-only):
    Show state.phpmyadmin.sites[site].db_name in TEXT_SECONDARY
    Below: small text "Detected from .env / wp-config.php"

══════════════════════════════════════════════════════════
STEP 10 — DASHBOARD INTEGRATION
══════════════════════════════════════════════════════════

In src/ui/panels/dashboard.rs quick actions row, add:
  If state.phpmyadmin.installed AND any site has phpmyadmin.enabled:
    ghost_button("🗄 phpMyAdmin") → dispatch!(OpenPhpMyAdmin {
      site: first_pma_enabled_site,
      scope: PmaDbScope::SiteOnly
    })

In Sites panel context menu (src/ui/panels/sites.rs):
  Add to ⋮ menu for each site where site_config.phpmyadmin.enabled:
    "Open phpMyAdmin →" → dispatch!(OpenPhpMyAdmin { site, scope: SiteOnly })
    "Open phpMyAdmin (all DBs) →" → dispatch!(OpenPhpMyAdmin { site, scope: AllDatabases })

══════════════════════════════════════════════════════════
TESTS — tests/integration/phpmyadmin_test.rs
══════════════════════════════════════════════════════════

Write tests covering:

  test_config_render_site_only():
    Call render_site_config(Some("mydb"), "root", "pass", "127.0.0.1", 3306, "secret", Config)
    Assert: output contains "only_db = 'mydb'"
    Assert: output contains "auth_type'] = 'config'"
    Assert: output contains "blowfish_secret'] = 'secret'"

  test_config_render_all_databases():
    Call render_site_config(None, ..., Cookie)
    Assert: output does NOT contain "only_db"
    Assert: output does NOT contain "password" (cookie auth omits credentials)
    Assert: output contains "auth_type'] = 'cookie'"

  test_ini_overrides_is_empty():
    Default PhpIniOverrides → is_empty() == true
    With one field set → is_empty() == false

  test_resolve_env_credentials():
    Create a tempdir with a .env containing DB_DATABASE=testdb etc.
    Call parse_env_key on the content, assert correct values extracted.

  test_resolve_wp_credentials():
    Create a tempdir with wp-config.php containing define('DB_NAME', 'wpdb')
    Call parse_wp_constant, assert "wpdb" returned.

  test_nginx_sentinel_injection():
    Start with a sample nginx config string (no phpMyAdmin block)
    Call inject logic (extract the sentinel insertion logic as pure fn)
    Assert the block appears before the last closing brace
    Call inject again with updated block
    Assert the block was REPLACED not duplicated (only one BEGIN sentinel)

  test_nginx_sentinel_removal():
    Start with nginx config containing sentinel block
    Call remove logic
    Assert sentinels and content between them are gone
    Assert rest of config is intact

══════════════════════════════════════════════════════════
ACCEPTANCE CRITERIA — Phase 11c
══════════════════════════════════════════════════════════
  □ cargo test --workspace — all tests pass including phpmyadmin_test.rs
  □ cargo clippy -- -D warnings — zero warnings
  □ phpMyAdmin detected correctly at /usr/share/phpmyadmin if installed
  □ Generated config.inc.php contains correct only_db for site-only mode
  □ Generated config.inc.php has NO only_db line for all-databases mode
  □ config.inc.php file permissions are 640 (not world-readable)
  □ Nginx path-alias block injected correctly — opens in browser at /_pma/
  □ Re-applying Nginx block replaces not duplicates (sentinel pattern works)
  □ Removing phpMyAdmin cleans both config file and Nginx block
  □ Global phpmyadmin.{tld} site accessible at https://phpmyadmin.test/
  □ "Open phpMyAdmin" in Sites panel ⋮ menu opens the correct URL
  □ DB credentials auto-detected from .env for Laravel sites
  □ DB credentials auto-detected from wp-config.php for WordPress sites
  □ Manual phpMyAdmin install via tar.gz extracts and is detected
  □ blowfish_secret generated once and persisted in config.toml
  □ Dashboard quick actions shows phpMyAdmin button when a site has it enabled
```

---

## 11. Complete Updated Feature Scope Table

Update section 1 of per-site-config-spec-and-prompts.md:

| Domain | What's configurable |
|---|---|
| PHP | Version, INI overrides (.user.ini), Xdebug toggle, FPM pool settings |
| Framework | Auto-detected version display, manual override |
| WordPress | Multisite (subdomain/subdir), WP_DEBUG flags, wp-config.php extras, network sites |
| Laravel | Environment, Octane server, Horizon/Telescope/Pulse/Reverb |
| **phpMyAdmin** | **Per-site install, site-DB-only or all-DB mode, path-alias or subdomain access** |
| Database | Connection preset, table prefix, backup policy |
| HTTP server | Nginx, FrankenPHP, Caddy, Apache config per site |
| Development | Node version, package manager, dev server port |

---

*Author: Al Amin Ahamed (@mralaminahamed)*
