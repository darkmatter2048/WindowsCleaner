use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use tauri::Manager;

// ---------------------------------------------------------------------------
// Logger — file-based, auto-rotation (keep latest 6)
// ---------------------------------------------------------------------------

static LOG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Call once at startup.
pub fn init(app_handle: &tauri::AppHandle) {
    let dir = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs");

    if let Ok(mut d) = LOG_DIR.lock() {
        *d = Some(dir.clone());
    }

    let _ = fs::create_dir_all(&dir);
    rotate(&dir, 6);

    info("wc", "========== 启动 v{} ==========", env!("CARGO_PKG_VERSION"));
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn info(tag: &str, msg: &str, extra: &str) {
    write("INFO", tag, msg, extra);
}

pub fn warn(tag: &str, msg: &str, extra: &str) {
    write("WARN", tag, msg, extra);
}

pub fn error(tag: &str, msg: &str, extra: &str) {
    write("ERROR", tag, msg, extra);
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(dur) => {
            let secs = dur.as_secs();
            format_unix(secs)
        }
        Err(_) => String::from("????-??-?? ??:??:??"),
    }
}

fn format_unix(secs: u64) -> String {
    // Convert unix timestamp to YYYY-MM-DD HH:MM:SS in local time
    // Manual computation
    let (year, month, day, hour, min, sec) = unix_to_local(secs);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}

fn unix_to_local(ts: u64) -> (u32, u32, u32, u32, u32, u32) {
    // Local time offset (China = UTC+8)
    let local_secs = ts + 8 * 3600;
    let mut days = (local_secs / 86400) as u32;
    let time_of_day = (local_secs % 86400) as u32;
    let hour = time_of_day / 3600;
    let min = (time_of_day % 3600) / 60;
    let sec = time_of_day % 60;

    // Days since 1970-01-01
    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;

    (year, month, day, hour, min, sec)
}

fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn write(level: &str, tag: &str, msg: &str, extra: &str) {
    let dir = match LOG_DIR.lock() {
        Ok(d) => d.clone(),
        Err(_) => return,
    };
    let dir = match dir {
        Some(d) => d,
        None => return,
    };

    let ts = timestamp();
    let line = if extra.is_empty() {
        format!("[{}] [{}] [{}] {}\n", ts, level, tag, msg)
    } else {
        format!("[{}] [{}] [{}] {} | {}\n", ts, level, tag, msg, extra)
    };

    let today = {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let (y, m, d, _, _, _) = unix_to_local(secs);
        format!("{:04}-{:02}-{:02}", y, m, d)
    };
    let path = dir.join(format!("wc_{}.log", today));

    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

fn rotate(dir: &PathBuf, max_files: usize) {
    let mut files: Vec<_> = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |x| x == "log")
            })
            .collect(),
        Err(_) => return,
    };

    if files.len() <= max_files {
        return;
    }

    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    let to_delete = files.len().saturating_sub(max_files);
    for entry in files.iter().take(to_delete) {
        let _ = fs::remove_file(entry.path());
    }
}
