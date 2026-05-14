#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareTool {
    Ngrok,
    Expose,
    Cloudflared,
}

impl ShareTool {
    pub fn label(&self) -> &'static str {
        match self {
            ShareTool::Ngrok => "ngrok",
            ShareTool::Expose => "expose",
            ShareTool::Cloudflared => "cloudflared",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShareSession {
    pub site: String,
    pub tool: ShareTool,
    pub public_url: Option<String>,
}

pub async fn start(
    site: &str,
    tool: ShareTool,
    token: &str,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let cmd: (&str, Vec<&str>) = match tool {
        ShareTool::Ngrok => ("valet", vec!["share", site]),
        ShareTool::Expose => ("expose", vec!["share", site]),
        ShareTool::Cloudflared => ("valet", vec!["share-cloudflared", site]),
    };
    let _ = token; // CLIs assumed pre-authenticated
    let args_refs: Vec<&str> = cmd.1.iter().copied().collect();
    let status = crate::creator::output_streamer::stream_command(
        cmd.0, &args_refs, None, tx, cancel_rx,
    )
    .await?;
    Ok(status.success())
}

/// Parse a tunnel-tool output line and extract a public URL.
/// Recognises patterns like:
///   `Forwarding https://abc123.ngrok-free.app -> http://localhost:8000`
///   `https://random.trycloudflare.com`
pub fn parse_public_url(line: &str) -> Option<String> {
    if let Some(idx) = line.find("https://") {
        let rest = &line[idx..];
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let url = &rest[..end];
        if url.contains('.') {
            return Some(url.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_public_url_finds_ngrok() {
        let line = "Forwarding https://abc123.ngrok-free.app -> http://localhost:8000";
        assert_eq!(
            parse_public_url(line),
            Some("https://abc123.ngrok-free.app".to_string())
        );
    }

    #[test]
    fn parse_public_url_finds_cloudflared() {
        let line = "INFO: https://random-name.trycloudflare.com";
        assert_eq!(
            parse_public_url(line),
            Some("https://random-name.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn parse_public_url_none_for_plain_text() {
        assert!(parse_public_url("just a log line").is_none());
    }

    #[test]
    fn parse_public_url_none_for_http_only() {
        // Function only accepts https:// scheme.
        assert!(parse_public_url("http://localhost:8000").is_none());
    }

    #[test]
    fn share_tool_label_is_distinct() {
        assert_ne!(ShareTool::Ngrok.label(), ShareTool::Expose.label());
        assert_ne!(ShareTool::Expose.label(), ShareTool::Cloudflared.label());
    }
}
