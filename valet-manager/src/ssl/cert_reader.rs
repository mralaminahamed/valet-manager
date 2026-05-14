use std::path::{Path, PathBuf};
use chrono::{Local, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CertStatus { Ok, Warning, Critical, Expired }

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CertInfo {
    pub domain: String,
    pub issuer: String,
    pub expires_at: NaiveDate,
    pub days_remaining: i64,
    pub status: CertStatus,
    pub pem_path: PathBuf,
}

pub fn status_from_days(days: i64) -> CertStatus {
    if days < 0       { CertStatus::Expired }
    else if days < 7  { CertStatus::Critical }
    else if days < 30 { CertStatus::Warning }
    else              { CertStatus::Ok }
}

/// Parse `openssl x509 -enddate -noout -in {path}` output line "notAfter=Jan 12 23:59:59 2026 GMT"
fn parse_enddate_line(s: &str) -> Option<NaiveDate> {
    let stripped = s.trim().trim_start_matches("notAfter=").trim();
    // chrono parse: "%b %e %H:%M:%S %Y %Z"
    let naive = chrono::NaiveDateTime::parse_from_str(stripped, "%b %e %H:%M:%S %Y %Z").ok()?;
    Some(naive.date())
}

fn parse_issuer_line(s: &str) -> String {
    let stripped = s.trim().trim_start_matches("issuer=").trim();
    // shorten CN= portion (accept "CN=value" or "CN = value")
    for part in stripped.split(',') {
        let p = part.trim();
        let cn_val = p
            .strip_prefix("CN=")
            .or_else(|| p.strip_prefix("CN ="))
            .or_else(|| {
                p.strip_prefix("CN").and_then(|rest| {
                    let r = rest.trim_start();
                    r.strip_prefix('=').map(|x| x.trim_start())
                })
            });
        if let Some(cn) = cn_val {
            return cn.trim().to_string();
        }
    }
    stripped.to_string()
}

#[allow(dead_code)]
pub async fn read_cert(domain: &str, pem_path: &Path) -> anyhow::Result<CertInfo> {
    let path_s = pem_path.to_string_lossy().to_string();
    let enddate = tokio::process::Command::new("openssl")
        .args(["x509", "-enddate", "-noout", "-in", &path_s])
        .output().await?;
    let issuer = tokio::process::Command::new("openssl")
        .args(["x509", "-issuer", "-noout", "-in", &path_s])
        .output().await?;
    let enddate_s = String::from_utf8_lossy(&enddate.stdout);
    let issuer_s = String::from_utf8_lossy(&issuer.stdout);
    let expires_at = parse_enddate_line(&enddate_s).ok_or_else(|| anyhow::anyhow!("cannot parse notAfter"))?;
    let issuer_str = parse_issuer_line(&issuer_s);
    let today = Local::now().date_naive();
    let days = expires_at.signed_duration_since(today).num_days();
    Ok(CertInfo {
        domain: domain.to_string(),
        issuer: issuer_str,
        expires_at,
        days_remaining: days,
        status: status_from_days(days),
        pem_path: pem_path.to_path_buf(),
    })
}

/// Scan a directory of *.crt files. Domain inferred from filename stem.
#[allow(dead_code)]
pub async fn scan(certificates_dir: &Path) -> Vec<CertInfo> {
    let mut out = Vec::new();
    let Ok(mut rd) = tokio::fs::read_dir(certificates_dir).await else { return out; };
    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "crt" || ext == "pem" {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if stem.is_empty() { continue; }
            if let Ok(info) = read_cert(&stem, &path).await {
                out.push(info);
            }
        }
    }
    out
}

/// Sort by status priority: Critical, Warning, Ok, Expired.
#[allow(dead_code)]
pub fn sort_by_status_priority(certs: &mut Vec<CertInfo>) {
    fn rank(s: CertStatus) -> i32 {
        match s {
            CertStatus::Critical => 0,
            CertStatus::Warning  => 1,
            CertStatus::Ok       => 2,
            CertStatus::Expired  => 3,
        }
    }
    certs.sort_by(|a, b| rank(a.status).cmp(&rank(b.status)).then(a.days_remaining.cmp(&b.days_remaining)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_thresholds() {
        assert_eq!(status_from_days(-1), CertStatus::Expired);
        assert_eq!(status_from_days(0),  CertStatus::Critical);
        assert_eq!(status_from_days(6),  CertStatus::Critical);
        assert_eq!(status_from_days(7),  CertStatus::Warning);
        assert_eq!(status_from_days(29), CertStatus::Warning);
        assert_eq!(status_from_days(30), CertStatus::Ok);
        assert_eq!(status_from_days(365), CertStatus::Ok);
    }
    #[test]
    fn parses_enddate_line() {
        let d = parse_enddate_line("notAfter=Jan 12 23:59:59 2026 GMT").unwrap();
        assert_eq!(d.to_string(), "2026-01-12");
    }
    #[test]
    fn parses_issuer_cn() {
        assert_eq!(parse_issuer_line("issuer=O = Valet+ CA, CN = Valet+ CA Self Signed"), "Valet+ CA Self Signed");
        // No CN — keep the rest
        let s = parse_issuer_line("issuer=O = Anonymous");
        assert!(s.contains("Anonymous"));
    }
    #[test]
    fn sort_priority() {
        fn mk(name: &str, status: CertStatus, days: i64) -> CertInfo {
            CertInfo { domain: name.into(), issuer: "x".into(), expires_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), days_remaining: days, status, pem_path: PathBuf::new() }
        }
        let mut v = vec![
            mk("ok", CertStatus::Ok, 100),
            mk("crit", CertStatus::Critical, 2),
            mk("expired", CertStatus::Expired, -10),
            mk("warn", CertStatus::Warning, 20),
        ];
        sort_by_status_priority(&mut v);
        assert_eq!(v[0].domain, "crit");
        assert_eq!(v[1].domain, "warn");
        assert_eq!(v[2].domain, "ok");
        assert_eq!(v[3].domain, "expired");
    }
}
