#![allow(dead_code)]

use std::path::Path;

use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::creator::output_streamer::{stream_command, OutputLine};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WpPlugin {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub update: String,
    #[serde(default)]
    pub version: String,
}

/// Run `wp core version` and return the trimmed string.
pub async fn core_version(site_path: &Path) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("wp")
        .args(["core", "version"])
        .current_dir(site_path)
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("wp core version failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run `wp plugin list --format=json` and parse.
pub async fn plugin_list(site_path: &Path) -> anyhow::Result<Vec<WpPlugin>> {
    let output = tokio::process::Command::new("wp")
        .args(["plugin", "list", "--format=json"])
        .current_dir(site_path)
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("wp plugin list failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_plugin_list(&stdout)
}

pub(crate) fn parse_plugin_list(json: &str) -> anyhow::Result<Vec<WpPlugin>> {
    let trimmed = json.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(trimmed)?)
}

pub async fn plugin_activate(
    site_path: &Path,
    plugin: &str,
    network: bool,
    tx: Sender<OutputLine>,
) -> anyhow::Result<()> {
    let mut args: Vec<&str> = vec!["plugin", "activate", plugin];
    if network { args.push("--network"); }
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &args, Some(&site_path.to_path_buf()), tx, cancel_rx).await;
    Ok(())
}

pub async fn plugin_deactivate(
    site_path: &Path,
    plugin: &str,
    network: bool,
    tx: Sender<OutputLine>,
) -> anyhow::Result<()> {
    let mut args: Vec<&str> = vec!["plugin", "deactivate", plugin];
    if network { args.push("--network"); }
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &args, Some(&site_path.to_path_buf()), tx, cancel_rx).await;
    Ok(())
}

pub async fn core_update(site_path: &Path, tx: Sender<OutputLine>) -> anyhow::Result<()> {
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &["core", "update"], Some(&site_path.to_path_buf()), tx, cancel_rx).await;
    Ok(())
}

pub async fn plugin_update_all(site_path: &Path, tx: Sender<OutputLine>) -> anyhow::Result<()> {
    let (_c, cancel_rx) = tokio::sync::watch::channel(false);
    let _ = stream_command("wp", &["plugin", "update", "--all"], Some(&site_path.to_path_buf()), tx, cancel_rx).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plugin_list_handles_typical_output() {
        let json = r#"[{"name":"akismet","status":"active","update":"none","version":"5.0"},{"name":"hello","status":"inactive","update":"available","version":"1.0"}]"#;
        let list = parse_plugin_list(json).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "akismet");
        assert_eq!(list[0].status, "active");
        assert_eq!(list[1].update, "available");
    }

    #[test]
    fn parse_plugin_list_empty_string() {
        let list = parse_plugin_list("").unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn parse_plugin_list_malformed_errors() {
        assert!(parse_plugin_list("bad json").is_err());
    }
}
