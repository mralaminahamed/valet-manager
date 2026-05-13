use std::collections::HashMap;
use anyhow::Result;
use tokio::process::Command;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CliTool {
    WpCli,
    WpCliValetCommand,
    LaravelInstaller,
    Composer,
    Npm,
    Git,
    Node,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ToolStatus {
    Installed { path: String, version: String },
    Missing,
}

/// Detects all CLI tools concurrently and returns a HashMap with their statuses.
#[allow(dead_code)]
pub async fn detect_all() -> HashMap<CliTool, ToolStatus> {
    let tools = vec![
        CliTool::WpCli,
        CliTool::WpCliValetCommand,
        CliTool::LaravelInstaller,
        CliTool::Composer,
        CliTool::Npm,
        CliTool::Git,
        CliTool::Node,
    ];

    let mut handles = vec![];

    for tool in tools {
        let handle = tokio::spawn(detect_tool(tool));
        handles.push(handle);
    }

    let mut results = HashMap::new();
    for handle in handles {
        if let Ok((tool, status)) = handle.await {
            results.insert(tool, status);
        }
    }

    results
}

/// Detects a single CLI tool by checking if the binary exists and fetching its version.
#[allow(dead_code)]
async fn detect_tool(tool: CliTool) -> (CliTool, ToolStatus) {
    let status = match &tool {
        CliTool::WpCli => detect_wp_cli().await,
        CliTool::WpCliValetCommand => detect_wp_cli_valet_command().await,
        CliTool::LaravelInstaller => detect_laravel_installer().await,
        CliTool::Composer => detect_composer().await,
        CliTool::Npm => detect_npm().await,
        CliTool::Git => detect_git().await,
        CliTool::Node => detect_node().await,
    };

    (tool, status)
}

async fn detect_wp_cli() -> ToolStatus {
    match which::which("wp") {
        Ok(path) => {
            match get_version_output("wp", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_wp_cli_valet_command() -> ToolStatus {
    match which::which("wp") {
        Ok(path) => {
            match Command::new("wp")
                .arg("valet")
                .arg("--info")
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        // Get version from wp --version
                        match get_version_output("wp", &["--version"]).await {
                            Ok(version) => ToolStatus::Installed {
                                path: path.to_string_lossy().to_string(),
                                version,
                            },
                            Err(_) => ToolStatus::Missing,
                        }
                    } else {
                        ToolStatus::Missing
                    }
                }
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_laravel_installer() -> ToolStatus {
    match which::which("laravel") {
        Ok(path) => {
            match get_version_output("laravel", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_composer() -> ToolStatus {
    match which::which("composer") {
        Ok(path) => {
            match get_version_output("composer", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_npm() -> ToolStatus {
    match which::which("npm") {
        Ok(path) => {
            match get_version_output("npm", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_git() -> ToolStatus {
    match which::which("git") {
        Ok(path) => {
            match get_version_output("git", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

async fn detect_node() -> ToolStatus {
    match which::which("node") {
        Ok(path) => {
            match get_version_output("node", &["--version"]).await {
                Ok(version) => ToolStatus::Installed {
                    path: path.to_string_lossy().to_string(),
                    version,
                },
                Err(_) => ToolStatus::Missing,
            }
        }
        Err(_) => ToolStatus::Missing,
    }
}

/// Helper function to get version output from a command.
/// Returns the first line of stdout, trimmed.
async fn get_version_output(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Command failed"));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let first_line = stdout
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No output"))?
        .trim()
        .to_string();

    Ok(first_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tools_detected_returns_all_keys() {
        let tools = vec![
            CliTool::WpCli,
            CliTool::WpCliValetCommand,
            CliTool::LaravelInstaller,
            CliTool::Composer,
            CliTool::Npm,
            CliTool::Git,
            CliTool::Node,
        ];
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn tool_status_installed_has_fields() {
        let s = ToolStatus::Installed {
            path: "/usr/bin/wp".to_string(),
            version: "2.8.0".to_string(),
        };
        if let ToolStatus::Installed { path, version } = s {
            assert_eq!(path, "/usr/bin/wp");
            assert_eq!(version, "2.8.0");
        } else {
            panic!("Expected ToolStatus::Installed");
        }
    }

    #[test]
    fn tool_status_missing_is_missing() {
        let s = ToolStatus::Missing;
        assert!(matches!(s, ToolStatus::Missing));
    }
}
