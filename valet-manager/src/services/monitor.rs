use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

impl ServiceStatus {
    pub fn label(&self) -> &str {
        match self {
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::Failed  => "Failed",
            ServiceStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ManagedService {
    pub name: String,
    pub display_name: String,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub unread_count: Option<u32>,
}

pub async fn query_service_status(name: &str) -> ServiceStatus {
    let Ok(status) = tokio::process::Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .await
    else {
        return ServiceStatus::Unknown;
    };

    if status.success() {
        return ServiceStatus::Running;
    }

    let failed = tokio::process::Command::new("systemctl")
        .args(["is-failed", "--quiet", name])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    if failed { ServiceStatus::Failed } else { ServiceStatus::Stopped }
}

pub async fn poll_services(service_names: Vec<String>, tx: mpsc::Sender<Vec<ManagedService>>) {
    loop {
        let mut results = Vec::new();
        for name in &service_names {
            let status = query_service_status(name).await;
            results.push(ManagedService {
                name: name.clone(),
                display_name: friendly_name(name),
                status,
                pid: None,
                unread_count: None,
            });
        }
        if tx.send(results).await.is_err() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

fn friendly_name(service: &str) -> String {
    match service {
        s if s.starts_with("php") && s.ends_with("-fpm") => {
            format!("PHP {}", s.trim_start_matches("php").trim_end_matches("-fpm"))
        }
        "nginx"   => "Nginx".to_string(),
        "mysql" | "mysqld" => "MySQL".to_string(),
        "mariadb" => "MariaDB".to_string(),
        "dnsmasq" => "Dnsmasq".to_string(),
        "mailpit" => "Mailpit".to_string(),
        other     => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_name_php_fpm() {
        assert_eq!(friendly_name("php8.3-fpm"), "PHP 8.3");
    }

    #[test]
    fn friendly_name_nginx() {
        assert_eq!(friendly_name("nginx"), "Nginx");
    }

    #[test]
    fn friendly_name_passthrough() {
        assert_eq!(friendly_name("some-service"), "some-service");
    }

    #[test]
    fn friendly_name_mailpit() {
        assert_eq!(friendly_name("mailpit"), "Mailpit");
    }

    #[test]
    fn service_status_labels() {
        assert_eq!(ServiceStatus::Running.label(), "Running");
        assert_eq!(ServiceStatus::Stopped.label(), "Stopped");
        assert_eq!(ServiceStatus::Failed.label(),  "Failed");
        assert_eq!(ServiceStatus::Unknown.label(), "Unknown");
    }
}
