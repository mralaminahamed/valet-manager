use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct ComposerJson {
    require: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CompatStatus {
    Compatible,
    Incompatible,
    NoRequirement,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SiteCompat {
    pub site_name: String,
    pub php_required: Option<String>,
    pub php_in_use: String,
    pub status: CompatStatus,
}

/// Read composer.json `require."php"` if any, return raw constraint string.
#[allow(dead_code)]
pub fn read_required_php(path: &Path) -> Option<String> {
    let composer_path = path.join("composer.json");
    let content = std::fs::read_to_string(composer_path).ok()?;
    let parsed: ComposerJson = serde_json::from_str(&content).ok()?;
    parsed.require?.get("php").cloned()
}

/// Check `version` satisfies the composer constraint.
/// Composer constraints may include "^7.4|^8.0", ">=8.1", etc.
/// We attempt semver matching; if parsing fails, return NoRequirement.
#[allow(dead_code)]
pub fn matches(version: &str, constraint: &str) -> CompatStatus {
    use semver::Version;
    let Ok(v) = Version::parse(&extend_to_three(version)) else {
        return CompatStatus::NoRequirement;
    };
    // Split alternatives on "|" or "||"
    let parts: Vec<&str> = constraint
        .split(|c: char| c == '|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let any_match = parts.iter().any(|p| satisfies_one(&v, p));
    if any_match {
        CompatStatus::Compatible
    } else {
        CompatStatus::Incompatible
    }
}

fn satisfies_one(v: &semver::Version, p: &str) -> bool {
    use semver::VersionReq;
    if let Ok(req) = VersionReq::parse(p) {
        return req.matches(v);
    }
    // composer often uses "8.1" or "8.*" — coerce to ">=8.1, <9" or "^8.1"
    let normalized = p.replace(".*", ".0");
    if let Ok(req) = VersionReq::parse(&format!("^{}", normalized)) {
        return req.matches(v);
    }
    false
}

fn extend_to_three(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0", parts[0]),
        2 => format!("{}.{}.0", parts[0], parts[1]),
        _ => s.to_string(),
    }
}

#[allow(dead_code)]
pub fn evaluate_site(name: &str, path: &Path, in_use: &str) -> SiteCompat {
    match read_required_php(path) {
        Some(req) => SiteCompat {
            site_name: name.to_string(),
            php_required: Some(req.clone()),
            php_in_use: in_use.to_string(),
            status: matches(in_use, &req),
        },
        None => SiteCompat {
            site_name: name.to_string(),
            php_required: None,
            php_in_use: in_use.to_string(),
            status: CompatStatus::NoRequirement,
        },
    }
}

/// Try to extract a major.minor PHP version from a composer constraint.
/// Examples: `^8.1` → `8.1`, `>=7.4` → `7.4`, `8.2.*` → `8.2`, `^7.4|^8.0` → `8.0`.
/// Returns the highest version found among alternatives.
#[allow(dead_code)]
pub fn suggest_version(constraint: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new(r"(\d+)\.(\d+)").ok()?;
    let mut best: Option<(u32, u32)> = None;
    for cap in re.captures_iter(constraint) {
        let maj: u32 = cap.get(1)?.as_str().parse().ok()?;
        let min: u32 = cap.get(2)?.as_str().parse().ok()?;
        best = match best {
            None => Some((maj, min)),
            Some((bm, bn)) if (maj, min) > (bm, bn) => Some((maj, min)),
            other => other,
        };
    }
    best.map(|(m, n)| format!("{}.{}", m, n))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_caret_constraint() {
        assert_eq!(matches("8.3", "^8.0"), CompatStatus::Compatible);
        assert_eq!(matches("7.4", "^8.0"), CompatStatus::Incompatible);
    }
    #[test]
    fn matches_alt_constraint() {
        assert_eq!(matches("8.3", "^7.4|^8.0"), CompatStatus::Compatible);
    }
    #[test]
    fn matches_star_constraint() {
        assert_eq!(matches("8.1", "8.*"), CompatStatus::Compatible);
    }
    #[test]
    fn invalid_constraint_returns_incompatible() {
        // bogus constraint can't match; we want Incompatible (not NoRequirement)
        assert_eq!(matches("8.3", "garbage"), CompatStatus::Incompatible);
    }
    #[test]
    fn suggest_version_from_caret() {
        assert_eq!(suggest_version("^8.1"), Some("8.1".to_string()));
    }
    #[test]
    fn suggest_version_from_alt_picks_highest() {
        assert_eq!(suggest_version("^7.4|^8.0"), Some("8.0".to_string()));
    }
    #[test]
    fn suggest_version_none_when_no_digits() {
        assert_eq!(suggest_version("garbage"), None);
    }
}
