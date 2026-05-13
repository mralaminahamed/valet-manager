#![allow(dead_code)]

use crate::valet::variant::ValetPaths;

/// Read the nginx config file for a named site.
/// Returns empty string if file doesn't exist.
pub async fn read_site_config(site: &str, valet_paths: &ValetPaths) -> String {
    let path = valet_paths.nginx_dir.join(site);
    tokio::fs::read_to_string(&path).await.unwrap_or_default()
}

/// Write nginx config for a site via privilege helper.
/// Uses HelperRequest::SaveIni with the nginx config path.
/// (Reusing SaveIni since it does the same thing: write content to a path)
pub async fn write_site_config(site: &str, content: &str, valet_paths: &ValetPaths) -> anyhow::Result<()> {
    use crate::php::privilege::{run_privileged, HelperRequest};
    let path = valet_paths.nginx_dir.join(site)
        .to_string_lossy()
        .to_string();
    run_privileged(&HelperRequest::SaveIni { path, content: content.to_string() }).await?;
    Ok(())
}

/// Reload nginx via systemctl.
pub async fn reload() -> anyhow::Result<()> {
    let output = tokio::process::Command::new("systemctl")
        .args(["reload", "nginx"])
        .output()
        .await?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("nginx reload failed: {}", err);
    }
    Ok(())
}

/// Inject directives into an nginx site config using sentinel comments.
/// The block is placed between:
///   # BEGIN valet-manager
///   <content>
///   # END valet-manager
/// If sentinel section already exists, it's replaced.
/// If not, the block is appended.
pub fn inject_directives(config: &str, block: &str) -> String {
    const BEGIN: &str = "# BEGIN valet-manager";
    const END: &str = "# END valet-manager";

    let injected = format!("{}\n{}\n{}", BEGIN, block.trim(), END);

    if config.contains(BEGIN) {
        // Replace existing block
        let start = config.find(BEGIN).unwrap_or(0);
        let end = config.find(END).map(|i| i + END.len()).unwrap_or(config.len());
        format!("{}{}{}", &config[..start], injected, &config[end..])
    } else {
        format!("{}\n\n{}\n", config.trim_end(), injected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_directives_appends_when_no_sentinel() {
        let config = "server { listen 80; }";
        let result = inject_directives(config, "add_header X-Test 1;");
        assert!(result.contains("# BEGIN valet-manager"));
        assert!(result.contains("add_header X-Test 1;"));
        assert!(result.contains("# END valet-manager"));
    }

    #[test]
    fn inject_directives_replaces_existing_sentinel() {
        let config = "server {\n# BEGIN valet-manager\nold directive;\n# END valet-manager\n}";
        let result = inject_directives(config, "new directive;");
        assert!(result.contains("new directive;"));
        assert!(!result.contains("old directive;"));
        assert!(result.contains("# BEGIN valet-manager"));
        assert!(result.contains("# END valet-manager"));
    }

    #[test]
    fn inject_directives_preserves_surrounding_config() {
        let config = "# top comment\nserver { }\n# bottom";
        let result = inject_directives(config, "foo bar;");
        assert!(result.contains("# top comment"));
        assert!(result.contains("foo bar;"));
    }
}
