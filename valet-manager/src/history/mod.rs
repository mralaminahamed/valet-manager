use chrono::{DateTime, Local};
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HistoryEntry {
    pub id: i64,
    pub started_at: DateTime<Local>,
    pub command: String,        // e.g. "php"
    pub args: String,           // space-joined
    pub duration_ms: i64,
    pub exit_code: Option<i32>, // None = killed/no exit
    pub source: String,         // panel name or "creator"
    pub output_preview: String, // first 1000 chars
}

#[allow(dead_code)]
pub fn db_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/valet-manager/history.sqlite")
}

#[allow(dead_code)]
pub fn open() -> rusqlite::Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT NOT NULL,
            command TEXT NOT NULL,
            args TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            exit_code INTEGER,
            source TEXT NOT NULL,
            output_preview TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_history_started ON history(started_at DESC);
    "#,
    )?;
    Ok(conn)
}

#[allow(dead_code)]
pub fn record(
    conn: &Connection,
    started: DateTime<Local>,
    command: &str,
    args: &[&str],
    duration_ms: i64,
    exit_code: Option<i32>,
    source: &str,
    output_preview: &str,
) -> rusqlite::Result<()> {
    let preview = if output_preview.len() > 1000 {
        &output_preview[..1000]
    } else {
        output_preview
    };
    conn.execute(
        "INSERT INTO history (started_at, command, args, duration_ms, exit_code, source, output_preview)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            started.to_rfc3339(),
            command,
            args.join(" "),
            duration_ms,
            exit_code,
            source,
            preview
        ],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn list_recent(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, command, args, duration_ms, exit_code, source, output_preview
         FROM history ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |row| {
        let started_str: String = row.get(1)?;
        let started_at = DateTime::parse_from_rfc3339(&started_str)
            .map(|d| d.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());
        Ok(HistoryEntry {
            id: row.get(0)?,
            started_at,
            command: row.get(2)?,
            args: row.get(3)?,
            duration_ms: row.get(4)?,
            exit_code: row.get(5)?,
            source: row.get(6)?,
            output_preview: row.get(7)?,
        })
    })?;
    rows.collect()
}

#[allow(dead_code)]
pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, command, args, duration_ms, exit_code, source, output_preview
         FROM history WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map([id], |row| {
        let started_str: String = row.get(1)?;
        let started_at = DateTime::parse_from_rfc3339(&started_str)
            .map(|d| d.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());
        Ok(HistoryEntry {
            id: row.get(0)?,
            started_at,
            command: row.get(2)?,
            args: row.get(3)?,
            duration_ms: row.get(4)?,
            exit_code: row.get(5)?,
            source: row.get(6)?,
            output_preview: row.get(7)?,
        })
    })?;
    match rows.next() {
        Some(Ok(entry)) => Ok(Some(entry)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

#[allow(dead_code)]
pub fn ago(when: DateTime<Local>) -> String {
    let now = Local::now();
    let diff = now.signed_duration_since(when);
    let s = diff.num_seconds();
    if s < 60 {
        format!("{}s ago", s)
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86400)
    }
}

/// Wrapper used by panels that want history-tracked commands.
#[allow(dead_code)]
pub async fn run_tracked(
    cmd: &str,
    args: &[&str],
    cwd: Option<&std::path::PathBuf>,
    source: &str,
    tx: tokio::sync::mpsc::Sender<crate::creator::output_streamer::OutputLine>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<bool> {
    let started = chrono::Local::now();
    let t0 = std::time::Instant::now();
    let status =
        crate::creator::output_streamer::stream_command(cmd, args, cwd, tx, cancel_rx).await;
    let duration_ms = t0.elapsed().as_millis() as i64;
    let (exit, success) = match &status {
        Ok(s) => (s.code(), s.success()),
        Err(_) => (None, false),
    };
    if let Ok(conn) = open() {
        let _ = record(&conn, started, cmd, args, duration_ms, exit, source, "");
    }
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                command TEXT NOT NULL,
                args TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                exit_code INTEGER,
                source TEXT NOT NULL,
                output_preview TEXT NOT NULL DEFAULT ''
            );
        "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn record_and_list_roundtrips() {
        let conn = fresh_conn();
        record(
            &conn,
            Local::now(),
            "php",
            &["-v"],
            12,
            Some(0),
            "panel:test",
            "PHP 8.3",
        )
        .unwrap();
        let rows = list_recent(&conn, 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].command, "php");
        assert_eq!(rows[0].exit_code, Some(0));
        assert_eq!(rows[0].args, "-v");
    }

    #[test]
    fn list_recent_orders_desc_by_id() {
        let conn = fresh_conn();
        record(&conn, Local::now(), "a", &[], 1, Some(0), "s", "").unwrap();
        record(&conn, Local::now(), "b", &[], 1, Some(0), "s", "").unwrap();
        record(&conn, Local::now(), "c", &[], 1, Some(0), "s", "").unwrap();
        let rows = list_recent(&conn, 10).unwrap();
        assert_eq!(rows[0].command, "c");
        assert_eq!(rows[1].command, "b");
        assert_eq!(rows[2].command, "a");
    }

    #[test]
    fn find_by_id_returns_entry() {
        let conn = fresh_conn();
        record(&conn, Local::now(), "php", &["-v"], 12, Some(0), "panel:test", "out").unwrap();
        let entry = find_by_id(&conn, 1).unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().command, "php");
    }

    #[test]
    fn find_by_id_none_when_missing() {
        let conn = fresh_conn();
        let entry = find_by_id(&conn, 999).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn ago_seconds_format() {
        let when = Local::now() - chrono::Duration::seconds(45);
        let s = ago(when);
        assert!(s.contains("s ago"));
    }

    #[test]
    fn ago_minutes_format() {
        let when = Local::now() - chrono::Duration::minutes(5);
        let s = ago(when);
        assert!(s.contains("m ago"));
    }

    #[test]
    fn ago_hours_format() {
        let when = Local::now() - chrono::Duration::hours(2);
        let s = ago(when);
        assert!(s.contains("h ago"));
    }

    #[test]
    fn db_path_under_home() {
        let p = db_path();
        assert!(p.to_string_lossy().contains("valet-manager"));
        assert!(p.to_string_lossy().ends_with("history.sqlite"));
    }
}
