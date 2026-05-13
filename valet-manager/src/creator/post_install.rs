use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::{OutputLine, Stream};
use chrono::Local;

/// Configuration for post-install steps (valet link, secure, isolate, open browser).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PostInstallConfig {
    pub site_name: String,
    pub site_path: PathBuf,
    pub php_version: Option<String>, // Some = isolate to this version, None = use global
    pub secure: bool,
    pub open_browser: bool,
    pub tld: String, // e.g. "test"
}

/// Run valet link, optionally valet secure, optionally valet isolate, optionally open browser.
/// Sends progress OutputLine messages to tx.
#[allow(dead_code)]
pub async fn run(config: &PostInstallConfig, tx: Sender<OutputLine>) -> anyhow::Result<()> {
    // Step 1: valet link {site_name}
    send_info(&tx, &format!("Linking site: {}...", config.site_name)).await;
    run_step("valet", &["link", &config.site_name], &config.site_path, &tx).await?;

    // Step 2: valet secure (if requested)
    if config.secure {
        send_info(&tx, "Securing with SSL...").await;
        run_step("valet", &["secure", &config.site_name], &config.site_path, &tx).await?;
    }

    // Step 3: valet isolate (if php_version is Some)
    if let Some(ref version) = config.php_version {
        send_info(&tx, &format!("Isolating to PHP {}...", version)).await;
        run_step(
            "valet",
            &["isolate", &format!("php@{}", version)],
            &config.site_path,
            &tx,
        )
        .await?;
    }

    // Step 4: open browser
    if config.open_browser {
        let scheme = if config.secure { "https" } else { "http" };
        let url = format!("{scheme}://{}.{}", config.site_name, config.tld);
        send_info(&tx, &format!("Opening {}...", url)).await;
        let _ = tokio::process::Command::new("xdg-open").arg(&url).spawn();
    }

    Ok(())
}

async fn send_info(tx: &Sender<OutputLine>, text: &str) {
    let _ = tx
        .send(OutputLine {
            text: text.to_string(),
            stream: Stream::Stdout,
            timestamp: Local::now(),
        })
        .await;
}

async fn run_step(
    cmd: &str,
    args: &[&str],
    cwd: &PathBuf,
    tx: &Sender<OutputLine>,
) -> anyhow::Result<()> {
    let (_, cancel_rx) = tokio::sync::watch::channel(false);
    let status = crate::creator::output_streamer::stream_command(
        cmd,
        args,
        Some(cwd),
        tx.clone(),
        cancel_rx,
    )
    .await?;
    if !status.success() {
        anyhow::bail!("{} {:?} failed", cmd, args);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_install_config_fields() {
        let config = PostInstallConfig {
            site_name: "mysite".to_string(),
            site_path: std::path::PathBuf::from("/tmp/mysite"),
            php_version: Some("8.3".to_string()),
            secure: true,
            open_browser: false,
            tld: "test".to_string(),
        };
        assert!(config.secure);
        assert_eq!(config.tld, "test");
    }

    #[test]
    fn post_install_config_no_php_isolation() {
        let config = PostInstallConfig {
            site_name: "mysite".to_string(),
            site_path: std::path::PathBuf::from("/tmp/mysite"),
            php_version: None,
            secure: false,
            open_browser: false,
            tld: "test".to_string(),
        };
        assert!(config.php_version.is_none());
        assert!(!config.secure);
    }

    #[test]
    fn post_install_config_open_browser() {
        let config = PostInstallConfig {
            site_name: "mysite".to_string(),
            site_path: std::path::PathBuf::from("/tmp/mysite"),
            php_version: None,
            secure: false,
            open_browser: true,
            tld: "test".to_string(),
        };
        assert!(config.open_browser);
    }

    #[test]
    fn post_install_config_site_name() {
        let config = PostInstallConfig {
            site_name: "awesome-app".to_string(),
            site_path: std::path::PathBuf::from("/var/www/awesome-app"),
            php_version: Some("8.2".to_string()),
            secure: true,
            open_browser: true,
            tld: "local".to_string(),
        };
        assert_eq!(config.site_name, "awesome-app");
        assert_eq!(config.php_version, Some("8.2".to_string()));
        assert_eq!(config.tld, "local");
    }
}
