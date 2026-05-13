use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OutputLine {
    pub text: String,
    pub stream: Stream,
    pub timestamp: DateTime<Local>,
}

/// Token used to cancel a running command. Drop or send to cancel.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CancelToken(pub tokio::sync::watch::Sender<bool>);

impl CancelToken {
    #[allow(dead_code)]
    pub fn new() -> (Self, tokio::sync::watch::Receiver<bool>) {
        let (tx, rx) = tokio::sync::watch::channel(false);
        (CancelToken(tx), rx)
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        let _ = self.0.send(true);
    }
}

/// Stream stdout/stderr lines from a child process to `tx`.
/// Returns the process exit status, or an IO error if spawning/waiting fails.
/// If `cancel_rx` receives `true`, sends SIGTERM to the child.
#[allow(dead_code)]
pub async fn stream_command(
    cmd: &str,
    args: &[&str],
    cwd: Option<&PathBuf>,
    tx: Sender<OutputLine>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut child = {
        let mut command = Command::new(cmd);
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        command
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?
    };

    let stdout = child.stdout.take().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "failed to pipe stdout")
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "failed to pipe stderr")
    })?;

    let tx_stdout = tx.clone();
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let output = OutputLine {
                text: line,
                stream: Stream::Stdout,
                timestamp: Local::now(),
            };
            if tx_stdout.send(output).await.is_err() {
                break;
            }
        }
    });

    let tx_stderr = tx.clone();
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let output = OutputLine {
                text: line,
                stream: Stream::Stderr,
                timestamp: Local::now(),
            };
            if tx_stderr.send(output).await.is_err() {
                break;
            }
        }
    });

    let exit_status = loop {
        tokio::select! {
            result = child.wait() => {
                break result?;
            }
            changed = cancel_rx.changed() => {
                if changed.is_ok() && *cancel_rx.borrow() {
                    if let Some(pid) = child.id() {
                        let _ = nix::sys::signal::kill(
                            nix::unistd::Pid::from_raw(pid as i32),
                            nix::sys::signal::Signal::SIGTERM,
                        );
                    }
                    break child.wait().await?;
                }
            }
        }
    };

    // Ensure tasks finish
    let _ = tokio::join!(stdout_task, stderr_task);

    Ok(exit_status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_line_has_correct_fields() {
        let line = OutputLine {
            text: "hello".to_string(),
            stream: Stream::Stdout,
            timestamp: chrono::Local::now(),
        };
        assert_eq!(line.text, "hello");
        assert_eq!(line.stream, Stream::Stdout);
    }

    #[test]
    fn stream_enum_variants() {
        assert_ne!(Stream::Stdout, Stream::Stderr);
    }

    #[test]
    fn cancel_token_new_creates_pair() {
        let (token, rx) = CancelToken::new();
        assert!(!*rx.borrow());
        token.cancel();
        // After cancel, receiver should see true
        assert!(*rx.borrow());
    }

    #[tokio::test]
    async fn stream_command_captures_stdout() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let (_, cancel_rx) = tokio::sync::watch::channel(false);
        let status = stream_command("echo", &["hello world"], None, tx, cancel_rx)
            .await
            .expect("stream_command failed");
        assert!(status.success());
        let line = rx.try_recv().unwrap();
        assert!(line.text.contains("hello world"));
        assert_eq!(line.stream, Stream::Stdout);
    }
}
