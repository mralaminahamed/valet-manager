use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PhpVersion {
    pub version: String,
    pub full_version: String,
    pub binary_path: PathBuf,
    pub fpm_service: String,
    pub cli_ini_path: PathBuf,
    pub fpm_ini_path: PathBuf,
    pub conf_d_path: PathBuf,
    pub is_active: bool,
    pub fpm_running: bool,
}

impl PhpVersion {
    pub fn from_version_str(version: &str) -> Self {
        PhpVersion {
            binary_path:  PathBuf::from(format!("/usr/bin/php{}", version)),
            fpm_service:  format!("php{}-fpm", version),
            cli_ini_path: PathBuf::from(format!("/etc/php/{}/cli/php.ini", version)),
            fpm_ini_path: PathBuf::from(format!("/etc/php/{}/fpm/php.ini", version)),
            conf_d_path:  PathBuf::from(format!("/etc/php/{}/cli/conf.d", version)),
            version:      version.to_string(),
            full_version: String::new(),
            is_active:    false,
            fpm_running:  false,
        }
    }
}

pub async fn detect_installed_versions() -> anyhow::Result<Vec<PhpVersion>> {
    let mut versions: HashMap<String, PhpVersion> = HashMap::new();

    // Scan /usr/bin/ for php<major>.<minor> binaries
    if let Ok(mut entries) = tokio::fs::read_dir("/usr/bin").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(ver) = parse_php_binary_name(&name) {
                let mut php = PhpVersion::from_version_str(&ver);
                php.binary_path = entry.path();
                if let Some(full) = query_full_version(&php.binary_path).await {
                    php.full_version = full;
                }
                versions.insert(ver, php);
            }
        }
    }

    // Merge update-alternatives
    if let Ok(output) = tokio::process::Command::new("update-alternatives")
        .args(["--list", "php"])
        .output()
        .await
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let path = PathBuf::from(line.trim());
            if let Some(ver) = extract_version_from_path(&path) {
                versions.entry(ver.clone()).or_insert_with(|| {
                    let mut php = PhpVersion::from_version_str(&ver);
                    php.binary_path = path;
                    php
                });
            }
        }
    }

    let active = detect_active_version().await;
    let mut result: Vec<PhpVersion> = versions.into_values().collect();
    for php in &mut result {
        php.is_active = php.version == active;
    }
    result.sort_by(|a, b| b.version.cmp(&a.version));
    Ok(result)
}

pub async fn detect_active_version() -> String {
    let Ok(output) = tokio::process::Command::new("php")
        .arg("--version")
        .output()
        .await
    else {
        return "unknown".to_string();
    };
    parse_php_version_output(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_php_binary_name(name: &str) -> Option<String> {
    let re = regex::Regex::new(r"^php(\d+\.\d+)$").ok()?;
    re.captures(name).map(|c| c[1].to_string())
}

fn extract_version_from_path(path: &Path) -> Option<String> {
    parse_php_binary_name(&path.file_name()?.to_string_lossy())
}

pub fn parse_php_version_output(output: &str) -> String {
    output
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .map(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() >= 2 {
                format!("{}.{}", parts[0], parts[1])
            } else {
                v.to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

async fn query_full_version(binary: &Path) -> Option<String> {
    let output = tokio::process::Command::new(binary)
        .arg("--version")
        .output()
        .await
        .ok()?;
    Some(String::from_utf8_lossy(&output.stdout).lines().next()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_php83_binary() {
        assert_eq!(parse_php_binary_name("php8.3"), Some("8.3".to_string()));
    }

    #[test]
    fn parse_php74_binary() {
        assert_eq!(parse_php_binary_name("php7.4"), Some("7.4".to_string()));
    }

    #[test]
    fn rejects_plain_php() {
        assert_eq!(parse_php_binary_name("php"), None);
    }

    #[test]
    fn rejects_phpize() {
        assert_eq!(parse_php_binary_name("phpize"), None);
    }

    #[test]
    fn rejects_php_fpm() {
        assert_eq!(parse_php_binary_name("php-fpm8.2"), None);
    }

    #[test]
    fn version_output_extracts_major_minor() {
        assert_eq!(parse_php_version_output("PHP 8.3.12 (cli) (built: ...)\n"), "8.3");
    }

    #[test]
    fn version_output_handles_empty() {
        assert_eq!(parse_php_version_output(""), "unknown");
    }

    #[test]
    fn from_version_str_builds_debian_paths() {
        let php = PhpVersion::from_version_str("8.2");
        assert_eq!(php.cli_ini_path, PathBuf::from("/etc/php/8.2/cli/php.ini"));
        assert_eq!(php.fpm_ini_path, PathBuf::from("/etc/php/8.2/fpm/php.ini"));
        assert_eq!(php.fpm_service, "php8.2-fpm");
    }
}
