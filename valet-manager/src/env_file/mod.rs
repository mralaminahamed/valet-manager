use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EnvEntry {
    Kv { key: String, value: String, raw_line: String, is_secret: bool },
    Comment(String),
    Blank,
}

#[allow(dead_code)]
pub fn parse(content: &str) -> Vec<EnvEntry> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            out.push(EnvEntry::Blank);
        } else if trimmed.starts_with('#') {
            out.push(EnvEntry::Comment(line.to_string()));
        } else if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_string();
            let value = unquote(v.trim());
            let is_secret = looks_like_secret(&key);
            out.push(EnvEntry::Kv { key, value, raw_line: line.to_string(), is_secret });
        } else {
            out.push(EnvEntry::Comment(line.to_string()));
        }
    }
    out
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2) {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

fn looks_like_secret(key: &str) -> bool {
    let up = key.to_uppercase();
    up.contains("PASS") || up.contains("SECRET") || up.contains("TOKEN")
        || up.contains("KEY") || up.ends_with("_DSN") || up.contains("PRIVATE")
}

#[allow(dead_code)]
pub fn group_key(key: &str) -> &'static str {
    let up = key.to_uppercase();
    if up.starts_with("APP_") { "APP" }
    else if up.starts_with("DB_") || up.starts_with("DATABASE_") { "DB" }
    else if up.starts_with("MAIL_") { "MAIL" }
    else if up.starts_with("REDIS_") { "REDIS" }
    else { "Other" }
}

#[allow(dead_code)]
pub fn render(entries: &[EnvEntry]) -> String {
    let mut out = String::new();
    for e in entries {
        match e {
            EnvEntry::Blank => out.push('\n'),
            EnvEntry::Comment(line) => { out.push_str(line); out.push('\n'); }
            EnvEntry::Kv { key, value, .. } => {
                let needs_quote = value.contains(' ') || value.contains('#');
                let val = if needs_quote { format!("\"{}\"", value.replace('"', "\\\"")) } else { value.clone() };
                out.push_str(&format!("{}={}\n", key, val));
            }
        }
    }
    out
}

/// Compare local entries against `.env.example` to find missing/extra keys.
#[allow(dead_code)]
pub fn diff_keys(local: &[EnvEntry], example: &[EnvEntry]) -> (Vec<String>, Vec<String>) {
    let local_keys: std::collections::HashSet<String> = local.iter().filter_map(|e| if let EnvEntry::Kv { key, .. } = e { Some(key.clone()) } else { None }).collect();
    let example_keys: std::collections::HashSet<String> = example.iter().filter_map(|e| if let EnvEntry::Kv { key, .. } = e { Some(key.clone()) } else { None }).collect();
    let mut missing: Vec<String> = example_keys.difference(&local_keys).cloned().collect();
    let mut extra: Vec<String>   = local_keys.difference(&example_keys).cloned().collect();
    missing.sort();
    extra.sort();
    (missing, extra)
}

/// Atomic write: write to .tmp, fsync, rename.
#[allow(dead_code)]
pub fn save_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension("env.tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_kv_and_comments_and_blanks() {
        let s = "APP_NAME=Foo\n# a comment\n\nDB_PASSWORD=\"hello world\"\n";
        let entries = parse(s);
        assert_eq!(entries.len(), 4);
        match &entries[0] { EnvEntry::Kv { key, value, is_secret, .. } => { assert_eq!(key, "APP_NAME"); assert_eq!(value, "Foo"); assert!(!*is_secret); }, _ => panic!() }
        match &entries[3] { EnvEntry::Kv { key, value, is_secret, .. } => { assert_eq!(key, "DB_PASSWORD"); assert_eq!(value, "hello world"); assert!(*is_secret); }, _ => panic!() }
    }
    #[test]
    fn group_key_known_prefixes() {
        assert_eq!(group_key("APP_NAME"), "APP");
        assert_eq!(group_key("DB_PASSWORD"), "DB");
        assert_eq!(group_key("MAIL_HOST"), "MAIL");
        assert_eq!(group_key("REDIS_URL"), "REDIS");
        assert_eq!(group_key("RANDOM_THING"), "Other");
    }
    #[test]
    fn diff_keys_reports_missing_and_extra() {
        let local = parse("A=1\nB=2\n");
        let example = parse("A=1\nC=3\n");
        let (missing, extra) = diff_keys(&local, &example);
        assert_eq!(missing, vec!["C".to_string()]);
        assert_eq!(extra, vec!["B".to_string()]);
    }
    #[test]
    fn render_roundtrips_simple() {
        let s = "A=1\n# c\nB=\"hello\"\n";
        let e = parse(s);
        let out = render(&e);
        assert!(out.contains("A=1"));
        assert!(out.contains("# c"));
        assert!(out.contains("B=hello") || out.contains("B=\"hello\""));
    }
    #[test]
    fn secret_detection() {
        assert!(looks_like_secret("DB_PASSWORD"));
        assert!(looks_like_secret("APP_KEY"));
        assert!(looks_like_secret("API_TOKEN"));
        assert!(looks_like_secret("STRIPE_SECRET"));
        assert!(!looks_like_secret("APP_NAME"));
    }
    #[test]
    fn save_atomic_writes_file_in_tempdir() {
        let dir = std::env::temp_dir().join(format!("vm-env-test-{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        save_atomic(&path, "X=1\n").unwrap();
        let read = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read, "X=1\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
