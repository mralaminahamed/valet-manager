#![allow(dead_code)]

use std::path::{Path, PathBuf};

use super::merger::merge;
use super::models::SiteConfig;

pub fn centralized_path(site_name: &str) -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".config/valet-manager/sites").join(format!("{site_name}.toml"))
}

pub fn site_root_path(site_path: &Path) -> PathBuf {
    site_path.join(".valet-manager.toml")
}

async fn try_load(path: &Path) -> SiteConfig {
    match tokio::fs::read_to_string(path).await {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => SiteConfig::default(),
    }
}

/// Load both config files (centralized and site-root) and merge them.
/// Site-root values override centralized values.
pub async fn load(site_name: &str, site_path: &Path) -> SiteConfig {
    let centralized = try_load(&centralized_path(site_name)).await;
    let site_root = try_load(&site_root_path(site_path)).await;
    merge(centralized, site_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centralized_path_is_under_home() {
        let p = centralized_path("myapp");
        assert!(p.to_string_lossy().contains("valet-manager/sites/myapp.toml"));
    }

    #[test]
    fn site_root_path_appends_filename() {
        let p = site_root_path(Path::new("/srv/myapp"));
        assert_eq!(p, PathBuf::from("/srv/myapp/.valet-manager.toml"));
    }

    #[tokio::test]
    async fn load_returns_default_when_neither_file_exists() {
        let tmp = std::env::temp_dir().join(format!("vm-load-empty-{}", std::process::id()));
        let _ = tokio::fs::create_dir_all(&tmp).await;
        let cfg = load("does-not-exist-site", &tmp).await;
        assert_eq!(cfg, SiteConfig::default());
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }

    #[tokio::test]
    async fn load_reads_site_root_file() {
        let tmp = std::env::temp_dir().join(format!("vm-load-root-{}", std::process::id()));
        let _ = tokio::fs::create_dir_all(&tmp).await;
        let toml_text = r#"
[php]
version = "8.3"
"#;
        let path = tmp.join(".valet-manager.toml");
        tokio::fs::write(&path, toml_text).await.unwrap();
        let cfg = load("anything", &tmp).await;
        assert_eq!(cfg.php.version, Some("8.3".to_string()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }
}
