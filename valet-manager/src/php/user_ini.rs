#![allow(dead_code)]

use std::path::PathBuf;

use crate::site_config::models::PhpIniOverrides;
use crate::ui::DetectedFramework;
use crate::valet::site_scanner::ValetSite;

const KNOWN_TIMEZONES: &[&str] = &[
    "UTC",
    "GMT",
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "America/Toronto",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Moscow",
    "Asia/Dubai",
    "Asia/Karachi",
    "Asia/Kolkata",
    "Asia/Dhaka",
    "Asia/Bangkok",
    "Asia/Singapore",
    "Asia/Shanghai",
    "Asia/Tokyo",
    "Asia/Seoul",
    "Australia/Sydney",
    "Pacific/Auckland",
    "Africa/Cairo",
    "Africa/Johannesburg",
    "America/Sao_Paulo",
    "America/Argentina/Buenos_Aires",
    "America/Mexico_City",
    "Asia/Tehran",
];

/// Return the absolute path where .user.ini should be written for this site,
/// based on the framework's document root.
pub fn user_ini_path(site: &ValetSite) -> PathBuf {
    match site.framework {
        DetectedFramework::Laravel
        | DetectedFramework::Symfony
        | DetectedFramework::Slim => site.path.join("public/.user.ini"),
        DetectedFramework::WordPress | DetectedFramework::Bedrock => site.path.join(".user.ini"),
        _ => site.path.join("public/.user.ini"),
    }
}

/// Write .user.ini with the given overrides (or remove if empty).
pub async fn apply(site: &ValetSite, overrides: &PhpIniOverrides) -> anyhow::Result<()> {
    if overrides.is_empty() {
        return remove(site).await;
    }
    let path = user_ini_path(site);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = overrides.render();
    tokio::fs::write(&path, content).await?;
    tracing::info!("Wrote .user.ini to {}", path.display());
    Ok(())
}

/// Delete .user.ini if it exists.
pub async fn remove(site: &ValetSite) -> anyhow::Result<()> {
    let path = user_ini_path(site);
    if tokio::fs::metadata(&path).await.is_ok() {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}

/// Read the current .user.ini content (if any).
pub async fn read_current(site: &ValetSite) -> Option<String> {
    tokio::fs::read_to_string(user_ini_path(site)).await.ok()
}

/// Validate a single key=value. Returns `Some(error_message)` if invalid, `None` if valid.
pub fn validate_ini_value(key: &str, value: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return Some("value cannot be empty".into());
    }
    match key {
        "memory_limit" | "upload_max_filesize" | "post_max_size" => {
            if v == "-1" { return None; }
            // Match `\d+(K|M|G)$`
            let bytes = v.as_bytes();
            if bytes.len() < 2 { return Some("expected like 512M or -1".into()); }
            let suffix = bytes[bytes.len() - 1];
            if !matches!(suffix, b'K' | b'M' | b'G') {
                return Some("expected like 512M or -1".into());
            }
            let digits = &v[..v.len() - 1];
            if digits.chars().all(|c| c.is_ascii_digit()) && !digits.is_empty() {
                None
            } else {
                Some("expected like 512M or -1".into())
            }
        }
        "max_execution_time" | "max_input_vars" | "max_file_uploads" | "session_gc_maxlifetime" => {
            if v.parse::<u32>().is_ok() {
                None
            } else {
                Some("expected non-negative integer".into())
            }
        }
        "date_timezone" | "date.timezone" => {
            if KNOWN_TIMEZONES.iter().any(|tz| tz.eq_ignore_ascii_case(v)) {
                None
            } else {
                Some("expected an IANA timezone (e.g. UTC, Asia/Dhaka)".into())
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_site(framework: DetectedFramework, path: PathBuf) -> ValetSite {
        ValetSite {
            name: "demo".to_string(),
            domain: "demo.test".to_string(),
            path,
            site_type: crate::valet::site_scanner::SiteType::Parked,
            framework,
            php_version: None,
            is_secured: false,
            ssl_expiry: None,
            is_favorite: false,
        }
    }

    #[test]
    fn user_ini_path_for_laravel_goes_under_public() {
        let s = fixture_site(DetectedFramework::Laravel, PathBuf::from("/srv/app"));
        assert_eq!(user_ini_path(&s), PathBuf::from("/srv/app/public/.user.ini"));
    }

    #[test]
    fn user_ini_path_for_wordpress_is_site_root() {
        let s = fixture_site(DetectedFramework::WordPress, PathBuf::from("/srv/blog"));
        assert_eq!(user_ini_path(&s), PathBuf::from("/srv/blog/.user.ini"));
    }

    #[test]
    fn user_ini_path_for_bedrock_is_site_root() {
        let s = fixture_site(DetectedFramework::Bedrock, PathBuf::from("/srv/blog"));
        assert_eq!(user_ini_path(&s), PathBuf::from("/srv/blog/.user.ini"));
    }

    #[test]
    fn user_ini_path_for_unknown_is_public() {
        let s = fixture_site(DetectedFramework::Unknown, PathBuf::from("/srv/app"));
        assert_eq!(user_ini_path(&s), PathBuf::from("/srv/app/public/.user.ini"));
    }

    #[test]
    fn validate_memory_limit_accepts_suffixed() {
        assert!(validate_ini_value("memory_limit", "512M").is_none());
        assert!(validate_ini_value("memory_limit", "2G").is_none());
        assert!(validate_ini_value("memory_limit", "-1").is_none());
    }

    #[test]
    fn validate_memory_limit_rejects_bad_input() {
        assert!(validate_ini_value("memory_limit", "abc").is_some());
        assert!(validate_ini_value("memory_limit", "100").is_some());
        assert!(validate_ini_value("memory_limit", "").is_some());
    }

    #[test]
    fn validate_max_execution_time_accepts_integers() {
        assert!(validate_ini_value("max_execution_time", "120").is_none());
        assert!(validate_ini_value("max_execution_time", "0").is_none());
        assert!(validate_ini_value("max_execution_time", "abc").is_some());
    }

    #[test]
    fn validate_timezone_accepts_known_iana() {
        assert!(validate_ini_value("date_timezone", "UTC").is_none());
        assert!(validate_ini_value("date_timezone", "Asia/Dhaka").is_none());
        assert!(validate_ini_value("date.timezone", "Europe/London").is_none());
    }

    #[test]
    fn validate_timezone_rejects_unknown() {
        assert!(validate_ini_value("date_timezone", "Bogus/Place").is_some());
    }

    #[test]
    fn validate_unknown_key_is_permissive() {
        assert!(validate_ini_value("some_random_key", "anything").is_none());
    }

    #[tokio::test]
    async fn apply_writes_file_then_remove_deletes_it() {
        let tmp = std::env::temp_dir().join(format!("vm-user-ini-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(tmp.join("public")).await.unwrap();
        let s = fixture_site(DetectedFramework::Laravel, tmp.clone());
        let o = PhpIniOverrides {
            memory_limit: Some("512M".into()),
            ..Default::default()
        };
        apply(&s, &o).await.unwrap();
        let written = tokio::fs::read_to_string(tmp.join("public/.user.ini")).await.unwrap();
        assert!(written.contains("memory_limit = 512M"));
        remove(&s).await.unwrap();
        assert!(tokio::fs::metadata(tmp.join("public/.user.ini")).await.is_err());
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }

    #[tokio::test]
    async fn apply_with_empty_overrides_does_not_create_file() {
        let tmp = std::env::temp_dir().join(format!("vm-user-ini-empty-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(tmp.join("public")).await.unwrap();
        let s = fixture_site(DetectedFramework::Laravel, tmp.clone());
        apply(&s, &PhpIniOverrides::default()).await.unwrap();
        assert!(tokio::fs::metadata(tmp.join("public/.user.ini")).await.is_err());
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }
}
