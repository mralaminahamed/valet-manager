#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize)]
#[serde(tag = "action")]
pub enum HelperRequest {
    SwitchPhp { version: String },
    StartFpm { version: String },
    StopFpm { version: String },
    RestartFpm { version: String },
    EnableExtension { version: String, extension: String },
    DisableExtension { version: String, extension: String },
    SaveIni { path: String, content: String },
}

#[derive(Debug, Deserialize)]
pub struct HelperResponse {
    pub success: bool,
    pub message: String,
}

const HELPER_PATH: &str = "/usr/local/bin/valet-manager-helper";

pub async fn run_privileged(request: &HelperRequest) -> anyhow::Result<HelperResponse> {
    let json = serde_json::to_string(request)?;

    let mut child = tokio::process::Command::new("pkexec")
        .arg(HELPER_PATH)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
    }

    let output = child.wait_with_output().await?;
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout_str.lines().next().unwrap_or("{}");
    let resp: HelperResponse = serde_json::from_str(first_line)?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_to_json() {
        let req = HelperRequest::StartFpm { version: "8.3".to_string() };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("StartFpm"));
        assert!(json.contains("8.3"));
    }

    #[test]
    fn response_deserializes_from_json() {
        let json = r#"{"success":true,"message":"OK"}"#;
        let resp: HelperResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.message, "OK");
    }
}
