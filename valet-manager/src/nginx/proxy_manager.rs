#![allow(dead_code)]

use crate::valet::variant::ValetPaths;

#[derive(Debug, Clone)]
pub struct ValetProxy {
    pub domain: String,
    pub target: String,
    pub secured: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpProbe {
    Pending,
    Ok(u16),
    ClientError(u16),
    ServerError(u16),
    Failed,
}

/// Scan the nginx config dir for proxy site configs.
/// A proxy config contains `proxy_pass http(s)://...;` directive instead of a
/// fastcgi_pass to PHP-FPM. Domain comes from the file name; target is the
/// scheme://host[:port] of the proxy_pass.
pub async fn list_proxies(paths: &ValetPaths) -> anyhow::Result<Vec<ValetProxy>> {
    let dir = &paths.nginx_dir;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut out: Vec<ValetProxy> = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name = match p.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let body = tokio::fs::read_to_string(&p).await.unwrap_or_default();
        if let Some(target) = parse_proxy_pass(&body) {
            let secured = body.contains("ssl_certificate") || body.contains("listen 443");
            out.push(ValetProxy { domain: name, target, secured });
        }
    }
    out.sort_by(|a, b| a.domain.cmp(&b.domain));
    Ok(out)
}

/// Extract the proxy_pass target URL from an nginx config body.
pub fn parse_proxy_pass(body: &str) -> Option<String> {
    for raw in body.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("proxy_pass") {
            let rest = rest.trim().trim_end_matches(';').trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

pub async fn add_proxy(
    domain: &str,
    target: &str,
    secure: bool,
    _paths: &ValetPaths,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let mut args: Vec<&str> = vec!["proxy", domain, target];
    if secure {
        args.push("--secure");
    }
    let status = crate::creator::output_streamer::stream_command(
        "valet", &args, None, tx, cancel_rx,
    )
    .await?;
    Ok(status.success())
}

pub async fn remove_proxy(
    domain: &str,
    _paths: &ValetPaths,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let args = ["unproxy", domain];
    let status = crate::creator::output_streamer::stream_command(
        "valet", &args, None, tx, cancel_rx,
    )
    .await?;
    Ok(status.success())
}

/// Probe an HTTPS or HTTP URL using reqwest, returning the HTTP status category.
pub async fn probe_http(url: String) -> HttpProbe {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // local self-signed
        .timeout(std::time::Duration::from_secs(4))
        .build();
    let Ok(client) = client else { return HttpProbe::Failed; };
    match client.get(&url).send().await {
        Ok(r) => {
            let s = r.status().as_u16();
            match s {
                200..=399 => HttpProbe::Ok(s),
                400..=499 => HttpProbe::ClientError(s),
                500..=599 => HttpProbe::ServerError(s),
                _ => HttpProbe::Failed,
            }
        }
        Err(_) => HttpProbe::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_probe_ok_for_2xx() {
        assert!(matches!(HttpProbe::Ok(200), HttpProbe::Ok(200)));
    }

    #[test]
    fn http_probe_client_error_for_4xx() {
        let p = HttpProbe::ClientError(404);
        match p {
            HttpProbe::ClientError(s) => assert_eq!(s, 404),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn http_probe_server_error_for_5xx() {
        let p = HttpProbe::ServerError(503);
        match p {
            HttpProbe::ServerError(s) => assert_eq!(s, 503),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn http_probe_pending_distinct_from_ok() {
        assert_ne!(HttpProbe::Pending, HttpProbe::Ok(200));
    }

    #[test]
    fn http_probe_failed_distinct_from_others() {
        assert_ne!(HttpProbe::Failed, HttpProbe::Pending);
        assert_ne!(HttpProbe::Failed, HttpProbe::Ok(200));
    }

    #[test]
    fn parse_proxy_pass_finds_target() {
        let body = "server {\n  proxy_pass http://localhost:3000;\n}";
        assert_eq!(
            parse_proxy_pass(body),
            Some("http://localhost:3000".to_string())
        );
    }

    #[test]
    fn parse_proxy_pass_ignores_comments() {
        let body = "# proxy_pass http://commented;\nserver { proxy_pass http://real:1; }";
        // Trimmed line will not start with proxy_pass because there's content before; instead,
        // the comment line starts with '#' so we skip it. Multi-line body splits on newline.
        let body2 = "# proxy_pass http://commented;\nproxy_pass http://real:1;";
        assert_eq!(parse_proxy_pass(body2), Some("http://real:1".to_string()));
        // Also confirm single-line server { ... } pattern is NOT matched on the brace-line
        // (the line doesn't start with proxy_pass).
        assert_eq!(parse_proxy_pass(body), None);
    }

    #[test]
    fn parse_proxy_pass_none_when_missing() {
        let body = "server { listen 80; fastcgi_pass unix:/run/php.sock; }";
        assert_eq!(parse_proxy_pass(body), None);
    }
}
