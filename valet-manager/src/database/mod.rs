use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DbEngine { MySql, PostgreSql, Sqlite }

impl DbEngine {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MySql => "MySQL",
            Self::PostgreSql => "PostgreSQL",
            Self::Sqlite => "SQLite",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DbDatabase {
    pub name: String,
    pub table_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DbTable {
    pub name: String,
    pub rows: u64,
    pub engine: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DbCredentials {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl Default for DbCredentials {
    fn default() -> Self { Self { host: "127.0.0.1".into(), port: 3306, user: "root".into(), password: "".into() } }
}

/// Format a u64 with commas: 12450 -> "12,450"
#[allow(dead_code)]
pub fn format_thousands(n: u64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(*c);
    }
    out.chars().rev().collect()
}

/// Use the `mysql` command-line client to list databases.
/// Returns an empty vec if the binary is unavailable — UI handles empty state.
#[allow(dead_code)]
pub async fn list_databases(creds: &DbCredentials) -> anyhow::Result<Vec<DbDatabase>> {
    let out = tokio::process::Command::new("mysql")
        .args(["-h", &creds.host, "-P", &creds.port.to_string(), "-u", &creds.user, "-N", "-e", "SHOW DATABASES;"])
        .env("MYSQL_PWD", &creds.password)
        .output().await;
    let stdout = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return Ok(Vec::new()),
    };
    let names: Vec<String> = stdout.lines()
        .map(|l| l.trim().to_string())
        .filter(|n| !n.is_empty() && !matches!(n.as_str(), "information_schema" | "performance_schema" | "mysql" | "sys"))
        .collect();
    let mut dbs = Vec::with_capacity(names.len());
    for name in names {
        // Get table count quickly (best-effort)
        let tc_out = tokio::process::Command::new("mysql")
            .args(["-h", &creds.host, "-P", &creds.port.to_string(), "-u", &creds.user, "-N", "-e",
                   &format!("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{}';", name.replace('\'', ""))])
            .env("MYSQL_PWD", &creds.password)
            .output().await;
        let table_count = tc_out.ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().ok()).unwrap_or(0);
        dbs.push(DbDatabase { name, table_count });
    }
    Ok(dbs)
}

#[allow(dead_code)]
pub async fn list_tables(creds: &DbCredentials, database: &str) -> anyhow::Result<Vec<DbTable>> {
    let query = format!("SELECT TABLE_NAME, IFNULL(TABLE_ROWS,0), IFNULL(ENGINE,'') FROM information_schema.tables WHERE table_schema='{}' ORDER BY TABLE_NAME;", database.replace('\'', ""));
    let out = tokio::process::Command::new("mysql")
        .args(["-h", &creds.host, "-P", &creds.port.to_string(), "-u", &creds.user, "-N", "-B", "-e", &query])
        .env("MYSQL_PWD", &creds.password)
        .output().await;
    let stdout = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return Ok(Vec::new()),
    };
    let mut tables = Vec::new();
    for line in stdout.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 3 {
            tables.push(DbTable {
                name: cols[0].to_string(),
                rows: cols[1].parse().unwrap_or(0),
                engine: cols[2].to_string(),
            });
        }
    }
    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_thousands_basic() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(42), "42");
        assert_eq!(format_thousands(1234), "1,234");
        assert_eq!(format_thousands(12450), "12,450");
        assert_eq!(format_thousands(1_000_000), "1,000,000");
    }
    #[test]
    fn db_engine_variants() {
        assert_ne!(DbEngine::MySql, DbEngine::PostgreSql);
        assert_ne!(DbEngine::MySql, DbEngine::Sqlite);
    }
    #[test]
    fn default_credentials_are_local_mysql() {
        let c = DbCredentials::default();
        assert_eq!(c.host, "127.0.0.1");
        assert_eq!(c.port, 3306);
        assert_eq!(c.user, "root");
    }
    #[test]
    fn db_engine_display_name() {
        assert_eq!(DbEngine::MySql.display_name(), "MySQL");
        assert_eq!(DbEngine::PostgreSql.display_name(), "PostgreSQL");
        assert_eq!(DbEngine::Sqlite.display_name(), "SQLite");
    }
}
