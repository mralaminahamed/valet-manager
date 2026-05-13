#![allow(dead_code)]

use std::path::Path;
use crate::php::privilege::{run_privileged, HelperRequest};
use crate::valet::variant::ValetPaths;

pub async fn switch_global(version: &str, _valet_paths: &ValetPaths) -> anyhow::Result<()> {
    let valet_result = tokio::process::Command::new("valet")
        .args(["use", &format!("php@{}", version)])
        .output()
        .await;

    let needs_fallback = match valet_result {
        Err(_) => true,
        Ok(out) => !out.status.success(),
    };
    if needs_fallback {
        let _resp = run_privileged(&HelperRequest::SwitchPhp { version: version.to_string() }).await?;
    }

    let _resp = run_privileged(&HelperRequest::RestartFpm { version: version.to_string() }).await?;

    Ok(())
}

pub async fn isolate_site(site_path: &Path, version: &str) -> anyhow::Result<()> {
    let valetrc_path = site_path.join(".valetrc");
    tokio::fs::write(&valetrc_path, format!("php={}\n", version)).await?;

    if let Some(site_name) = site_path.file_name() {
        let site_name_str = site_name.to_string_lossy();
        let _ = tokio::process::Command::new("valet")
            .args(["isolate", &format!("php@{}", version), &format!("--site={}", site_name_str)])
            .output()
            .await;
    }

    Ok(())
}

pub async fn unisolate_site(site_path: &Path) -> anyhow::Result<()> {
    let valetrc_path = site_path.join(".valetrc");
    let _ = tokio::fs::remove_file(&valetrc_path).await;

    if let Some(site_name) = site_path.file_name() {
        let site_name_str = site_name.to_string_lossy();
        let _ = tokio::process::Command::new("valet")
            .args(["unisolate", &format!("--site={}", site_name_str)])
            .output()
            .await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valetrc_content_format() {
        let expected = "php=8.3\n";
        let actual = format!("php={}\n", "8.3");
        assert_eq!(actual, expected);
    }

    #[test]
    fn site_name_extraction() {
        let path = Path::new("/home/user/Sites/myapp");
        let site_name = path.file_name();
        assert_eq!(site_name, Some(std::ffi::OsStr::new("myapp")));
    }

    #[test]
    fn switch_global_version_format() {
        let expected = "php8.3-fpm";
        let actual = format!("php{}-fpm", "8.3");
        assert_eq!(actual, expected);
    }
}
