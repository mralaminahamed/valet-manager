#![allow(dead_code)]

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ValetConfig {
    #[serde(rename = "domain")]
    pub tld: String,
    pub loopback: String,
    pub paths: Vec<String>,
}

impl Default for ValetConfig {
    fn default() -> Self {
        Self {
            tld: "test".to_string(),
            loopback: "127.0.0.1".to_string(),
            paths: Vec::new(),
        }
    }
}

pub async fn load(valet_paths: &crate::valet::variant::ValetPaths) -> anyhow::Result<ValetConfig> {
    let path = &valet_paths.config_json;
    let Ok(content) = tokio::fs::read_to_string(path).await else {
        return Ok(ValetConfig::default());
    };
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tld_is_test() {
        assert_eq!(ValetConfig::default().tld, "test");
    }

    #[test]
    fn default_loopback() {
        assert_eq!(ValetConfig::default().loopback, "127.0.0.1");
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{"domain":"local","loopback":"127.0.0.1","paths":[]}"#;
        let config: ValetConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.tld, "local");
    }
}
