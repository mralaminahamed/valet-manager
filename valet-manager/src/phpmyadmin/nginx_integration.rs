#![allow(dead_code)]

//! Injects phpMyAdmin path-alias / subdomain blocks into Valet's nginx site
//! files using sentinel comments. Idempotent: re-injecting replaces an
//! existing block in-place.

use std::path::Path;

use crate::valet::site_scanner::ValetSite;
use crate::valet::variant::ValetPaths;

pub const BEGIN: &str = "# BEGIN valet-manager phpmyadmin";
pub const END: &str = "# END valet-manager phpmyadmin";

/// Render the nginx directives that serve phpMyAdmin under `path_alias`
/// (e.g. `/phpmyadmin`) of the host site, reading PHP via FPM and pulling
/// per-site overrides from `config_file`.
pub fn render_path_alias_block(
    path_alias: &str,
    pma_install_path: &Path,
    config_file: &Path,
    php_fpm_socket: &Path,
) -> String {
    let alias = normalize_alias(path_alias);
    let trimmed = alias.trim_end_matches('/');
    let mut s = String::new();
    s.push_str(&format!("location {} {{\n", trimmed));
    s.push_str(&format!("    alias {};\n", pma_install_path.display()));
    s.push_str(&format!("    index index.php;\n"));
    s.push_str(&format!("    try_files $uri $uri/ {}/index.php?$args;\n", trimmed));
    s.push_str("\n");
    s.push_str(&format!("    location ~ ^{}/(.+\\.php)$ {{\n", trimmed));
    s.push_str(&format!("        alias {}/$1;\n", pma_install_path.display()));
    s.push_str("        fastcgi_split_path_info ^(.+\\.php)(/.+)$;\n");
    s.push_str(&format!("        fastcgi_pass unix:{};\n", php_fpm_socket.display()));
    s.push_str("        fastcgi_index index.php;\n");
    s.push_str("        include fastcgi_params;\n");
    s.push_str("        fastcgi_param SCRIPT_FILENAME $request_filename;\n");
    s.push_str(&format!(
        "        fastcgi_param PHPMYADMIN_CONFIG_FILE {};\n",
        config_file.display(),
    ));
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

fn normalize_alias(path_alias: &str) -> String {
    let trimmed = path_alias.trim();
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}

/// Idempotent inject. Replaces an existing block bounded by BEGIN/END
/// sentinels, otherwise inserts the block just before the outer `}`.
pub fn inject_block(config: &str, body: &str) -> String {
    let block = format!("{}\n{}\n{}", BEGIN, body.trim_matches('\n'), END);
    if let Some(start_idx) = config.find(BEGIN) {
        let end_idx = config
            .find(END)
            .map(|i| i + END.len())
            .unwrap_or(config.len());
        let mut out = String::new();
        out.push_str(&config[..start_idx]);
        out.push_str(&block);
        out.push_str(&config[end_idx..]);
        return out;
    }
    if let Some(idx) = config.rfind('}') {
        let mut out = String::new();
        out.push_str(&config[..idx]);
        out.push_str("\n    ");
        out.push_str(&block.replace('\n', "\n    "));
        out.push('\n');
        out.push_str(&config[idx..]);
        out
    } else {
        format!("{}\n{}\n", config.trim_end(), block)
    }
}

/// Strip the phpmyadmin block from `config`. No-op if sentinels are absent.
pub fn remove_block(config: &str) -> String {
    let Some(start_idx) = config.find(BEGIN) else { return config.to_string() };
    let end_idx = config
        .find(END)
        .map(|i| i + END.len())
        .unwrap_or(config.len());
    let mut out = String::new();
    out.push_str(&config[..start_idx]);
    // Drop the trailing newline if present so we don't accumulate blank lines.
    let rest = &config[end_idx..];
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    out.push_str(rest);
    out
}

/// Locate a site's nginx config file. Valet writes them as `{site}` (no
/// extension) under `nginx_dir`; some setups use `{site}.conf` instead.
pub fn nginx_config_path(site: &ValetSite, valet_paths: &ValetPaths) -> std::path::PathBuf {
    let plain = valet_paths.nginx_dir.join(&site.name);
    if plain.exists() {
        return plain;
    }
    let with_ext = valet_paths.nginx_dir.join(format!("{}.conf", site.name));
    if with_ext.exists() {
        return with_ext;
    }
    plain
}

/// Atomically write `content` to `path` using a `.tmp` + rename.
pub async fn write_atomic(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub async fn inject_into_site(
    site: &ValetSite,
    body: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    let path = nginx_config_path(site, valet_paths);
    let cur = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let new = inject_block(&cur, body);
    write_atomic(&path, &new).await
}

pub async fn remove_from_site(
    site: &ValetSite,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    let path = nginx_config_path(site, valet_paths);
    let cur = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };
    let new = remove_block(&cur);
    if new != cur {
        write_atomic(&path, &new).await?;
    }
    Ok(())
}

/// Stub: in a real setup this would generate `~/.valet/Nginx/{site}.conf`
/// for the per-site subdomain (e.g. `pma.myapp.test`). For Phase 12 scope
/// we return the URL only.
pub fn subdomain_url(site_domain: &str) -> String {
    format!("https://pma.{}/", site_domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn render_path_alias_block_uses_alias() {
        let s = render_path_alias_block(
            "/phpmyadmin",
            &PathBuf::from("/usr/share/phpmyadmin"),
            &PathBuf::from("/home/user/.config/valet-manager/phpmyadmin/site.config.inc.php"),
            &PathBuf::from("/run/php/php8.3-fpm.sock"),
        );
        assert!(s.contains("location /phpmyadmin"));
        assert!(s.contains("alias /usr/share/phpmyadmin"));
        assert!(s.contains("fastcgi_pass unix:/run/php/php8.3-fpm.sock"));
        assert!(s.contains("PHPMYADMIN_CONFIG_FILE"));
    }

    #[test]
    fn render_block_normalizes_missing_leading_slash() {
        let s = render_path_alias_block(
            "phpmyadmin",
            &PathBuf::from("/usr/share/phpmyadmin"),
            &PathBuf::from("/cfg"),
            &PathBuf::from("/sock"),
        );
        assert!(s.contains("location /phpmyadmin"));
    }

    #[test]
    fn inject_block_inserts_before_last_brace() {
        let cfg = "server {\n    listen 80;\n}\n";
        let out = inject_block(cfg, "location /pma { return 200; }");
        assert!(out.contains(BEGIN));
        assert!(out.contains(END));
        assert!(out.contains("location /pma"));
        // Before closing brace
        assert!(out.find(BEGIN).unwrap() < out.rfind('}').unwrap());
    }

    #[test]
    fn inject_block_is_idempotent() {
        let cfg = "server {\n    listen 80;\n}\n";
        let once = inject_block(cfg, "first");
        let twice = inject_block(&once, "second");
        assert_eq!(twice.matches(BEGIN).count(), 1);
        assert_eq!(twice.matches(END).count(), 1);
        assert!(twice.contains("second"));
        assert!(!twice.contains("first"));
    }

    #[test]
    fn inject_block_handles_no_braces() {
        let cfg = "# just a comment\n";
        let out = inject_block(cfg, "x");
        assert!(out.contains(BEGIN));
        assert!(out.contains("x"));
    }

    #[test]
    fn remove_block_strips_sentinels_only() {
        let cfg = "server {\n    listen 80;\n}\n";
        let injected = inject_block(cfg, "body");
        let stripped = remove_block(&injected);
        assert!(!stripped.contains(BEGIN));
        assert!(!stripped.contains(END));
        assert!(!stripped.contains("body"));
        assert!(stripped.contains("server {"));
        assert!(stripped.contains("listen 80;"));
    }

    #[test]
    fn remove_block_is_noop_when_absent() {
        let cfg = "server { listen 80; }\n";
        assert_eq!(remove_block(cfg), cfg);
    }

    #[test]
    fn subdomain_url_includes_pma_prefix() {
        assert_eq!(subdomain_url("myapp.test"), "https://pma.myapp.test/");
    }
}
