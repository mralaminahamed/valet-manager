# HTTP Server Config Spec
## Nginx · FrankenPHP · Caddy · Apache — per-site abstraction

---

## Supported servers

| Server | How detected | Config path | Reload |
|---|---|---|---|
| **Nginx** | `which nginx` | `{valet_nginx_dir}/{site}.{tld}` | `systemctl reload nginx` |
| **FrankenPHP** | `which frankenphp` | `/etc/frankenphp/conf.d/{site}.caddy` (standalone) | `systemctl reload frankenphp` |
| **Caddy** | `which caddy` | `/etc/caddy/sites/{site}.caddy` | `systemctl reload caddy` |
| **Apache** | `which apache2` or `httpd` | `/etc/apache2/sites-available/{site}.conf` | `systemctl reload apache2` |

Detection runs at app startup. Only installed servers shown in site config UI.

---

## FrankenPHP modes

### Mode A — Octane backend (Nginx frontend)
```
Browser → Nginx (port 80/443) → FrankenPHP (127.0.0.1:8000)
```
Nginx `proxy_pass http://127.0.0.1:8000;` replaces `fastcgi_pass`.
FrankenPHP started via `php artisan octane:start --server=frankenphp`.

### Mode B — Standalone (FrankenPHP own HTTPS)
```
Browser → FrankenPHP (port 80/443, embedded Caddy)
```
Nginx site config removed. Caddyfile written to `/etc/frankenphp/conf.d/{site}.caddy`.

---

## Nginx custom directive injection

Directives injected between sentinel comments inside `server {}`:

```nginx
server {
    listen 80;
    server_name mysite.test;
    root /home/user/Sites/mysite/public;

    # ... existing valet config ...

    # BEGIN valet-manager custom
    client_max_body_size 64m;
    proxy_read_timeout   120;
    add_header X-Frame-Options "SAMEORIGIN";
    # END valet-manager custom

    # ... rest of config ...
}
```

Re-applying replaces the sentinel block — never duplicates.

---

## Caddy site template

```caddy
mysite.test {
    root * /home/user/Sites/mysite/public
    php_fastcgi unix//run/php/php8.3-fpm.sock
    file_server

    tls /path/to/cert.pem /path/to/key.pem

    basicauth /* {
        admin $2b$12$...
    }

    header X-Frame-Options "SAMEORIGIN"

    redir /old /new 301
}
```

---

## Apache VirtualHost template

```apache
<VirtualHost *:80>
    ServerName mysite.test
    DocumentRoot /home/user/Sites/mysite/public

    <Directory /home/user/Sites/mysite/public>
        AllowOverride All
        Require all granted
    </Directory>

    <FilesMatch \.php$>
        SetHandler "proxy:unix:/run/php/php8.3-fpm.sock|fcgi://localhost/"
    </FilesMatch>

    ErrorLog  ${APACHE_LOG_DIR}/mysite-error.log
    CustomLog ${APACHE_LOG_DIR}/mysite-access.log combined
</VirtualHost>
```

Apache requires `a2ensite {site}.conf` after writing.

---

## Basic auth

htpasswd stored at: `~/.config/valet-manager/auth/{site-name}.htpasswd`

Passwords stored as bcrypt hashes — never plaintext.
The htpasswd file is user-owned and referenced from Nginx/Caddy/Apache config.

---

## Per-site HttpServerSiteConfig TOML

```toml
[server]
server_type          = "nginx"
custom_directives    = "client_max_body_size 64m;\nproxy_read_timeout 120;"
client_max_body_size = "64m"
read_timeout         = 120

[server.extra_headers]
"X-Frame-Options"         = "SAMEORIGIN"
"X-Content-Type-Options"  = "nosniff"
"Referrer-Policy"         = "strict-origin-when-cross-origin"

[server.basic_auth]
enabled = true
realm   = "Staging"
users   = [
    { username = "demo", password_hash = "$2b$12$hash..." }
]

[[server.redirects]]
from    = "^/blog/(.*)$"
to      = "/posts/$1"
code    = 301
enabled = true
```

---

## Template files (assets/templates/servers/)

```
nginx-site.conf.hbs               standard PHP-FPM site
nginx-site-octane.conf.hbs        Octane proxy_pass site
nginx-proxy.conf.hbs              valet proxy site
frankenphp-standalone.caddy.hbs   FrankenPHP standalone Caddyfile
frankenphp-octane.caddy.hbs       FrankenPHP Octane embedded Caddyfile
caddy-site.caddy.hbs              Caddy site with PHP-FPM
caddy-proxy.caddy.hbs             Caddy reverse-proxy site
apache-site.conf.hbs              Apache VirtualHost
```

All templates are Handlebars. Variables: `domain`, `tld`, `document_root`,
`php_version`, `php_fpm_socket`, `tls_cert`, `tls_key`, `octane_port`,
`basic_auth_enabled`, `auth_users`, `extra_headers`, `redirects`, `custom_directives`.

---

*Author: Al Amin Ahamed (@mralaminahamed)*
