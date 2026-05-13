#![allow(dead_code)]

use anyhow::Result;
use regex::Regex;

use crate::php::privilege::{run_privileged, HelperRequest};
use crate::php::types::{IniEntry, IniSection, IniType};

pub fn parse(raw: &str) -> Vec<IniSection> {
    let mut sections: Vec<IniSection> = Vec::new();
    let mut current = IniSection {
        name: "General".to_string(),
        entries: Vec::new(),
    };

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        if trimmed.starts_with('[') {
            let name = trimmed
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_string();
            let finished = std::mem::replace(
                &mut current,
                IniSection {
                    name,
                    entries: Vec::new(),
                },
            );
            sections.push(finished);
        } else if let Some((key, value)) = trimmed.split_once('=') {
            current.entries.push(IniEntry {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
                comment: None,
            });
        }
    }

    sections.push(current);
    sections
}

pub fn render(sections: &[IniSection]) -> String {
    let mut out = String::new();

    for (i, section) in sections.iter().enumerate() {
        if !(i == 0 && section.name == "General") {
            out.push_str(&format!("[{}]\n", section.name));
        }
        for entry in &section.entries {
            out.push_str(&format!("{} = {}\n", entry.key, entry.value));
        }
        out.push('\n');
    }

    out
}

pub fn validate_value(key: &str, value: &str) -> Option<String> {
    let is_memory_key =
        key.ends_with("_size") || key.ends_with("_limit");

    if is_memory_key {
        let memory_re = Regex::new(r"^\d+[KMGBkmgb]?$").unwrap();
        if !memory_re.is_match(value) {
            return Some(format!(
                "Invalid memory value '{}': expected digits followed by optional K, M, G, or B",
                value
            ));
        }
        return None;
    }

    if key == "max_execution_time" || key == "max_input_time" {
        if value.parse::<u64>().is_err() {
            return Some(format!(
                "Invalid value '{}' for '{}': expected a non-negative integer",
                value, key
            ));
        }
        return None;
    }

    None
}

pub async fn save(version: &str, ini_type: &IniType, content: &str) -> Result<()> {
    let path = match ini_type {
        IniType::Cli => format!("/etc/php/{}/cli/php.ini", version),
        IniType::Fpm => format!("/etc/php/{}/fpm/php.ini", version),
    };

    run_privileged(&HelperRequest::SaveIni {
        path,
        content: content.to_string(),
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        let sections = parse("");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "General");
        assert!(sections[0].entries.is_empty());
    }

    #[test]
    fn parse_section_header() {
        let sections = parse("[PHP]\nfoo = bar");
        let php = sections.iter().find(|s| s.name == "PHP").expect("PHP section");
        assert_eq!(php.entries.len(), 1);
        assert_eq!(php.entries[0].key, "foo");
        assert_eq!(php.entries[0].value, "bar");
    }

    #[test]
    fn parse_skips_comments() {
        let sections = parse(";comment\nfoo = bar");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].entries.len(), 1);
        assert_eq!(sections[0].entries[0].key, "foo");
    }

    #[test]
    fn validate_memory_value_valid() {
        assert!(validate_value("memory_limit", "128M").is_none());
    }

    #[test]
    fn validate_memory_value_invalid() {
        assert!(validate_value("memory_limit", "abc").is_some());
    }
}
