use serde::{Deserialize, Serialize};
use tauri::Emitter;
use winreg::enums::*;
use winreg::RegKey;

// ---------------------------------------------------------------------------
// Combined status (single invoke → no focus flicker)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedStatus {
    pub defender_disabled: bool,
    pub update_disabled: bool,
    pub page_file: PageFileInfo,
}

#[tauri::command]
pub async fn get_advanced_status() -> Result<AdvancedStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let defender_disabled = get_defender_status_raw()?;
        let update_disabled = match get_update_status_raw() {
            Ok(s) => s.disabled,
            Err(_) => false,
        };
        let page_file = get_page_file_info_raw()?;
        Ok(AdvancedStatus {
            defender_disabled,
            update_disabled,
            page_file,
        })
    })
    .await
    .unwrap_or_else(|_| {
        Err("Failed to read advanced status".into())
    })
}

fn get_defender_status_raw() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey(DEFENDER_KEY) {
        Ok(k) => {
            let val: u32 = k.get_value("DisableAntiSpyware").unwrap_or(0);
            Ok(val == 1)
        }
        Err(_) => Ok(false),
    }
}

fn get_update_status_raw() -> Result<UpdateStatus, String> {
    let disabled = read_service_disabled("wuauserv");
    Ok(UpdateStatus {
        disabled,
        service_running: !disabled,
    })
}

fn get_page_file_info_raw() -> Result<PageFileInfo, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;
    let paging_files: String = key.get_value("PagingFiles").unwrap_or_default();
    if paging_files.is_empty() || paging_files.trim().is_empty() {
        return Ok(PageFileInfo { initial_mb: 0, max_mb: 0, system_managed: true });
    }
    let first_line = paging_files.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let (initial, max) = if parts.len() >= 3 {
        (parts[1].parse::<u32>().unwrap_or(0), parts[2].parse::<u32>().unwrap_or(0))
    } else {
        (0, 0)
    };
    Ok(PageFileInfo { initial_mb: initial, max_mb: max, system_managed: initial == 0 && max == 0 })
}

// ---------------------------------------------------------------------------
// Windows Defender
// ---------------------------------------------------------------------------

const DEFENDER_KEY: &str = "SOFTWARE\\Policies\\Microsoft\\Windows Defender";

#[tauri::command]
pub fn get_defender_status() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(DEFENDER_KEY);
    match key {
        Ok(k) => {
            let val: u32 = k.get_value("DisableAntiSpyware").unwrap_or(0);
            Ok(val == 1)
        }
        Err(_) => Ok(false), // key doesn't exist = defender is ON (not disabled)
    }
}

#[tauri::command]
pub fn set_defender_disabled(disabled: bool) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm.create_subkey(DEFENDER_KEY).map_err(|e| e.to_string())?;

    if disabled {
        key.set_value("DisableAntiSpyware", &1u32)
            .map_err(|e| format!("Failed to disable Defender: {}", e))?;
        // Also disable real-time monitoring via policy
        let _ = key.set_value("DisableRealtimeMonitoring", &1u32);
        let _ = key.set_value("DisableBehaviorMonitoring", &1u32);
        let _ = key.set_value("DisableOnAccessProtection", &1u32);
        let _ = key.set_value("DisableScanOnRealtimeEnable", &1u32);
    } else {
        // Delete the values to re-enable
        let _ = key.delete_value("DisableAntiSpyware");
        let _ = key.delete_value("DisableRealtimeMonitoring");
        let _ = key.delete_value("DisableBehaviorMonitoring");
        let _ = key.delete_value("DisableOnAccessProtection");
        let _ = key.delete_value("DisableScanOnRealtimeEnable");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Windows Update
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct UpdateStatus {
    pub disabled: bool,
    pub service_running: bool,
}

/// Read service Start value from registry — no process spawn.
/// HKLM\SYSTEM\CurrentControlSet\Services\<name>\Start:
///   2=AUTO, 3=MANUAL, 4=DISABLED
fn read_service_disabled(name: &str) -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", name);
    hklm.open_subkey(&path)
        .ok()
        .and_then(|k| k.get_value::<u32, _>("Start").ok())
        .map(|v| v == 4)
        .unwrap_or(false)
}

/// Write service Start value via registry — no process spawn.
fn write_service_disabled(name: &str, disabled: bool) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", name);
    let (key, _) = hklm.create_subkey(&path).map_err(|e| e.to_string())?;
    let start_val: u32 = if disabled { 4 } else { 2 }; // 4=DISABLED, 2=AUTO
    key.set_value("Start", &start_val)
        .map_err(|e| format!("Failed to set {} Start: {}", name, e))
}

#[tauri::command]
pub fn get_update_status() -> Result<UpdateStatus, String> {
    let disabled = read_service_disabled("wuauserv");
    Ok(UpdateStatus {
        disabled,
        service_running: !disabled,
    })
}

#[tauri::command]
pub fn set_update_disabled(disabled: bool) -> Result<(), String> {
    write_service_disabled("wuauserv", disabled)?;
    let _ = write_service_disabled("UsoSvc", disabled);
    let _ = write_service_disabled("WaaSMedicSvc", disabled);
    // Registry change takes effect on next reboot
    Ok(())
}

// ---------------------------------------------------------------------------
// Virtual Memory (Page File)
// ---------------------------------------------------------------------------

const MEMORY_MGMT_KEY: &str =
    "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageFileInfo {
    pub initial_mb: u32,
    pub max_mb: u32,
    /// System-managed = true, custom size = false
    pub system_managed: bool,
}

#[tauri::command]
pub fn get_page_file_info() -> Result<PageFileInfo, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    let paging_files: String = key
        .get_value("PagingFiles")
        .unwrap_or_default();

    // "PagingFiles" is REG_MULTI_SZ but may be read as REG_SZ.
    // Format per line: "<disk>:\pagefile.sys <min> <max>"
    if paging_files.is_empty() || paging_files.trim().is_empty() {
        return Ok(PageFileInfo {
            initial_mb: 0,
            max_mb: 0,
            system_managed: true,
        });
    }

    let first_line = paging_files.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    // Expected: "C:\pagefile.sys 0 0" or "C:\pagefile.sys 2048 4096"
    let (initial, max) = if parts.len() >= 3 {
        (parts[1].parse::<u32>().unwrap_or(0), parts[2].parse::<u32>().unwrap_or(0))
    } else {
        (0, 0)
    };

    Ok(PageFileInfo {
        initial_mb: initial,
        max_mb: max,
        // When both are 0 it means system-managed
        system_managed: initial == 0 && max == 0,
    })
}

#[tauri::command]
pub fn set_page_file(initial_mb: u32, max_mb: u32) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    // Find the system drive letter
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let entry = format!("{}\\pagefile.sys {} {}", system_drive, initial_mb, max_mb);
    key.set_value("PagingFiles", &entry)
        .map_err(|e| format!("Failed to set page file: {}", e))?;

    // Also set the existing page file to avoid conflicts
    key.set_value("ExistingPagingFiles", &entry)
        .map_err(|e| format!("Failed to set existing page file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_page_file_to_system_managed() -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(MEMORY_MGMT_KEY)
        .map_err(|e| format!("Failed to open memory management key: {}", e))?;

    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let entry = format!("{}\\pagefile.sys 0 0", system_drive);
    key.set_value("PagingFiles", &entry)
        .map_err(|e| format!("Failed to reset page file: {}", e))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// User Folder Migration
// ---------------------------------------------------------------------------

const USER_SHELL_FOLDERS: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders";

const SHELL_FOLDERS: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserFolderInfo {
    /// Registry value name
    pub key: String,
    /// Display name
    pub display_name: String,
    /// Current full path
    pub current_path: String,
    /// Current drive letter
    pub drive: String,
    /// Size in bytes (0 if not yet scanned)
    pub size_bytes: u64,
    /// Whether folder is on C:
    pub on_c_drive: bool,
}

static KNOWN_FOLDERS: &[(&str, &str)] = &[
    ("Personal", "Documents"),
    ("My Pictures", "Pictures"),
    ("My Video", "Videos"),
    ("My Music", "Music"),
    ("Desktop", "Desktop"),
    ("{374DE290-123F-4565-9164-39C4925E467B}", "Downloads"),
];

fn expand_env_vars(raw: &str) -> String {
    let mut result = raw.to_string();
    let vars = [("USERPROFILE", "USERPROFILE"), ("APPDATA", "APPDATA"),
        ("LOCALAPPDATA", "LOCALAPPDATA"), ("TEMP", "TEMP")];
    for (env_key, _) in &vars {
        if let Ok(val) = std::env::var(env_key) {
            result = result.replace(&format!("%{}%", env_key), &val);
        }
    }
    result
}

fn read_user_shell_folder(key_name: &str) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(USER_SHELL_FOLDERS)
        .map_err(|e| format!("Failed to open User Shell Folders: {}", e))?;
    let raw: String = key
        .get_value(key_name)
        .map_err(|e| format!("Failed to read {}: {}", key_name, e))?;
    // Expand %USERPROFILE% etc.
    let expanded = expand_env_vars(&raw);
    Ok(expanded)
}

fn write_user_shell_folder(key_name: &str, new_path: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Update User Shell Folders
    let (key, _) = hkcu
        .create_subkey(USER_SHELL_FOLDERS)
        .map_err(|e| e.to_string())?;
    key.set_value(key_name, &new_path)
        .map_err(|e| format!("Failed to write {}: {}", key_name, e))?;
    // Also update Shell Folders (without "User" prefix) for compatibility
    if let Ok((k, _)) = hkcu.create_subkey(SHELL_FOLDERS) {
        let _ = k.set_value(key_name, &new_path);
    }
    Ok(())
}

fn dir_size_fast(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && !p.is_symlink() {
                total += dir_size_fast(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[tauri::command]
pub fn get_user_folders() -> Result<Vec<UserFolderInfo>, String> {
    let mut folders = Vec::new();

    for (key, display_name) in KNOWN_FOLDERS {
        let current_path = match read_user_shell_folder(key) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let drive = current_path
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_default();
        let on_c_drive = drive == "C";
        folders.push(UserFolderInfo {
            key: key.to_string(),
            display_name: display_name.to_string(),
            current_path,
            drive,
            size_bytes: 0,
            on_c_drive,
        });
    }

    Ok(folders)
}

#[tauri::command]
pub fn scan_user_folder_sizes() -> Result<Vec<UserFolderInfo>, String> {
    let mut folders = Vec::new();

    for (key, display_name) in KNOWN_FOLDERS {
        let current_path = match read_user_shell_folder(key) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let drive = current_path
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_default();
        let on_c_drive = drive == "C";
        let p = std::path::Path::new(&current_path);
        let size_bytes = if p.exists() { dir_size_fast(p) } else { 0 };
        folders.push(UserFolderInfo {
            key: key.to_string(),
            display_name: display_name.to_string(),
            current_path,
            drive,
            size_bytes,
            on_c_drive,
        });
    }

    Ok(folders)
}

#[tauri::command]
pub async fn move_user_folder(
    app: tauri::AppHandle,
    folder_key: String,
    target_drive: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let src_str = read_user_shell_folder(&folder_key)?;
        let src = std::path::Path::new(&src_str);

        if !src.exists() {
            return Err(format!("Source folder not found: {}", src_str));
        }

        let folder_name = src
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&folder_key);
        let dst_str = format!("{}:\\Users\\{}\\{}",
            target_drive,
            std::env::var("USERNAME").unwrap_or_default(),
            folder_name,
        );
        let dst = std::path::Path::new(&dst_str);

        if dst.exists() {
            return Err(format!("Target already exists: {}", dst_str));
        }

        // Step 1: Copy all files to target
        let total_size = dir_size_fast(src);
        if total_size > 0 {
            let mut copied: u64 = 0;
            let mut skipped: Vec<String> = Vec::new();
            copy_dir_with_progress(
                &app, src, dst, total_size, &mut copied, &folder_key, &mut skipped,
            );
            if !skipped.is_empty() {
                let _ = std::fs::remove_dir_all(dst);
                return Err(format!(
                    "{} 个文件无法复制。源文件夹未变动。请关闭相关程序后重试。",
                    skipped.len()
                ));
            }
        } else {
            std::fs::create_dir_all(dst)
                .map_err(|e| format!("Failed to create: {}", e))?;
        }

        // Step 2: Update registry — after this, Windows uses the new path
        write_user_shell_folder(&folder_key, &dst_str)?;

        // Step 3: Delete source to actually free up space on C:
        if let Err(e) = std::fs::remove_dir_all(src) {
            // Non-fatal: data is safe at new location. Old files waste space but no data loss.
            let _ = app.emit("folder-move-progress", serde_json::json!({
                "key": folder_key,
                "percent": 100u64,
                "status": "partial",
                "message": format!("Source cleanup failed: {} — old files remain on C:", e)
            }));
        }

        // Step 4: Notify Explorer
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/c", "explorer.exe", "/update"])
            .creation_flags(0x08000000)
            .spawn();

        let _ = app.emit("folder-move-progress", serde_json::json!({
            "key": folder_key,
            "percent": 100u64,
            "status": "done"
        }));

        Ok(())
    })
    .await
    .unwrap_or_else(|_| Err("Move task panicked".into()))
}

#[tauri::command]
pub async fn undo_user_folder(
    app: tauri::AppHandle,
    folder_key: String,
    original_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let current_path = read_user_shell_folder(&folder_key)?;
        let src = std::path::Path::new(&current_path);
        let dst = std::path::Path::new(&original_path);

        if src == dst {
            return Err("Already at original location".into());
        }
        if !src.exists() {
            // Registry points somewhere but folder doesn't exist — just restore registry
            write_user_shell_folder(&folder_key, &original_path)?;
            return Ok(());
        }

        let total_size = dir_size_fast(src);
        let mut copied: u64 = 0;
        let mut skipped: Vec<String> = Vec::new();
        copy_dir_with_progress(&app, src, dst, total_size, &mut copied, &folder_key, &mut skipped);

        if !skipped.is_empty() {
            let _ = std::fs::remove_dir_all(dst);
            return Err(format!("{} 个文件无法移回。请关闭相关程序后重试。", skipped.len()));
        }

        // Update registry to original path
        write_user_shell_folder(&folder_key, &original_path)?;

        // Remove the moved copy
        let _ = std::fs::remove_dir_all(src);

        // Notify shell
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/c", "explorer.exe", "/update"])
            .creation_flags(0x08000000)
            .spawn();

        let _ = app.emit("folder-move-progress", serde_json::json!({
            "key": folder_key,
            "percent": 100u64,
            "status": "done"
        }));

        Ok(())
    })
    .await
    .unwrap_or_else(|_| Err("Undo task panicked".into()))
}

/// Re-export copy_dir_with_progress for use here
fn copy_dir_with_progress(
    app: &tauri::AppHandle,
    src: &std::path::Path,
    dst: &std::path::Path,
    total_size: u64,
    copied: &mut u64,
    app_key: &str,
    skipped: &mut Vec<String>,
) {
    let _ = std::fs::create_dir_all(dst);
    let entries = match std::fs::read_dir(src) {
        Ok(e) => e,
        Err(e) => {
            skipped.push(format!("[dir] {} (read failed: {})", src.display(), e));
            return;
        }
    };
    for entry in entries.flatten() {
        let sp = entry.path();
        let dp = dst.join(entry.file_name());
        if sp.is_dir() {
            if sp.is_symlink() { continue; }
            copy_dir_with_progress(app, &sp, &dp, total_size, copied, app_key, skipped);
        } else {
            match std::fs::copy(&sp, &dp) {
                Ok(_) => {
                    *copied += entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let pct = ((*copied as f64 / total_size as f64) * 100.0).min(99.0) as u64;
                    let _ = app.emit("folder-move-progress", serde_json::json!({
                        "key": app_key, "percent": pct, "status": "copying"
                    }));
                }
                Err(_) => {
                    skipped.push(sp.display().to_string());
                }
            }
        }
    }
}
