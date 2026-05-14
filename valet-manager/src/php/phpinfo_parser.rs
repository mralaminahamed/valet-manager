use std::collections::BTreeMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PhpInfoEntry {
    pub key: String,
    pub local_value: String,
    pub master_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PhpInfoSection {
    pub name: String,
    pub entries: Vec<PhpInfoEntry>,
}

/// Parse `php -r "phpinfo();"` HTML output into sections.
/// Sections are delimited by `<h2>Section Name</h2>`.
/// Entries are rows like `<tr><td class="e">key</td><td class="v">local</td><td class="v">master</td></tr>`
/// Some rows have only 2 cells (key + value); treat the value as local AND master.
#[allow(dead_code)]
pub fn parse(html: &str) -> Vec<PhpInfoSection> {
    use regex::Regex;
    let mut sections: Vec<PhpInfoSection> = Vec::new();
    let section_re = Regex::new(r#"(?is)<h2[^>]*>(.*?)</h2>"#).unwrap();
    let row_re = Regex::new(r#"(?is)<tr[^>]*>(.*?)</tr>"#).unwrap();
    let cell_re = Regex::new(r#"(?is)<td[^>]*>(.*?)</td>"#).unwrap();
    let strip_tags = Regex::new(r#"<[^>]+>"#).unwrap();
    let entity_re = Regex::new(r#"&nbsp;|&#160;"#).unwrap();

    let parse_body = |body: &str, sec: &mut PhpInfoSection| {
        for row_cap in row_re.captures_iter(body) {
            let row = row_cap.get(1).unwrap().as_str();
            let cells: Vec<String> = cell_re
                .captures_iter(row)
                .map(|c| {
                    let inner = c.get(1).unwrap().as_str();
                    let stripped = strip_tags.replace_all(inner, "");
                    let cleaned = entity_re.replace_all(&stripped, " ");
                    cleaned.trim().to_string()
                })
                .collect();
            if cells.len() >= 2 {
                let key = cells[0].clone();
                let local = cells[1].clone();
                let master = cells.get(2).cloned().unwrap_or_else(|| local.clone());
                if !key.is_empty() {
                    sec.entries.push(PhpInfoEntry {
                        key,
                        local_value: local,
                        master_value: master,
                    });
                }
            }
        }
    };

    let mut last_pos = 0usize;
    let mut current: Option<PhpInfoSection> = None;
    for cap in section_re.captures_iter(html) {
        let m = cap.get(0).unwrap();
        if let Some(mut sec) = current.take() {
            let body = &html[last_pos..m.start()];
            parse_body(body, &mut sec);
            sections.push(sec);
        }
        let name = strip_tags
            .replace_all(cap.get(1).unwrap().as_str(), "")
            .trim()
            .to_string();
        current = Some(PhpInfoSection {
            name,
            entries: Vec::new(),
        });
        last_pos = m.end();
    }
    if let Some(mut sec) = current.take() {
        let body = &html[last_pos..];
        parse_body(body, &mut sec);
        sections.push(sec);
    }
    sections
}

/// Run `php -r "phpinfo();"` and return raw HTML output.
#[allow(dead_code)]
pub async fn load(version: &str) -> anyhow::Result<String> {
    let bin = if version == "system" {
        "php".to_string()
    } else {
        format!("php{}", version)
    };
    let out = tokio::process::Command::new(&bin)
        .arg("-r")
        .arg("phpinfo();")
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[allow(dead_code)]
pub fn to_index(sections: &[PhpInfoSection]) -> BTreeMap<String, usize> {
    sections
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.clone(), i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = r#"<h2>Core</h2><tr><td class="e">memory_limit</td><td class="v">128M</td><td class="v">128M</td></tr><h2>date</h2><tr><td>date.timezone</td><td>UTC</td></tr>"#;

    #[test]
    fn parses_section_names() {
        let s = parse(SAMPLE);
        let names: Vec<&str> = s.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, vec!["Core", "date"]);
    }
    #[test]
    fn parses_entries_with_local_and_master() {
        let s = parse(SAMPLE);
        let core = &s[0];
        assert_eq!(core.entries.len(), 1);
        assert_eq!(core.entries[0].key, "memory_limit");
        assert_eq!(core.entries[0].local_value, "128M");
        assert_eq!(core.entries[0].master_value, "128M");
    }
    #[test]
    fn parses_two_cell_row_fills_master_with_local() {
        let s = parse(SAMPLE);
        let date = &s[1];
        assert_eq!(date.entries[0].key, "date.timezone");
        assert_eq!(date.entries[0].local_value, "UTC");
        assert_eq!(date.entries[0].master_value, "UTC");
    }
    #[test]
    fn empty_html_returns_empty_vec() {
        assert!(parse("").is_empty());
    }
    #[test]
    fn to_index_maps_names() {
        let s = parse(SAMPLE);
        let idx = to_index(&s);
        assert_eq!(idx.get("Core"), Some(&0));
        assert_eq!(idx.get("date"), Some(&1));
    }
}
