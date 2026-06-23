use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static CRASH_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Set up the global panic hook. Call once at startup.
pub fn init(crashes_dir: PathBuf) {
    fs::create_dir_all(&crashes_dir).ok();
    *CRASH_DIR.lock().unwrap() = Some(crashes_dir.clone());

    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let crash_dir = CRASH_DIR.lock().unwrap();
        write_crash_dump(crash_dir.as_deref(), info);

        // Still run the default panic handler (which prints + aborts)
        old_hook(info);
    }));
}

/// Write crash info to a timestamped file
fn write_crash_dump(crashes_dir: Option<&std::path::Path>, info: &std::panic::PanicHookInfo<'_>) {
    let Some(dir) = crashes_dir else { return };
    let now = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!("crash_{}.log", now));

    let msg = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.clone()))
        .unwrap_or_else(|| "(unknown panic payload)".to_string());

    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "(unknown location)".to_string());

    let mut f = match fs::File::create(&path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let _ = writeln!(f, "Workbase Crash Report");
    let _ = writeln!(f, "Time: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    let _ = writeln!(f, "Location: {}", location);
    let _ = writeln!(f, "Message: {}", msg);
    let _ = writeln!(f, "---");
    let _ = writeln!(f, "{}", std::backtrace::Backtrace::force_capture());

    eprintln!(
        "Workbase crashed! Crash report written to: {}",
        path.display()
    );
}

/// Check for crash dumps from previous runs.
/// Returns the path of the most recent crash dump, or None if clean.
pub fn check_previous_crash(crashes_dir: &PathBuf) -> Option<PathBuf> {
    let dir = match fs::read_dir(crashes_dir) {
        Ok(d) => d,
        Err(_) => return None,
    };

    let mut crashes: Vec<_> = dir
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("crash_"))
        .collect();

    if crashes.is_empty() {
        return None;
    }

    // Return the most recent (sorted by name = timestamp)
    crashes.sort_by_key(|e| e.file_name());
    crashes.pop().map(|e| e.path())
}

/// Delete all crash dumps after user has acknowledged them
pub fn clear_crashes(crashes_dir: &PathBuf) {
    let Ok(dir) = fs::read_dir(crashes_dir) else { return };
    for entry in dir.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().starts_with("crash_") {
            let _ = fs::remove_file(entry.path());
        }
    }
}
