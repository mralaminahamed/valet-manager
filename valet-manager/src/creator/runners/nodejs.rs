use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::OutputLine;

/// Configuration for a Node.js project creation run.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NodeJsConfig {
    pub name: String,
    pub directory: PathBuf,
    pub framework_id: String, // "nextjs" | "nuxt" | "react-vite" | "vue-vite" | "sveltekit" | "astro"
    pub port: u16,            // dev-server port
    pub with_systemd: bool,
    pub tld: String, // for proxy domain
}

impl NodeJsConfig {
    /// Build from form_values HashMap (keys match project_types.rs option keys).
    #[allow(dead_code)]
    pub fn from_form(values: &HashMap<String, String>, framework_id: &str, tld: &str) -> Self {
        let port = values
            .get("port")
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000);
        NodeJsConfig {
            name: values.get("name").cloned().unwrap_or_default(),
            directory: PathBuf::from(values.get("directory").cloned().unwrap_or_default()),
            framework_id: framework_id.to_string(),
            port,
            with_systemd: values
                .get("with_systemd")
                .map(|v| v == "true")
                .unwrap_or(false),
            tld: tld.to_string(),
        }
    }

    /// Returns the create-command and args for this framework.
    pub fn create_command(&self) -> (String, Vec<String>) {
        match self.framework_id.as_str() {
            "nextjs" => (
                "npx".into(),
                vec![
                    "--yes".into(),
                    "create-next-app".into(),
                    self.name.clone(),
                    "--ts".into(),
                    "--app".into(),
                    "--no-eslint".into(),
                    "--use-npm".into(),
                ],
            ),
            "nuxt" => (
                "npx".into(),
                vec![
                    "--yes".into(),
                    "nuxi@latest".into(),
                    "init".into(),
                    self.name.clone(),
                ],
            ),
            "react-vite" => (
                "npm".into(),
                vec![
                    "create".into(),
                    "vite@latest".into(),
                    self.name.clone(),
                    "--".into(),
                    "--template".into(),
                    "react-ts".into(),
                ],
            ),
            "vue-vite" => (
                "npm".into(),
                vec![
                    "create".into(),
                    "vite@latest".into(),
                    self.name.clone(),
                    "--".into(),
                    "--template".into(),
                    "vue-ts".into(),
                ],
            ),
            "sveltekit" => (
                "npm".into(),
                vec![
                    "create".into(),
                    "svelte@latest".into(),
                    self.name.clone(),
                ],
            ),
            "astro" => (
                "npm".into(),
                vec![
                    "create".into(),
                    "astro@latest".into(),
                    self.name.clone(),
                    "--".into(),
                    "--template".into(),
                    "minimal".into(),
                    "--typescript".into(),
                    "strict".into(),
                    "--no-install".into(),
                ],
            ),
            _ => ("npm".into(), vec!["init".into(), "-y".into()]),
        }
    }
}

/// Run the Node.js project creator, then register a Valet proxy and optionally a systemd unit.
/// Streams output lines to `tx`. Returns Ok(true) on success, Ok(false) on failure.
#[allow(dead_code)]
pub async fn run(
    config: &NodeJsConfig,
    tx: Sender<OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    // 1. Create the project via npx/npm command
    let (cmd, args) = config.create_command();
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let status = crate::creator::output_streamer::stream_command(
        &cmd,
        &arg_refs,
        Some(&config.directory),
        tx.clone(),
        cancel_rx.clone(),
    )
    .await?;
    if !status.success() {
        return Ok(false);
    }

    // 2. valet proxy {name} http://localhost:{port}
    let proxy_target = format!("http://localhost:{}", config.port);
    let proxy_args = ["proxy", config.name.as_str(), proxy_target.as_str()];
    let proxy_status = crate::creator::output_streamer::stream_command(
        "valet",
        &proxy_args,
        Some(&config.directory),
        tx.clone(),
        cancel_rx.clone(),
    )
    .await?;
    if !proxy_status.success() {
        return Ok(false);
    }

    // 3. Optional: systemd user service to keep dev server running
    if config.with_systemd {
        let _ = write_systemd_unit(config, &tx).await;
    }

    Ok(true)
}

#[allow(dead_code)]
async fn write_systemd_unit(
    config: &NodeJsConfig,
    tx: &Sender<OutputLine>,
) -> anyhow::Result<()> {
    let proj_dir = config.directory.join(&config.name);
    let unit_name = format!("valet-{}.service", config.name);
    let unit_path = std::env::var("HOME")
        .map(|h| {
            std::path::PathBuf::from(h)
                .join(".config/systemd/user")
                .join(&unit_name)
        })
        .unwrap_or_else(|_| std::path::PathBuf::from(&unit_name));

    let exec_start = match config.framework_id.as_str() {
        "nextjs" | "nuxt" => "npm run dev",
        "react-vite" | "vue-vite" | "sveltekit" | "astro" => "npm run dev",
        _ => "npm run dev",
    };

    let content = format!(
        "[Unit]\nDescription=Valet dev server for {name}\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory={dir}\nEnvironment=PORT={port}\nExecStart=/usr/bin/env {exec}\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        name = config.name,
        dir = proj_dir.display(),
        port = config.port,
        exec = exec_start,
    );

    if let Some(parent) = unit_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    tokio::fs::write(&unit_path, content).await?;

    let _ = tx
        .send(OutputLine {
            text: format!("Wrote systemd unit: {}", unit_path.display()),
            stream: crate::creator::output_streamer::Stream::Stdout,
            timestamp: chrono::Local::now(),
        })
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nodejs_config_from_form_defaults_port_3000() {
        let values = HashMap::new();
        let config = NodeJsConfig::from_form(&values, "nextjs", "test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.framework_id, "nextjs");
        assert_eq!(config.tld, "test");
        assert!(!config.with_systemd);
    }

    #[test]
    fn nodejs_config_from_form_parses_port() {
        let mut values = HashMap::new();
        values.insert("port".to_string(), "5173".to_string());
        let config = NodeJsConfig::from_form(&values, "react-vite", "test");
        assert_eq!(config.port, 5173);
    }

    #[test]
    fn nodejs_config_from_form_invalid_port_falls_back() {
        let mut values = HashMap::new();
        values.insert("port".to_string(), "not-a-number".to_string());
        let config = NodeJsConfig::from_form(&values, "nextjs", "test");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn nodejs_config_from_form_with_systemd() {
        let mut values = HashMap::new();
        values.insert("with_systemd".to_string(), "true".to_string());
        let config = NodeJsConfig::from_form(&values, "nextjs", "test");
        assert!(config.with_systemd);
    }

    #[test]
    fn nodejs_config_from_form_directory() {
        let mut values = HashMap::new();
        values.insert("directory".to_string(), "/srv/apps".to_string());
        let config = NodeJsConfig::from_form(&values, "nextjs", "test");
        assert_eq!(config.directory, PathBuf::from("/srv/apps"));
    }

    #[test]
    fn create_command_nextjs_uses_npx() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "nextjs".to_string(),
            port: 3000,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npx");
        assert!(args.contains(&"create-next-app".to_string()));
        assert!(args.contains(&"myapp".to_string()));
    }

    #[test]
    fn create_command_nuxt_uses_npx() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "nuxt".to_string(),
            port: 3000,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npx");
        assert!(args.contains(&"nuxi@latest".to_string()));
        assert!(args.contains(&"init".to_string()));
    }

    #[test]
    fn create_command_react_vite_uses_npm() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "react-vite".to_string(),
            port: 5173,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"vite@latest".to_string()));
        assert!(args.contains(&"react-ts".to_string()));
    }

    #[test]
    fn create_command_vue_vite_uses_npm() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "vue-vite".to_string(),
            port: 5173,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"vue-ts".to_string()));
    }

    #[test]
    fn create_command_sveltekit_uses_npm() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "sveltekit".to_string(),
            port: 5173,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"svelte@latest".to_string()));
    }

    #[test]
    fn create_command_astro_uses_npm() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "astro".to_string(),
            port: 4321,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npm");
        assert!(args.contains(&"astro@latest".to_string()));
        assert!(args.contains(&"minimal".to_string()));
    }

    #[test]
    fn create_command_unknown_falls_back_to_npm_init() {
        let config = NodeJsConfig {
            name: "myapp".to_string(),
            directory: PathBuf::from("/tmp"),
            framework_id: "mystery".to_string(),
            port: 3000,
            with_systemd: false,
            tld: "test".to_string(),
        };
        let (cmd, args) = config.create_command();
        assert_eq!(cmd, "npm");
        assert_eq!(args, vec!["init".to_string(), "-y".to_string()]);
    }
}
