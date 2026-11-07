use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::os::windows::process::CommandExt;
use std::process::Command;
use tauri::Emitter;
use winreg::enums::*;
use winreg::RegKey;

// ---------------------------------------------------------------------------
// Combined status (single invoke → no focus flicker)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedStatus {
    pub defender_disabled: Option<bool>,
    pub update_disabled: Option<bool>,
    pub update_service_disabled: Option<bool>,
    pub page_file: PageFileInfo,
}

#[tauri::command]
pub async fn get_advanced_status() -> Result<AdvancedStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let defender_disabled = get_defender_status_raw().ok();
        let update_status = get_update_status_raw().ok();
        let update_disabled = update_status.as_ref().map(|s| s.disabled);
        let update_service_disabled = update_status.as_ref().map(|s| s.service_disabled);
        let page_file = get_page_file_info_raw()?;
        Ok(AdvancedStatus {
            defender_disabled,
            update_disabled,
            update_service_disabled,
            page_file,
        })
    })
    .await
    .unwrap_or_else(|_| {
        Err("Failed to read advanced status".into())
    })
}

fn get_defender_status_raw() -> Result<bool, String> {
    let output = run_defender_command(
        "$ErrorActionPreference = 'Stop'; $status = Get-MpComputerStatus; \
         if ($null -eq $status) { throw 'Defender status unavailable' }; \
         [Console]::Out.WriteLine($status.RealTimeProtectionEnabled)",
    )?;
    match output.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(false),
        "false" => Ok(true),
        _ => Err("Unexpected Defender status response".into()),
    }
}

fn run_defender_command(script: &str) -> Result<String, String> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Could not run Windows Defender command: {e}"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Windows Defender command failed: {}", error.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn get_update_status_raw() -> Result<UpdateStatus, String> {
    let policy_disabled = read_update_policy_disabled()?;
    let service_disabled = read_service_disabled("wuauserv")?
        || read_service_disabled("UsoSvc")?
        || read_service_disabled("WaaSMedicSvc")?;
    Ok(UpdateStatus {
        disabled: policy_disabled || service_disabled,
        service_disabled,
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

#[tauri::command]
pub fn get_defender_status() -> Result<bool, String> {
    get_defender_status_raw()
}

#[tauri::command]
pub async fn set_defender_disabled(disabled: bool) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let value = if disabled { "$true" } else { "$false" };
        run_defender_command(&format!(
            "$ErrorActionPreference = 'Stop'; Set-MpPreference -DisableRealtimeMonitoring {value} -ErrorAction Stop"
        ))?;
        let actual = get_defender_status_raw()?;
        if actual != disabled {
            return Err("Windows did not apply the Defender change. Check Tamper Protection or device policy.".into());
        }
        if disabled {
            crate::logger::warn("defender", "已关闭 Windows Defender 实时保护", "");
        } else {
            crate::logger::info("defender", "已恢复 Windows Defender 实时保护", "");
        }
        Ok(actual)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Windows Update
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub disabled: bool,
    pub service_disabled: bool,
}

const UPDATE_POLICY_KEY: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU";

fn read_service_disabled(name: &str) -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", name);
    let key = hklm
        .open_subkey(&path)
        .map_err(|e| format!("Could not read {name}: {e}"))?;
    let start: u32 = key
        .get_value("Start")
        .map_err(|e| format!("Could not read {name} startup type: {e}"))?;
    Ok(start == 4)
}

fn read_update_policy_disabled() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey(UPDATE_POLICY_KEY) {
        Ok(key) => key,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(format!("Could not read Windows Update policy: {e}")),
    };
    match key.get_value::<u32, _>("NoAutoUpdate") {
        Ok(value) => Ok(value == 1),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Could not read automatic update setting: {e}")),
    }
}

#[tauri::command]
pub fn get_update_status() -> Result<UpdateStatus, String> {
    get_update_status_raw()
}

#[tauri::command]
pub fn set_update_disabled(disabled: bool) -> Result<UpdateStatus, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if disabled {
        let (key, _) = hklm
            .create_subkey(UPDATE_POLICY_KEY)
            .map_err(|e| format!("Could not open Windows Update policy: {e}"))?;
        key.set_value("NoAutoUpdate", &1u32)
            .map_err(|e| format!("Could not disable automatic updates: {e}"))?;
    } else {
        match hklm.open_subkey_with_flags(UPDATE_POLICY_KEY, KEY_SET_VALUE) {
            Ok(key) => match key.delete_value("NoAutoUpdate") {
                Ok(()) => (),
                Err(e) if e.kind() == ErrorKind::NotFound => (),
                Err(e) => return Err(format!("Could not restore automatic updates: {e}")),
            },
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(format!("Could not open Windows Update policy: {e}")),
        }
    }
    if read_update_policy_disabled()? != disabled {
        return Err("Windows did not apply the automatic update policy.".into());
    }
    let status = get_update_status_raw()?;
    if status.disabled != disabled {
        return Err(if status.service_disabled {
            "One or more Windows Update services are still disabled by an older service setting.".into()
        } else {
            "Windows did not apply the automatic update policy.".into()
        });
    }
    if disabled {
        crate::logger::warn("update", "已关闭 Windows 自动更新", "");
    } else {
        crate::logger::info("update", "已恢复 Windows 自动更新", "");
    }
    Ok(status)
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
    key.set_value("ExistingPagingFiles", &entry)
        .map_err(|e| format!("Failed to set existing page file: {}", e))?;

    crate::logger::info("pagefile", &format!("虚拟内存: {} MB - {} MB", initial_mb, max_mb), "");
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
        crate::logger::info("folders", &format!("迁移文件夹 {}: {} → {}", folder_key, src_str, dst_str), "");
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
        crate::logger::info("folders", &format!("还原文件夹 {}: {} → {}", folder_key, current_path, original_path), "");
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
