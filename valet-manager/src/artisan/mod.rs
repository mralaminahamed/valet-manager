use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ArtisanTool { Artisan, BinConsole, BinMagento, Drush }

impl ArtisanTool {
    #[allow(dead_code)]
    pub fn binary(&self) -> &'static str {
        match self {
            Self::Artisan => "php",
            Self::BinConsole => "php",
            Self::BinMagento => "php",
            Self::Drush => "drush",
        }
    }
    #[allow(dead_code)]
    pub fn script(&self) -> &'static str {
        match self {
            Self::Artisan     => "artisan",
            Self::BinConsole  => "bin/console",
            Self::BinMagento  => "bin/magento",
            Self::Drush       => "", // standalone binary
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArtisanCommand {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct ListJson {
    commands: Vec<JsonCmd>,
}
#[derive(Debug, Deserialize)]
struct JsonCmd { name: String, description: Option<String> }

/// Detect which tool to use given a site path.
#[allow(dead_code)]
pub fn detect_tool(path: &Path) -> Option<ArtisanTool> {
    if path.join("artisan").exists() { return Some(ArtisanTool::Artisan); }
    if path.join("bin/console").exists() { return Some(ArtisanTool::BinConsole); }
    if path.join("bin/magento").exists() { return Some(ArtisanTool::BinMagento); }
    None
}

/// Run `{tool} list --format=json` and return parsed commands.
#[allow(dead_code)]
pub async fn discover(path: &Path, tool: ArtisanTool) -> anyhow::Result<Vec<ArtisanCommand>> {
    let (cmd, args): (&str, Vec<&str>) = match tool {
        ArtisanTool::Artisan    => ("php", vec!["artisan", "list", "--format=json"]),
        ArtisanTool::BinConsole => ("php", vec!["bin/console", "list", "--format=json"]),
        ArtisanTool::BinMagento => ("php", vec!["bin/magento", "list", "--format=json"]),
        ArtisanTool::Drush      => ("drush", vec!["list", "--format=json"]),
    };
    let out = tokio::process::Command::new(cmd)
        .args(&args)
        .current_dir(path)
        .output().await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: ListJson = serde_json::from_str(&stdout).unwrap_or(ListJson { commands: vec![] });
    Ok(parsed.commands.into_iter().map(|c| ArtisanCommand { name: c.name, description: c.description.unwrap_or_default() }).collect())
}

#[allow(dead_code)]
pub fn fuzzy_filter(cmds: &[ArtisanCommand], q: &str) -> Vec<ArtisanCommand> {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;
    if q.is_empty() { return cmds.iter().take(8).cloned().collect(); }
    let m = SkimMatcherV2::default();
    let mut scored: Vec<(i64, &ArtisanCommand)> = cmds.iter()
        .filter_map(|c| m.fuzzy_match(&c.name, q).map(|s| (s, c)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(8).map(|(_, c)| c.clone()).collect()
}

#[allow(dead_code)]
pub fn quick_commands(tool: ArtisanTool) -> Vec<&'static str> {
    match tool {
        ArtisanTool::Artisan    => vec!["migrate", "migrate:fresh", "db:seed", "route:list", "cache:clear", "tinker"],
        ArtisanTool::BinConsole => vec!["cache:clear", "doctrine:migrations:migrate", "make:controller"],
        ArtisanTool::BinMagento => vec!["cache:clean", "cache:flush", "indexer:reindex", "setup:upgrade"],
        ArtisanTool::Drush      => vec!["cache-rebuild", "updatedb", "config-import"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_tool_artisan() {
        let dir = std::env::temp_dir().join(format!("vm-art-{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("artisan"), "#!/bin/sh").unwrap();
        assert_eq!(detect_tool(&dir), Some(ArtisanTool::Artisan));
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn detect_tool_none_when_empty() {
        let dir = std::env::temp_dir().join(format!("vm-art-none-{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(detect_tool(&dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn fuzzy_filter_matches() {
        let cmds = vec![
            ArtisanCommand { name: "migrate".into(), description: "".into() },
            ArtisanCommand { name: "make:controller".into(), description: "".into() },
        ];
        let r = fuzzy_filter(&cmds, "mig");
        assert!(r.iter().any(|c| c.name == "migrate"));
    }
    #[test]
    fn fuzzy_filter_empty_returns_first_8() {
        let cmds: Vec<ArtisanCommand> = (0..20).map(|i| ArtisanCommand {
            name: format!("cmd{}", i), description: String::new()
        }).collect();
        assert_eq!(fuzzy_filter(&cmds, "").len(), 8);
    }
    #[test]
    fn quick_commands_per_tool() {
        assert!(quick_commands(ArtisanTool::Artisan).contains(&"migrate"));
        assert!(quick_commands(ArtisanTool::BinMagento).contains(&"cache:flush"));
        assert!(quick_commands(ArtisanTool::Drush).contains(&"cache-rebuild"));
        assert!(quick_commands(ArtisanTool::BinConsole).contains(&"cache:clear"));
    }
    #[test]
    fn tool_binary_and_script() {
        assert_eq!(ArtisanTool::Artisan.binary(), "php");
        assert_eq!(ArtisanTool::Artisan.script(), "artisan");
        assert_eq!(ArtisanTool::Drush.binary(), "drush");
    }
}
