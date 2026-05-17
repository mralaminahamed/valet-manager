use std::path::PathBuf;

use crate::version_registry::models::VersionRegistryCache;

#[allow(dead_code)]
pub fn cache_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("valet-manager")
        .join("version-registry.json")
}

/// Read cache from disk. Returns Err on missing file or parse failure.
#[allow(dead_code)]
pub async fn load() -> anyhow::Result<VersionRegistryCache> {
    let path = cache_path();
    let content = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&content)?)
}

/// Write cache atomically: write to <path>.tmp then rename.
#[allow(dead_code)]
pub async fn save(cache: &VersionRegistryCache) -> anyhow::Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let json = serde_json::to_string_pretty(cache)?;
    let tmp = path.with_extension("json.tmp");
    tokio::fs::write(&tmp, json).await?;
    tokio::fs::rename(&tmp, &path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_ends_with_version_registry_json() {
        let p = cache_path();
        let s = p.to_string_lossy();
        assert!(s.ends_with("version-registry.json"));
        assert!(s.contains("valet-manager"));
    }
}
