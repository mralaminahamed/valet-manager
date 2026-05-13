#![allow(dead_code)]

use crate::php::privilege::{run_privileged, HelperRequest};
use crate::services::monitor::ServiceStatus;

pub async fn start(version: &str) -> anyhow::Result<()> {
    run_privileged(&HelperRequest::StartFpm { version: version.to_string() }).await?;
    Ok(())
}

pub async fn stop(version: &str) -> anyhow::Result<()> {
    run_privileged(&HelperRequest::StopFpm { version: version.to_string() }).await?;
    Ok(())
}

pub async fn restart(version: &str) -> anyhow::Result<()> {
    run_privileged(&HelperRequest::RestartFpm { version: version.to_string() }).await?;
    Ok(())
}

pub async fn status(version: &str) -> ServiceStatus {
    let Ok(output) = tokio::process::Command::new("systemctl")
        .args(["is-active", &format!("php{version}-fpm")])
        .output()
        .await
    else {
        return ServiceStatus::Unknown;
    };

    map_status(String::from_utf8_lossy(&output.stdout).trim())
}

fn map_status(s: &str) -> ServiceStatus {
    match s {
        "active" => ServiceStatus::Running,
        "inactive" | "deactivating" => ServiceStatus::Stopped,
        "failed" => ServiceStatus::Failed,
        _ => ServiceStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_active_to_running() {
        assert_eq!(map_status("active"), ServiceStatus::Running);
    }

    #[test]
    fn status_maps_failed_to_failed() {
        assert_eq!(map_status("failed"), ServiceStatus::Failed);
    }

    #[test]
    fn status_maps_unknown_strings() {
        assert_eq!(map_status("activating"), ServiceStatus::Unknown);
        assert_eq!(map_status(""), ServiceStatus::Unknown);
        assert_eq!(map_status("some-garbage"), ServiceStatus::Unknown);
    }

    #[test]
    fn status_maps_inactive_to_stopped() {
        assert_eq!(map_status("inactive"), ServiceStatus::Stopped);
        assert_eq!(map_status("deactivating"), ServiceStatus::Stopped);
    }
}
