use crate::version_registry::models::FrameworkVersionInfo;

#[allow(dead_code)]
pub async fn fetch(
    client: &reqwest::Client,
    package: &str,
    name: &str,
) -> anyhow::Result<FrameworkVersionInfo> {
    let url = format!("https://repo.packagist.org/p2/{package}.json");
    let j: serde_json::Value = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let versions = j
        .get("packages")
        .and_then(|p| p.get(package))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for v in versions {
        let ver = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
        let lower = ver.to_lowercase();
        if lower.contains("dev")
            || lower.contains("alpha")
            || lower.contains("beta")
            || lower.contains("rc")
        {
            continue;
        }
        let min_php = v
            .get("require")
            .and_then(|r| r.get("php"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());
        return Ok(FrameworkVersionInfo {
            name: name.to_string(),
            latest_version: ver.trim_start_matches('v').to_string(),
            min_php,
            release_url: Some(format!("https://packagist.org/packages/{package}")),
        });
    }
    anyhow::bail!("no stable version found");
}
