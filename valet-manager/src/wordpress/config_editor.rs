#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::site_config::models::WordPressConfig;

const STOP_EDITING_MARKER: &str = "/* That's all, stop editing!";

/// Parse `define('NAME', value);` lines from wp-config.php and return a map.
/// Values are stringified (booleans and integers stripped of any quotes).
pub async fn read_constants(wp_path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let content = tokio::fs::read_to_string(wp_path.join("wp-config.php"))
        .await
        .unwrap_or_default();
    Ok(parse_constants(&content))
}

pub(crate) fn parse_constants(content: &str) -> HashMap<String, String> {
    let re = Regex::new(r#"define\s*\(\s*['"]([^'"]+)['"]\s*,\s*(.+?)\s*\)\s*;"#).unwrap();
    let mut out = HashMap::new();
    for cap in re.captures_iter(content) {
        let name = cap[1].to_string();
        let raw = cap[2].trim().to_string();
        let value = if (raw.starts_with('\'') && raw.ends_with('\''))
            || (raw.starts_with('"') && raw.ends_with('"'))
        {
            // Strip quotes
            raw[1..raw.len() - 1].to_string()
        } else {
            raw
        };
        out.insert(name, value);
    }
    out
}

/// Render a `define('NAME', value);` line. Booleans/integers are unquoted; everything else is single-quoted.
pub fn format_constant(name: &str, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "true" || trimmed == "false" {
        format!("define('{name}', {trimmed});")
    } else if trimmed.parse::<i64>().is_ok() {
        format!("define('{name}', {trimmed});")
    } else {
        // Escape single quotes inside.
        let escaped = trimmed.replace('\'', "\\'");
        format!("define('{name}', '{escaped}');")
    }
}

/// Backup wp-config.php to a timestamped sibling. Returns the backup path.
pub async fn backup(wp_path: &Path) -> anyhow::Result<PathBuf> {
    let src = wp_path.join("wp-config.php");
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let dst = wp_path.join(format!("wp-config.php.bak.{timestamp}"));
    let content = tokio::fs::read(&src).await?;
    tokio::fs::write(&dst, content).await?;
    Ok(dst)
}

/// Generate the list of (constant_name, value_string) pairs to write to wp-config.php
/// based on a WordPressConfig draft.
pub(crate) fn collect_constants(config: &WordPressConfig) -> Vec<(&'static str, String)> {
    let mut out: Vec<(&'static str, String)> = Vec::new();

    if config.multisite_enabled {
        out.push(("WP_ALLOW_MULTISITE", "true".into()));
        out.push(("MULTISITE", "true".into()));
        out.push((
            "SUBDOMAIN_INSTALL",
            match config.multisite_type {
                crate::site_config::models::MultisiteType::Subdomain => "true".into(),
                crate::site_config::models::MultisiteType::Subdirectory => "false".into(),
            },
        ));
        if let Some(d) = &config.domain_current_site {
            out.push(("DOMAIN_CURRENT_SITE", d.clone()));
        }
        out.push(("PATH_CURRENT_SITE", "/".into()));
        out.push(("SITE_ID_CURRENT_SITE", "1".into()));
        out.push(("BLOG_ID_CURRENT_SITE", "1".into()));
    }

    out.push(("WP_DEBUG", config.wp_debug.to_string()));
    out.push(("WP_DEBUG_LOG", config.wp_debug_log.to_string()));
    out.push(("WP_DEBUG_DISPLAY", config.wp_debug_display.to_string()));
    out.push(("SCRIPT_DEBUG", config.script_debug.to_string()));
    if config.savequeries {
        out.push(("SAVEQUERIES", "true".into()));
    }

    if let Some(home) = &config.wp_home {
        out.push(("WP_HOME", home.clone()));
    }
    if let Some(siteurl) = &config.wp_siteurl {
        out.push(("WP_SITEURL", siteurl.clone()));
    }

    if let Some(cache) = config.wp_cache {
        out.push(("WP_CACHE", cache.to_string()));
    }
    if let Some(ml) = &config.wp_memory_limit {
        out.push(("WP_MEMORY_LIMIT", ml.clone()));
    }
    if let Some(ml) = &config.wp_max_memory_limit {
        out.push(("WP_MAX_MEMORY_LIMIT", ml.clone()));
    }

    if let Some(b) = config.disallow_file_edit {
        out.push(("DISALLOW_FILE_EDIT", b.to_string()));
    }
    if let Some(b) = config.disallow_file_mods {
        out.push(("DISALLOW_FILE_MODS", b.to_string()));
    }
    if let Some(b) = config.force_ssl_admin {
        out.push(("FORCE_SSL_ADMIN", b.to_string()));
    }

    out
}

/// Replace or insert `define()` lines in wp-config.php content for the given constants.
pub(crate) fn merge_into_content(content: &str, constants: &[(&str, String)]) -> String {
    let mut out = content.to_string();
    for (name, value) in constants {
        let line = format_constant(name, value);
        let pattern = format!(
            r#"define\s*\(\s*['"]({})['"]\s*,\s*(.+?)\s*\)\s*;"#,
            regex::escape(name)
        );
        let re = Regex::new(&pattern).unwrap();
        if re.is_match(&out) {
            out = re.replace(&out, line.as_str()).to_string();
        } else if let Some(idx) = out.find(STOP_EDITING_MARKER) {
            // Insert before the marker, on its own line.
            let insertion = format!("{line}\n");
            out.insert_str(idx, &insertion);
        } else if let Some(idx) = out.rfind("?>") {
            let insertion = format!("{line}\n");
            out.insert_str(idx, &insertion);
        } else {
            // Append at end, with newline.
            if !out.ends_with('\n') { out.push('\n'); }
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Backup wp-config.php and write the requested constants. Atomic write.
pub async fn write_constants(wp_path: &Path, config: &WordPressConfig) -> anyhow::Result<()> {
    let src = wp_path.join("wp-config.php");
    let content = tokio::fs::read_to_string(&src).await.unwrap_or_default();
    let _ = backup(wp_path).await; // best-effort
    let constants = collect_constants(config);
    let new_content = merge_into_content(&content, &constants);
    // Append extra_config before "/* That's all */" if present.
    let final_content = if !config.extra_config.trim().is_empty() {
        let block = format!("\n{}\n", config.extra_config.trim());
        if let Some(idx) = new_content.find(STOP_EDITING_MARKER) {
            let mut s = new_content.clone();
            s.insert_str(idx, &block);
            s
        } else {
            format!("{}\n{}\n", new_content.trim_end(), config.extra_config.trim())
        }
    } else {
        new_content
    };
    let tmp = wp_path.join("wp-config.php.tmp");
    tokio::fs::write(&tmp, final_content).await?;
    tokio::fs::rename(&tmp, &src).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_constants_extracts_simple_defines() {
        let input = r#"
<?php
define('DB_NAME', 'mydb');
define('DB_USER', "root");
define('WP_DEBUG', true);
define('WP_MEMORY_LIMIT', '256M');
?>"#;
        let map = parse_constants(input);
        assert_eq!(map.get("DB_NAME"), Some(&"mydb".to_string()));
        assert_eq!(map.get("DB_USER"), Some(&"root".to_string()));
        assert_eq!(map.get("WP_DEBUG"), Some(&"true".to_string()));
        assert_eq!(map.get("WP_MEMORY_LIMIT"), Some(&"256M".to_string()));
    }

    #[test]
    fn format_constant_bool_unquoted() {
        assert_eq!(format_constant("WP_DEBUG", "true"), "define('WP_DEBUG', true);");
        assert_eq!(format_constant("WP_DEBUG", "false"), "define('WP_DEBUG', false);");
    }

    #[test]
    fn format_constant_int_unquoted() {
        assert_eq!(format_constant("SITE_ID_CURRENT_SITE", "1"), "define('SITE_ID_CURRENT_SITE', 1);");
    }

    #[test]
    fn format_constant_string_quoted() {
        assert_eq!(format_constant("WP_HOME", "https://x.test"), "define('WP_HOME', 'https://x.test');");
    }

    #[test]
    fn merge_replaces_existing_define() {
        let content = "<?php\ndefine('WP_DEBUG', false);\n/* That's all, stop editing! */";
        let out = merge_into_content(content, &[("WP_DEBUG", "true".into())]);
        assert!(out.contains("define('WP_DEBUG', true);"));
        assert!(!out.contains("define('WP_DEBUG', false);"));
    }

    #[test]
    fn merge_inserts_new_define_before_stop_marker() {
        let content = "<?php\ndefine('DB_NAME', 'mydb');\n/* That's all, stop editing! Happy publishing. */";
        let out = merge_into_content(content, &[("WP_DEBUG", "true".into())]);
        let idx_debug = out.find("define('WP_DEBUG', true);").unwrap();
        let idx_marker = out.find("/* That's all").unwrap();
        assert!(idx_debug < idx_marker);
    }

    #[test]
    fn merge_appends_when_no_markers() {
        let content = "<?php\ndefine('DB_NAME', 'mydb');\n";
        let out = merge_into_content(content, &[("WP_DEBUG", "true".into())]);
        assert!(out.contains("define('WP_DEBUG', true);"));
    }

    #[test]
    fn collect_constants_includes_multisite_when_enabled() {
        let mut wp = WordPressConfig::default();
        wp.multisite_enabled = true;
        wp.domain_current_site = Some("myblog.test".into());
        let consts = collect_constants(&wp);
        let names: Vec<&'static str> = consts.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"WP_ALLOW_MULTISITE"));
        assert!(names.contains(&"MULTISITE"));
        assert!(names.contains(&"SUBDOMAIN_INSTALL"));
        assert!(names.contains(&"DOMAIN_CURRENT_SITE"));
    }

    #[tokio::test]
    async fn backup_creates_timestamped_copy() {
        let tmp = std::env::temp_dir().join(format!("vm-wp-bak-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(&tmp).await.unwrap();
        let src = tmp.join("wp-config.php");
        tokio::fs::write(&src, "<?php // hello").await.unwrap();
        let bak = backup(&tmp).await.unwrap();
        assert!(bak.file_name().unwrap().to_string_lossy().starts_with("wp-config.php.bak."));
        let content = tokio::fs::read_to_string(&bak).await.unwrap();
        assert_eq!(content, "<?php // hello");
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }

    #[tokio::test]
    async fn write_constants_creates_backup_and_updates_file() {
        let tmp = std::env::temp_dir().join(format!("vm-wp-write-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&tmp).await;
        tokio::fs::create_dir_all(&tmp).await.unwrap();
        let src = tmp.join("wp-config.php");
        tokio::fs::write(&src, "<?php\ndefine('DB_NAME', 'mydb');\n/* That's all, stop editing! */\n").await.unwrap();
        let mut wp = WordPressConfig::default();
        wp.wp_debug = true;
        write_constants(&tmp, &wp).await.unwrap();
        let content = tokio::fs::read_to_string(&src).await.unwrap();
        assert!(content.contains("define('WP_DEBUG', true);"));
        // A backup file should exist.
        let mut entries = tokio::fs::read_dir(&tmp).await.unwrap();
        let mut found_backup = false;
        while let Some(e) = entries.next_entry().await.unwrap() {
            if e.file_name().to_string_lossy().starts_with("wp-config.php.bak.") {
                found_backup = true;
            }
        }
        assert!(found_backup, "expected at least one wp-config.php.bak.* file");
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }
}
