use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone)]
pub struct CleanResult {
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CleanOption {
    Prefetch,
    UserTemp,
    WindowsTemp,
    SystemLogs,
    SoftwareDistribution,
    BrowserCache,
    TempFilesC,
    RestorePoints,
    Hibernation,
    DeliveryOptimization,
    Thumbnails,
    Defender,
    InetCache,
    WerReports,
    ShaderCache,
    NvidiaDebugLogs,
    WpsBackups,
    Winapp2,
    MspFiles,
}

#[derive(Deserialize)]
pub struct CleanRequest {
    pub options: Vec<CleanOption>,
}

impl CleanResult {
    pub(crate) fn new() -> Self {
        CleanResult {
            bytes_freed: 0,
            errors: Vec::new(),
        }
    }

    pub(crate) fn add_freed(&mut self, bytes: u64) {
        self.bytes_freed += bytes;
    }

    pub(crate) fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub(crate) fn merge(&mut self, other: CleanResult) {
        self.bytes_freed += other.bytes_freed;
        self.errors.extend(other.errors);
    }
}

/// Delete all files and subdirectories inside a folder.
/// Each file that is locked by another process will be skipped (error logged, not fatal).
fn clean_folder_contents(folder: &Path) -> CleanResult {
    let mut result = CleanResult::new();

    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", folder.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            match fs::remove_dir_all(&path) {
                Ok(_) => {}
                Err(e) => result.add_error(format!("Cannot delete {}: {}", path.display(), e)),
            }
        } else {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            match fs::remove_file(&path) {
                Ok(_) => result.add_freed(size),
                Err(e) => {
                    // File may be in use — skip, don't treat as fatal
                    result.add_error(format!("Skip {}: {}", path.display(), e))
                }
            }
        }
    }

    result
}

// ─── Helpers ───

fn windows_dir() -> PathBuf {
    PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()))
}

fn local_appdata() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
}

// ─── Individual cleaning tasks ───

/// C:\Windows\Prefetch — only delete .pf application prefetch files.
/// ReadyBoot folder (boot optimization) and system files (Layout.ini, etc.) are left untouched.
fn clean_prefetch() -> CleanResult {
    let mut result = CleanResult::new();
    let path = windows_dir().join("Prefetch");
    if !path.exists() {
        return result;
    }

    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", path.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        // Skip ALL subdirectories (especially ReadyBoot)
        if p.is_dir() {
            continue;
        }
        // Only delete .pf files — leave Layout.ini and other system files alone
        let is_pf = p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pf"))
            .unwrap_or(false);
        if !is_pf {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        match fs::remove_file(&p) {
            Ok(_) => result.add_freed(size),
            Err(e) => result.add_error(format!("Skip {}: {}", p.display(), e)),
        }
    }

    result
}

/// %TEMP% — user temp folder. Only deletes files not currently in use.
fn clean_user_temp() -> CleanResult {
    let path = PathBuf::from(std::env::var("TEMP").unwrap_or_default());
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// C:\Windows\Temp — system temp folder.
fn clean_windows_temp() -> CleanResult {
    let path = windows_dir().join("Temp");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// C:\Windows\Logs — system log files (not the System32 event logs).
fn clean_system_logs() -> CleanResult {
    let path = windows_dir().join("Logs");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// Check if the Windows Update service (wuauserv) is currently running.
fn is_windows_update_active() -> bool {
    // sc query wuauserv | findstr RUNNING — service name is language-independent
    std::process::Command::new("sc")
        .args(["query", "wuauserv"])
        .output()
        .map(|out| {
            let text = String::from_utf8_lossy(&out.stdout);
            text.contains("RUNNING")
        })
        .unwrap_or(false)
}

/// C:\Windows\SoftwareDistribution\Download — Windows Update download cache.
/// Skipped entirely if Windows Update service is currently running.
fn clean_software_distribution() -> CleanResult {
    let mut result = CleanResult::new();

    if is_windows_update_active() {
        result.add_error("Windows Update is active, skipping SoftwareDistribution cleanup".into());
        return result;
    }

    let path = windows_dir()
        .join("SoftwareDistribution")
        .join("Download");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        result
    }
}

/// Delete system restore points on C: drive via vssadmin.
/// App is already elevated at startup, so direct call is sufficient.
fn clean_restore_points() -> CleanResult {
    let mut result = CleanResult::new();

    let out = std::process::Command::new("vssadmin")
        .args(["delete", "shadows", "/for=C:", "/all", "/quiet"])
        .output();

    match out {
        Ok(o) if !o.status.success() => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            result.add_error(format!("vssadmin: {}", stderr.trim()));
        }
        Err(e) => {
            result.add_error(format!("Cannot run vssadmin: {}", e));
        }
        _ => {}
    }

    result
}

/// Recursively delete .tmp and .cache files under the given root.
/// .msp is intentionally excluded — those are Windows Installer patches.
fn clean_temp_by_ext(root: &Path) -> CleanResult {
    let mut result = CleanResult::new();
    let exts = ["tmp", "cache"];

    fn walk(dir: &Path, exts: &[&str], result: &mut CleanResult) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return, // skip inaccessible dirs silently
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip system junctions and protected dirs
                walk(&path, exts, result);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    match fs::remove_file(&path) {
                        Ok(_) => result.add_freed(size),
                        Err(_) => {} // in-use or protected — skip silently
                    }
                }
            }
        }
    }

    walk(root, &exts, &mut result);
    result
}

fn clean_temp_files_c() -> CleanResult {
    let root = PathBuf::from("C:\\");
    if root.exists() {
        clean_temp_by_ext(&root)
    } else {
        CleanResult::new()
    }
}

/// Explicit custom-clean option for recursively deleting .msp files under C:.
/// Kept separate from quick clean because Windows Installer patches can be important.
fn clean_msp_files() -> CleanResult {
    fn walk(dir: &Path, result: &mut CleanResult) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, result);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("msp"))
                .unwrap_or(false)
            {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    result.add_freed(size);
                }
            }
        }
    }

    let mut result = CleanResult::new();
    let root = PathBuf::from("C:\\");
    if root.exists() {
        walk(&root, &mut result);
    }
    result
}

/// Chrome / Edge browser cache — safe, browsers rebuild on next launch.
fn clean_browser_cache() -> CleanResult {
    let mut result = CleanResult::new();

    let chrome = local_appdata()
        .join("Google")
        .join("Chrome")
        .join("User Data")
        .join("Default")
        .join("Cache");
    if chrome.exists() {
        result.merge(clean_folder_contents(&chrome));
    }

    let edge = local_appdata()
        .join("Microsoft")
        .join("Edge")
        .join("User Data")
        .join("Default")
        .join("Cache");
    if edge.exists() {
        result.merge(clean_folder_contents(&edge));
    }

    result
}

/// Disable system hibernation to free hiberfil.sys (typically = RAM size).
/// No-op if hibernation is already off.
fn disable_hibernation() -> CleanResult {
    let mut result = CleanResult::new();

    match std::process::Command::new("powercfg")
        .args(["/h", "off"])
        .output()
    {
        Ok(out) => {
            if out.status.success() {
                // hiberfil.sys is deleted, but we can't measure exact bytes
                result.add_freed(0);
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                // "Hibernate is already disabled" is not an error
                let msg = stderr.trim();
                if !msg.is_empty() {
                    result.add_error(format!("powercfg: {}", msg));
                }
            }
        }
        Err(e) => {
            result.add_error(format!("Cannot run powercfg: {}", e));
        }
    }

    result
}

/// Delivery Optimization files — cached Windows Update peer-sharing data.
fn clean_delivery_optimization() -> CleanResult {
    let path = windows_dir()
        .join("SoftwareDistribution")
        .join("DeliveryOptimization");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// Thumbnail cache — Explorer thumbnail .db files.
fn clean_thumbnails() -> CleanResult {
    let path = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("Explorer");
    if !path.exists() {
        return CleanResult::new();
    }

    let mut result = CleanResult::new();
    let entries = match fs::read_dir(&path) {
        Ok(e) => e,
        Err(e) => {
            result.add_error(format!("Cannot read {}: {}", path.display(), e));
            return result;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("thumbcache_") && name.ends_with(".db") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    match fs::remove_file(&p) {
                        Ok(_) => result.add_freed(size),
                        Err(e) => result.add_error(format!("Skip {}: {}", p.display(), e)),
                    }
                }
            }
        }
    }

    result
}

/// Microsoft Defender scan history & cache.
fn clean_defender() -> CleanResult {
    let path = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows Defender\\Scans\\History");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// Internet temporary files (INetCache).
fn clean_inet_cache() -> CleanResult {
    let path = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("INetCache");
    if path.exists() {
        clean_folder_contents(&path)
    } else {
        CleanResult::new()
    }
}

/// Windows Error Reporting logs + feedback diagnostics.
fn clean_wer_reports() -> CleanResult {
    let mut result = CleanResult::new();

    // System-wide WER
    let system_wer = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\WER");
    if system_wer.exists() {
        result.merge(clean_folder_contents(&system_wer));
    }

    // Per-user WER
    let user_wer = local_appdata()
        .join("Microsoft")
        .join("Windows")
        .join("WER");
    if user_wer.exists() {
        result.merge(clean_folder_contents(&user_wer));
    }

    result
}

/// DirectX shader cache — both Microsoft and GPU vendor caches.
fn clean_shader_cache() -> CleanResult {
    let mut result = CleanResult::new();

    let dx_cache = local_appdata()
        .join("Microsoft")
        .join("DirectX Shader Cache");
    if dx_cache.exists() {
        result.merge(clean_folder_contents(&dx_cache));
    }

    let nv_cache = local_appdata().join("NVIDIA").join("DXCache");
    if nv_cache.exists() {
        result.merge(clean_folder_contents(&nv_cache));
    }

    let amd_cache = local_appdata().join("AMD").join("DxCache");
    if amd_cache.exists() {
        result.merge(clean_folder_contents(&amd_cache));
    }

    let intel_cache = local_appdata().join("Intel").join("ShaderCache");
    if intel_cache.exists() {
        result.merge(clean_folder_contents(&intel_cache));
    }

    result
}

/// NVIDIA debug.log — driver/GeForce Experience debug logs that can grow large.
fn clean_nvidia_debug_logs() -> CleanResult {
    let mut result = CleanResult::new();

    let roots = [
        PathBuf::from("C:\\ProgramData\\NVIDIA Corporation"),
        PathBuf::from(
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into())
        ).join("NVIDIA Corporation"),
        local_appdata().join("NVIDIA Corporation"),
    ];

    for root in &roots {
        if !root.exists() {
            continue;
        }
        delete_named_files_recursive(root, "debug.log", &mut result);
    }

    result
}

/// WPS Office backup files (autosave .bak / .wbk).
/// Only cleans the dedicated backup folder, not WPS config data.
fn clean_wps_backups() -> CleanResult {
    let mut result = CleanResult::new();

    let appdata = PathBuf::from(std::env::var("APPDATA").unwrap_or_default());

    // Dedicated backup folder — safe to clean entirely
    let backup_dir = appdata.join("Kingsoft").join("office6").join("backup");
    if backup_dir.exists() {
        result.merge(clean_folder_contents(&backup_dir));
    }

    // Also check %LocalAppData%\Kingsoft for .bak / .wbk in backup folders
    let local = local_appdata().join("Kingsoft");
    if local.exists() {
        collect_and_clean_backup_dirs(&local, &mut result);
    }

    // %AppData%\Kingsoft — scan for nested backup dirs
    let roaming = appdata.join("Kingsoft");
    if roaming.exists() {
        collect_and_clean_backup_dirs(&roaming, &mut result);
    }

    result
}

fn collect_and_clean_backup_dirs(dir: &Path, result: &mut CleanResult) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            // Clean any directory named "backup" or "Backup"
            if dir_name.eq_ignore_ascii_case("backup") {
                result.merge(clean_folder_contents(&path));
            } else {
                collect_and_clean_backup_dirs(&path, result);
            }
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Delete individual .bak / .wbk files outside backup dirs too
            let lower = name.to_lowercase();
            if lower.ends_with(".bak") || lower.ends_with(".wbk") {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                match fs::remove_file(&path) {
                    Ok(_) => result.add_freed(size),
                    Err(_) => {}
                }
            }
        }
    }
}

fn delete_named_files_recursive(dir: &Path, target_name: &str, result: &mut CleanResult) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            delete_named_files_recursive(&path, target_name, result);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case(target_name) {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                match fs::remove_file(&path) {
                    Ok(_) => result.add_freed(size),
                    Err(_) => {} // locked
                }
            }
        }
    }
}

// ─── Top-level commands ───

const QUICK_CLEAN_OPTIONS: &[CleanOption] = &[
    CleanOption::Prefetch,
    CleanOption::UserTemp,
    CleanOption::WindowsTemp,
    CleanOption::SystemLogs,
    CleanOption::SoftwareDistribution,
    CleanOption::BrowserCache,
    CleanOption::TempFilesC,
    CleanOption::RestorePoints,
    CleanOption::Hibernation,
    CleanOption::DeliveryOptimization,
    CleanOption::Thumbnails,
    CleanOption::Defender,
    CleanOption::InetCache,
    CleanOption::WerReports,
    CleanOption::ShaderCache,
    CleanOption::NvidiaDebugLogs,
    CleanOption::WpsBackups,
    CleanOption::Winapp2,
];

fn run_clean_option(option: CleanOption) -> CleanResult {
    match option {
        CleanOption::Prefetch => clean_prefetch(),
        CleanOption::UserTemp => clean_user_temp(),
        CleanOption::WindowsTemp => clean_windows_temp(),
        CleanOption::SystemLogs => clean_system_logs(),
        CleanOption::SoftwareDistribution => clean_software_distribution(),
        CleanOption::BrowserCache => clean_browser_cache(),
        CleanOption::TempFilesC => clean_temp_files_c(),
        CleanOption::RestorePoints => clean_restore_points(),
        CleanOption::Hibernation => disable_hibernation(),
        CleanOption::DeliveryOptimization => clean_delivery_optimization(),
        CleanOption::Thumbnails => clean_thumbnails(),
        CleanOption::Defender => clean_defender(),
        CleanOption::InetCache => clean_inet_cache(),
        CleanOption::WerReports => clean_wer_reports(),
        CleanOption::ShaderCache => clean_shader_cache(),
        CleanOption::NvidiaDebugLogs => clean_nvidia_debug_logs(),
        CleanOption::WpsBackups => clean_wps_backups(),
        CleanOption::Winapp2 => crate::winapp2::clean_winapp2(),
        CleanOption::MspFiles => clean_msp_files(),
    }
}

fn run_clean_options(options: &[CleanOption]) -> CleanResult {
    let mut result = CleanResult::new();
    let before = measure_c_free().unwrap_or(0);

    for option in options {
        result.merge(run_clean_option(*option));
    }

    let after = measure_c_free().unwrap_or(before);
    result.bytes_freed = after.saturating_sub(before);
    result
}

/// Measure C: drive free space in bytes.
fn measure_c_free() -> Result<u64, String> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy();
        if mount.starts_with("C:") || mount.starts_with("c:") {
            return Ok(disk.available_space());
        }
    }
    Err("C: drive not found".into())
}

/// Quick clean (小白一键清) — all cleaning items.
/// Measures free space before and after, returns the delta.
#[tauri::command]
pub async fn quick_clean() -> CleanResult {
    tauri::async_runtime::spawn_blocking(|| run_clean_options(QUICK_CLEAN_OPTIONS))
        .await
        .unwrap_or_else(|_| {
            let mut err = CleanResult::new();
            err.add_error("Clean task panicked".into());
            err
        })
}

/// Deep clean (自定义清理) — same as quick clean for now.
/// Reserved for future user-configurable cleaning paths.
#[tauri::command]
pub async fn deep_clean() -> CleanResult {
    quick_clean().await
}

#[tauri::command]
pub async fn clean_selected(request: CleanRequest) -> CleanResult {
    tauri::async_runtime::spawn_blocking(move || run_clean_options(&request.options))
        .await
        .unwrap_or_else(|_| {
            let mut err = CleanResult::new();
            err.add_error("Clean task panicked".into());
            err
        })
}
