#![allow(dead_code)]

//! Global `phpmyadmin.{tld}` site bootstrapping.
//!
//! For Phase 12 scope: we resolve the URL and ack the request without
//! performing the privileged nginx writes. Returning the URL is sufficient
//! for the UI to launch `xdg-open` and for tests to verify the planning.

use crate::valet::variant::ValetPaths;

pub fn global_url(tld: &str) -> String {
    format!("https://phpmyadmin.{}/", tld)
}

pub async fn setup_global_site(tld: &str) -> anyhow::Result<String> {
    Ok(global_url(tld))
}

pub fn is_global_site_configured(tld: &str, valet_paths: &ValetPaths) -> bool {
    let path = valet_paths
        .nginx_dir
        .join(format!("phpmyadmin.{}.conf", tld));
    path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn global_url_uses_tld() {
        assert_eq!(global_url("test"), "https://phpmyadmin.test/");
        assert_eq!(global_url("local"), "https://phpmyadmin.local/");
    }

    #[tokio::test]
    async fn setup_global_site_returns_url() {
        let url = setup_global_site("test").await.unwrap();
        assert_eq!(url, "https://phpmyadmin.test/");
    }

    #[test]
    fn is_global_site_configured_false_for_empty_dir() {
        let paths = ValetPaths {
            config_root: PathBuf::from("/tmp/vm-pma-noexist"),
            nginx_dir: PathBuf::from("/tmp/vm-pma-noexist/Nginx"),
            sites_dir: PathBuf::from("/tmp/vm-pma-noexist/Sites"),
            drivers_dir: PathBuf::from("/tmp/vm-pma-noexist/Drivers"),
            log_dir: PathBuf::from("/tmp/vm-pma-noexist/Log"),
            config_json: PathBuf::from("/tmp/vm-pma-noexist/config.json"),
            ca_dir: PathBuf::from("/tmp/vm-pma-noexist/CA"),
        };
        assert!(!is_global_site_configured("test", &paths));
    }
}
