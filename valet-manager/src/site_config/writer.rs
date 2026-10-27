#![allow(dead_code)]

use std::path::Path;

use super::models::SiteConfig;
use super::reader::{centralized_dir, site_root_path};

async fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("tmp")
    ));
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

/// Save to the centralized config (under ~/.config/valet-manager/sites).
pub async fn save(site_name: &str, config: &SiteConfig) -> anyhow::Result<()> {
    save_in(&centralized_dir(), site_name, config).await
}

async fn save_in(dir: &Path, site_name: &str, config: &SiteConfig) -> anyhow::Result<()> {
    let path = dir.join(format!("{site_name}.toml"));
    let serialized = toml::to_string_pretty(config)?;
    atomic_write(&path, &serialized).await
}

/// Save to the site-root .valet-manager.toml (user explicitly opted in).
pub async fn save_to_site_root(site_path: &Path, config: &SiteConfig) -> anyhow::Result<()> {
    let path = site_root_path(site_path);
    let serialized = toml::to_string_pretty(config)?;
    atomic_write(&path, &serialized).await
}

/// Delete the centralized config file (if any). Does not touch site root.
pub async fn reset(site_name: &str) -> anyhow::Result<()> {
    reset_in(&centralized_dir(), site_name).await
}

async fn reset_in(dir: &Path, site_name: &str) -> anyhow::Result<()> {
    let path = dir.join(format!("{site_name}.toml"));
    if tokio::fs::metadata(&path).await.is_ok() {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn save_to_site_root_writes_atomically() {
        let tmp = std::env::temp_dir().join(format!("vm-save-root-{}", std::process::id()));
        let _ = tokio::fs::create_dir_all(&tmp).await;
        let mut cfg = SiteConfig::default();
        cfg.php.version = Some("8.2".into());
        save_to_site_root(&tmp, &cfg).await.unwrap();
        let path = tmp.join(".valet-manager.toml");
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(content.contains("8.2"));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }

    #[tokio::test]
    async fn reset_removes_file_when_present() {
        let dir = std::env::temp_dir().join(format!("vm-reset-test-{}", std::process::id()));
        let site_name = "site";
        let mut cfg = SiteConfig::default();
        cfg.php.version = Some("8.3".into());
        save_in(&dir, site_name, &cfg).await.unwrap();
        let path = dir.join("site.toml");
        assert!(tokio::fs::metadata(&path).await.is_ok());
        reset_in(&dir, site_name).await.unwrap();
        assert!(tokio::fs::metadata(&path).await.is_err());
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn reset_is_idempotent_when_no_file() {
        let dir = std::env::temp_dir().join(format!("vm-reset-missing-{}", std::process::id()));
        // Should not error out.
        reset_in(&dir, "missing").await.unwrap();
    }
}
