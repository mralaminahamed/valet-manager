# Per-Site Configuration Spec
## SiteConfig TOML · PHP overrides · WordPress · Laravel · HTTP servers

---

## Config file locations

```
Priority 1 — site root (portable, git-committable):
  {site_root}/.valet-manager.toml

Priority 2 — centralized (machine-local, not in git):
  ~/.config/valet-manager/sites/{site-name}.toml
```

Site-root file wins on every field conflict. Field-level deep-merge
(see `src/site_config/merger.rs`).

---

## Complete SiteConfig TOML

```toml
# .valet-manager.toml

[php]
version = "8.3"          # None = use global active version

[php.ini_overrides]
memory_limit          = "512M"
upload_max_filesize   = "64M"
post_max_size         = "64M"
max_execution_time    = 120
max_input_vars        = 3000
max_file_uploads      = 20
session_gc_maxlifetime = 1440
date_timezone         = "Asia/Dhaka"
# extra = { "custom.key" = "value" }

[php.ini_overrides.extra]
"realpath_cache_size" = "4096K"

xdebug_enabled = false
xdebug_mode    = "debug"    # off | debug | profile | coverage | trace

[framework]
# framework_override = "laravel"  # override auto-detection if needed

[server]
server_type          = "nginx"    # nginx | frankenphp | caddy | apache
custom_directives    = ""         # raw block injected inside server{}
client_max_body_size = "64m"
read_timeout         = 120

[server.extra_headers]
"X-Frame-Options"       = "SAMEORIGIN"
"X-Content-Type-Options" = "nosniff"

[server.basic_auth]
enabled = false
realm   = "Restricted"
# users = [{ username = "admin", password_hash = "$2b$..." }]

# [[server.redirects]]
# from    = "^/old-path(.*)$"
# to      = "/new-path$1"
# code    = 301
# enabled = true

[wordpress]
multisite_enabled    = false
multisite_type       = "subdomain"    # subdomain | subdirectory
wp_debug             = true
wp_debug_log         = true
wp_debug_display     = false
script_debug         = false
savequeries          = false
table_prefix         = "wp_"
wp_cache             = false
wp_memory_limit      = "256M"
wp_max_memory_limit  = "256M"
disallow_file_edit   = true
disallow_file_mods   = false
force_ssl_admin      = true
extra_config         = ""    # raw PHP appended before "/* That's all */"

[laravel]
octane_enabled  = false
octane_server   = "swoole"    # swoole | roadrunner | frankenphp
octane_port     = 8000
octane_workers  = 4
horizon_enabled   = false
telescope_enabled = false
pulse_enabled     = false
reverb_enabled    = false
reverb_port       = 8080

[phpmyadmin]
enabled     = true
access_mode = "path_alias"    # path_alias | subdomain | global_only
db_scope    = "site_only"     # site_only | all_databases
path_alias  = "_pma"
# db_name_override = "custom_db_name"

[database]
engine                = "mysql"    # auto-detected from .env
backup_before_destroy = true

[development]
node_version    = "20"         # nvm / fnm version string
package_manager = "npm"        # npm | pnpm | yarn | bun
# dev_server_port = 3000       # override default port
# start_command   = "npm run dev"
```

---

## PHP INI override mechanism

PHP reads `.user.ini` from the document root automatically.
Document root path by framework:

| Framework | .user.ini location |
|---|---|
| Laravel, Symfony, Slim | `{site_root}/public/.user.ini` |
| WordPress, Bedrock | `{site_root}/.user.ini` |
| Others | `{site_root}/public/.user.ini` |

No privilege helper required — developer owns the site directory.

File permissions: `640` (owner rw, group r, world none).

---

## WordPress multisite flow

1. Backup `wp-config.php` to `wp-config.php.bak.{timestamp}`
2. Write `WP_ALLOW_MULTISITE = true`
3. Run `wp core multisite-install [--subdomains]` (streaming)
4. Write remaining constants: `MULTISITE`, `SUBDOMAIN_INSTALL`, `DOMAIN_CURRENT_SITE`, etc.
5. Inject Nginx multisite rewrite rules (sentinel blocks)
6. Reload Nginx

Reverting: remove constants → remove Nginx rewrite block → reload.

---

## HTTP server config per site

Each server type uses a Handlebars template from `assets/templates/servers/`:

| Server | Template | Reload command |
|---|---|---|
| Nginx | `nginx-site.conf.hbs` | `systemctl reload nginx` |
| FrankenPHP standalone | `frankenphp-standalone.caddy.hbs` | `systemctl reload frankenphp` |
| FrankenPHP Octane | N/A (artisan octane:start) | restart process |
| Caddy | `caddy-site.caddy.hbs` | `systemctl reload caddy` |
| Apache | `apache-site.conf.hbs` | `systemctl reload apache2` |

Custom directives are injected between sentinel comments:
```nginx
# BEGIN valet-manager custom
{your directives here}
# END valet-manager custom
```

Re-applying replaces the block between sentinels — idempotent.

---

## Module structure

```
src/
├── site_config/
│   ├── models.rs        SiteConfig and all sub-structs
│   ├── reader.rs        load() + merge both file locations
│   ├── writer.rs        save() atomic write, save_to_site_root()
│   └── merger.rs        field-level deep merge (site-root wins)
├── php/
│   └── user_ini.rs      write/remove .user.ini per site
├── wordpress/
│   ├── config_editor.rs wp-config.php constant read/write
│   ├── multisite.rs     enable/disable + network site management
│   └── wp_cli.rs        wp-cli command wrappers
├── laravel/
│   ├── octane.rs        start/stop Octane as background process
│   └── packages.rs      detect Horizon/Telescope/Pulse/Reverb/Octane
└── http_servers/
    ├── detector.rs      detect Nginx/FrankenPHP/Caddy/Apache
    ├── nginx.rs         extend existing + directive injection
    ├── frankenphp.rs    standalone + Octane proxy modes
    ├── caddy.rs         Caddyfile per-site
    ├── apache.rs        VirtualHost + .htaccess
    ├── template.rs      Handlebars rendering
    └── basic_auth.rs    htpasswd generation (bcrypt)
```

---

*Author: Al Amin Ahamed (@mralaminahamed)*
