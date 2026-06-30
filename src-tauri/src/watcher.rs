use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use tauri::{AppHandle, Emitter};

pub struct WatchManager {
    _watcher: Option<notify::RecommendedWatcher>,
    _tx: Option<mpsc::Sender<()>>,
}

impl WatchManager {
    pub fn new() -> Self {
        WatchManager { _watcher: None, _tx: None }
    }

    pub fn start(&mut self, app: AppHandle, path: String) -> Result<(), String> {
        self.stop();

        let (tx, rx) = mpsc::channel::<()>();
        let tx_watch = tx.clone();
        let app_clone = app.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            let event = match res {
                Ok(e) => e,
                Err(err) => { eprintln!("[watch] error: {}", err); return; }
            };
            // Filter out .git, node_modules, and common sync/temp file churn
            let has_fs_change = event.paths.iter().any(|p| {
                let s = p.to_string_lossy();
                if s.contains("/.git/") || s.contains("\\.git\\") || s.ends_with("/.git") || s.ends_with("\\.git") {
                    return false;
                }
                if s.contains("/node_modules/") || s.contains("\\node_modules\\")
                    || s.contains("/target/") || s.contains("\\target\\")
                {
                    return false;
                }
                let lower = s.to_lowercase();
                if lower.contains(".tmp")
                    || lower.contains("~$")
                    || lower.contains(".odtmp")
                    || lower.ends_with("desktop.ini")
                    || lower.contains("\\.sync\\")
                    || lower.contains("/.sync/")
                {
                    return false;
                }
                true
            });
            if !has_fs_change { return; }
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    let _ = tx_watch.send(());
                }
                _ => {}
            }
        })
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        let watch_path = PathBuf::from(&path);
        watcher
            .watch(&watch_path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch: {}", e))?;

        let path_for_log = path.clone();
        std::thread::spawn(move || {
            loop {
                if rx.recv().is_err() { break; }
                let mut quiet = false;
                while !quiet {
                    match rx.recv_timeout(std::time::Duration::from_millis(1500)) {
                        Ok(()) => {}
                        Err(_) => quiet = true,
                    }
                }
                let _ = app_clone.emit("file-changed", &path_for_log);
            }
        });

        self._watcher = Some(watcher);
        self._tx = Some(tx);
        Ok(())
    }

    pub fn stop(&mut self) {
        self._tx = None;
        self._watcher = None;
    }
}
