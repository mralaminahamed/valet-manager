use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum MailTool { Mailpit, MailHog }

impl MailTool {
    #[allow(dead_code)]
    pub fn binary(&self) -> &'static str {
        match self { Self::Mailpit => "mailpit", Self::MailHog => "MailHog" }
    }
    pub fn label(&self) -> &'static str {
        match self { Self::Mailpit => "Mailpit", Self::MailHog => "MailHog" }
    }
    #[allow(dead_code)]
    pub fn default_smtp_port(&self) -> u16 {
        match self { Self::Mailpit => 1025, Self::MailHog => 1025 }
    }
    #[allow(dead_code)]
    pub fn default_http_port(&self) -> u16 {
        match self { Self::Mailpit => 8025, Self::MailHog => 8025 }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MailStatus {
    pub tool: MailTool,
    pub running: bool,
    pub smtp_port: u16,
    pub http_port: u16,
    pub unread: u64,
}

/// HTTP GET /api/v1/messages and parse `unread` count.
#[allow(dead_code)]
pub async fn fetch_unread(http_port: u16) -> anyhow::Result<u64> {
    let url = format!("http://127.0.0.1:{}/api/v1/messages?start=0&limit=1", http_port);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(3)).build()?;
    let json: serde_json::Value = client.get(&url).send().await?.json().await?;
    Ok(json.get("unread").and_then(|v| v.as_u64()).unwrap_or(0))
}

/// Render the .env lines to apply for this mail tool.
#[allow(dead_code)]
pub fn env_lines(tool: MailTool, smtp_port: u16) -> String {
    format!(
        "MAIL_MAILER=smtp\nMAIL_HOST=127.0.0.1\nMAIL_PORT={}\nMAIL_USERNAME=null\nMAIL_PASSWORD=null\nMAIL_ENCRYPTION=null\nMAIL_FROM_ADDRESS=\"hello@local\"\nMAIL_FROM_NAME=\"{}\"\n",
        smtp_port,
        tool.label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_ports() {
        assert_eq!(MailTool::Mailpit.default_smtp_port(), 1025);
        assert_eq!(MailTool::Mailpit.default_http_port(), 8025);
    }
    #[test]
    fn env_lines_contains_smtp_settings() {
        let s = env_lines(MailTool::Mailpit, 1025);
        assert!(s.contains("MAIL_MAILER=smtp"));
        assert!(s.contains("MAIL_PORT=1025"));
        assert!(s.contains("Mailpit"));
    }
    #[test]
    fn tool_labels() {
        assert_eq!(MailTool::Mailpit.label(), "Mailpit");
        assert_eq!(MailTool::MailHog.label(), "MailHog");
    }
}
