use crate::version_registry::models::{default_php_versions, PhpVersionInfo};

#[allow(dead_code)]
pub async fn fetch(client: &reqwest::Client) -> Vec<PhpVersionInfo> {
    let url = "https://endoflife.date/api/v1/products/php/";
    let response: serde_json::Value = match client.get(url).send().await {
        Ok(r) => match r.error_for_status() {
            Ok(r) => match r.json().await {
                Ok(j) => j,
                Err(_) => return default_php_versions(),
            },
            Err(_) => return default_php_versions(),
        },
        Err(_) => return default_php_versions(),
    };
    let entries = response
        .get("result")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| response.as_array().cloned())
        .unwrap_or_default();
    let mut out = Vec::new();
    for e in entries {
        let cycle = e
            .get("name")
            .or_else(|| e.get("cycle"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !cycle.starts_with("8.") {
            continue;
        }
        if let Ok(num) = cycle.parse::<f32>() {
            if num < 8.1 {
                continue;
            }
        }
        let latest = e
            .get("latest")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    v.as_object()
                        .and_then(|o| o.get("name"))
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string())
                }
            })
            .unwrap_or_else(|| cycle.to_string());
        let support = e
            .get("activeSupportTill")
            .or_else(|| e.get("support"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let eol = e
            .get("eolFrom")
            .or_else(|| e.get("eol"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let (Some(support), Some(eol)) = (support, eol) else {
            continue;
        };
        out.push(PhpVersionInfo {
            minor: cycle.to_string(),
            latest_patch: latest,
            active_support_until: support,
            security_support_until: eol,
            release_url: format!("https://www.php.net/releases/{}_0.php", cycle.replace('.', "_")),
        });
    }
    if out.is_empty() {
        default_php_versions()
    } else {
        out
    }
}
