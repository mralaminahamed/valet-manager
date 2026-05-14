use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::OutputLine;

/// Generic Composer-based runner. Used by every PHP type whose install_command
/// starts with `composer create-project`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ComposerConfig {
    pub name: String,
    pub directory: PathBuf,
    pub php_version: String,
    pub package: String,                    // e.g. "symfony/website-skeleton"
    pub version_constraint: Option<String>, // e.g. Some("5.*") for CakePHP version select
    pub extra_args: Vec<String>,            // any additional --flags
}

impl ComposerConfig {
    /// Build from form_values HashMap (keys match project_types.rs option keys).
    #[allow(dead_code)]
    pub fn from_form(values: &HashMap<String, String>, package: &str) -> Self {
        ComposerConfig {
            name: values.get("name").cloned().unwrap_or_default(),
            directory: PathBuf::from(values.get("directory").cloned().unwrap_or_default()),
            php_version: values
                .get("php_version")
                .cloned()
                .unwrap_or_else(|| "8.3".to_string()),
            package: package.to_string(),
            version_constraint: values.get("version").cloned(),
            extra_args: Vec::new(),
        }
    }
}

/// Run `composer create-project <package>[:<version>] <name> --no-interaction`.
/// Streams output lines to `tx`. Returns Ok(true) on success, Ok(false) on failure.
#[allow(dead_code)]
pub async fn run(
    config: &ComposerConfig,
    tx: Sender<OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let mut args: Vec<String> = vec!["create-project".to_string()];
    let pkg_arg = match &config.version_constraint {
        Some(v) if !v.is_empty() => format!("{}:{}", config.package, v),
        _ => config.package.clone(),
    };
    args.push(pkg_arg);
    args.push(config.name.clone());
    args.push("--no-interaction".to_string());
    for a in &config.extra_args {
        args.push(a.clone());
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let status = crate::creator::output_streamer::stream_command(
        "composer",
        &arg_refs,
        Some(&config.directory),
        tx,
        cancel_rx,
    )
    .await?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_args(config: &ComposerConfig) -> Vec<String> {
        let mut args: Vec<String> = vec!["create-project".to_string()];
        let pkg_arg = match &config.version_constraint {
            Some(v) if !v.is_empty() => format!("{}:{}", config.package, v),
            _ => config.package.clone(),
        };
        args.push(pkg_arg);
        args.push(config.name.clone());
        args.push("--no-interaction".to_string());
        for a in &config.extra_args {
            args.push(a.clone());
        }
        args
    }

    #[test]
    fn composer_config_from_form_defaults() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "myapp".to_string());
        let config = ComposerConfig::from_form(&values, "symfony/website-skeleton");
        assert_eq!(config.name, "myapp");
        assert_eq!(config.package, "symfony/website-skeleton");
        assert_eq!(config.php_version, "8.3");
        assert!(config.version_constraint.is_none());
        assert!(config.extra_args.is_empty());
    }

    #[test]
    fn composer_config_from_form_with_version() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "cake-app".to_string());
        values.insert("version".to_string(), "5.*".to_string());
        let config = ComposerConfig::from_form(&values, "cakephp/app");
        assert_eq!(config.version_constraint, Some("5.*".to_string()));
    }

    #[test]
    fn composer_config_directory_from_form() {
        let mut values = HashMap::new();
        values.insert("directory".to_string(), "/var/www".to_string());
        let config = ComposerConfig::from_form(&values, "slim/slim-skeleton");
        assert_eq!(config.directory, PathBuf::from("/var/www"));
    }

    #[test]
    fn composer_config_php_version_custom() {
        let mut values = HashMap::new();
        values.insert("php_version".to_string(), "8.4".to_string());
        let config = ComposerConfig::from_form(&values, "drupal/recommended-project");
        assert_eq!(config.php_version, "8.4");
    }

    #[test]
    fn build_args_without_version() {
        let config = ComposerConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            php_version: "8.3".to_string(),
            package: "symfony/website-skeleton".to_string(),
            version_constraint: None,
            extra_args: Vec::new(),
        };
        let args = build_args(&config);
        assert_eq!(
            args,
            vec![
                "create-project".to_string(),
                "symfony/website-skeleton".to_string(),
                "myapp".to_string(),
                "--no-interaction".to_string(),
            ]
        );
    }

    #[test]
    fn build_args_with_version() {
        let config = ComposerConfig {
            name: "cake-app".to_string(),
            directory: PathBuf::from("/tmp"),
            php_version: "8.3".to_string(),
            package: "cakephp/app".to_string(),
            version_constraint: Some("5.*".to_string()),
            extra_args: Vec::new(),
        };
        let args = build_args(&config);
        assert_eq!(args[1], "cakephp/app:5.*");
        assert_eq!(args[2], "cake-app");
    }

    #[test]
    fn build_args_with_empty_version_treated_as_none() {
        let config = ComposerConfig {
            name: "app".to_string(),
            directory: PathBuf::from("/tmp"),
            php_version: "8.3".to_string(),
            package: "slim/slim-skeleton".to_string(),
            version_constraint: Some("".to_string()),
            extra_args: Vec::new(),
        };
        let args = build_args(&config);
        assert_eq!(args[1], "slim/slim-skeleton");
    }

    #[test]
    fn build_args_with_extra_args_appended() {
        let config = ComposerConfig {
            name: "app".to_string(),
            directory: PathBuf::from("/tmp"),
            php_version: "8.3".to_string(),
            package: "drupal/recommended-project".to_string(),
            version_constraint: None,
            extra_args: vec!["--prefer-dist".to_string(), "--no-dev".to_string()],
        };
        let args = build_args(&config);
        assert!(args.contains(&"--prefer-dist".to_string()));
        assert!(args.contains(&"--no-dev".to_string()));
        // extras come after --no-interaction
        let no_int_idx = args.iter().position(|a| a == "--no-interaction").expect("flag present");
        let prefer_idx = args.iter().position(|a| a == "--prefer-dist").expect("flag present");
        assert!(prefer_idx > no_int_idx);
    }
}
