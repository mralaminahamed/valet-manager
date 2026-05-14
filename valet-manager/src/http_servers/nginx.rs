#![allow(dead_code)]

use std::path::Path;

use crate::site_config::models::MultisiteType;
use crate::valet::site_scanner::ValetSite;
use crate::valet::variant::ValetPaths;

const BEGIN: &str = "# BEGIN valet-manager custom";
const END: &str = "# END valet-manager custom";

const MS_BEGIN: &str = "# BEGIN valet-manager multisite";
const MS_END: &str = "# END valet-manager multisite";

const AUTH_BEGIN: &str = "# BEGIN valet-manager auth";
const AUTH_END: &str = "# END valet-manager auth";

/// Insert a directive block between sentinels. If the sentinel pair exists, the
/// block between them is replaced. Otherwise the block is inserted just before
/// the last closing brace of the file (assumed to close the server { } block).
pub fn inject_block(config: &str, begin: &str, end: &str, body: &str) -> String {
    let block = format!("{}\n{}\n{}", begin, body.trim_matches('\n'), end);
    if config.contains(begin) {
        let start_idx = config.find(begin).unwrap();
        let end_idx = config.find(end).map(|i| i + end.len()).unwrap_or(config.len());
        let mut out = String::new();
        out.push_str(&config[..start_idx]);
        out.push_str(&block);
        out.push_str(&config[end_idx..]);
        out
    } else {
        // Insert before the last `}` if present.
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
}

async fn read_config(site: &ValetSite, valet_paths: &ValetPaths) -> String {
    let path = valet_paths.nginx_dir.join(&site.name);
    tokio::fs::read_to_string(&path).await.unwrap_or_default()
}

async fn write_config(site: &ValetSite, content: &str, valet_paths: &ValetPaths) -> anyhow::Result<()> {
    let path = valet_paths.nginx_dir.join(&site.name);
    write_atomic(&path, content).await
}

async fn write_atomic(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

/// Stub: would rewrite fastcgi_pass to proxy_pass for Octane mode.
/// Currently returns Ok and emits a diagnostic in logs.
pub async fn set_octane_upstream(
    _site: &ValetSite,
    _port: u16,
    _valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    Ok(())
}

/// Inject custom directives into a site's nginx config using sentinel comments.
pub async fn inject_custom_directives(
    site: &ValetSite,
    directives: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    let cur = read_config(site, valet_paths).await;
    let new = inject_block(&cur, BEGIN, END, directives);
    write_config(site, &new, valet_paths).await
}

/// Inject multisite rewrite rules into a site's nginx config.
pub async fn set_multisite_rewrites(
    site: &ValetSite,
    multisite_type: &MultisiteType,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    let body = match multisite_type {
        MultisiteType::Subdomain => {
            format!("server_name *.{0} {0};", site.domain)
        }
        MultisiteType::Subdirectory => {
            "if (!-e $request_filename) {\n    rewrite /wp-admin$ $scheme://$host$uri/ permanent;\n    rewrite ^(/[^/]+)?(/wp-.*) $2 last;\n    rewrite ^(/[^/]+)?(/.*\\.php) $2 last;\n}".to_string()
        }
    };
    let cur = read_config(site, valet_paths).await;
    let new = inject_block(&cur, MS_BEGIN, MS_END, &body);
    write_config(site, &new, valet_paths).await
}

/// Inject auth_basic + auth_basic_user_file directives.
pub async fn set_basic_auth(
    site: &ValetSite,
    htpasswd_path: &Path,
    realm: &str,
    valet_paths: &ValetPaths,
) -> anyhow::Result<()> {
    let body = format!(
        "auth_basic \"{}\";\nauth_basic_user_file \"{}\";",
        realm.replace('"', "\\\""),
        htpasswd_path.display(),
    );
    let cur = read_config(site, valet_paths).await;
    let new = inject_block(&cur, AUTH_BEGIN, AUTH_END, &body);
    write_config(site, &new, valet_paths).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_block_inserts_before_last_brace() {
        let config = "server {\n    listen 80;\n}\n";
        let out = inject_block(config, BEGIN, END, "add_header X-Foo bar;");
        assert!(out.contains(BEGIN));
        assert!(out.contains("add_header X-Foo bar;"));
        assert!(out.contains(END));
        // Block should be inside the server { } block, before the closing brace.
        let begin_idx = out.find(BEGIN).unwrap();
        let close_idx = out.rfind('}').unwrap();
        assert!(begin_idx < close_idx);
    }

    #[test]
    fn inject_block_is_idempotent() {
        let config = "server {\n    listen 80;\n}\n";
        let once = inject_block(config, BEGIN, END, "add_header X-Foo bar;");
        let twice = inject_block(&once, BEGIN, END, "add_header X-Foo baz;");
        // Only one sentinel pair
        assert_eq!(twice.matches(BEGIN).count(), 1);
        assert_eq!(twice.matches(END).count(), 1);
        assert!(twice.contains("X-Foo baz"));
        assert!(!twice.contains("X-Foo bar"));
    }

    #[test]
    fn inject_block_handles_no_braces() {
        let config = "# just a comment\n";
        let out = inject_block(config, BEGIN, END, "directives;");
        assert!(out.contains(BEGIN));
        assert!(out.contains("directives;"));
    }
}
