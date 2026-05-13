# phpMyAdmin Per-Site Service Spec

---

## Access modes

| Mode | URL | DB scope | Auth |
|---|---|---|---|
| Path alias (default) | `https://mysite.test/_pma/` | Site DB only | Auto-login (config auth) |
| Subdomain | `https://pma.mysite.test/` | Site DB only | Auto-login |
| Global only | `https://phpmyadmin.test/` | All databases | Login prompt (cookie auth) |
| All databases (any mode) | same URL | All databases | Login prompt |

---

## Config isolation mechanism

Each site gets its own `config.inc.php` in:
```
~/.config/valet-manager/phpmyadmin/sites/{site-name}/config.inc.php
```

Nginx passes `PMA_CONFIG_DIR` via `fastcgi_param` pointing to this directory.
phpMyAdmin reads the site-specific config automatically.

**Site-only mode:** `$cfg['Servers'][$i]['only_db'] = 'mydb';` + config auth (auto-login).
**All-DB mode:** No `only_db` line + cookie auth (login prompt for security).

---

## DB credential resolution order

1. Site `.env` → `DB_DATABASE`, `DB_USERNAME`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`
2. Site `wp-config.php` → `define('DB_NAME', ...)`, `define('DB_USER', ...)`, etc.
3. `AppConfig.database.*` defaults (global MySQL credentials)
4. `db_name_override` in site's `[phpmyadmin]` config overrides step 1/2 result

---

## Nginx injection (path alias mode)

```nginx
# BEGIN valet-manager phpmyadmin
location /_pma {
    return 301 /_pma/;
}
location /_pma/ {
    alias /usr/share/phpmyadmin/;
    index index.php;

    location ~ ^/_pma/(.+\.php)$ {
        fastcgi_pass unix:/run/php/php8.3-fpm.sock;
        fastcgi_param SCRIPT_FILENAME /usr/share/phpmyadmin/$1;
        fastcgi_param PMA_CONFIG_DIR  /home/user/.config/valet-manager/phpmyadmin/sites/mysite/;
        fastcgi_param SCRIPT_NAME /_pma/$1;
        include fastcgi_params;
    }
    location ~* \.(jpg|gif|css|png|js|ico|html)$ {
        alias /usr/share/phpmyadmin/$1;
        expires 1d;
    }
}
# END valet-manager phpmyadmin
```

Re-applying replaces the block between sentinels — idempotent.

---

## Global site (phpmyadmin.test)

- Nginx config written to `{valet_nginx_dir}/phpmyadmin.{tld}`
- Symlink: `{valet_sites_dir}/phpmyadmin` → `/usr/share/phpmyadmin`
- Global config uses cookie auth (login prompt) — no `only_db`
- `valet secure phpmyadmin` for HTTPS

---

## Installation methods

| Method | Command | Notes |
|---|---|---|
| apt | `DEBIAN_FRONTEND=noninteractive apt install phpmyadmin` | Pre-seed debconf to skip web server config |
| Manual | Download tar.gz from GitHub releases | Extract to `~/.config/valet-manager/phpmyadmin/app/` |

---

## SiteConfig TOML

```toml
[phpmyadmin]
enabled     = true
access_mode = "path_alias"    # path_alias | subdomain | global_only
db_scope    = "site_only"     # site_only | all_databases
path_alias  = "_pma"
# db_name_override = "my_custom_db"
```

---

## Module structure

```
src/phpmyadmin/
├── mod.rs                orchestrator: configure_for_site(), remove_from_site()
├── installer.rs          detect(), install_via_apt(), install_manual()
├── config_generator.rs   render_site_config(), write_site_config(), blowfish_secret
├── nginx_integration.rs  inject_into_site(), remove_from_site(), create_subdomain_site()
└── global_site.rs        setup_global_site(), is_global_site_configured()
```

---

## Security notes

- `config.inc.php` permissions: `640` (owner rw, group r, world none)
- `blowfish_secret` generated once, stored in `AppConfig`, never logged
- DB passwords never appear in `OutputLine` messages (replaced with `●●●●`)
- Cookie auth forced when all-DB scope (protects multi-DB exposure)
- Global site uses cookie auth only — always requires login

---

*Author: Al Amin Ahamed (@mralaminahamed)*
