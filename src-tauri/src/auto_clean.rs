use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use sysinfo::Disks;
use tauri::Manager;

use crate::clean::{run_clean_option, CleanOption, CleanResult};

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AutoCleanMode {
    Scheduled,
    LowDisk,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoCleanSettings {
    pub enabled: bool,
    pub mode: AutoCleanMode,
    pub interval_days: u32,
    pub threshold_gb: u32,
    pub last_clean_time: String, // "YYYY-MM-DD"
    pub options: Vec<CleanOption>,
}

impl Default for AutoCleanSettings {
    fn default() -> Self {
        AutoCleanSettings {
            enabled: false,
            mode: AutoCleanMode::Scheduled,
            interval_days: 7,
            threshold_gb: 10,
            last_clean_time: String::new(),
            options: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&dir).ok();
    dir.join("auto_clean.json")
}

fn load_settings(app: &tauri::AppHandle) -> AutoCleanSettings {
    let path = settings_path(app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(app: &tauri::AppHandle, settings: &AutoCleanSettings) -> Result<(), String> {
    let path = settings_path(app);
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_auto_clean_settings(app: tauri::AppHandle) -> AutoCleanSettings {
    load_settings(&app)
}

#[tauri::command]
pub fn save_auto_clean_settings(
    app: tauri::AppHandle,
    settings: AutoCleanSettings,
) -> Result<(), String> {
    save_settings(&app, &settings)
}

// ---------------------------------------------------------------------------
// Background task helpers
// ---------------------------------------------------------------------------

fn c_free_gb() -> u64 {
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy();
        if mount.starts_with("C:") || mount.starts_with("c:") {
            return disk.available_space() / (1024 * 1024 * 1024);
        }
    }
    u64::MAX
}

fn today_string() -> String {
    // Use chrono-like manual calculation for zero-dependency date
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    days_since_epoch_to_ymd(elapsed.as_secs() / 86400)
}

fn days_since_epoch_to_ymd(days: u64) -> String {
    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era as i64 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_between(from: &str, to: &str) -> i64 {
    let (y1, m1, d1) = match parse_ymd(from) {
        Some(v) => v,
        None => return 0,
    };
    let (y2, m2, d2) = match parse_ymd(to) {
        Some(v) => v,
        None => return 0,
    };
    let days1 = y1 as i64 * 365 + y1 as i64 / 4 - y1 as i64 / 100 + y1 as i64 / 400
        + month_day_offset(m1) + d1 as i64;
    let days2 = y2 as i64 * 365 + y2 as i64 / 4 - y2 as i64 / 100 + y2 as i64 / 400
        + month_day_offset(m2) + d2 as i64;
    days2 - days1
}

fn parse_ymd(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn month_day_offset(m: u32) -> i64 {
    const OFFSETS: [i64; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    OFFSETS[m as usize]
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

fn run_auto_clean_if_needed(app: &tauri::AppHandle) {
    let settings = load_settings(app);
    if !settings.enabled || settings.options.is_empty() {
        return;
    }

    let should_clean = match settings.mode {
        AutoCleanMode::Scheduled => {
            if settings.last_clean_time.is_empty() {
                true
            } else {
                let today = today_string();
                today >= settings.last_clean_time
                    && days_between(&settings.last_clean_time, &today)
                        >= settings.interval_days as i64
            }
        }
        AutoCleanMode::LowDisk => c_free_gb() < settings.threshold_gb as u64,
    };

    if !should_clean {
        return;
    }

    // Run clean synchronously then update last clean time
    let mut result = CleanResult {
        bytes_freed: 0,
        errors: Vec::new(),
    };
    for option in &settings.options {
        result.merge(run_clean_option(*option));
    }

    let mut updated = load_settings(app);
    updated.last_clean_time = today_string();
    let _ = save_settings(app, &updated);
}

// ---------------------------------------------------------------------------
// Spawn the background watcher (called once from setup)
// ---------------------------------------------------------------------------

pub fn spawn_auto_clean_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(300));
        run_auto_clean_if_needed(&app);
    });
}
