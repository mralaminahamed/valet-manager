use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use crate::creator::output_streamer::{OutputLine, Stream};

/// Template choice for the in-process static-site generator.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StaticTemplate {
    Blank,
    Tailwind,
    Bootstrap,
}

impl StaticTemplate {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "tailwind" => Self::Tailwind,
            "bootstrap" => Self::Bootstrap,
            _ => Self::Blank,
        }
    }
}

/// Configuration for a static HTML site generation run.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StaticHtmlConfig {
    pub name: String,
    pub directory: PathBuf,
    pub template: StaticTemplate,
}

impl StaticHtmlConfig {
    /// Build from form_values HashMap (keys match project_types.rs option keys).
    #[allow(dead_code)]
    pub fn from_form(values: &HashMap<String, String>) -> Self {
        StaticHtmlConfig {
            name: values.get("name").cloned().unwrap_or_default(),
            directory: PathBuf::from(values.get("directory").cloned().unwrap_or_default()),
            template: StaticTemplate::from_str(
                values.get("template").map(|s| s.as_str()).unwrap_or("blank"),
            ),
        }
    }

    pub fn site_dir(&self) -> PathBuf {
        self.directory.join(&self.name)
    }

    pub fn render_index(&self) -> String {
        match self.template {
            StaticTemplate::Blank => format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n  <title>{name}</title>\n</head>\n<body>\n  <h1>{name}</h1>\n  <p>Welcome to your new site.</p>\n</body>\n</html>\n",
                name = self.name
            ),
            StaticTemplate::Tailwind => format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n  <title>{name}</title>\n  <script src=\"https://cdn.tailwindcss.com\"></script>\n</head>\n<body class=\"min-h-screen flex items-center justify-center bg-slate-900 text-slate-100\">\n  <main class=\"text-center\">\n    <h1 class=\"text-3xl font-semibold\">{name}</h1>\n    <p class=\"mt-2 text-slate-400\">Powered by Tailwind CDN.</p>\n  </main>\n</body>\n</html>\n",
                name = self.name
            ),
            StaticTemplate::Bootstrap => format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n  <title>{name}</title>\n  <link href=\"https://cdn.jsdelivr.net/npm/bootstrap@5/dist/css/bootstrap.min.css\" rel=\"stylesheet\">\n</head>\n<body class=\"d-flex align-items-center justify-content-center min-vh-100 bg-dark text-light\">\n  <main class=\"text-center\">\n    <h1>{name}</h1>\n    <p class=\"text-secondary\">Powered by Bootstrap CDN.</p>\n  </main>\n</body>\n</html>\n",
                name = self.name
            ),
        }
    }
}

/// Generate the static site files in-process. No subprocess.
/// Streams synthetic OutputLine messages to `tx`.
#[allow(dead_code)]
pub async fn run(
    config: &StaticHtmlConfig,
    tx: Sender<OutputLine>,
) -> anyhow::Result<bool> {
    let dir = config.site_dir();

    let _ = tx
        .send(OutputLine {
            text: format!("Creating directory {}", dir.display()),
            stream: Stream::Stdout,
            timestamp: chrono::Local::now(),
        })
        .await;

    tokio::fs::create_dir_all(&dir).await?;

    let index = dir.join("index.html");
    let html = config.render_index();
    tokio::fs::write(&index, html).await?;

    let _ = tx
        .send(OutputLine {
            text: format!("Wrote {}", index.display()),
            stream: Stream::Stdout,
            timestamp: chrono::Local::now(),
        })
        .await;

    let _ = tx
        .send(OutputLine {
            text: "Static site ready.".to_string(),
            stream: Stream::Stdout,
            timestamp: chrono::Local::now(),
        })
        .await;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_from_str_blank_default() {
        assert_eq!(StaticTemplate::from_str("blank"), StaticTemplate::Blank);
        assert_eq!(StaticTemplate::from_str(""), StaticTemplate::Blank);
        assert_eq!(StaticTemplate::from_str("unknown"), StaticTemplate::Blank);
    }

    #[test]
    fn template_from_str_tailwind() {
        assert_eq!(StaticTemplate::from_str("tailwind"), StaticTemplate::Tailwind);
    }

    #[test]
    fn template_from_str_bootstrap() {
        assert_eq!(StaticTemplate::from_str("bootstrap"), StaticTemplate::Bootstrap);
    }

    #[test]
    fn site_dir_joins_correctly() {
        let config = StaticHtmlConfig {
            name: "mysite".to_string(),
            directory: PathBuf::from("/var/www"),
            template: StaticTemplate::Blank,
        };
        assert_eq!(config.site_dir(), PathBuf::from("/var/www/mysite"));
    }

    #[test]
    fn render_index_blank_contains_name() {
        let config = StaticHtmlConfig {
            name: "hello-world".to_string(),
            directory: PathBuf::from("/tmp"),
            template: StaticTemplate::Blank,
        };
        let html = config.render_index();
        assert!(html.contains("hello-world"));
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("Welcome to your new site."));
    }

    #[test]
    fn render_index_tailwind_contains_cdn() {
        let config = StaticHtmlConfig {
            name: "tw-site".to_string(),
            directory: PathBuf::from("/tmp"),
            template: StaticTemplate::Tailwind,
        };
        let html = config.render_index();
        assert!(html.contains("cdn.tailwindcss.com"));
        assert!(html.contains("tw-site"));
    }

    #[test]
    fn render_index_bootstrap_contains_cdn() {
        let config = StaticHtmlConfig {
            name: "bs-site".to_string(),
            directory: PathBuf::from("/tmp"),
            template: StaticTemplate::Bootstrap,
        };
        let html = config.render_index();
        assert!(html.contains("cdn.jsdelivr.net"));
        assert!(html.contains("bootstrap"));
        assert!(html.contains("bs-site"));
    }

    #[test]
    fn from_form_defaults_to_blank_template() {
        let values = HashMap::new();
        let config = StaticHtmlConfig::from_form(&values);
        assert_eq!(config.template, StaticTemplate::Blank);
    }

    #[test]
    fn from_form_picks_up_template() {
        let mut values = HashMap::new();
        values.insert("template".to_string(), "tailwind".to_string());
        let config = StaticHtmlConfig::from_form(&values);
        assert_eq!(config.template, StaticTemplate::Tailwind);
    }

    #[test]
    fn from_form_picks_up_name_and_directory() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "demo".to_string());
        values.insert("directory".to_string(), "/srv/sites".to_string());
        let config = StaticHtmlConfig::from_form(&values);
        assert_eq!(config.name, "demo");
        assert_eq!(config.directory, PathBuf::from("/srv/sites"));
    }
}
