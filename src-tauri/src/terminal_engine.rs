use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

pub struct TerminalEngine {
    sessions: HashMap<String, Arc<Mutex<TermSession>>>,
    pty_system: NativePtySystem,
}

struct TermSession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn std::io::Write + Send>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PtyConfig {
    pub shell: String,
    pub working_directory: String,
    pub cols: u16,
    pub rows: u16,
    pub env: HashMap<String, String>,
}

impl TerminalEngine {
    pub fn new() -> Self {
        TerminalEngine {
            sessions: HashMap::new(),
            pty_system: NativePtySystem::default(),
        }
    }

    pub fn spawn(
        &mut self,
        session_id: &str,
        config: PtyConfig,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let id = session_id.to_string();
        eprintln!(
            "[workbase] pty_spawn: {} shell={} cwd={}",
            id, config.shell, config.working_directory
        );

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
        cmd.cwd(&config.working_directory);

        let shell_lower = config.shell.to_lowercase();
        if shell_lower.ends_with("powershell.exe") || shell_lower.ends_with("pwsh.exe") {
            cmd.arg("-NoLogo");
        }

        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let child: Box<dyn portable_pty::Child + Send + Sync> = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let master = pair.master;
        let writer = master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let session = Arc::new(Mutex::new(TermSession {
            master,
            writer,
            child: Some(child),
        }));

        self.sessions.insert(id.clone(), session.clone());

        let reader = {
            let s = session.lock().map_err(|e| format!("Lock: {}", e))?;
            Box::new(
                s.master
                    .try_clone_reader()
                    .map_err(|e| format!("Failed to clone reader: {}", e))?,
            )
        };

        std::thread::spawn(move || {
            Self::reader_loop(id, reader, session, app);
        });

        Ok(())
    }

    fn reader_loop(
        id: String,
        mut reader: Box<dyn Read + Send>,
        _session: Arc<Mutex<TermSession>>,
        app: AppHandle,
    ) {
        let mut buf = [0u8; 4096];
        let mut batch: Vec<u8> = Vec::with_capacity(32_768);
        eprintln!("[workbase] reader_loop start: {}", id);

        let flush_batch = |batch: &mut Vec<u8>, app: &AppHandle| {
            if !batch.is_empty() {
                if let Some(cwd) = extract_osc7(batch) {
                    let event = format!("pty:cwd:{}", id);
                    let _ = app.emit(event.as_str(), cwd);
                }
                let event = format!("pty:data:{}", id);
                let text = String::from_utf8_lossy(batch).into_owned();
                let _ = app.emit(event.as_str(), text);
                batch.clear();
            }
        };

        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    eprintln!("[workbase] pty exit (EOF): {}", id);
                    flush_batch(&mut batch, &app);
                    let event = format!("pty:exit:{}", id);
                    let _ = app.emit(event.as_str(), 0_i32);
                    log::info!("PTY {}: process exited (EOF)", id);
                    break;
                }
                Ok(n) => {
                    batch.extend_from_slice(&buf[..n]);
                    // Flush immediately for interactive response (small reads).
                    // For TUI bursts, accumulate up to the size cap below.
                    if batch.len() < 8192 {
                        flush_batch(&mut batch, &app);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if !batch.is_empty() {
                        flush_batch(&mut batch, &app);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(e) => {
                    eprintln!("[workbase] pty read error: {} err={}", id, e);
                    log::error!("PTY {}: read error: {}", id, e);
                    flush_batch(&mut batch, &app);
                    let event = format!("pty:exit:{}", id);
                    let _ = app.emit(event.as_str(), 1_i32);
                    break;
                }
            }

            // Size cap: prevent unbounded batch growth during heavy TUI output
            if batch.len() >= 65_536 {
                flush_batch(&mut batch, &app);
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

// ── OSC 7 parser ──

fn extract_osc7(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    let pat = "\x1b]7;file://";
    if let Some(start) = text.find(pat) {
        let after = &text[start + pat.len()..];
        if let Some(path_start) = after.find('/') {
            let path = &after[path_start..];
            let end = path
                .find('\x07')
                .or_else(|| path.find("\x1b\\"))
                .unwrap_or(path.len());
            let raw = &path[..end];
            if let Ok(decoded) = url_decode(raw) {
                return Some(decoded);
            }
        }
    }
    None
}

fn url_decode(s: &str) -> Result<String, ()> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(b) = u8::from_str_radix(&hex, 16) {
                        out.push(b as char);
                    } else {
                        out.push('%');
                        out.push_str(&hex);
                    }
                } else {
                    out.push('%');
                    out.push_str(&hex);
                }
            }
            '+' => out.push(' '),
            other => out.push(other),
        }
    }
    Ok(out)
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
