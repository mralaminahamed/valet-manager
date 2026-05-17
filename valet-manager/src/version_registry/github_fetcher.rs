use crate::version_registry::models::FrameworkVersionInfo;

#[allow(dead_code)]
pub async fn fetch(
    client: &reqwest::Client,
    repo: &str,
    name: &str,
    min_php: Option<&str>,
) -> anyhow::Result<FrameworkVersionInfo> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .error_for_status()?;
    let j: serde_json::Value = resp.json().await?;
    let tag = j
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing tag_name"))?;
    let latest = tag.trim_start_matches('v').to_string();
    let html_url = j
        .get("html_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(FrameworkVersionInfo {
        name: name.to_string(),
        latest_version: latest,
        min_php: min_php.map(|s| s.to_string()),
        release_url: html_url,
    })
}
