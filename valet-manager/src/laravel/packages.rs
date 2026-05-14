#![allow(dead_code)]

use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LaravelPackages {
    pub horizon: bool,
    pub telescope: bool,
    pub pulse: bool,
    pub reverb: bool,
    pub octane: bool,
    pub filament: bool,
    pub livewire: bool,
    pub inertia: bool,
}

const PACKAGES: &[(&str, fn(&mut LaravelPackages))] = &[
    ("laravel/horizon", |p: &mut LaravelPackages| p.horizon = true),
    ("laravel/telescope", |p: &mut LaravelPackages| p.telescope = true),
    ("laravel/pulse", |p: &mut LaravelPackages| p.pulse = true),
    ("laravel/reverb", |p: &mut LaravelPackages| p.reverb = true),
    ("laravel/octane", |p: &mut LaravelPackages| p.octane = true),
    ("filament/filament", |p: &mut LaravelPackages| p.filament = true),
    ("livewire/livewire", |p: &mut LaravelPackages| p.livewire = true),
    ("inertiajs/inertia-laravel", |p: &mut LaravelPackages| p.inertia = true),
];

pub fn detect_from_json(content: &str) -> LaravelPackages {
    let mut out = LaravelPackages::default();
    let value: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return out,
    };
    for section in ["require", "require-dev"] {
        if let Some(obj) = value.get(section).and_then(|v| v.as_object()) {
            for (pkg, setter) in PACKAGES {
                if obj.contains_key(*pkg) {
                    setter(&mut out);
                }
            }
        }
    }
    out
}

pub async fn detect(site_path: &Path) -> anyhow::Result<LaravelPackages> {
    let composer = site_path.join("composer.json");
    let content = tokio::fs::read_to_string(&composer).await.unwrap_or_default();
    Ok(detect_from_json(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_from_json_finds_horizon_and_telescope() {
        let json = r#"{
            "require": { "laravel/horizon": "^5.0", "php": "^8.2" },
            "require-dev": { "laravel/telescope": "^4.0" }
        }"#;
        let p = detect_from_json(json);
        assert!(p.horizon);
        assert!(p.telescope);
        assert!(!p.pulse);
        assert!(!p.octane);
    }

    #[test]
    fn detect_from_json_finds_octane_and_filament() {
        let json = r#"{
            "require": {
                "laravel/octane": "^2.0",
                "filament/filament": "^3.0",
                "livewire/livewire": "^3.0",
                "inertiajs/inertia-laravel": "^1.0"
            }
        }"#;
        let p = detect_from_json(json);
        assert!(p.octane);
        assert!(p.filament);
        assert!(p.livewire);
        assert!(p.inertia);
    }

    #[test]
    fn detect_from_json_returns_default_on_malformed() {
        let p = detect_from_json("not json");
        assert_eq!(p, LaravelPackages::default());
    }

    #[test]
    fn detect_from_json_returns_default_on_missing_sections() {
        let p = detect_from_json("{}");
        assert_eq!(p, LaravelPackages::default());
    }
}
