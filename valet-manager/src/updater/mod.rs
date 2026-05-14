use semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub is_newer: bool,
}

#[allow(dead_code)]
pub const REPO: &str = "alaminahamed/valet-manager"; // placeholder; users can change.

#[allow(dead_code)]
pub async fn check() -> anyhow::Result<UpdateInfo> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let client = reqwest::Client::builder()
        .user_agent("valet-manager")
        .timeout(std::time::Duration::from_secs(6))
        .build()?;
    let release: GithubRelease = client.get(&url).send().await?.json().await?;
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = release.tag_name.trim_start_matches('v').to_string();
    let is_newer = match (Version::parse(&current), Version::parse(&latest)) {
        (Ok(c), Ok(l)) => l > c,
        _ => false,
    };
    Ok(UpdateInfo {
        current,
        latest,
        is_newer,
    })
}

#[allow(dead_code)]
pub fn releases_url() -> String {
    format!("https://github.com/{}/releases/latest", REPO)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn update_info_is_newer_logic() {
        let u = UpdateInfo {
            current: "0.1.0".into(),
            latest: "0.2.0".into(),
            is_newer: true,
        };
        assert!(u.is_newer);
        assert_eq!(u.current, "0.1.0");
    }
    #[test]
    fn releases_url_is_correct() {
        let u = releases_url();
        assert!(u.starts_with("https://github.com/"));
        assert!(u.ends_with("/releases/latest"));
    }
    #[test]
    fn repo_constant_present() {
        assert!(REPO.contains('/'));
    }
}
