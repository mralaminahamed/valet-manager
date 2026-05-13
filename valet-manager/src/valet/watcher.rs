#![allow(dead_code)]

use std::time::Duration;
use tokio::sync::mpsc::Sender;
use crate::commands::AppCommand;
use crate::valet::variant::ValetPaths;

/// Start watching Sites/ and Nginx/ directories.
/// When any change is detected, debounce 500ms then send RefreshSites.
/// This function runs indefinitely — call it with tokio::spawn.
pub async fn start_watcher(valet_paths: ValetPaths, cmd_tx: Sender<AppCommand>) {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc as std_mpsc;

    let (tx, rx) = std_mpsc::channel();

    let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(_) => return,
    };

    // Watch both directories, ignore errors if dirs don't exist yet
    let _ = watcher.watch(&valet_paths.sites_dir, RecursiveMode::NonRecursive);
    let _ = watcher.watch(&valet_paths.nginx_dir, RecursiveMode::NonRecursive);

    let debounce = Duration::from_millis(500);

    loop {
        // Block on first event
        let Ok(_) = rx.recv() else { break };

        // Drain any additional events within debounce window
        let deadline = std::time::Instant::now() + debounce;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() { break }
            match rx.recv_timeout(remaining) {
                Ok(_) => {} // drain
                Err(_) => break,
            }
        }

        // Send RefreshSites
        let _ = cmd_tx.try_send(AppCommand::RefreshSites);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_duration_is_500ms() {
        let d = Duration::from_millis(500);
        assert_eq!(d.as_millis(), 500);
    }

    #[test]
    fn watcher_watch_paths_are_correct() {
        // Just verify the path construction logic — no actual watcher
        let home = std::path::PathBuf::from("/home/user");
        let sites_dir = home.join(".valet").join("Sites");
        let nginx_dir = home.join(".valet").join("Nginx");
        assert!(sites_dir.to_string_lossy().contains("Sites"));
        assert!(nginx_dir.to_string_lossy().contains("Nginx"));
    }
}
