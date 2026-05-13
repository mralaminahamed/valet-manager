use std::path::PathBuf;
use dirs::home_dir;

#[derive(Debug, Clone, PartialEq)]
pub enum ValetVariant {
    ValetLinux,
    ValetOfficial,
    ValetLinuxPlus,
}

impl ValetVariant {
    pub fn display_name(&self) -> &str {
        match self {
            ValetVariant::ValetLinux => "Valet Linux",
            ValetVariant::ValetOfficial => "Valet",
            ValetVariant::ValetLinuxPlus => "Valet Linux+",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValetPaths {
    pub config_root: PathBuf,
    pub nginx_dir: PathBuf,
    pub sites_dir: PathBuf,
    pub drivers_dir: PathBuf,
    pub log_dir: PathBuf,
    pub config_json: PathBuf,
    pub ca_dir: PathBuf,
}

pub fn for_variant(v: &ValetVariant) -> ValetPaths {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    let valet_root = match v {
        ValetVariant::ValetLinux | ValetVariant::ValetLinuxPlus => home.join(".valet"),
        ValetVariant::ValetOfficial => home.join(".config").join("valet"),
    };
    ValetPaths {
        nginx_dir:   valet_root.join("Nginx"),
        sites_dir:   valet_root.join("Sites"),
        drivers_dir: valet_root.join("Drivers"),
        log_dir:     valet_root.join("Log"),
        config_json: valet_root.join("config.json"),
        ca_dir:      valet_root.join("CA"),
        config_root: valet_root,
    }
}

pub async fn detect_valet_variant() -> anyhow::Result<ValetVariant> {
    if let Ok(output) = tokio::process::Command::new("composer")
        .args(["global", "show", "--format=json"])
        .output()
        .await
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(installed) = json["installed"].as_array() {
                    for pkg in installed {
                        match pkg["name"].as_str() {
                            Some("genesisweb/valet-linux-plus") => {
                                return Ok(ValetVariant::ValetLinuxPlus);
                            }
                            Some("cpriego/valet-linux") => {
                                return Ok(ValetVariant::ValetLinux);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    let home = home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    if home.join(".valet").join("config.json").exists() {
        return Ok(ValetVariant::ValetLinux);
    }
    if home.join(".config").join("valet").join("config.json").exists() {
        return Ok(ValetVariant::ValetOfficial);
    }

    anyhow::bail!("No Valet installation detected")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_variant_linux_uses_dot_valet() {
        let paths = for_variant(&ValetVariant::ValetLinux);
        assert!(paths.config_root.to_string_lossy().contains(".valet"));
        assert_eq!(paths.config_json, paths.config_root.join("config.json"));
    }

    #[test]
    fn for_variant_official_uses_dot_config_valet() {
        let paths = for_variant(&ValetVariant::ValetOfficial);
        let s = paths.config_root.to_string_lossy().to_string();
        assert!(s.contains(".config") && s.contains("valet"), "expected .config/valet in {}", s);
    }

    #[test]
    fn for_variant_plus_uses_dot_valet() {
        let paths = for_variant(&ValetVariant::ValetLinuxPlus);
        assert!(paths.config_root.to_string_lossy().contains(".valet"));
    }

    #[test]
    fn display_names_are_distinct() {
        assert_ne!(ValetVariant::ValetLinux.display_name(), ValetVariant::ValetOfficial.display_name());
        assert_ne!(ValetVariant::ValetLinux.display_name(), ValetVariant::ValetLinuxPlus.display_name());
    }
}
