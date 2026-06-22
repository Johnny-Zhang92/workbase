use rusqlite::{Connection, Error as RusqliteError, Result as SqliteResult, params};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub root_path: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub cwd: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub last_active_at: String,
    pub launch_command: String,
    pub launch_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionTemplate {
    pub id: i64,
    pub name: String,
    pub launch_command: String,
    pub icon: String,
    pub sort_order: i32,
    pub created_at: String,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from(".")))
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {}", e))?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS _meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT OR IGNORE INTO _meta (key, value) VALUES ('schema_version', '1');

            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_projects_sort ON projects(sort_order);

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                cwd TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_active_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_id);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .map_err(|e| format!("Migration v1 failed: {}", e))?;

        // Migration v2: session launch_command/launch_type + session_templates
        let version: String = conn
            .query_row(
                "SELECT value FROM _meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "1".to_string());

        if version == "1" {
            conn.execute_batch(
                "
                ALTER TABLE sessions ADD COLUMN launch_command TEXT DEFAULT '';
                ALTER TABLE sessions ADD COLUMN launch_type TEXT DEFAULT 'shell';

                CREATE TABLE IF NOT EXISTS session_templates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    launch_command TEXT NOT NULL,
                    icon TEXT DEFAULT '',
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                UPDATE _meta SET value = '2' WHERE key = 'schema_version';
                ",
            )
            .map_err(|e| format!("Migration v2 failed: {}", e))?;
        }

        Ok(())
    }

    // ── Projects ──

    pub fn insert_project(&self, name: &str, root_path: &str) -> Result<Project, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO projects (name, root_path) VALUES (?1, ?2)",
            params![name, root_path],
        )
        .map_err(|e| format!("Insert project failed: {}", e))?;

        let id = conn.last_insert_rowid();
        // Inline query to avoid deadlock (don't call self.get_project while holding lock)
        conn.query_row(
            "SELECT id, name, root_path, sort_order, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("Get inserted project failed: {}", e))
    }

    pub fn get_all_projects(&self) -> Result<Vec<Project>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, root_path, sort_order, created_at, updated_at FROM projects ORDER BY sort_order, created_at")
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query projects failed: {}", e))?
            .collect::<SqliteResult<Vec<Project>>>()
            .map_err(|e| format!("Collect projects failed: {}", e))?;

        Ok(projects)
    }

    pub fn get_project(&self, id: i64) -> Result<Project, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.query_row(
            "SELECT id, name, root_path, sort_order, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("Get project failed: {}", e))
    }

    pub fn delete_project(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Delete associated sessions first (cascade handles this, but be explicit)
        conn.execute("DELETE FROM sessions WHERE project_id = ?1", params![id])
            .map_err(|e| format!("Delete sessions failed: {}", e))?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete project failed: {}", e))?;
        Ok(())
    }

    pub fn update_project_name(&self, id: i64, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, id],
        )
        .map_err(|e| format!("Update project failed: {}", e))?;
        Ok(())
    }

    pub fn update_project_sort(&self, id: i64, sort_order: i32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE projects SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![sort_order, id],
        )
        .map_err(|e| format!("Update project sort failed: {}", e))?;
        Ok(())
    }

    // ── Sessions ──

    pub fn insert_session(&self, project_id: i64, name: &str, cwd: Option<&str>, launch_command: &str, launch_type: &str) -> Result<Session, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO sessions (project_id, name, cwd, launch_command, launch_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![project_id, name, cwd, launch_command, launch_type],
        )
        .map_err(|e| format!("Insert session failed: {}", e))?;

        let id = conn.last_insert_rowid();
        // Inline query to avoid deadlock
        conn.query_row(
            "SELECT id, project_id, name, cwd, sort_order, created_at, last_active_at, launch_command, launch_type FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    cwd: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                    last_active_at: row.get(6)?,
                    launch_command: row.get(7)?,
                    launch_type: row.get(8)?,
                })
            },
        )
        .map_err(|e| format!("Get inserted session failed: {}", e))
    }

    pub fn get_session(&self, id: i64) -> Result<Session, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.query_row(
            "SELECT id, project_id, name, cwd, sort_order, created_at, last_active_at, launch_command, launch_type FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    cwd: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                    last_active_at: row.get(6)?,
                    launch_command: row.get(7)?,
                    launch_type: row.get(8)?,
                })
            },
        )
        .map_err(|e| format!("Get session failed: {}", e))
    }

    pub fn get_sessions_for_project(&self, project_id: i64) -> Result<Vec<Session>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, name, cwd, sort_order, created_at, last_active_at, launch_command, launch_type FROM sessions WHERE project_id = ?1 ORDER BY sort_order, created_at")
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    cwd: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                    last_active_at: row.get(6)?,
                    launch_command: row.get(7)?,
                    launch_type: row.get(8)?,
                })
            })
            .map_err(|e| format!("Query sessions failed: {}", e))?
            .collect::<SqliteResult<Vec<Session>>>()
            .map_err(|e| format!("Collect sessions failed: {}", e))?;

        Ok(sessions)
    }

    pub fn delete_session(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete session failed: {}", e))?;
        Ok(())
    }

    pub fn update_session_name(&self, id: i64, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET name = ?1 WHERE id = ?2",
            params![name, id],
        )
        .map_err(|e| format!("Update session name failed: {}", e))?;
        Ok(())
    }

    pub fn update_session_cwd(&self, id: i64, cwd: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET cwd = ?1, last_active_at = datetime('now') WHERE id = ?2",
            params![cwd, id],
        )
        .map_err(|e| format!("Update session cwd failed: {}", e))?;
        Ok(())
    }

    pub fn update_session_active(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET last_active_at = datetime('now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Update session active failed: {}", e))?;
        Ok(())
    }

    // ── Session Templates ──

    pub fn insert_template(&self, name: &str, launch_command: &str, icon: &str) -> Result<SessionTemplate, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Get next sort_order
        let max_order: i32 = conn
            .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM session_templates", [], |row| row.get(0))
            .unwrap_or(-1);
        conn.execute(
            "INSERT INTO session_templates (name, launch_command, icon, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![name, launch_command, icon, max_order + 1],
        )
        .map_err(|e| format!("Insert template failed: {}", e))?;

        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, name, launch_command, icon, sort_order, created_at FROM session_templates WHERE id = ?1",
            params![id],
            |row| {
                Ok(SessionTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    launch_command: row.get(2)?,
                    icon: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("Get inserted template failed: {}", e))
    }

    pub fn get_all_templates(&self) -> Result<Vec<SessionTemplate>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, launch_command, icon, sort_order, created_at FROM session_templates ORDER BY sort_order, created_at")
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let templates = stmt
            .query_map([], |row| {
                Ok(SessionTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    launch_command: row.get(2)?,
                    icon: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query templates failed: {}", e))?
            .collect::<SqliteResult<Vec<SessionTemplate>>>()
            .map_err(|e| format!("Collect templates failed: {}", e))?;

        Ok(templates)
    }

    pub fn update_template(&self, id: i64, name: &str, launch_command: &str, icon: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE session_templates SET name = ?1, launch_command = ?2, icon = ?3 WHERE id = ?4",
            params![name, launch_command, icon, id],
        )
        .map_err(|e| format!("Update template failed: {}", e))?;
        Ok(())
    }

    pub fn delete_template(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM session_templates WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete template failed: {}", e))?;
        Ok(())
    }

    // ── Settings ──

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(RusqliteError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Get setting failed: {}", e)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
            params![key, value],
        )
        .map_err(|e| format!("Set setting failed: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup() -> Database {
        let dir = std::env::temp_dir().join("devterm_test");
        std::fs::create_dir_all(&dir).ok();
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = dir.join(format!("test_{}_{}.db", std::process::id(), n));
        let _ = std::fs::remove_file(&path); // clean slate
        let db = Database::open(&path).expect("open test db");
        db.migrate().expect("migrate test db");
        db
    }

    #[test]
    fn test_open_and_migrate() {
        let db = setup();
        // Verify tables exist by querying them
        let conn = db.conn.lock().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"_meta".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_insert_and_get_project() {
        let db = setup();
        let p = db.insert_project("Test Project", "/home/user/test").unwrap();
        assert_eq!(p.name, "Test Project");
        assert_eq!(p.root_path, "/home/user/test");
        assert!(p.id > 0);

        let fetched = db.get_project(p.id).unwrap();
        assert_eq!(fetched.name, p.name);
        assert_eq!(fetched.root_path, p.root_path);
    }

    #[test]
    fn test_get_all_projects_order() {
        let db = setup();
        db.insert_project("B", "/b").unwrap();
        db.insert_project("A", "/a").unwrap();
        db.insert_project("C", "/c").unwrap();

        let projects = db.get_all_projects().unwrap();
        let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
        // Default sort_order is 0 for all, so they should be ordered by created_at
        assert_eq!(names, vec!["B", "A", "C"]);
    }

    #[test]
    fn test_get_nonexistent_project() {
        let db = setup();
        let result = db.get_project(99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_project_name() {
        let db = setup();
        let p = db.insert_project("Old Name", "/test").unwrap();
        db.update_project_name(p.id, "New Name").unwrap();
        let updated = db.get_project(p.id).unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_update_project_sort() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        db.update_project_sort(p.id, 42).unwrap();
        let updated = db.get_project(p.id).unwrap();
        assert_eq!(updated.sort_order, 42);
    }

    #[test]
    fn test_delete_project_cascades_sessions() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "S", Some("/p"), "", "shell").unwrap();
        assert!(db.get_session(s.id).is_ok());

        db.delete_project(p.id).unwrap();
        assert!(db.get_project(p.id).is_err());
        assert!(db.get_session(s.id).is_err());
    }

    #[test]
    fn test_insert_and_get_session() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "Session 1", Some("/p/sub"), "", "shell").unwrap();
        assert_eq!(s.name, "Session 1");
        assert_eq!(s.cwd.as_deref(), Some("/p/sub"));
        assert_eq!(s.project_id, p.id);

        let fetched = db.get_session(s.id).unwrap();
        assert_eq!(fetched.name, s.name);
        assert_eq!(fetched.cwd, s.cwd);
    }

    #[test]
    fn test_session_cwd_null() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "No CWD", None, "", "shell").unwrap();
        assert_eq!(s.cwd, None);
    }

    #[test]
    fn test_get_sessions_for_project() {
        let db = setup();
        let p1 = db.insert_project("P1", "/p1").unwrap();
        let p2 = db.insert_project("P2", "/p2").unwrap();
        db.insert_session(p1.id, "S1", None, "", "shell").unwrap();
        db.insert_session(p1.id, "S2", None, "", "shell").unwrap();
        db.insert_session(p2.id, "S3", None, "", "shell").unwrap();

        let s_p1 = db.get_sessions_for_project(p1.id).unwrap();
        assert_eq!(s_p1.len(), 2);
        let s_p2 = db.get_sessions_for_project(p2.id).unwrap();
        assert_eq!(s_p2.len(), 1);
    }

    #[test]
    fn test_update_session_name() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "Old", None, "", "shell").unwrap();
        db.update_session_name(s.id, "New").unwrap();
        assert_eq!(db.get_session(s.id).unwrap().name, "New");
    }

    #[test]
    fn test_update_session_cwd() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "S", Some("/old"), "", "shell").unwrap();
        db.update_session_cwd(s.id, "/new/path").unwrap();
        assert_eq!(db.get_session(s.id).unwrap().cwd.as_deref(), Some("/new/path"));
    }

    #[test]
    fn test_update_session_active() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "S", None, "", "shell").unwrap();
        let before = db.get_session(s.id).unwrap().last_active_at;
        std::thread::sleep(std::time::Duration::from_secs(1));
        db.update_session_active(s.id).unwrap();
        let after = db.get_session(s.id).unwrap().last_active_at;
        assert_ne!(before, after);
    }

    #[test]
    fn test_delete_session() {
        let db = setup();
        let p = db.insert_project("P", "/p").unwrap();
        let s = db.insert_session(p.id, "S", None, "", "shell").unwrap();
        db.delete_session(s.id).unwrap();
        assert!(db.get_session(s.id).is_err());
    }

    #[test]
    fn test_get_nonexistent_session() {
        let db = setup();
        assert!(db.get_session(99999).is_err());
    }

    #[test]
    fn test_settings_get_set() {
        let db = setup();
        assert_eq!(db.get_setting("foo").unwrap(), None);
        db.set_setting("foo", "bar").unwrap();
        assert_eq!(db.get_setting("foo").unwrap().as_deref(), Some("bar"));
    }

    #[test]
    fn test_settings_upsert() {
        let db = setup();
        db.set_setting("key", "v1").unwrap();
        db.set_setting("key", "v2").unwrap();
        assert_eq!(db.get_setting("key").unwrap().as_deref(), Some("v2"));
    }

    #[test]
    fn test_get_db_path_returns_path() {
        let path = get_db_path();
        assert!(path.ends_with("devterm.db"));
        assert!(path.to_string_lossy().contains("devterm"));
    }
}

/// Get the platform-specific database path
pub fn get_db_path() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("USERPROFILE").unwrap_or_default();
                PathBuf::from(home).join("AppData").join("Roaming")
            })
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("Library").join("Application Support")
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".local").join("share")
            })
    };

    base.join("devterm").join("devterm.db")
}
