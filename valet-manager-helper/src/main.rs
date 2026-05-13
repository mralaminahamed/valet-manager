use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum HelperRequest {
    SwitchPhp { version: String },
    StartFpm { version: String },
    StopFpm { version: String },
    RestartFpm { version: String },
    EnableExtension { version: String, extension: String },
    DisableExtension { version: String, extension: String },
    SaveIni { path: String, content: String },
}

#[derive(Debug, Serialize)]
struct HelperResponse {
    success: bool,
    message: String,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let response = match serde_json::from_str::<HelperRequest>(&line) {
            Ok(req) => handle_request(req),
            Err(e) => HelperResponse { success: false, message: format!("parse error: {}", e) },
        };
        let json = serde_json::to_string(&response).unwrap_or_default();
        let _ = writeln!(stdout, "{}", json);
        let _ = stdout.flush();
    }
}

fn handle_request(req: HelperRequest) -> HelperResponse {
    match req {
        HelperRequest::SwitchPhp { version } => {
            let binary = format!("/usr/bin/php{}", version);
            run_cmd("update-alternatives", &["--set", "php", &binary])
        }
        HelperRequest::StartFpm { version } => {
            run_cmd("systemctl", &["start", &format!("php{}-fpm", version)])
        }
        HelperRequest::StopFpm { version } => {
            run_cmd("systemctl", &["stop", &format!("php{}-fpm", version)])
        }
        HelperRequest::RestartFpm { version } => {
            run_cmd("systemctl", &["restart", &format!("php{}-fpm", version)])
        }
        HelperRequest::EnableExtension { version, extension } => {
            run_cmd("phpenmod", &["-v", &version, &extension])
        }
        HelperRequest::DisableExtension { version, extension } => {
            run_cmd("phpdismod", &["-v", &version, &extension])
        }
        HelperRequest::SaveIni { path, content } => match fs::write(&path, content) {
            Ok(_) => HelperResponse { success: true, message: "OK".to_string() },
            Err(e) => HelperResponse { success: false, message: e.to_string() },
        },
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> HelperResponse {
    match Command::new(cmd).args(args).output() {
        Ok(out) if out.status.success() => HelperResponse {
            success: true,
            message: "OK".to_string(),
        },
        Ok(out) => HelperResponse {
            success: false,
            message: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => HelperResponse { success: false, message: e.to_string() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_request_switch_php_parses() {
        let json = r#"{"action":"SwitchPhp","version":"8.3"}"#;
        let req: HelperRequest = serde_json::from_str(json).unwrap();
        if let HelperRequest::SwitchPhp { version } = req {
            assert_eq!(version, "8.3");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn helper_request_enable_ext_parses() {
        let json = r#"{"action":"EnableExtension","version":"8.3","extension":"xdebug"}"#;
        let req: HelperRequest = serde_json::from_str(json).unwrap();
        if let HelperRequest::EnableExtension { version, extension } = req {
            assert_eq!(version, "8.3");
            assert_eq!(extension, "xdebug");
        } else {
            panic!("wrong variant");
        }
    }
}
