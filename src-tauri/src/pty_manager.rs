use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty, Child};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct PtyManager {
    sessions: HashMap<String, Arc<Mutex<PtySession>>>,
    app: AppHandle,
    pty_system: NativePtySystem,
}

struct PtySession {
    id: String,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Option<Box<dyn Child + Send + Sync>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PtyConfig {
    pub shell: String,
    pub working_directory: String,
    pub cols: u16,
    pub rows: u16,
    pub env: HashMap<String, String>,
}

impl PtyManager {
    pub fn new(app: AppHandle) -> Self {
        PtyManager {
            sessions: HashMap::new(),
            app,
            pty_system: NativePtySystem::default(),
        }
    }

    pub fn spawn(&mut self, session_id: &str, config: PtyConfig) -> Result<(), String> {
        let id = session_id.to_string();

        let pty_size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = self
            .pty_system
            .openpty(pty_size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(&config.shell);
        cmd.cwd(config.working_directory);

        // Suppress PowerShell startup banner
        let shell_lower = config.shell.to_lowercase();
        if shell_lower.ends_with("powershell.exe") || shell_lower.ends_with("pwsh.exe") {
            cmd.arg("-NoLogo");
        }

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let child: Box<dyn Child + Send + Sync> = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let master = pair.master;
        let writer = master.take_writer().map_err(|e| format!("Failed to take writer: {}", e))?;

        let session = Arc::new(Mutex::new(PtySession {
            id: id.clone(),
            master,
            writer,
            child: Some(child),
        }));

        self.sessions.insert(id.clone(), session.clone());

        // Start the reader thread using try_clone_reader
        let reader = {
            let s = session.lock().map_err(|e| format!("Lock: {}", e))?;
            s.master.try_clone_reader().map_err(|e| format!("Failed to clone reader: {}", e))?
        };

        let app = self.app.clone();
        std::thread::spawn(move || {
            Self::reader_loop(id, reader, app);
        });

        Ok(())
    }

    fn reader_loop(id: String, mut reader: Box<dyn Read + Send>, app: AppHandle) {
        let mut buf = [0u8; 4096];

        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = app.emit(&format!("pty:exit:{}", id), 0_i32);
                    log::info!("PTY {}: process exited (EOF)", id);
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app.emit(&format!("pty:data:{}", id), data);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => {
                    log::error!("PTY {}: read error: {}", id, e);
                    let _ = app.emit(&format!("pty:exit:{}", id), 1_i32);
                    break;
                }
            }
        }
    }

    pub fn write(&mut self, id: &str, data: &[u8]) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("PTY session not found: {}", id))?;

        let mut s = session.lock().map_err(|e| format!("Lock error: {}", e))?;
        s.writer
            .write_all(data)
            .map_err(|e| format!("Write error: {}", e))
    }

    pub fn resize(&mut self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("PTY session not found: {}", id))?;

        let s = session.lock().map_err(|e| format!("Lock error: {}", e))?;
        s.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize error: {}", e))
    }

    pub fn kill(&mut self, id: &str) -> Result<(), String> {
        if let Some(session) = self.sessions.remove(id) {
            let mut s = session.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(mut child) = s.child.take() {
                child.kill().ok();
                let _ = child.wait();
            }
            log::info!("PTY {}: killed", id);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn kill_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            self.kill(&id).ok();
        }
    }

    pub fn list_ids(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }
}

/// Detect the user's default shell
pub fn detect_shell() -> String {
    if cfg!(target_os = "windows") {
        for shell in &["pwsh.exe", "powershell.exe", "cmd.exe"] {
            if let Ok(path) = which::which(shell) {
                return path.to_string_lossy().to_string();
            }
        }
        "cmd.exe".to_string()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| {
            for shell in &["/bin/zsh", "/bin/bash", "/bin/sh"] {
                if PathBuf::from(shell).exists() {
                    return shell.to_string();
                }
            }
            "/bin/sh".to_string()
        })
    }
}
