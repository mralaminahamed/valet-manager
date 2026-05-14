#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::site_config::models::BasicAuthUser;

/// Bcrypt-hash a password (uses bcrypt::DEFAULT_COST).
pub fn hash_password(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap_or_else(|_| String::new())
}

/// Path where the per-site htpasswd file lives.
pub fn htpasswd_path(site_name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".config/valet-manager/auth").join(format!("{site_name}.htpasswd"))
}

/// Render the contents of an htpasswd file. Each user becomes `username:hash`.
pub fn generate_htpasswd(users: &[BasicAuthUser]) -> String {
    let mut lines: Vec<String> = users
        .iter()
        .map(|u| format!("{}:{}", u.username, u.password_hash))
        .collect();
    if !lines.is_empty() {
        lines.push(String::new()); // trailing newline
    }
    lines.join("\n")
}

async fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

/// Write the htpasswd file for a site.
pub async fn write_htpasswd(site_name: &str, users: &[BasicAuthUser]) -> anyhow::Result<()> {
    let path = htpasswd_path(site_name);
    let content = generate_htpasswd(users);
    atomic_write(&path, &content).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_produces_bcrypt_format() {
        let h = hash_password("password");
        assert!(h.starts_with("$2") && h.len() > 20);
    }

    #[test]
    fn hash_password_is_verifiable() {
        let h = hash_password("hunter2");
        assert!(bcrypt::verify("hunter2", &h).unwrap());
        assert!(!bcrypt::verify("wrong", &h).unwrap());
    }

    #[test]
    fn htpasswd_path_lives_in_config_home() {
        let p = htpasswd_path("myapp");
        let s = p.to_string_lossy();
        assert!(s.contains("valet-manager/auth/myapp.htpasswd"));
    }

    #[test]
    fn generate_htpasswd_joins_users_with_newlines() {
        let users = vec![
            BasicAuthUser { username: "alice".into(), password_hash: "h1".into() },
            BasicAuthUser { username: "bob".into(), password_hash: "h2".into() },
        ];
        let out = generate_htpasswd(&users);
        assert!(out.contains("alice:h1"));
        assert!(out.contains("bob:h2"));
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn generate_htpasswd_empty_when_no_users() {
        assert_eq!(generate_htpasswd(&[]), "");
    }

    #[tokio::test]
    async fn write_htpasswd_round_trips() {
        // Write to a temp-path-like name; redirect via HOME override would be ideal.
        let users = vec![BasicAuthUser { username: "u".into(), password_hash: "h".into() }];
        // Use a unique site name so we don't clobber anything real.
        let site_name = format!("vm-htpasswd-test-{}", std::process::id());
        write_htpasswd(&site_name, &users).await.unwrap();
        let content = tokio::fs::read_to_string(htpasswd_path(&site_name)).await.unwrap();
        assert!(content.contains("u:h"));
        let _ = tokio::fs::remove_file(htpasswd_path(&site_name)).await;
    }
}
