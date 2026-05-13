#![allow(dead_code)]

use std::path::Path;

use crate::php::privilege::{run_privileged, HelperRequest};
use crate::php::types::{ExtensionType, PhpExtension};

pub async fn list(version: &str) -> Vec<PhpExtension> {
    let mods_dir = format!("/etc/php/{}/mods-available", version);
    let conf_dir = format!("/etc/php/{}/conf.d", version);

    let mods_path = Path::new(&mods_dir);
    if !mods_path.exists() {
        return Vec::new();
    }

    let conf_entries: Vec<String> = std::fs::read_dir(&conf_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    let mut extensions: Vec<PhpExtension> = std::fs::read_dir(mods_path)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let fname = e.file_name().into_string().ok()?;
                    let name = fname.strip_suffix(".ini")?.to_string();
                    let enabled = conf_entries.iter().any(|entry| entry.contains(&name));
                    Some(PhpExtension {
                        name,
                        enabled,
                        ext_type: ExtensionType::Other,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    extensions.sort_by(|a, b| {
        b.enabled
            .cmp(&a.enabled)
            .then_with(|| a.name.cmp(&b.name))
    });

    extensions
}

pub async fn enable(version: &str, name: &str) -> anyhow::Result<()> {
    run_privileged(&HelperRequest::EnableExtension {
        version: version.to_string(),
        extension: name.to_string(),
    })
    .await?;
    Ok(())
}

pub async fn disable(version: &str, name: &str) -> anyhow::Result<()> {
    run_privileged(&HelperRequest::DisableExtension {
        version: version.to_string(),
        extension: name.to_string(),
    })
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ext_name_from_ini_filename() {
        let fname = "curl.ini";
        let name = fname.strip_suffix(".ini").unwrap();
        assert_eq!(name, "curl");
    }

    #[test]
    fn ext_enabled_check_logic() {
        let conf_entries = vec![
            "20-curl.ini".to_string(),
            "10-opcache.ini".to_string(),
            "20-mbstring.ini".to_string(),
        ];
        let name = "curl";
        let enabled = conf_entries.iter().any(|entry| entry.contains(name));
        assert!(enabled);

        let name_disabled = "xdebug";
        let not_enabled = conf_entries.iter().any(|entry| entry.contains(name_disabled));
        assert!(!not_enabled);
    }

    #[tokio::test]
    async fn list_returns_empty_for_missing_dir() {
        let result = list("99.99").await;
        assert!(result.is_empty());
    }
}
