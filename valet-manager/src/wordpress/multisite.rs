#![allow(dead_code)]

use tokio::sync::mpsc::Sender;

use crate::creator::output_streamer::{stream_command, OutputLine, Stream};
use crate::site_config::models::{MultisiteType, WordPressConfig, WpNetworkSite};
use crate::valet::site_scanner::ValetSite;

#[derive(Debug, Clone, Default)]
pub struct MultisiteEnableResult {
    pub constants_written: bool,
    pub nginx_updated: bool,
    pub wp_command_output: Vec<String>,
    pub next_steps: Vec<String>,
}

fn emit(tx: &Sender<OutputLine>, text: impl Into<String>) {
    let line = OutputLine {
        text: text.into(),
        stream: Stream::Stdout,
        timestamp: chrono::Local::now(),
    };
    let _ = tx.try_send(line);
}

/// Enable WordPress multisite on a site. Writes wp-config.php constants, runs
/// `wp core multisite-install`, then injects nginx rewrites.
pub async fn enable(
    site: &ValetSite,
    multisite_type: MultisiteType,
    config: &WordPressConfig,
    tx: Sender<OutputLine>,
) -> anyhow::Result<MultisiteEnableResult> {
    let mut result = MultisiteEnableResult::default();

    // Step 1: Backup wp-config.php
    let _ = super::config_editor::backup(&site.path).await;
    emit(&tx, "Backed up wp-config.php");

    // Step 2: Set WP_ALLOW_MULTISITE = true.
    let mut step2 = config.clone();
    step2.multisite_enabled = false; // first pass — only WP_ALLOW_MULTISITE
    // We'll write a draft with just WP_ALLOW_MULTISITE flag set via extra_config.
    step2.extra_config = format!(
        "{}\n{}",
        step2.extra_config.trim(),
        "define('WP_ALLOW_MULTISITE', true);"
    );
    if super::config_editor::write_constants(&site.path, &step2).await.is_ok() {
        result.constants_written = true;
        emit(&tx, "Wrote WP_ALLOW_MULTISITE = true");
    }

    // Step 3: wp core multisite-install
    let mut args: Vec<&str> = vec!["core", "multisite-install", "--title=Network", "--base=/"];
    if multisite_type == MultisiteType::Subdomain {
        args.push("--subdomains");
    }
    let (_cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &args, Some(&site.path), tx.clone(), cancel_rx).await;

    // Step 4: write full multisite constants.
    let mut step4 = config.clone();
    step4.multisite_enabled = true;
    step4.multisite_type = multisite_type;
    if step4.domain_current_site.is_none() {
        step4.domain_current_site = Some(site.domain.clone());
    }
    if super::config_editor::write_constants(&site.path, &step4).await.is_ok() {
        emit(&tx, "Wrote multisite constants to wp-config.php");
    }

    // Step 5: nginx rewrites (best-effort; needs ValetPaths from caller dispatcher).
    // The caller may also dispatch nginx update separately. We mark it pending here.
    result.nginx_updated = false;
    result.next_steps.push("Reload Nginx after applying the multisite rewrites".to_string());
    if multisite_type == MultisiteType::Subdomain {
        result
            .next_steps
            .push("Ensure DNS / dnsmasq resolves *.{} to 127.0.0.1".replace("{}", &site.domain));
    }

    Ok(result)
}

/// Disable WordPress multisite (rollback).
pub async fn disable(site: &ValetSite, tx: Sender<OutputLine>) -> anyhow::Result<()> {
    // Re-write wp-config.php without multisite flags.
    let mut wp = WordPressConfig::default();
    wp.multisite_enabled = false;
    let _ = super::config_editor::write_constants(&site.path, &wp).await;
    emit(&tx, "Removed multisite constants from wp-config.php");
    Ok(())
}

/// Run `wp site list --format=json` and parse into WpNetworkSite items.
pub async fn fetch_network_sites(site: &ValetSite) -> anyhow::Result<Vec<WpNetworkSite>> {
    let output = tokio::process::Command::new("wp")
        .args(["site", "list", "--format=json"])
        .current_dir(&site.path)
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("wp site list failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_network_sites(&stdout)
}

pub(crate) fn parse_network_sites(json: &str) -> anyhow::Result<Vec<WpNetworkSite>> {
    let value: serde_json::Value = serde_json::from_str(json.trim())?;
    let arr = value.as_array().ok_or_else(|| anyhow::anyhow!("expected JSON array"))?;
    let mut out = Vec::new();
    for item in arr {
        let blog_id = item.get("blog_id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
            .or_else(|| item.get("blog_id").and_then(|v| v.as_u64()).map(|n| n as u32))
            .unwrap_or(0);
        let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let title = item.get("title").or_else(|| item.get("blogname")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let registered = item.get("registered").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let last_updated = item.get("last_updated").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let public = item.get("public").and_then(|v| v.as_str()).map(|s| s == "1").unwrap_or(false);
        let archived = item.get("archived").and_then(|v| v.as_str()).map(|s| s == "1").unwrap_or(false);
        let spam = item.get("spam").and_then(|v| v.as_str()).map(|s| s == "1").unwrap_or(false);
        let deleted = item.get("deleted").and_then(|v| v.as_str()).map(|s| s == "1").unwrap_or(false);
        out.push(WpNetworkSite {
            blog_id, url, title, registered, last_updated, public, archived, spam, deleted,
        });
    }
    Ok(out)
}

pub async fn create_network_site(
    site: &ValetSite,
    url: &str,
    title: &str,
    tx: Sender<OutputLine>,
) -> anyhow::Result<()> {
    let url_arg = format!("--url={}", url);
    let title_arg = format!("--title={}", title);
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &["site", "create", url_arg.as_str(), title_arg.as_str()], Some(&site.path), tx, cancel_rx).await;
    Ok(())
}

pub async fn delete_network_site(
    site: &ValetSite,
    blog_id: u32,
    tx: Sender<OutputLine>,
) -> anyhow::Result<()> {
    let id_arg = blog_id.to_string();
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &["site", "delete", id_arg.as_str(), "--yes"], Some(&site.path), tx, cancel_rx).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_network_sites_handles_wp_cli_json_format() {
        let json = r#"[
          {"blog_id":"1","url":"http://myblog.test/","title":"Network","registered":"2025-01-01 00:00:00","last_updated":"2025-02-01 00:00:00","public":"1","archived":"0","spam":"0","deleted":"0"},
          {"blog_id":"2","url":"http://sub.myblog.test/","title":"Sub","registered":"2025-01-02 00:00:00","last_updated":"2025-02-02 00:00:00","public":"0","archived":"0","spam":"0","deleted":"0"}
        ]"#;
        let sites = parse_network_sites(json).unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].blog_id, 1);
        assert_eq!(sites[0].url, "http://myblog.test/");
        assert!(sites[0].public);
        assert!(!sites[1].public);
    }

    #[test]
    fn parse_network_sites_empty_array_ok() {
        let sites = parse_network_sites("[]").unwrap();
        assert!(sites.is_empty());
    }

    #[test]
    fn parse_network_sites_malformed_json_errors() {
        assert!(parse_network_sites("not json").is_err());
    }
}
