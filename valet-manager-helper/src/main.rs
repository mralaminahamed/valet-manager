use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const PHP_CONFIG_ROOT: &str = "/etc/php";
const MAX_PATH_LEN: usize = 4096;
const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum HelperRequest {
    SwitchPhp { version: String },
    StartFpm { version: String },
    StopFpm { version: String },
    RestartFpm { version: String },
    EnableExtension { version: String, extension: String },
    DisableExtension { version: String, extension: String },
    SaveIni { path: String, content: String },
}

#[derive(Debug, Serialize)]
struct HelperResponse {
    success: bool,
    message: String,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut rejected = false;
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let response = match serde_json::from_str::<HelperRequest>(&line) {
            Ok(req) => handle_request(req),
            Err(e) => HelperResponse {
                success: false,
                message: format!("parse error: {}", e),
            },
        };
        rejected |= !response.success;
        let json = serde_json::to_string(&response).unwrap_or_default();
        let _ = writeln!(stdout, "{}", json);
        let _ = stdout.flush();
    }
    if rejected {
        std::process::exit(1);
    }
}

fn validate_request(req: &HelperRequest) -> Result<(), String> {
    let (version, extension) = match req {
        HelperRequest::SwitchPhp { version }
        | HelperRequest::StartFpm { version }
        | HelperRequest::StopFpm { version }
        | HelperRequest::RestartFpm { version } => (Some(version), None),
        HelperRequest::EnableExtension { version, extension }
        | HelperRequest::DisableExtension { version, extension } => {
            (Some(version), Some(extension))
        }
        HelperRequest::SaveIni { .. } => (None, None),
    };
    if let Some(v) = version {
        if !valid_version(v) {
            return Err(format!("invalid PHP version: {:?}", truncate(v)));
        }
    }
    if let Some(e) = extension {
        if !valid_extension(e) {
            return Err(format!("invalid extension name: {:?}", truncate(e)));
        }
    }
    Ok(())
}

fn truncate(s: &str) -> String {
    s.chars().take(32).collect()
}

fn reject(message: impl Into<String>) -> HelperResponse {
    HelperResponse {
        success: false,
        message: message.into(),
    }
}

/// Accepts `major.minor` with one or two digits per part, for example `8.3`.
fn valid_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let (Some(major), Some(minor), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let ok = |p: &str| (1..=2).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit());
    ok(major) && ok(minor)
}

/// Accepts `[a-z0-9_]+`, at most `MAX_NAME_LEN` bytes.
fn valid_extension(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Accepts `php.ini` or `<name>.ini` where name is `[A-Za-z0-9_.-]+`.
fn valid_ini_file_name(name: &str) -> bool {
    if name == "php.ini" {
        return true;
    }
    let Some(stem) = name.strip_suffix(".ini") else {
        return false;
    };
    !stem.is_empty()
        && name.len() <= MAX_NAME_LEN
        && !stem.starts_with('.')
        && stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Checks that `path` is a PHP configuration file location under `root` and
/// returns the path to write. Accepted shapes, with `V` a valid version:
/// `root/V/{cli,fpm}/php.ini`, `root/V/{cli,fpm}/conf.d/<name>.ini` and
/// `root/V/mods-available/<name>.ini`. Symlinks are resolved and the result
/// must stay inside `root`.
fn validate_ini_path(path: &str, root: &Path) -> Result<PathBuf, String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN {
        return Err("path has an invalid length".to_string());
    }
    if path.contains('\0') {
        return Err("path contains a NUL byte".to_string());
    }
    if path.split('/').any(|seg| seg == "." || seg == "..") {
        return Err("path must not contain . or .. segments".to_string());
    }
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err("path must be absolute".to_string());
    }
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("path is outside {}", root.display()))?;
    let parts: Vec<&str> = rel
        .components()
        .map(|c| c.as_os_str().to_str().unwrap_or(""))
        .collect();
    let shape_ok = match parts.as_slice() {
        [v, "mods-available", file] => valid_version(v) && valid_ini_file_name(file),
        [v, sapi, file] => valid_version(v) && matches!(*sapi, "cli" | "fpm") && *file == "php.ini",
        [v, sapi, "conf.d", file] => {
            valid_version(v) && matches!(*sapi, "cli" | "fpm") && valid_ini_file_name(file)
        }
        _ => false,
    };
    if !shape_ok {
        return Err("path is not an allowed PHP configuration file".to_string());
    }

    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {}", root.display(), e))?;
    let parent = path.parent().ok_or("path has no parent")?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("cannot resolve directory: {}", e))?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err("directory resolves outside the PHP configuration root".to_string());
    }
    let target = canonical_parent.join(path.file_name().ok_or("path has no file name")?);
    if std::fs::symlink_metadata(&target).is_ok() {
        let canonical_target = target
            .canonicalize()
            .map_err(|e| format!("cannot resolve file: {}", e))?;
        if !canonical_target.starts_with(&canonical_root) {
            return Err("file resolves outside the PHP configuration root".to_string());
        }
        return Ok(canonical_target);
    }
    Ok(target)
}

fn handle_request(req: HelperRequest) -> HelperResponse {
    match validate_request(&req) {
        Ok(()) => {}
        Err(message) => return reject(message),
    }
    match req {
        HelperRequest::SwitchPhp { version } => {
            let binary = format!("/usr/bin/php{}", version);
            run_cmd("update-alternatives", &["--set", "php", &binary])
        }
        HelperRequest::StartFpm { version } => {
            run_cmd("systemctl", &["start", &format!("php{}-fpm", version)])
        }
        HelperRequest::StopFpm { version } => {
            run_cmd("systemctl", &["stop", &format!("php{}-fpm", version)])
        }
        HelperRequest::RestartFpm { version } => {
            run_cmd("systemctl", &["restart", &format!("php{}-fpm", version)])
        }
        HelperRequest::EnableExtension { version, extension } => {
            run_cmd("phpenmod", &["-v", &version, &extension])
        }
        HelperRequest::DisableExtension { version, extension } => {
            run_cmd("phpdismod", &["-v", &version, &extension])
        }
        HelperRequest::SaveIni { path, content } => {
            let target = match validate_ini_path(&path, Path::new(PHP_CONFIG_ROOT)) {
                Ok(t) => t,
                Err(message) => return reject(message),
            };
            match fs::write(&target, content) {
                Ok(_) => HelperResponse {
                    success: true,
                    message: "OK".to_string(),
                },
                Err(e) => HelperResponse {
                    success: false,
                    message: e.to_string(),
                },
            }
        }
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> HelperResponse {
    match Command::new(cmd).args(args).output() {
        Ok(out) if out.status.success() => HelperResponse {
            success: true,
            message: "OK".to_string(),
        },
        Ok(out) => HelperResponse {
            success: false,
            message: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => HelperResponse {
            success: false,
            message: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_request_switch_php_parses() {
        let json = r#"{"action":"SwitchPhp","version":"8.3"}"#;
        let req: HelperRequest = serde_json::from_str(json).unwrap();
        if let HelperRequest::SwitchPhp { version } = req {
            assert_eq!(version, "8.3");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn helper_request_enable_ext_parses() {
        let json = r#"{"action":"EnableExtension","version":"8.3","extension":"xdebug"}"#;
        let req: HelperRequest = serde_json::from_str(json).unwrap();
        if let HelperRequest::EnableExtension { version, extension } = req {
            assert_eq!(version, "8.3");
            assert_eq!(extension, "xdebug");
        } else {
            panic!("wrong variant");
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("vm-helper-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for d in ["8.3/cli/conf.d", "8.3/fpm/conf.d", "8.3/mods-available"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(root.join("8.3/cli/php.ini"), "x").unwrap();
        root.canonicalize().unwrap()
    }

    fn check(root: &Path, rel_or_abs: &str) -> Result<PathBuf, String> {
        let p = if rel_or_abs.starts_with('/') {
            rel_or_abs.to_string()
        } else {
            format!("{}/{}", root.display(), rel_or_abs)
        };
        validate_ini_path(&p, root)
    }

    #[test]
    fn versions_accepted() {
        for v in ["8.3", "7.4", "10.0", "8.12"] {
            assert!(valid_version(v), "{v}");
        }
    }

    #[test]
    fn versions_rejected() {
        let long = "1".repeat(5000) + ".1";
        for v in [
            "",
            "8",
            "8.",
            ".3",
            "8.3.1",
            "8.3 ",
            "8.3;id",
            "8.3\n",
            "../8.3",
            "a.b",
            "123.4",
            "8.3\0",
            "$(id).1",
            "8.3&&ls",
            long.as_str(),
        ] {
            assert!(!valid_version(v), "{v:?}");
        }
    }

    #[test]
    fn extensions_accepted() {
        for e in ["xdebug", "pdo_mysql", "mbstring", "sqlite3", "a"] {
            assert!(valid_extension(e), "{e}");
        }
    }

    #[test]
    fn extensions_rejected() {
        let long = "a".repeat(MAX_NAME_LEN + 1);
        for e in [
            "",
            "XDebug",
            "x-debug",
            "x debug",
            "x;id",
            "x|y",
            "$(id)",
            "`id`",
            "x&y",
            "x>y",
            "../x",
            "x\0",
            "x\n",
            long.as_str(),
        ] {
            assert!(!valid_extension(e), "{e:?}");
        }
    }

    #[test]
    fn request_validation_rejects_bad_fields() {
        let bad = HelperRequest::EnableExtension {
            version: "8.3".into(),
            extension: "x;id".into(),
        };
        assert!(validate_request(&bad).is_err());
        let bad = HelperRequest::RestartFpm {
            version: "8.3; reboot".into(),
        };
        assert!(validate_request(&bad).is_err());
        let ok = HelperRequest::DisableExtension {
            version: "8.3".into(),
            extension: "xdebug".into(),
        };
        assert!(validate_request(&ok).is_ok());
    }

    #[test]
    fn ini_paths_accepted() {
        let root = temp_root("ok");
        for rel in [
            "8.3/cli/php.ini",
            "8.3/fpm/php.ini",
            "8.3/cli/conf.d/20-xdebug.ini",
            "8.3/fpm/conf.d/new.ini",
            "8.3/mods-available/xdebug.ini",
        ] {
            assert!(check(&root, rel).is_ok(), "{rel}: {:?}", check(&root, rel));
        }
    }

    #[test]
    fn ini_paths_rejected() {
        let root = temp_root("bad");
        let r = root.display().to_string();
        let cases = [
            "8.3/cli/php.ini/../../../../etc/passwd".to_string(),
            "8.3/cli/../../x/php.ini".to_string(),
            format!("{r}/8.3/cli/./php.ini"),
            "/etc/passwd".to_string(),
            "/etc/php/8.3/cli/php.ini".to_string(),
            "8.3/cli/php.ini\0.txt".to_string(),
            "8.3/cli/conf.d/x;id.ini".to_string(),
            "8.3/cli/conf.d/$(id).ini".to_string(),
            "8.3/cli/conf.d/a b.ini".to_string(),
            "8.3/cli/conf.d/x.sh".to_string(),
            "8.3/cli/conf.d/.ini".to_string(),
            "8.3/apache2/php.ini".to_string(),
            "8.3/cli/other.ini".to_string(),
            "8.3/php.ini".to_string(),
            "x.y/cli/php.ini".to_string(),
            "8.3/cli/conf.d/sub/x.ini".to_string(),
            "relative/8.3/cli/php.ini".to_string(),
            String::new(),
            format!("8.3/cli/conf.d/{}.ini", "a".repeat(5000)),
        ];
        for c in cases {
            assert!(check(&root, &c).is_err(), "{c:?}");
        }
        assert!(validate_ini_path("relative.ini", &root).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escapes_rejected() {
        use std::os::unix::fs::symlink;
        let root = temp_root("link");
        let outside = std::env::temp_dir().join(format!("vm-helper-out-{}", std::process::id()));
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("target.ini"), "x").unwrap();

        // File symlink pointing outside the root.
        symlink(
            outside.join("target.ini"),
            root.join("8.3/cli/conf.d/evil.ini"),
        )
        .unwrap();
        assert!(check(&root, "8.3/cli/conf.d/evil.ini").is_err());

        // Directory symlink pointing outside the root.
        symlink(&outside, root.join("8.3/fpm/conf.d2")).unwrap();
        fs::remove_dir_all(root.join("8.3/fpm/conf.d")).unwrap();
        symlink(&outside, root.join("8.3/fpm/conf.d")).unwrap();
        assert!(check(&root, "8.3/fpm/conf.d/new.ini").is_err());

        // File symlink that stays inside the root is allowed and resolved.
        symlink(
            root.join("8.3/mods-available/xdebug.ini"),
            root.join("8.3/cli/conf.d/20-xdebug.ini"),
        )
        .unwrap();
        fs::write(root.join("8.3/mods-available/xdebug.ini"), "x").unwrap();
        let resolved = check(&root, "8.3/cli/conf.d/20-xdebug.ini").unwrap();
        assert_eq!(resolved, root.join("8.3/mods-available/xdebug.ini"));
    }
}
