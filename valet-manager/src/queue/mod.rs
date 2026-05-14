use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QueueWorker {
    pub id: String,                 // service-name without .service
    pub site: String,               // domain
    pub site_path: PathBuf,
    pub connection: String,         // "redis" | "database" | "sqs" etc.
    pub queue: String,              // "default"
    pub start_on_boot: bool,
    pub status: String,             // "active" | "inactive" | "failed"
    pub jobs_processed: u64,
    pub jobs_failed: u64,
}

#[allow(dead_code)]
pub fn unit_path(id: &str) -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".config/systemd/user").join(format!("valet-queue-{}.service", id))
}

/// Render a user systemd unit file.
#[allow(dead_code)]
pub fn render_unit(worker: &QueueWorker) -> String {
    format!(
        "[Unit]\nDescription=Valet queue worker {id}\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory={dir}\nExecStart=/usr/bin/env php artisan queue:work {conn} --queue={queue} --tries=3\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        id = worker.id,
        dir = worker.site_path.display(),
        conn = worker.connection,
        queue = worker.queue,
    )
}

/// Read all valet-queue-*.service unit files and parse minimal metadata.
#[allow(dead_code)]
pub async fn list_workers() -> Vec<QueueWorker> {
    let dir = dirs::home_dir().unwrap_or_default().join(".config/systemd/user");
    let mut out = Vec::new();
    let Ok(mut rd) = tokio::fs::read_dir(&dir).await else { return out; };
    while let Ok(Some(entry)) = rd.next_entry().await {
        let name = entry.file_name();
        let Some(n) = name.to_str().map(|s| s.to_string()) else { continue; };
        if !n.starts_with("valet-queue-") || !n.ends_with(".service") { continue; }
        let id = n.trim_start_matches("valet-queue-").trim_end_matches(".service").to_string();
        let content = tokio::fs::read_to_string(entry.path()).await.unwrap_or_default();
        // crude extraction
        let workdir = content.lines().find(|l| l.starts_with("WorkingDirectory="))
            .map(|l| l.trim_start_matches("WorkingDirectory=").trim().to_string())
            .unwrap_or_default();
        let exec = content.lines().find(|l| l.starts_with("ExecStart="))
            .map(|l| l.trim_start_matches("ExecStart=").trim().to_string())
            .unwrap_or_default();
        // Parse `... queue:work {conn} --queue={queue}`
        let mut tokens = exec.split_whitespace().peekable();
        let mut conn = "default".to_string();
        let mut queue = "default".to_string();
        while let Some(t) = tokens.next() {
            if t == "queue:work" {
                if let Some(c) = tokens.next() { conn = c.to_string(); }
            }
            if let Some(rest) = t.strip_prefix("--queue=") {
                queue = rest.to_string();
            }
        }
        out.push(QueueWorker {
            id: id.clone(),
            site: id.clone(),
            site_path: PathBuf::from(&workdir),
            connection: conn,
            queue,
            start_on_boot: false,
            status: "unknown".to_string(),
            jobs_processed: 0,
            jobs_failed: 0,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_path_uses_id() {
        let p = unit_path("myapp");
        assert!(p.to_string_lossy().ends_with("valet-queue-myapp.service"));
    }
    #[test]
    fn render_unit_contains_workdir_and_queue() {
        let w = QueueWorker {
            id: "myapp".into(), site: "myapp.test".into(), site_path: PathBuf::from("/tmp/myapp"),
            connection: "redis".into(), queue: "high".into(),
            start_on_boot: false, status: "".into(), jobs_processed: 0, jobs_failed: 0,
        };
        let s = render_unit(&w);
        assert!(s.contains("WorkingDirectory=/tmp/myapp"));
        assert!(s.contains("queue:work redis"));
        assert!(s.contains("--queue=high"));
    }
}
