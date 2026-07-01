mod crash;
mod db;
mod files;
mod git;
mod terminal_engine;
mod watcher;

use db::{Database, Project, Session, SessionTemplate};
use terminal_engine::{PtyConfig, TerminalEngine};
use watcher::WatchManager;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

struct AppState {
    db: Arc<Database>,
    pty: Arc<Mutex<TerminalEngine>>,
    watcher: Arc<Mutex<WatchManager>>,
}

// ── Project commands ──

#[tauri::command]
fn create_project(state: State<AppState>, name: String, root_path: String) -> Result<Project, String> {
    state.db.insert_project(&name, &root_path)
}

#[tauri::command]
fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    state.db.get_all_projects()
}

#[tauri::command]
fn delete_project(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_project(id)
}

#[tauri::command]
fn rename_project(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    state.db.update_project_name(id, &name)
}

#[tauri::command]
fn reorder_projects(state: State<AppState>, ids: Vec<i64>) -> Result<(), String> {
    for (i, id) in ids.iter().enumerate() {
        state.db.update_project_sort(*id, i as i32)?;
    }
    Ok(())
}

// ── Session commands ──

#[tauri::command]
fn create_session(state: State<AppState>, project_id: i64, name: String, cwd: Option<String>, launch_command: Option<String>, launch_type: Option<String>) -> Result<Session, String> {
    state.db.insert_session(project_id, &name, cwd.as_deref(), launch_command.as_deref().unwrap_or(""), launch_type.as_deref().unwrap_or("shell"))
}

#[tauri::command]
fn list_sessions(state: State<AppState>, project_id: i64) -> Result<Vec<Session>, String> {
    state.db.get_sessions_for_project(project_id)
}

#[tauri::command]
fn delete_session_cmd(state: State<AppState>, id: i64, pty_id: String) -> Result<(), String> {
    state.pty.lock().map_err(|e| format!("Lock: {}", e))?.kill(&pty_id).ok();
    state.db.delete_session(id)
}

#[tauri::command]
fn rename_session(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    state.db.update_session_name(id, &name)
}

#[tauri::command]
fn update_session_cwd(state: State<AppState>, id: i64, cwd: String) -> Result<(), String> {
    state.db.update_session_cwd(id, &cwd)
}

// ── Session template commands ──

#[tauri::command]
fn create_template(state: State<AppState>, name: String, launch_command: String, icon: Option<String>) -> Result<SessionTemplate, String> {
    state.db.insert_template(&name, &launch_command, icon.as_deref().unwrap_or(""))
}

#[tauri::command]
fn list_templates(state: State<AppState>) -> Result<Vec<SessionTemplate>, String> {
    state.db.get_all_templates()
}

#[tauri::command]
fn update_template(state: State<AppState>, id: i64, name: String, launch_command: String, icon: Option<String>) -> Result<(), String> {
    state.db.update_template(id, &name, &launch_command, icon.as_deref().unwrap_or(""))
}

#[tauri::command]
fn delete_template(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_template(id)
}

// ── PTY commands ──

#[tauri::command]
fn pty_spawn(app: AppHandle, state: State<AppState>, session_id: String, shell: String, working_directory: String, cols: u16, rows: u16) -> Result<(), String> {
    let config = PtyConfig {
        shell,
        working_directory,
        cols,
        rows,
        env: HashMap::new(),
    };
    state.pty.lock().map_err(|e| format!("Lock: {}", e))?.spawn(&session_id, config, app)
}

#[tauri::command]
fn pty_write(state: State<AppState>, id: String, data: String) -> Result<(), String> {
    state.pty.lock().map_err(|e| format!("Lock: {}", e))?.write(&id, data.as_bytes())
}

#[tauri::command]
fn pty_resize(state: State<AppState>, id: String, cols: u16, rows: u16) -> Result<(), String> {
    state.pty.lock().map_err(|e| format!("Lock: {}", e))?.resize(&id, cols, rows)
}

#[tauri::command]
fn pty_kill(state: State<AppState>, id: String) -> Result<(), String> {
    state.pty.lock().map_err(|e| format!("Lock: {}", e))?.kill(&id)
}

#[tauri::command]
fn pty_list(state: State<AppState>) -> Result<Vec<String>, String> {
    Ok(state.pty.lock().map_err(|e| format!("Lock: {}", e))?.list_ids())
}

#[tauri::command]
fn detect_shell() -> String {
    terminal_engine::detect_shell()
}

// ── Settings commands ──

#[tauri::command]
fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    state.db.get_setting(&key)
}

#[tauri::command]
fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    state.db.set_setting(&key, &value)
}

#[tauri::command]
fn list_dir(path: String) -> Result<Vec<files::DirEntry>, String> {
    files::list_dir(&path)
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("cmd");
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd.args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn git_status(root_path: String) -> Result<git::GitStatus, String> {
    git::get_git_status(&root_path)
}

// ── Crash recovery ──

#[tauri::command]
fn check_crash() -> Result<Option<String>, String> {
    let dir = db::get_db_path().parent().unwrap().join("crashes");
    Ok(crash::check_previous_crash(&dir).map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
fn clear_crashes() -> Result<(), String> {
    let dir = db::get_db_path().parent().unwrap().join("crashes");
    crash::clear_crashes(&dir);
    Ok(())
}

// ── File watch commands ──

#[tauri::command]
fn start_watch(app: AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
    let mut watcher = state.watcher.lock().map_err(|e| format!("Lock: {}", e))?;
    watcher.start(app, path)
}

#[tauri::command]
fn stop_watch(state: State<AppState>) -> Result<(), String> {
    state.watcher.lock().map_err(|e| format!("Lock: {}", e))?.stop();
    Ok(())
}

#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let is_dir = p.is_dir();

    #[cfg(target_os = "windows")]
    {
        if is_dir {
            std::process::Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {}", e))?;
        } else {
            std::process::Command::new("explorer")
                .args(["/select,", &path])
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {}", e))?;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if is_dir {
            std::process::Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open: {}", e))?;
        } else {
            std::process::Command::new("open")
                .args(["-R", &path])
                .spawn()
                .map_err(|e| format!("Failed to open: {}", e))?;
        }
    }
    #[cfg(target_os = "linux")]
    {
        let target = if is_dir { &path } else { p.parent().and_then(|d| d.to_str()).unwrap_or(&path) };
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    Ok(())
}

// ── App entry point ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("[workbase] startup begin");
    env_logger::init();

    let db_path = db::get_db_path();
    eprintln!("[workbase] db_path: {:?}", db_path);
    log::info!("Database path: {:?}", db_path);

    // Initialize crash handler
    let crashes_dir = db_path.parent().unwrap().join("crashes");
    crash::init(crashes_dir);

    let database = Database::open(&db_path).expect("Failed to open database");
    eprintln!("[workbase] db opened");
    database.migrate().expect("Failed to run migrations");
    eprintln!("[workbase] migration done");

    let db = Arc::new(database);

    tauri::Builder::default()
        .setup(move |app| {
            let pty = TerminalEngine::new();
            let watch = WatchManager::new();
            app.manage(AppState {
                db: db.clone(),
                pty: Arc::new(Mutex::new(pty)),
                watcher: Arc::new(Mutex::new(watch)),
            });
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            create_project,
            list_projects,
            delete_project,
            rename_project,
            reorder_projects,
            create_session,
            list_sessions,
            delete_session_cmd,
            rename_session,
            update_session_cwd,
            create_template,
            list_templates,
            update_template,
            delete_template,
            pty_spawn,
            pty_write,
            pty_resize,
            pty_kill,
            pty_list,
            detect_shell,
            get_setting,
            set_setting,
            git_status,
            list_dir,
            open_file,
            open_in_explorer,
            start_watch,
            stop_watch,
            check_crash,
            clear_crashes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
