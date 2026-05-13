use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::OutputLine;

/// All the data collected from the user form for a WordPress project.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WordPressConfig {
    pub name: String,
    pub directory: PathBuf,
    pub php_version: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_host: String,
    pub db_prefix: String,
    pub site_title: String,
    pub admin_user: String,
    pub admin_pass: String,
    pub admin_email: String,
    pub locale: String,
    pub multisite: bool,
    pub variant: String, // "blank", "bedrock", "sage", "woocommerce", "multisite"
}

impl WordPressConfig {
    /// Build from form_values HashMap (keys match project_types.rs option keys).
    #[allow(dead_code)]
    pub fn from_form(values: &HashMap<String, String>, variant: &str) -> Self {
        WordPressConfig {
            name: values.get("name").cloned().unwrap_or_default(),
            directory: PathBuf::from(values.get("directory").cloned().unwrap_or_default()),
            php_version: values
                .get("php_version")
                .cloned()
                .unwrap_or_else(|| "8.3".to_string()),
            db_name: values.get("db_name").cloned().unwrap_or_default(),
            db_user: values
                .get("db_user")
                .cloned()
                .unwrap_or_else(|| "root".to_string()),
            db_pass: values.get("db_pass").cloned().unwrap_or_default(),
            db_host: values
                .get("db_host")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            db_prefix: values
                .get("db_prefix")
                .cloned()
                .unwrap_or_else(|| "wp_".to_string()),
            site_title: values.get("site_title").cloned().unwrap_or_default(),
            admin_user: values
                .get("admin_user")
                .cloned()
                .unwrap_or_else(|| "admin".to_string()),
            admin_pass: values.get("admin_pass").cloned().unwrap_or_default(),
            admin_email: values.get("admin_email").cloned().unwrap_or_default(),
            locale: values
                .get("locale")
                .cloned()
                .unwrap_or_else(|| "en_US".to_string()),
            multisite: values
                .get("multisite")
                .map(|v| v == "true")
                .unwrap_or(false),
            variant: variant.to_string(),
        }
    }
}

/// Run `wp valet new` with all configured options.
/// Streams output lines to `tx`. Returns Ok(true) on success, Ok(false) on failure.
#[allow(dead_code)]
pub async fn run(
    config: &WordPressConfig,
    tx: Sender<OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    // Build wp valet new command with all configured options
    let mut args = vec![
        "valet".to_string(),
        "new".to_string(),
        config.name.clone(),
        format!("--php={}", config.php_version),
        format!("--dbname={}", config.db_name),
        format!("--dbuser={}", config.db_user),
        format!("--dbprefix={}", config.db_prefix),
        format!("--dbhost={}", config.db_host),
        format!("--locale={}", config.locale),
    ];

    if !config.db_pass.is_empty() {
        args.push(format!("--dbpass={}", config.db_pass));
    }
    if !config.site_title.is_empty() {
        args.push(format!("--site_title={}", config.site_title));
    }
    if !config.admin_user.is_empty() && config.admin_user != "admin" {
        args.push(format!("--admin_user={}", config.admin_user));
    }
    if !config.admin_pass.is_empty() {
        args.push(format!("--admin_password={}", config.admin_pass));
    }
    if !config.admin_email.is_empty() {
        args.push(format!("--admin_email={}", config.admin_email));
    }
    if config.multisite {
        args.push("--multisite".to_string());
    }

    // First arg is "valet" (which maps to subcommand), remaining are actual args.
    // stream_command takes cmd="wp", args=["valet", "new", ...]
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let status = crate::creator::output_streamer::stream_command(
        "wp",
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
    fn wordpress_config_from_form_defaults() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "mysite".to_string());
        let config = WordPressConfig::from_form(&values, "blank");
        assert_eq!(config.name, "mysite");
        assert_eq!(config.db_user, "root");
        assert_eq!(config.db_prefix, "wp_");
        assert_eq!(config.locale, "en_US");
        assert!(!config.multisite);
    }

    #[test]
    fn wordpress_config_multisite_from_form() {
        let mut values = std::collections::HashMap::new();
        values.insert("multisite".to_string(), "true".to_string());
        let config = WordPressConfig::from_form(&values, "multisite");
        assert!(config.multisite);
    }

    #[test]
    fn wordpress_config_variant_stored() {
        let values = std::collections::HashMap::new();
        let config = WordPressConfig::from_form(&values, "bedrock");
        assert_eq!(config.variant, "bedrock");
    }

    #[test]
    fn wordpress_config_php_version_default() {
        let values = std::collections::HashMap::new();
        let config = WordPressConfig::from_form(&values, "blank");
        assert_eq!(config.php_version, "8.3");
    }

    #[test]
    fn wordpress_config_db_host_default() {
        let values = std::collections::HashMap::new();
        let config = WordPressConfig::from_form(&values, "blank");
        assert_eq!(config.db_host, "127.0.0.1");
    }

    #[test]
    fn wordpress_config_admin_user_default() {
        let values = std::collections::HashMap::new();
        let config = WordPressConfig::from_form(&values, "blank");
        assert_eq!(config.admin_user, "admin");
    }

    #[test]
    fn wordpress_config_multisite_false_by_default() {
        let values = std::collections::HashMap::new();
        let config = WordPressConfig::from_form(&values, "blank");
        assert!(!config.multisite);
    }

    #[test]
    fn wordpress_config_custom_values() {
        let mut values = std::collections::HashMap::new();
        values.insert("name".to_string(), "testsite".to_string());
        values.insert("db_name".to_string(), "testdb".to_string());
        values.insert("db_user".to_string(), "myuser".to_string());
        values.insert("db_pass".to_string(), "secret".to_string());
        values.insert("db_prefix".to_string(), "ts_".to_string());
        values.insert("locale".to_string(), "fr_FR".to_string());
        let config = WordPressConfig::from_form(&values, "blank");
        assert_eq!(config.name, "testsite");
        assert_eq!(config.db_name, "testdb");
        assert_eq!(config.db_user, "myuser");
        assert_eq!(config.db_pass, "secret");
        assert_eq!(config.db_prefix, "ts_");
        assert_eq!(config.locale, "fr_FR");
    }
}
