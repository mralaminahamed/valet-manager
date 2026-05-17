use crate::version_registry::models::ToolVersionInfo;

#[allow(dead_code)]
pub async fn fetch_lts(client: &reqwest::Client) -> anyhow::Result<ToolVersionInfo> {
    let j: serde_json::Value = client
        .get("https://nodejs.org/dist/index.json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let arr = j.as_array().ok_or_else(|| anyhow::anyhow!("not an array"))?;
    for entry in arr {
        let lts = entry.get("lts").cloned().unwrap_or(serde_json::Value::Bool(false));
        if matches!(lts, serde_json::Value::Bool(false) | serde_json::Value::Null) {
            continue;
        }
        let ver = entry.get("version").and_then(|v| v.as_str()).unwrap_or("");
        return Ok(ToolVersionInfo {
            name: "Node.js LTS".into(),
            latest_version: ver.trim_start_matches('v').to_string(),
            release_url: Some("https://nodejs.org/en/download/".into()),
        });
    }
    anyhow::bail!("no LTS found");
}
