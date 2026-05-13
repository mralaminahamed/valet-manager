use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::OutputLine;

/// All the data collected from the user form for a Laravel project.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LaravelConfig {
    pub name: String,
    pub directory: PathBuf,
    pub php_version: String,
    pub starter_kit: String, // "none", "breeze-blade", "breeze-react", "breeze-vue", "jetstream-livewire", "jetstream-inertia"
    pub with_pest: bool,
    pub with_git: bool,
    pub variant: String, // "blank", "breeze", "jetstream", "api"
}

impl LaravelConfig {
    /// Build from form_values HashMap (keys match project_types.rs option keys).
    #[allow(dead_code)]
    pub fn from_form(values: &HashMap<String, String>, variant: &str) -> Self {
        LaravelConfig {
            name: values.get("name").cloned().unwrap_or_default(),
            directory: PathBuf::from(values.get("directory").cloned().unwrap_or_default()),
            php_version: values
                .get("php_version")
                .cloned()
                .unwrap_or_else(|| "8.3".to_string()),
            starter_kit: values
                .get("starter_kit")
                .cloned()
                .unwrap_or_else(|| "none".to_string()),
            with_pest: values
                .get("with_pest")
                .map(|v| v == "true")
                .unwrap_or(false),
            with_git: values
                .get("with_git")
                .map(|v| v == "true")
                .unwrap_or(true),
            variant: variant.to_string(),
        }
    }
}

/// Run the laravel installer commands based on variant.
/// Streams output lines to `tx`. Returns Ok(true) on success, Ok(false) on failure.
///
/// Command strategy by variant:
/// - blank/api:  `laravel new {name} --no-interaction`
/// - breeze:     `laravel new {name} --breeze --no-interaction`
/// - jetstream:  `laravel new {name} --jet --no-interaction`
#[allow(dead_code)]
pub async fn run(
    config: &LaravelConfig,
    tx: Sender<OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    // Build the primary `laravel new` command arguments
    let mut args: Vec<String> = vec!["new".to_string(), config.name.clone()];

    match config.variant.as_str() {
        "breeze" => {
            args.push("--breeze".to_string());
        }
        "jetstream" => {
            args.push("--jet".to_string());
        }
        _ => {}
    }

    args.push("--no-interaction".to_string());

    if config.with_pest {
        args.push("--pest".to_string());
    }

    if !config.with_git {
        args.push("--no-git".to_string());
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let status = crate::creator::output_streamer::stream_command(
        "laravel",
        &args_refs,
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

    #[test]
    fn laravel_config_from_form_defaults() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "myapp".to_string());
        let config = LaravelConfig::from_form(&values, "blank");
        assert_eq!(config.name, "myapp");
        assert_eq!(config.starter_kit, "none");
        assert!(!config.with_pest);
        assert!(config.with_git);
    }

    #[test]
    fn laravel_config_variant_stored() {
        let values = std::collections::HashMap::new();
        let config = LaravelConfig::from_form(&values, "breeze");
        assert_eq!(config.variant, "breeze");
    }

    #[test]
    fn laravel_config_php_version_default() {
        let values = std::collections::HashMap::new();
        let config = LaravelConfig::from_form(&values, "blank");
        assert_eq!(config.php_version, "8.3");
    }

    #[test]
    fn laravel_config_with_pest_true() {
        let mut values = std::collections::HashMap::new();
        values.insert("with_pest".to_string(), "true".to_string());
        let config = LaravelConfig::from_form(&values, "blank");
        assert!(config.with_pest);
    }

    #[test]
    fn laravel_config_with_git_false() {
        let mut values = std::collections::HashMap::new();
        values.insert("with_git".to_string(), "false".to_string());
        let config = LaravelConfig::from_form(&values, "blank");
        assert!(!config.with_git);
    }

    #[test]
    fn laravel_config_starter_kit_custom() {
        let mut values = std::collections::HashMap::new();
        values.insert("starter_kit".to_string(), "breeze-react".to_string());
        let config = LaravelConfig::from_form(&values, "breeze");
        assert_eq!(config.starter_kit, "breeze-react");
    }

    #[test]
    fn laravel_config_directory_from_form() {
        let mut values = std::collections::HashMap::new();
        values.insert("directory".to_string(), "/var/www".to_string());
        let config = LaravelConfig::from_form(&values, "blank");
        assert_eq!(config.directory, PathBuf::from("/var/www"));
    }

    #[test]
    fn laravel_config_with_git_defaults_true_when_missing() {
        let values = std::collections::HashMap::new();
        let config = LaravelConfig::from_form(&values, "api");
        assert!(config.with_git);
    }
}
