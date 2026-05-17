use crate::version_registry::models::FrameworkVersionInfo;

#[allow(dead_code)]
pub async fn fetch(client: &reqwest::Client) -> anyhow::Result<FrameworkVersionInfo> {
    let j: serde_json::Value = client
        .get("https://api.wordpress.org/core/version-check/1.7/")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let ver = j
        .get("offers")
        .and_then(|o| o.as_array())
        .and_then(|a| a.first())
        .and_then(|f| f.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no version"))?;
    Ok(FrameworkVersionInfo {
        name: "WordPress".into(),
        latest_version: ver.to_string(),
        min_php: Some("7.2.24".into()),
        release_url: Some(format!("https://wordpress.org/download/releases/{}/", ver)),
    })
}
