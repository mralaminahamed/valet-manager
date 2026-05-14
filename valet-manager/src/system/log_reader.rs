#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    NginxError,
    PhpFpm,
    ValetFpm,
    Access,
}

impl LogSource {
    pub fn label(&self) -> &'static str {
        match self {
            LogSource::NginxError => "Nginx Error",
            LogSource::PhpFpm => "PHP-FPM",
            LogSource::ValetFpm => "Valet FPM",
            LogSource::Access => "Access",
        }
    }

    pub fn path(&self) -> std::path::PathBuf {
        match self {
            LogSource::NginxError => std::path::PathBuf::from("/var/log/nginx/error.log"),
            LogSource::PhpFpm => std::path::PathBuf::from("/var/log/php/error.log"),
            LogSource::ValetFpm => std::env::var("HOME")
                .map(|h| std::path::PathBuf::from(h).join(".config/valet-linux-plus/Log/fpm-php.www.log"))
                .unwrap_or_default(),
            LogSource::Access => std::path::PathBuf::from("/var/log/nginx/access.log"),
        }
    }
}

/// Read up to the last `max_lines` lines of the given log source.
/// If the file does not exist, returns a one-line placeholder.
pub async fn tail(source: LogSource, max_lines: usize) -> anyhow::Result<Vec<String>> {
    let path = source.path();
    if !path.exists() {
        return Ok(vec![format!("(no log file at {})", path.display())]);
    }
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let lines: Vec<String> = content
        .lines()
        .rev()
        .take(max_lines)
        .map(|s| s.to_string())
        .collect();
    Ok(lines.into_iter().rev().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSeverity {
    Error,
    Warn,
    Notice,
    Info,
}

/// Classify a log line by scanning for severity keywords.
pub fn classify(line: &str) -> LineSeverity {
    let lower = line.to_lowercase();
    if lower.contains("error") || lower.contains("emerg") || lower.contains("crit") {
        LineSeverity::Error
    } else if lower.contains("warn") {
        LineSeverity::Warn
    } else if lower.contains("notice") {
        LineSeverity::Notice
    } else {
        LineSeverity::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_error_keyword() {
        assert_eq!(classify("[ERROR] something broke"), LineSeverity::Error);
    }

    #[test]
    fn classify_emerg_keyword() {
        assert_eq!(classify("emerg: kernel panic"), LineSeverity::Error);
    }

    #[test]
    fn classify_crit_keyword() {
        assert_eq!(classify("crit: failure"), LineSeverity::Error);
    }

    #[test]
    fn classify_warn_keyword() {
        assert_eq!(classify("Warning: deprecated"), LineSeverity::Warn);
    }

    #[test]
    fn classify_notice_keyword() {
        assert_eq!(classify("[notice] starting"), LineSeverity::Notice);
    }

    #[test]
    fn classify_plain_is_info() {
        assert_eq!(classify("connection accepted"), LineSeverity::Info);
    }

    #[test]
    fn log_source_label_returns_expected() {
        assert_eq!(LogSource::NginxError.label(), "Nginx Error");
        assert_eq!(LogSource::PhpFpm.label(), "PHP-FPM");
        assert_eq!(LogSource::ValetFpm.label(), "Valet FPM");
        assert_eq!(LogSource::Access.label(), "Access");
    }

    #[test]
    fn log_source_paths_are_distinct() {
        assert_ne!(LogSource::NginxError.path(), LogSource::Access.path());
        assert_ne!(LogSource::NginxError.path(), LogSource::PhpFpm.path());
    }
}
